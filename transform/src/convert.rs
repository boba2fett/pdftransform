use std::sync::Arc;

use common::convert::BaseConvertService;
use common::download::{IDownloadService, DownloadedSourceFile};
use common::nats::subscribe::IWorker;
use common::persistence::{tempfiles::TempJobFileProvider};
use tracing::info;

use crate::transform::ITransformService;

pub struct ConvertService {
    pub base: Arc<BaseConvertService>,
    pub transform_service: Arc<dyn ITransformService>,
    pub download_service: Arc<dyn IDownloadService>,
}

#[async_trait::async_trait]
impl IWorker for ConvertService {
    #[tracing::instrument(skip(self))]
    async fn work(&self, job_id: String) -> Result<(), &'static str> {
        info!("Starting job");
        let job_model = self.base.transform_persistence._get_transform_job_model(&job_id).await;
        if let Ok(job_model) = job_model {
            let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
            let job_files = TempJobFileProvider::build(&job_id).await;
            let source_files = self.download_service.download_source_files(&client, job_model.data.source_files, &job_files).await;
            info!("Downloaded all files for job");

            let failed = source_files.iter().find(|source_file| source_file.is_err());

            match failed {
                None => {
                    let source_files: Vec<&DownloadedSourceFile> = source_files.iter().map(|source_file| source_file.as_ref().unwrap()).collect();
                    let results: Result<_, &str> = self.transform_service.get_transformation(&job_id, &job_model.token, &job_model.data.documents, source_files, &job_files).await;
                    match results {
                        Ok(results) => self.base.ready(&job_id, &job_model.callback_uri, &client, results, |self, job_id| self.transform_persistence._get_transform_job_dto(job_id)).await,
                        Err(err) => self.base.error(&job_id, &job_model.callback_uri, &client, err).await,
                    };
                }
                Some(err) => {
                    self.base.error(&job_id, &job_model.callback_uri, &client, err.as_ref().err().unwrap()).await;
                }
            }
            job_files.clean_up().await;
        }
        Ok(())
    }
}
