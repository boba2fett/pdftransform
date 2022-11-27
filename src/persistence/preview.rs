use bson::{doc, oid::ObjectId};
use mongodb::bson::DateTime;
use std::str::FromStr;

use crate::{
    models::{CreatePreviewJobDto, JobLinks, JobStatus, PreviewJobDto, PreviewJobModel},
    routes::preview_job_route,
};

use super::{generate_30_alphanumeric, get_previews};

pub async fn get_preview_job_dto(
    client: &mongodb::Client,
    job_id: &String,
    token: String,
) -> Result<PreviewJobDto, &'static str> {
    let job_model = get_preview_job_model(client, &job_id, &token).await?;
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

pub async fn _get_preview_job_dto(
    client: &mongodb::Client,
    job_id: &str,
) -> Result<PreviewJobDto, &'static str> {
    let job_model = _get_preview_job_model(client, &job_id).await?;
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

pub async fn get_preview_job_model(
    client: &mongodb::Client,
    job_id: &str,
    token: &str,
) -> Result<PreviewJobModel, &'static str> {
    let jobs = get_previews(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs
            .find_one(Some(doc! {"_id": id, "token": token}), None)
            .await
        {
            if let Some(job_model) = result {
                return Ok(job_model);
            }
        }
    }
    Err("Could not find job")
}

pub async fn _get_preview_job_model(
    client: &mongodb::Client,
    job_id: &str,
) -> Result<PreviewJobModel, &'static str> {
    let jobs = get_previews(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
            if let Some(job_model) = result {
                return Ok(job_model);
            }
        }
    }
    Err("Could not find job")
}

pub async fn create_new_preview_job(
    client: &mongodb::Client,
    create_job: CreatePreviewJobDto,
) -> Result<(PreviewJobDto, PreviewJobModel), &'static str> {
    let job = PreviewJobModel {
        id: None,
        status: JobStatus::InProgress,
        callback_uri: create_job.callback_uri,
        source_uri: Some(create_job.source_uri),
        result: None,
        message: None,
        token: generate_30_alphanumeric(),
        created: DateTime::now(),
    };
    save_new_preview_job(client, job).await
}

pub async fn save_new_preview_job(
    client: &mongodb::Client,
    job: PreviewJobModel,
) -> Result<(PreviewJobDto, PreviewJobModel), &'static str> {
    let jobs = get_previews(client);
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
            PreviewJobModel {
                id: Some(id),
                ..job_clone
            },
        ));
    }
    Err("Could not save job")
}
