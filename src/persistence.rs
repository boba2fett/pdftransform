use std::{env, error::Error, str::FromStr};
use bson::{doc, oid::ObjectId};
use mongodb::{options::ClientOptions, Client, Collection};

use crate::{consts::NAME, models::{JobDto, CreateJobDto, JobModel, JobStatus}};

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

pub async fn get_job(job_id: String) -> Result<JobDto, &'static str> {
    if let Ok(jobs) = get_jobs().await
    {
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
                if let Some(job_dto) = result {
                    return Ok(JobDto {
                        id: job_id,
                        status: job_dto.status,
                    })
                }
            }
        }
    }
    Err("Could not find Job")
}

pub async fn save_job<'a>(create_job: CreateJobDto<'a>) -> Result<JobDto, &'static str> {
    let job = JobModel {
        id: None,
        status: JobStatus::WaitingForFile,
        source_uri: create_job.source_uri.to_string(),
        callback_uri :  create_job.callback_uri.to_string(),
        source_mime_type:  create_job.source_mime_type.to_string(),
        destination_mime_type:  create_job.destination_mime_type.to_string(),
    };
    if let Ok(jobs) = get_jobs().await
    {
        if let Ok(insert_result) = jobs.insert_one(job, None).await {
            let id = insert_result
            .inserted_id
            .as_object_id().expect("msg");
            return Ok(JobDto {
                id: id.to_string(),
                status: JobStatus::WaitingForFile,
            })
        }
    }
    Err("Could not create Job")
}
