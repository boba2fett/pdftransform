use std::{str::FromStr, time::Duration};
use bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection, bson::DateTime, options::{ClientOptions, IndexOptions}, IndexModel, error::Error};
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use rocket_db_pools::Database;

use crate::{consts::NAME, models::{JobDto, CreateJobDto, JobModel, JobStatus, DocumentResult, ConvertLinks}, routes::job_route};

#[derive(Database)]
#[database("db")]
pub struct DbClient(pub Client);

pub async fn set_expire_after(mongo_uri: &str, seconds: u64) -> Result<Client, Error> {
    let options = ClientOptions::parse(&mongo_uri).await?;
    let client = Client::with_options(options)?;
    let jobs = get_jobs(&client);

    let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
    let index = IndexModel::builder()
        .keys(doc! {"created": 1})
        .options(options)
        .build();

    jobs.create_index(index.clone(), None).await?;

    Ok(client)
}

fn get_jobs(db_client: &mongodb::Client) -> Collection<JobModel> {
    db_client.database(NAME).collection("jobs")
}

pub async fn get_job_dto(client: &mongodb::Client, job_id: &String, token: String) -> Result<JobDto, &'static str> {
    let job_model = get_job_model(client, &job_id, &token).await?;
    let job_id = job_model.id.unwrap().to_string();
    Ok(JobDto {
        message: job_model.message,
        status: job_model.status,
        results: job_model.results,
        _links: ConvertLinks { _self: job_route(&job_id, &job_model.token) },
        id: job_id,
    })
}

pub async fn _get_job_dto(client: &mongodb::Client, job_id: &str) -> Result<JobDto, &'static str> {
    let job_model = _get_job_model(client, &job_id).await?;
    let job_id = job_model.id.unwrap().to_string();
    Ok(JobDto {
        message: job_model.message,
        status: job_model.status,
        results: job_model.results,
        _links: ConvertLinks { _self: job_route(&job_id, &job_model.token) },
        id: job_id,
    })
}

pub async fn get_job_model(client: &mongodb::Client, job_id: &str, token: &str) -> Result<JobModel, &'static str> {
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

pub async fn _get_job_model(client: &mongodb::Client, job_id: &str) -> Result<JobModel, &'static str> {
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

pub async fn create_new_job<'a>(client: &mongodb::Client, create_job: CreateJobDto) -> Result<(JobDto, JobModel), &'static str> {
    let job = JobModel {
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

pub async fn save_new_job(client: &mongodb::Client, job: JobModel) -> Result<(JobDto, JobModel), &'static str> {
    let jobs = get_jobs(client);
    let job_clone = job.clone();
    if let Ok(insert_result) = jobs.insert_one(job, None).await {
        let id = insert_result
        .inserted_id
        .as_object_id().expect("msg");
        let job_id = id.to_string();
        return Ok((JobDto {
            message: None,
            status: JobStatus::InProgress,
            results: vec![],
            _links: ConvertLinks { _self: job_route(&job_id, &job_clone.token) },
            id: job_id,
        }, JobModel {
            id: Some(id),
            ..job_clone
        }))
    }
    Err("Could not save job")
}

pub async fn set_ready(client: &mongodb::Client, job_id: &str, results: Vec<DocumentResult>) -> Result<(), &'static str> {
    let jobs = get_jobs(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.update_one(doc!{"_id": id}, doc!{"$set": {"status": JobStatus::Finished as u32 ,"results": bson::to_bson(&results).ok(), "finished": DateTime::now()}}, None).await {
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
