use std::sync::Arc;

use common::convert::BaseConvertService;
use common::nats::subscribe::IWorker;
use tracing::info;

use common::download::{IDownloadService};
use common::persistence::{tempfiles::TempJobFileProvider};

use super::preview::IPreviewService;

pub struct ConvertService {
    pub base: Arc<BaseConvertService>,
    pub preview_service: Arc<dyn IPreviewService>,
    pub download_service: Arc<dyn IDownloadService>,
}

#[async_trait::async_trait]
impl IWorker for ConvertService {
    #[tracing::instrument(skip(self))]
    async fn work(&self, job_id: &str) -> Result<(), &'static str> {
        info!("Starting job");
        let job_model = self.base.preview_persistence._get_preview_job_model(&job_id).await;
        if let Ok(job_model) = job_model {
            let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
            let job_files = TempJobFileProvider::build(&job_id).await;
            let source_file = self.download_service.download_source_bytes(&client, &job_model.data.source_uri.unwrap()).await;
            info!("Downloaded file for job");

            match source_file {
                Ok(source_file) => {
                    let result: Result<_, &str> = self.preview_service.get_preview(&job_id, &job_model.token, source_file.to_vec()).await;
                    match result {
                        Ok(result) => self.base.ready(&job_id, &job_model.callback_uri, &client, result, |self, job_id| self.preview_persistence._get_preview_job_dto(job_id)).await,
                        Err(err) => self.base.error(&job_id, &job_model.callback_uri, &client, err).await,
                    };
                }
                Err(err) => {
                    self.base.error(&job_id, &job_model.callback_uri, &client, err).await;
                }
            }
            job_files.clean_up().await;
        }
        Ok(())
    }
}
