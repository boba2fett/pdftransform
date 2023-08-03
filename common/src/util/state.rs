use std::{sync::Arc, time::Duration};

use crate::{persistence::{IJobPersistence, IFileStorage, s3::S3FileStorage}, nats::{base::BaseJetStream, kv_store::KeyValueStoreService}};

pub struct NatsBaseSettings<'a> {
    pub nats_uri: &'a str,
    pub bucket: String,
    pub max_age: Duration,
}

pub struct NatsBaseServiceCollection {
    pub base_jetstream: Arc<BaseJetStream>,
    pub job_persistence: Arc<dyn IJobPersistence>,
}

impl NatsBaseServiceCollection {
    pub async fn build(nats_settings: &NatsBaseSettings<'_>) -> Result<Arc<Self>, &'static str> {
        let base_jetstream = Arc::new(BaseJetStream::build(nats_settings.nats_uri).await?);
        Ok(Arc::new(NatsBaseServiceCollection{
            job_persistence: Arc::new(KeyValueStoreService::build(base_jetstream.clone(), nats_settings.bucket.clone(), nats_settings.max_age).await?),
            base_jetstream: base_jetstream
        }))
    }
}

pub struct S3BaseSettings {
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    pub expire_seconds: u32,
}

pub struct StorageBaseServiceCollection {
    pub base_jetstream: Arc<BaseJetStream>,
    pub job_persistence: Arc<dyn IJobPersistence>,
    pub file_storage: Arc<dyn IFileStorage>,
}

impl StorageBaseServiceCollection {
    pub async fn build(nats_settings: &NatsBaseSettings<'_>, s3_settings: S3BaseSettings) -> Result<Arc<Self>, &'static str> {
        let nats_base = NatsBaseServiceCollection::build(nats_settings).await?;
        Ok(Arc::new(StorageBaseServiceCollection {
            base_jetstream: nats_base.base_jetstream.clone(),
            job_persistence: nats_base.job_persistence.clone(),
            file_storage: Arc::new(S3FileStorage::build(s3_settings.endpoint, s3_settings.region, s3_settings.access_key_id, s3_settings.secret_access_key, s3_settings.bucket, s3_settings.expire_seconds).await?),
        }))
    }
}
