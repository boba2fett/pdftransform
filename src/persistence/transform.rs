use bson::{doc, oid::ObjectId};
use mongodb::bson::DateTime;
use std::str::FromStr;

use crate::{
    models::{CreateTransformJobDto, JobLinks, JobStatus, TransformJobDto, TransformJobModel},
    routes::transform_job_route,
};

use super::{generate_30_alphanumeric, get_transformations};

pub async fn get_transform_job_dto(client: &mongodb::Client, job_id: &String, token: String) -> Result<TransformJobDto, &'static str> {
    let job_model = get_transform_job_model(client, &job_id, &token).await?;
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

pub async fn _get_transform_job_dto(client: &mongodb::Client, job_id: &str) -> Result<TransformJobDto, &'static str> {
    let job_model = _get_transform_job_model(client, &job_id).await?;
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

pub async fn get_transform_job_model(client: &mongodb::Client, job_id: &str, token: &str) -> Result<TransformJobModel, &'static str> {
    let jobs = get_transformations(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.find_one(Some(doc! {"_id": id, "token": token}), None).await {
            if let Some(job_model) = result {
                return Ok(job_model);
            }
        }
    }
    Err("Could not find job")
}

pub async fn _get_transform_job_model(client: &mongodb::Client, job_id: &str) -> Result<TransformJobModel, &'static str> {
    let jobs = get_transformations(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
            if let Some(job_model) = result {
                return Ok(job_model);
            }
        }
    }
    Err("Could not find job")
}

pub async fn create_new_transform_job(client: &mongodb::Client, create_job: CreateTransformJobDto) -> Result<(TransformJobDto, TransformJobModel), &'static str> {
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
    save_new_transform_job(client, job).await
}

pub async fn save_new_transform_job(client: &mongodb::Client, job: TransformJobModel) -> Result<(TransformJobDto, TransformJobModel), &'static str> {
    let jobs = get_transformations(client);
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
