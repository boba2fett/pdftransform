use std::{env, error::Error, str::FromStr};
use bson::{doc, oid::ObjectId};
use mongodb::{options::ClientOptions, Client, Collection};
use rand::{thread_rng, Rng, distributions::Alphanumeric};

use crate::{consts::NAME, models::{JobDto, CreateJobDto, JobModel, JobStatus, DocumentResult, Links}};

async fn get_client() -> Result<Client, Box<dyn Error>> {
    let client_uri = env::var("MONGO_URI")?;
    let options = ClientOptions::parse(&client_uri).await?;
    let client = Client::with_options(options)?;
    Ok(client)
}

async fn get_jobs<'a>() -> Result<Collection<JobModel>, &'static str> {
    let client = get_client().await;
    match client {
        Ok(client) => Ok(client.database(NAME).collection("jobs")),
        Err(_) => Err("Database currently unreachable"),
    }
}

pub async fn get_job_dto(job_id: &String, token: String) -> Result<JobDto, &'static str> {
    let job_model = get_job_model(&job_id, &token).await?;
    let job_id = job_model.id.unwrap().to_string();
    return Ok(JobDto {
        status: job_model.status,
        results: job_model.results,
        _links: Links { _self: get_self_url(&job_id, &job_model.token) },
        id: job_id,
    })
}

pub async fn get_job_model(job_id: &str, token: &str) -> Result<JobModel, &'static str> {
    if let Ok(jobs) = get_jobs().await
    {
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc!{"_id": id, "token": token}), None).await {
                if let Some(job_model) = result {
                    return Ok(job_model)
                }
            }
        }
    }
    Err("Could not find job")
}

pub async fn _get_job_model(job_id: &String) -> Result<JobModel, &'static str> {
    if let Ok(jobs) = get_jobs().await
    {
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
                if let Some(job_model) = result {
                    return Ok(job_model)
                }
            }
        }
    }
    Err("Could not find job")
}

pub async fn create_new_job<'a>(create_job: CreateJobDto) -> Result<JobDto, &'static str> {
    let job = JobModel {
        id: None,
        status: JobStatus::InProgress,
        callback_uri :  create_job.callback_uri,
        documents: create_job.documents,
        source_files: create_job.source_files,
        results: vec![],
        message: None,
        token: generate_token()
    };
    save_new_job(job).await
}

pub fn generate_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

pub fn get_self_url(job_id: &String, job_token: &String) -> String {
    format!("/convert/{}?token={}", &job_id, job_token)
}

pub async fn save_new_job(job: JobModel) -> Result<JobDto, &'static str> {
    if let Ok(jobs) = get_jobs().await
    {
        let job_token = job.token.clone();
        if let Ok(insert_result) = jobs.insert_one(job, None).await {
            let id = insert_result
            .inserted_id
            .as_object_id().expect("msg");
            let job_id = id.to_string();
            return Ok(JobDto {
                status: JobStatus::InProgress,
                results: vec![],
                _links: Links { _self: get_self_url(&job_id, &job_token) },
                id: job_id,
            })
        }
    }
    Err("Could not save job")
}

pub async fn set_ready(job_id: &String, results: Vec<DocumentResult>) -> Result<(), &'static str> {
    if let Ok(jobs) = get_jobs().await
    {
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.update_one(doc!{"_id": id}, doc!{"$set": {"status": JobStatus::Finished as u32 ,"results": bson::to_bson(&results).ok()}}, None).await {
                if result.modified_count > 0 {
                    return Ok(())
                }
            }
        }
    }
    Err("Could not find job")
}

pub async fn set_error(job_id: &String, err: &'static str) -> Result<(), &'static str> {
    if let Ok(jobs) = get_jobs().await
    {
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.update_one(doc!{"_id": id}, doc!{"$set": {"status": JobStatus::Error as u32, "message": err}}, None).await {
                if result.modified_count > 0 {
                    return Ok(())
                }
            }
        }
    }
    Err("Could not find job")
}