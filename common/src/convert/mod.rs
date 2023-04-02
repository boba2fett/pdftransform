use std::sync::Arc;

use futures::Future;
use serde::Serialize;
use tracing::info;

use crate::{persistence::{JobsBasePersistence, PreviewPersistence, TransformPersistence}, models::BaseJobDto};

pub struct BaseConvertService {
    pub base_persistence: Arc<dyn JobsBasePersistence>,
    pub preview_persistence: Arc<dyn PreviewPersistence>,
    pub transform_persistence: Arc<dyn TransformPersistence>,
}

impl BaseConvertService {
    pub async fn ready<'a, 'b, ResultType: Serialize, JobType: Serialize + Sized, F, Fut>(&'b self, job_id: &'b str, callback_uri: &Option<String>, client: &reqwest::Client, result: ResultType, job_fn: F)
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

    pub async fn error(&self, job_id: &str, callback_uri: &Option<String>, client: &reqwest::Client, err: &str) {
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