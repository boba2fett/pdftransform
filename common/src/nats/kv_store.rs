use std::{sync::Arc, time::Duration};
use async_nats::jetstream::kv::{Store, Config};
use bytes::Bytes;

use crate::{models::ToIdJson, persistence::IJobPersistence};

use super::base::BaseJetStream;

pub struct KeyValueStoreService  {
    key_value: Store,
}

impl KeyValueStoreService {
    pub async fn build(base: Arc<BaseJetStream>, bucket: String, max_age: Duration) -> Result<Self, &'static str> {
        let key_value = base.jetstream.create_key_value(Config {
            bucket,
            max_age,
            ..Default::default()
        }).await.map_err(|_| "could not create key value store bucket")?;
        Ok(KeyValueStoreService {
            key_value
        })
    }
}

#[async_trait::async_trait]
impl IJobPersistence for KeyValueStoreService {
    async fn put(&self, job: &dyn ToIdJson) -> Result<(), &'static str> {
        let json = job.to_json()?;
        self.key_value.put(job.get_id(), json.into()).await.map_err(|_| "could not put job")?;
        Ok(())
    }
    async fn get(&self, job_id: &str) -> Result<Option<Bytes>, &'static str> {
        let stream = self.key_value.get(job_id).await.map_err(|_| "could not get job")?;
        Ok(stream)
    }
}