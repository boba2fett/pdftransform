use std::sync::Arc;
use serde::Serialize;
use tracing::info;

use crate::{persistence::IJobPersistence, models::JobModel, dtos::GetSelfRoute};

pub struct BaseConvertService {
    pub job_persistence: Arc<dyn IJobPersistence>,
}

impl BaseConvertService {
    pub async fn ready<'a, InputType, ResultType>(&'a self, job: &mut JobModel<InputType, ResultType>, client: &reqwest::Client, result: ResultType)
        where JobModel<InputType, ResultType>: GetSelfRoute, ResultType: Clone, JobModel<InputType, ResultType>: Serialize, ResultType: Serialize + Send + Sync, InputType: Serialize + Send + Sync
    {
        job.result = Some(result);
        let result = self.job_persistence.put(job).await;
        if let Err(err) = result {
            self.error(job, client, err).await;
            return;
        }
        self.callback(job, client).await
    }

    pub async fn error<'a, InputType, ResultType>(&'a self, job: &mut JobModel<InputType, ResultType>, client: &reqwest::Client, err: &str)
        where JobModel<InputType, ResultType>: GetSelfRoute, ResultType: Clone, JobModel<InputType, ResultType>: Serialize, ResultType: Serialize + Send + Sync, InputType: Serialize + Send + Sync
    {
        job.message = Some(err.to_string());
        _ = self.job_persistence.put(job).await;
        self.callback(job, client).await
    }

    async fn callback<'a, InputType, ResultType>(&'a self, job: &JobModel<InputType, ResultType>, client: &reqwest::Client)
        where JobModel<InputType, ResultType>: GetSelfRoute, ResultType: Clone, JobModel<InputType, ResultType>: Serialize, ResultType: Serialize, InputType: Serialize
    {
        if let Some(callback_uri) = &job.callback_uri {
            let dto = job.to_dto();
            let mut retries = 0;
            loop {
                let result = client.post(callback_uri).json(&dto).send().await;
                match result {
                    Ok(ok) => {
                        info!("Send callback '{}' to '{}', with {}", &job.id, callback_uri, ok.status());
                        break;
                    },
                    Err(err) => {
                        retries += 1;
                        info!("Error sending {} time callback '{}' to '{}', because of {}", retries, &job.id, callback_uri, err);
                        if retries >= 5 {
                            break;
                        }
                    },
                }
            }
        }
    }
}