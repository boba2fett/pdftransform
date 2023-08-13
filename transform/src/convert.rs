use std::str::FromStr;
use std::sync::Arc;

use common::convert::BaseConvertService;
use common::download::{IDownloadService, DownloadedSourceFile};
use common::models::TransformJobModel;
use common::nats::subscribe::{WorkError, IWorkerService};
use common::persistence::tempfiles::TempJobFileProvider;
use mime::Mime;
use tracing::info;

use crate::file_converter::IFileConverterService;
use crate::transform::ITransformService;

pub struct ConvertService {
    pub base: Arc<BaseConvertService>,
    pub transform_service: Arc<dyn ITransformService>,
    pub download_service: Arc<dyn IDownloadService>,
    pub file_converter_service: Arc<dyn IFileConverterService>,
}

#[async_trait::async_trait]
impl IWorkerService for ConvertService {
    #[tracing::instrument(skip(self))]
    async fn work(&self, job_id: &str) -> Result<(), WorkError> {
        info!("Starting job");
        let job_model = self.base.job_persistence.get(&job_id).await;
        if let Ok(Some(job_model)) = job_model {
            let mut job_model = TransformJobModel::from_json_slice(&job_model).map_err(|_| WorkError::NoRetry)?;
            let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();

            let files_for_conversion: Vec<_> = job_model.input.source_files.iter().filter(|source_file| {
                if let Some(mime_type) = &source_file.content_type {
                    if let Ok(mime_type) = Mime::from_str(mime_type) {
                        return !mime_type.eq(&mime::APPLICATION_PDF) && !self.transform_service.is_supported_image(&mime_type);
                    }
                }
                false
            }).collect();

            if files_for_conversion.len() > 0 {
                for file in files_for_conversion {
                    self.file_converter_service.convert(job_id, file).await; //TODO
                }
                return Ok(());
            }

            let job_files = TempJobFileProvider::build(&job_id).await;
            let source_files = self.download_service.download_source_files(&client, job_model.input.source_files.clone(), &job_files).await;
            info!("Downloaded all files for job");

            let failed = source_files.iter().find(|source_file| source_file.is_err());

            match failed {
                None => {
                    let source_files: Vec<&DownloadedSourceFile> = source_files.iter().map(|source_file| source_file.as_ref().unwrap()).collect();
                    let results: Result<_, &str> = self.transform_service.get_transformation(&job_id, &job_model.input.documents, source_files, &job_files).await;
                    match results {
                        Ok(results) => self.base.ready(&mut job_model, &client, results).await,
                        Err(err) => self.base.error(&mut job_model, &client, err).await,
                    };
                }
                Some(err) => {
                    self.base.error(&mut job_model, &client, err.as_ref().err().unwrap()).await;
                }
            }
            job_files.clean_up().await;
        }
        Ok(())
    }
}
