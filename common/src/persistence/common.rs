use bson::{doc, oid::ObjectId, Bson};
use futures::StreamExt;
use mongodb::{
    bson::DateTime,
    error::Error,
    options::{ClientOptions, IndexOptions},
    Client, Collection, IndexModel,
};
use serde::Serialize;
use std::{str::FromStr, time::Duration};

use crate::{
    models::{AvgTimeModel, BaseJobDto, JobModel, DummyJobModel, JobStatus},
    util::consts::NAME,
};

trait Serializable: Serialize + Send + Sync {}

#[async_trait::async_trait]
pub trait JobsBasePersistence: Send + Sync {
    async fn jobs_health(&self) -> Result<Vec<AvgTimeModel>, &'static str>;
    async fn set_ready(&self, job_id: &str, results_bson: Bson) -> Result<(), &'static str>;
    async fn set_error(&self, job_id: &str, err: &str) -> Result<(), &'static str>;
    async fn _get_error_dto(&self, job_id: &str) -> Result<BaseJobDto, &'static str>;
    async fn _get_base_job_model(&self, job_id: &str) -> Result<JobModel<()>, &'static str>;
}

pub struct MongoPersistenceBase {
    mongo_client: Client,
}

impl MongoPersistenceBase {
    pub async fn build(mongo_uri: &str, expire_seconds: u64) -> Result<Self, &'static str> {
        let options = ClientOptions::parse(&mongo_uri).await.map_err(|_| "Cloud not create mongo client")?;
        let oneself = MongoPersistenceBase {
            mongo_client: Client::with_options(options).map_err(|_| "Cloud not create mongo client")?,
        };
        oneself.set_expire_after(expire_seconds).await.map_err(|_| "Cloud not set expire time")?;
        Ok(oneself)
    }

    pub fn get_mongo_client(&self) -> &Client {
        &self.mongo_client
    }

    async fn set_expire_after(&self, seconds: u64) -> Result<(), Error> {
        let jobs = self.get_jobs::<DummyJobModel>();

        let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
        let index = IndexModel::builder().keys(doc! {"created": 1}).options(options).build();

        jobs.create_index(index.clone(), None).await?;

        Ok(())
    }

    pub fn get_jobs<T>(&self) -> Collection<T> {
        self.mongo_client.database(&NAME).collection("jobs")
    }
}

#[async_trait::async_trait]
impl JobsBasePersistence for MongoPersistenceBase {
    async fn jobs_health(&self) -> Result<Vec<AvgTimeModel>, &'static str> {
        let cursor = self
            .get_jobs::<DummyJobModel>()
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

    async fn set_ready(&self, job_id: &str, results_bson: Bson) -> Result<(), &'static str> {
        let jobs = self.get_jobs::<DummyJobModel>();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs
                .update_one(doc! {"_id": id}, doc! {"$set": {"status": JobStatus::Finished as u32 ,"data.result": results_bson, "finished": DateTime::now()}}, None)
                .await
            {
                if result.modified_count > 0 {
                    return Ok(());
                }
            }
        }
        Err("Could not find job")
    }

    async fn set_error(&self, job_id: &str, err: &str) -> Result<(), &'static str> {
        let jobs = self.get_jobs::<DummyJobModel>();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.update_one(doc! {"_id": id}, doc! {"$set": {"status": JobStatus::Error as u32, "message": err, "finished": DateTime::now()}}, None).await {
                if result.modified_count > 0 {
                    return Ok(());
                }
            }
        }
        Err("Could not find job")
    }

    async fn _get_error_dto(&self, job_id: &str) -> Result<BaseJobDto, &'static str> {
        let job_model = self._get_base_job_model(&job_id).await?;
        let job_id = job_model.id.unwrap().to_string();
        Ok(BaseJobDto {
            message: job_model.message,
            status: job_model.status,
            callback_uri: job_model.callback_uri,
            created: job_model.created,
            id: job_id,
        })
    }

    async fn _get_base_job_model(&self, job_id: &str) -> Result<JobModel<()>, &'static str> {
        let jobs = self.get_jobs::<JobModel<()>>();
        if let Ok(id) = ObjectId::from_str(&job_id) {
            if let Ok(result) = jobs.find_one(Some(doc!("_id": id)), None).await {
                if let Some(job_model) = result {
                    return Ok(job_model);
                }
            }
        }
        Err("Could not find job")
    }
}
