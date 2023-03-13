use bson::{doc, oid::ObjectId};
use mongodb::{bson::DateTime, Collection};
use std::{str::FromStr, sync::Arc};

use crate::{
    models::{CreateTransformJobDto, JobLinks, JobStatus, TransformJobDto, TransformJobModel}, routes::transform::transform_job_route, util::random::generate_30_alphanumeric,
};

use super::MongoPersistenceBase;

#[async_trait::async_trait]
pub trait TransformPersistence: Send + Sync {
    async fn get_transform_job_dto(&self, job_id: &String, token: &str) -> Result<TransformJobDto, &'static str>;
    async fn _get_transform_job_dto(&self, job_id: &str) -> Result<TransformJobDto, &'static str>;
    async fn get_transform_job_model(&self, job_id: &str, token: &str) -> Result<TransformJobModel, &'static str>;
    async fn _get_transform_job_model(&self, job_id: &str) -> Result<TransformJobModel, &'static str>;
    async fn create_new_transform_job(&self, create_job: CreateTransformJobDto) -> Result<(TransformJobDto, TransformJobModel), &'static str>;
    async fn save_new_transform_job(&self, job: TransformJobModel) -> Result<(TransformJobDto, TransformJobModel), &'static str>;
}

pub struct MongoTransformPersistence {
    pub base: Arc<MongoPersistenceBase>,
}

#[async_trait::async_trait]
impl TransformPersistence for MongoTransformPersistence {

    async fn get_transform_job_dto(&self, job_id: &String, token: &str) -> Result<TransformJobDto, &'static str> {
        let job_model = self.get_transform_job_model(&job_id, &token).await?;
        let job_id = job_model.id.unwrap().to_string();
        Ok(TransformJobDto {
            message: job_model.message,
            status: job_model.status,
            result: job_model.result,
            _links: JobLinks {
                _self: transform_job_route(&job_id, &job_model.token),
            },
            id: job_id,
        })
    }

    async fn _get_transform_job_dto(&self, job_id: &str) -> Result<TransformJobDto, &'static str> {
        let job_model = self._get_transform_job_model( &job_id).await?;
        let job_id = job_model.id.unwrap().to_string();
        Ok(TransformJobDto {
            message: job_model.message,
            status: job_model.status,
            result: job_model.result,
            _links: JobLinks {
                _self: transform_job_route(&job_id, &job_model.token),
            },
            id: job_id,
        })
    }

    async fn get_transform_job_model(&self, job_id: &str, token: &str) -> Result<TransformJobModel, &'static str> {
        let jobs = self.get_transformations();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc! {"_id": id, "token": token}), None).await {
                if let Some(job_model) = result {
                    return Ok(job_model);
                }
            }
        }
        Err("Could not find job")
    }

    async fn _get_transform_job_model(&self, job_id: &str) -> Result<TransformJobModel, &'static str> {
        let jobs = self.get_transformations();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
                if let Some(job_model) = result {
                    return Ok(job_model);
                }
            }
        }
        Err("Could not find job")
    }

    async fn create_new_transform_job(&self, create_job: CreateTransformJobDto) -> Result<(TransformJobDto, TransformJobModel), &'static str> {
        let job = TransformJobModel {
            id: None,
            status: JobStatus::InProgress,
            callback_uri: create_job.callback_uri,
            documents: create_job.documents,
            source_files: create_job.source_files,
            result: vec![],
            message: None,
            token: generate_30_alphanumeric(),
            created: DateTime::now(),
        };
        self.save_new_transform_job(job).await
    }

    async fn save_new_transform_job(&self, job: TransformJobModel) -> Result<(TransformJobDto, TransformJobModel), &'static str> {
        let jobs = self.get_transformations();
        let job_clone = job.clone();
        if let Ok(insert_result) = jobs.insert_one(job, None).await {
            let id = insert_result.inserted_id.as_object_id().unwrap();
            let job_id = id.to_string();
            return Ok((
                TransformJobDto {
                    message: None,
                    status: JobStatus::InProgress,
                    result: vec![],
                    _links: JobLinks {
                        _self: transform_job_route(&job_id, &job_clone.token),
                    },
                    id: job_id,
                },
                TransformJobModel { id: Some(id), ..job_clone },
            ));
        }
        Err("Could not save job")
    }
}

impl MongoTransformPersistence {
    fn get_transformations(&self) -> Collection<TransformJobModel> {
        self.base.get_jobs()
    }
}