use std::sync::Arc;

use common::convert::BaseConvertService;
use common::models::PreviewJobModel;
use common::nats::subscribe::{IWorkerService, WorkError};
use tracing::info;

use common::download::IDownloadService;
use common::persistence::tempfiles::TempJobFileProvider;

use super::preview::IPreviewService;

pub struct ConvertService {
    pub base: Arc<BaseConvertService>,
    pub preview_service: Arc<dyn IPreviewService>,
    pub download_service: Arc<dyn IDownloadService>,
}

#[async_trait::async_trait]
impl IWorkerService for ConvertService {
    #[tracing::instrument(skip(self))]
    async fn work(&self, job_id: &str) -> Result<(), WorkError> {
        info!("Starting job");
        let job_model = self.base.job_persistence.get(&job_id).await;
        if let Ok(Some(job_model)) = job_model {
            let mut job_model = PreviewJobModel::from_json_slice(&job_model).map_err(|_| WorkError::NoRetry)?;
            let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
            let job_files = TempJobFileProvider::build(&job_id).await;
            let source_file = self.download_service.download_source_bytes(&client, &job_model.input.source_uri).await;
            info!("Downloaded file for job");

            match source_file {
                Ok(source_file) => {
                    let result: Result<_, &str> = self.preview_service.get_preview(&job_model, source_file.to_vec()).await;
                    match result {
                        Ok(result) => self.base.ready(&mut job_model, &client, result).await,
                        Err(err) => self.base.error(&mut job_model, &client, err).await,
                    };
                }
                Err(err) => {
                    self.base.error(&mut job_model, &client, err).await;
                }
            }
            job_files.clean_up().await;
        }
        Ok(())
    }
}
