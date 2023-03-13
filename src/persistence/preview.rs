use bson::{doc, oid::ObjectId};
use mongodb::{bson::DateTime, Collection};
use std::{str::FromStr, sync::Arc};

use crate::{
    models::{CreatePreviewJobDto, JobLinks, JobStatus, PreviewJobDto, PreviewJobModel}, routes::preview::preview_job_route, util::random::generate_30_alphanumeric,
};

use super::MongoPersistenceBase;

#[async_trait::async_trait]
pub trait PreviewPersistence: Send + Sync {
    async fn get_preview_job_dto(&self, job_id: &String, token: &str) -> Result<PreviewJobDto, &'static str>;
    async fn _get_preview_job_dto(&self, job_id: &str) -> Result<PreviewJobDto, &'static str>;
    async fn get_preview_job_model(&self, job_id: &str, token: &str) -> Result<PreviewJobModel, &'static str>;
    async fn _get_preview_job_model(&self, job_id: &str) -> Result<PreviewJobModel, &'static str>;
    async fn create_new_preview_job(&self, create_job: CreatePreviewJobDto) -> Result<(PreviewJobDto, PreviewJobModel), &'static str>;
    async fn save_new_preview_job(&self, job: PreviewJobModel) -> Result<(PreviewJobDto, PreviewJobModel), &'static str>;
}

pub struct MongoPreviewPersistence {
    pub base: Arc<MongoPersistenceBase>,
}

#[async_trait::async_trait]
impl PreviewPersistence for MongoPreviewPersistence {

    async fn get_preview_job_dto(&self, job_id: &String, token: &str) -> Result<PreviewJobDto, &'static str> {
        let job_model = self.get_preview_job_model(&job_id, &token).await?;
        let job_id = job_model.id.unwrap().to_string();
        Ok(PreviewJobDto {
            message: job_model.message,
            status: job_model.status,
            result: job_model.result,
            _links: JobLinks {
                _self: preview_job_route(&job_id, &job_model.token),
            },
            id: job_id,
        })
    }

    async fn _get_preview_job_dto(&self, job_id: &str) -> Result<PreviewJobDto, &'static str> {
        let job_model = self._get_preview_job_model(&job_id).await?;
        let job_id = job_model.id.unwrap().to_string();
        Ok(PreviewJobDto {
            message: job_model.message,
            status: job_model.status,
            result: job_model.result,
            _links: JobLinks {
                _self: preview_job_route(&job_id, &job_model.token),
            },
            id: job_id,
        })
    }

    async fn get_preview_job_model(&self, job_id: &str, token: &str) -> Result<PreviewJobModel, &'static str> {
        let jobs = self.get_previews();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc! {"_id": id, "token": token}), None).await {
                if let Some(job_model) = result {
                    return Ok(job_model);
                }
            }
        }
        Err("Could not find job")
    }

    async fn _get_preview_job_model(&self, job_id: &str) -> Result<PreviewJobModel, &'static str> {
        let jobs = self.get_previews();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
                if let Some(job_model) = result {
                    return Ok(job_model);
                }
            }
        }
        Err("Could not find job")
    }

    async fn create_new_preview_job(&self, create_job: CreatePreviewJobDto) -> Result<(PreviewJobDto, PreviewJobModel), &'static str> {
        let job = PreviewJobModel {
            id: None,
            status: JobStatus::InProgress,
            callback_uri: create_job.callback_uri,
            source_uri: Some(create_job.source_uri),
            source_mime_type: create_job.source_mime_type.unwrap_or("application/pdf".to_string()),
            result: None,
            message: None,
            token: generate_30_alphanumeric(),
            created: DateTime::now(),
        };
        self.save_new_preview_job(job).await
    }

    async fn save_new_preview_job(&self, job: PreviewJobModel) -> Result<(PreviewJobDto, PreviewJobModel), &'static str> {
        let jobs = self.get_previews();
        let job_clone = job.clone();
        if let Ok(insert_result) = jobs.insert_one(job, None).await {
            let id = insert_result.inserted_id.as_object_id().unwrap();
            let job_id = id.to_string();
            return Ok((
                PreviewJobDto {
                    message: None,
                    status: JobStatus::InProgress,
                    result: None,
                    _links: JobLinks {
                        _self: preview_job_route(&job_id, &job_clone.token),
                    },
                    id: job_id,
                },
                PreviewJobModel { id: Some(id), ..job_clone },
            ));
        }
        Err("Could not save job")
    }
}

impl MongoPreviewPersistence {
    pub fn get_previews(&self) -> Collection<PreviewJobModel> {
        self.base.get_jobs()
    }
}