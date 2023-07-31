use std::{sync::Arc, time::Duration};

use crate::{persistence::IJobPersistence, nats::{base::BaseJetStream, kv_store::KeyValueStoreService}};

pub struct NatsBaseServiceCollection {
    pub base_jetstream: Arc<BaseJetStream>,
    pub job_persistence: Arc<dyn IJobPersistence>,
}

impl NatsBaseServiceCollection {
    pub async fn build(nats_uri: &str, bucket: String, max_age: Duration) -> Result<Arc<Self>, &'static str> {
        let base_jetstream = Arc::new(BaseJetStream::build(nats_uri).await?);
        Ok(Arc::new(NatsBaseServiceCollection{
            job_persistence: Arc::new(KeyValueStoreService::build(base_jetstream.clone(), bucket, max_age).await?),
            base_jetstream: base_jetstream
        }))
    }
}
