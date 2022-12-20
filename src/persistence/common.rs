use bson::{doc, oid::ObjectId};
use futures::StreamExt;
use mongodb::{
    bson::DateTime,
    error::Error,
    options::{ClientOptions, IndexOptions},
    Client, Collection, IndexModel,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket_db_pools::Database;
use serde::Serialize;
use std::{str::FromStr, time::Duration};

use crate::{
    consts::NAME,
    models::{AvgTimeModel, DummyModel, JobStatus, PreviewJobModel, TransformJobModel},
};

#[derive(Database)]
#[database("db")]
pub struct DbClient(pub Client);

pub async fn set_expire_after(mongo_uri: &str, seconds: u64) -> Result<Client, Error> {
    let options = ClientOptions::parse(&mongo_uri).await?;
    let client = Client::with_options(options)?;
    let jobs = get_jobs::<DummyModel>(&client);

    let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
    let index = IndexModel::builder().keys(doc! {"created": 1}).options(options).build();

    jobs.create_index(index.clone(), None).await?;

    Ok(client)
}

pub fn get_transformations(db_client: &mongodb::Client) -> Collection<TransformJobModel> {
    get_jobs(db_client)
}

pub fn get_previews(db_client: &mongodb::Client) -> Collection<PreviewJobModel> {
    get_jobs(db_client)
}

pub fn get_jobs<T>(db_client: &mongodb::Client) -> Collection<T> {
    db_client.database(NAME).collection("jobs")
}

pub async fn set_ready<ResultType: Serialize>(client: &mongodb::Client, job_id: &str, results: ResultType) -> Result<(), &'static str> {
    let jobs = get_jobs::<DummyModel>(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs
            .update_one(doc! {"_id": id}, doc! {"$set": {"status": JobStatus::Finished as u32 ,"result": bson::to_bson(&results).ok(), "finished": DateTime::now()}}, None)
            .await
        {
            if result.modified_count > 0 {
                return Ok(());
            }
        }
    }
    Err("Could not find job")
}

pub async fn set_error(client: &mongodb::Client, job_id: &str, err: &str) -> Result<(), &'static str> {
    let jobs = get_jobs::<DummyModel>(client);
    if let Ok(id) = ObjectId::from_str(&job_id) {
        if let Ok(result) = jobs.update_one(doc! {"_id": id}, doc! {"$set": {"status": JobStatus::Error as u32, "message": err, "finished": DateTime::now()}}, None).await {
            if result.modified_count > 0 {
                return Ok(());
            }
        }
    }
    Err("Could not find job")
}

pub fn generate_30_alphanumeric() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect()
}

pub async fn jobs_health(client: &mongodb::Client) -> Result<Vec<AvgTimeModel>, &'static str> {
    let cursor = get_jobs::<DummyModel>(client)
        .aggregate(
            [
                doc! {
                    "$set": {
                        "time": {
                            "$dateDiff": {
                                "startDate": "$created",
                                "endDate": { "$ifNull": ["$finished", "$$NOW"]},
                                "unit": "millisecond"
                            }
                        }
                    }
                },
                doc! {
                    "$group": {
                        "_id": {
                            "status": "$status"
                        },
                        "avgTimeMillis": {
                            "$avg": "$time"
                        },
                        "count": {
                            "$count": {}
                        }
                    }
                },
                doc! {
                    "$set": {
                        "status": "$_id.status"
                      }
                },
            ],
            None,
        )
        .await
        .map_err(|_| "Could not get heath.")?
        .with_type::<AvgTimeModel>();

    let documents: Vec<Result<AvgTimeModel, Error>> = cursor.collect().await;
    let mut results = Vec::with_capacity(documents.len());
    for document in documents {
        let document = document.map_err(|_| "Could not get health.")?;
        results.push(document);
    }
    Ok(results)
}
