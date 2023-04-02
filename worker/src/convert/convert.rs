use std::sync::Arc;

use futures::Future;
use serde::Serialize;
use tracing::info;

use crate::download::{DownloadService, DownloadedSourceFile};
use common::models::{BaseJobDto, PreviewJobModel, TransformJobModel};
use common::persistence::{tempfiles::TempJobFileProvider, JobsBasePersistence, PreviewPersistence, TransformPersistence};

use super::{preview::pdfium::PreviewService, transform::TransformService};

#[async_trait::async_trait]
pub trait ConvertService: Send + Sync {
    async fn process_transform_job(&self, job_id: String, job_model: Option<TransformJobModel>);
    async fn process_preview_job(&self, job_id: String, job_model: Option<PreviewJobModel>);
}

pub struct ConvertServiceImpl {
    pub base_persistence: Arc<dyn JobsBasePersistence>,
    pub transform_persistence: Arc<dyn TransformPersistence>,
    pub preview_persistence: Arc<dyn PreviewPersistence>,
    pub preview_service: Arc<dyn PreviewService>,
    pub transform_service: Arc<dyn TransformService>,
    pub download_service: Arc<dyn DownloadService>,
}

#[async_trait::async_trait]
impl ConvertService for ConvertServiceImpl {
    #[tracing::instrument(skip(self, job_model))]
    async fn process_transform_job(&self, job_id: String, job_model: Option<TransformJobModel>) {
        info!("Starting job");
        let job_model = job_model.ok_or(self.transform_persistence._get_transform_job_model(&job_id).await);
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
                        Ok(results) => self.ready(&job_id, &job_model.callback_uri, &client, results, |self, job_id| self.transform_persistence._get_transform_job_dto(job_id)).await,
                        Err(err) => self.error(&job_id, &job_model.callback_uri, &client, err).await,
                    };
                }
                Some(err) => {
                    self.error(&job_id, &job_model.callback_uri, &client, err.as_ref().err().unwrap()).await;
                }
            }
            job_files.clean_up().await;
        }
    }

    #[tracing::instrument(skip(self, job_model))]
    async fn process_preview_job(&self, job_id: String, job_model: Option<PreviewJobModel>) {
        info!("Starting job");
        let job_model = job_model.ok_or(self.preview_persistence._get_preview_job_model(&job_id).await);
        if let Ok(job_model) = job_model {
            let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
            let job_files = TempJobFileProvider::build(&job_id).await;
            let source_file = self.download_service.download_source_bytes(&client, &job_model.data.source_uri.unwrap()).await;
            info!("Downloaded file for job");

            match source_file {
                Ok(source_file) => {
                    let result: Result<_, &str> = self.preview_service.get_preview(&job_id, &job_model.token, source_file.to_vec()).await;
                    match result {
                        Ok(result) => self.ready(&job_id, &job_model.callback_uri, &client, result, |self, job_id| self.preview_persistence._get_preview_job_dto(job_id)).await,
                        Err(err) => self.error(&job_id, &job_model.callback_uri, &client, err).await,
                    };
                }
                Err(err) => {
                    self.error(&job_id, &job_model.callback_uri, &client, err).await;
                }
            }
            job_files.clean_up().await;
        }
    }
}

impl ConvertServiceImpl {
    async fn ready<'a, 'b, ResultType: Serialize, JobType: Serialize + Sized, F, Fut>(&'b self, job_id: &'b str, callback_uri: &Option<String>, client: &reqwest::Client, result: ResultType, job_fn: F)
    where
        F: Send + 'static,
        F: FnOnce(&'b Self, &'b str) -> Fut,
        Fut: Future<Output = Result<JobType, &'static str>> + Send,
    {
        info!("Finished job");
        let result_bson = bson::to_bson(&result).unwrap();
        let result = self.base_persistence.set_ready(job_id, result_bson).await;
        if let Err(err) = result {
            self.error(job_id, callback_uri, client, err).await;
            return;
        }
        if let Some(callback_uri) = callback_uri {
            let dto = job_fn(self, &job_id).await;
            if let Ok(dto) = dto {
                let result = client.post(callback_uri).json::<JobType>(&dto).send().await;
                if let Err(err) = result {
                    info!("Error sending callback '{}' to '{}', because of {}", &job_id, callback_uri, err);
                }
            }
        }
    }

    async fn error(&self, job_id: &str, callback_uri: &Option<String>, client: &reqwest::Client, err: &str) {
        info!("Finished job with error {}", err);
        let result = self.base_persistence.set_error(job_id, err).await;
        if result.is_err() {
            return;
        }
        if let Some(callback_uri) = callback_uri {
            let dto = self.base_persistence._get_error_dto(&job_id).await;
            if let Ok(dto) = dto {
                let result = client.post(callback_uri).json::<BaseJobDto>(&dto).send().await;
                if let Err(err) = result {
                    info!("Error sending error callback to '{}', because of {}", callback_uri, err);
                }
            }
        }
    }
}
