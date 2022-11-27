use std::{str::FromStr, time::Duration};
use bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection, bson::DateTime, options::{ClientOptions, IndexOptions}, IndexModel, error::Error};
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use rocket_db_pools::Database;
use serde::Serialize;

use crate::{consts::NAME, models::{JobStatus, TransformJobModel, PreviewJobModel, JobLinks, TransformJobDto, CreateTransformJobDto}, routes::job_route};

#[derive(Database)]
#[database("db")]
pub struct DbClient(pub Client);

pub async fn set_expire_after(mongo_uri: &str, seconds: u64) -> Result<Client, Error> {
    let options = ClientOptions::parse(&mongo_uri).await?;
    let client = Client::with_options(options)?;
    let jobs = get_jobs(&client);
    let previews = get_jobs(&client);

    let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
    let index = IndexModel::builder()
        .keys(doc! {"created": 1})
        .options(options)
        .build();

    jobs.create_index(index.clone(), None).await?;
    previews.create_index(index, None).await?;

    Ok(client)
}

fn get_jobs(db_client: &mongodb::Client) -> Collection<TransformJobModel> {
    db_client.database(NAME).collection("jobs")
}

fn get_previews(db_client: &mongodb::Client) -> Collection<PreviewJobModel> {
    db_client.database(NAME).collection("previews")
}

pub async fn get_job_dto(client: &mongodb::Client, job_id: &String, token: String) -> Result<TransformJobDto, &'static str> {
    let job_model = get_job_model(client, &job_id, &token).await?;
    let job_id = job_model.id.unwrap().to_string();
    Ok(TransformJobDto {
        message: job_model.message,
        status: job_model.status,
        result: job_model.results,
        _links: JobLinks { _self: job_route(&job_id, &job_model.token) },
        id: job_id,
    })
}

pub async fn _get_job_dto(client: &mongodb::Client, job_id: &str) -> Result<TransformJobDto, &'static str> {
    let job_model = _get_job_model(client, &job_id).await?;
    let job_id = job_model.id.unwrap().to_string();
    Ok(TransformJobDto {
        message: job_model.message,
        status: job_model.status,
        result: job_model.results,
        _links: JobLinks { _self: job_route(&job_id, &job_model.token) },
        id: job_id,
    })
}

pub async fn get_preview_model(client: &mongodb::Client, job_id: &str, token: &str) -> Result<PreviewJobModel, &'static str> {
    let previews = get_previews(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = previews.find_one(Some(doc!{"_id": id, "token": token}), None).await {
            if let Some(preview_model) = result {
                return Ok(preview_model)
            }
        }
    }
    Err("Could not find job")
}

pub async fn save_new_preview(client: &mongodb::Client, preview: PreviewJobModel) -> Result<PreviewJobModel, &'static str> {
    let previews = get_previews(client);
    let preview_clone = preview.clone();
    if let Ok(insert_result) = previews.insert_one(preview, None).await {
        let id = insert_result
        .inserted_id
        .as_object_id().unwrap();
        return Ok(PreviewJobModel {
            id: Some(id),
            ..preview_clone
        })
    }
    Err("Could not save job")
}

pub async fn get_job_model(client: &mongodb::Client, job_id: &str, token: &str) -> Result<TransformJobModel, &'static str> {
    let jobs = get_jobs(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.find_one(Some(doc!{"_id": id, "token": token}), None).await {
            if let Some(job_model) = result {
                return Ok(job_model)
            }
        }
    }
    Err("Could not find job")
}

pub async fn _get_job_model(client: &mongodb::Client, job_id: &str) -> Result<TransformJobModel, &'static str> {
    let jobs = get_jobs(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
            if let Some(job_model) = result {
                return Ok(job_model)
            }
        }
    }
    Err("Could not find job")
}

pub async fn create_new_job<'a>(client: &mongodb::Client, create_job: CreateTransformJobDto) -> Result<(TransformJobDto, TransformJobModel), &'static str> {
    let job = TransformJobModel {
        id: None,
        status: JobStatus::InProgress,
        callback_uri :  create_job.callback_uri,
        documents: create_job.documents,
        source_files: create_job.source_files,
        results: vec![],
        message: None,
        token: generate_30_alphanumeric(),
        created: DateTime::now()
    };
    save_new_job(client, job).await
}

pub fn generate_30_alphanumeric() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

pub async fn save_new_job(client: &mongodb::Client, job: TransformJobModel) -> Result<(TransformJobDto, TransformJobModel), &'static str> {
    let jobs = get_jobs(client);
    let job_clone = job.clone();
    if let Ok(insert_result) = jobs.insert_one(job, None).await {
        let id = insert_result
        .inserted_id
        .as_object_id().unwrap();
        let job_id = id.to_string();
        return Ok((TransformJobDto {
            message: None,
            status: JobStatus::InProgress,
            result: vec![],
            _links: JobLinks { _self: job_route(&job_id, &job_clone.token) },
            id: job_id,
        }, TransformJobModel {
            id: Some(id),
            ..job_clone
        }))
    }
    Err("Could not save job")
}

pub async fn set_ready<ResultType: Serialize>(client: &mongodb::Client, job_id: &str, results: ResultType) -> Result<(), &'static str> {
    let jobs = get_jobs(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.update_one(doc!{"_id": id}, doc!{"$set": {"status": JobStatus::Finished as u32 ,"result": bson::to_bson(&results).ok(), "finished": DateTime::now()}}, None).await {
            if result.modified_count > 0 {
                return Ok(())
            }
        }
    }
    Err("Could not find job")
}

pub async fn set_error(client: &mongodb::Client, job_id: &str, err: &str) -> Result<(), &'static str> {
    let jobs = get_jobs(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.update_one(doc!{"_id": id}, doc!{"$set": {"status": JobStatus::Error as u32, "message": err, "finished": DateTime::now()}}, None).await {
            if result.modified_count > 0 {
                return Ok(())
            }
        }
    }
    Err("Could not find job")
}
