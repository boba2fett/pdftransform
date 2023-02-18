use bson::{doc, oid::ObjectId};
use futures::StreamExt;
use mongodb::{
    bson::DateTime,
    options::{ClientOptions, IndexOptions},
    Client, Collection, IndexModel, error::Error,
};
use serde::Serialize;
use std::{str::FromStr, time::Duration};

use crate::{
    util::consts::NAME,
    models::{AvgTimeModel, DummyModel, JobStatus, PreviewJobModel, TransformJobModel},
};

pub struct MongoPersistenceBase {
    mongo_client: Client,
}

impl MongoPersistenceBase {
    pub async fn build(mongo_uri: &str) -> Result<Self, Error> {
        let options = ClientOptions::parse(&mongo_uri).await?;
        Ok(MongoPersistenceBase {
            mongo_client: Client::with_options(options)?
        })
    }

    pub fn get_mongo_client(&self) -> &Client {
        &self.mongo_client
    }

    pub async fn set_expire_after(&self, seconds: u64) -> Result<(), Error> {
        let jobs = self.get_jobs::<DummyModel>();

        let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
        let index = IndexModel::builder().keys(doc! {"created": 1}).options(options).build();

        jobs.create_index(index.clone(), None).await?;

        Ok(())
    }

    pub fn get_transformations(&self) -> Collection<TransformJobModel> {
        self.get_jobs()
    }

    pub fn get_previews(&self) -> Collection<PreviewJobModel> {
        self.get_jobs()
    }

    pub fn get_jobs<T>(&self) -> Collection<T> {
        self.mongo_client.database(&NAME).collection("jobs")
    }

    pub async fn set_ready<ResultType: Serialize>(&self, job_id: &str, results: ResultType) -> Result<(), &'static str> {
        let jobs = self.get_jobs::<DummyModel>();
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

    pub async fn set_error(&self, job_id: &str, err: &str) -> Result<(), &'static str> {
        let jobs = self.get_jobs::<DummyModel>();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.update_one(doc! {"_id": id}, doc! {"$set": {"status": JobStatus::Error as u32, "message": err, "finished": DateTime::now()}}, None).await {
                if result.modified_count > 0 {
                    return Ok(());
                }
            }
        }
        Err("Could not find job")
    }

    pub async fn jobs_health(&self) -> Result<Vec<AvgTimeModel>, &'static str> {
        let cursor = self.get_jobs::<DummyModel>()
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
}
