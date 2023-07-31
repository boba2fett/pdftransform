use std::{sync::Arc, time::Duration};

use axum::extract::State;
use common::{nats::{publish::{PublishService, IPublishService}}, util::state::NatsBaseServiceCollection, persistence::IJobPersistence};

pub type Services = State<Arc<ServiceCollection>>;

pub struct ServiceCollection {
    pub publish_service: Arc<dyn IPublishService>,
    pub job_persistence: Arc<dyn IJobPersistence>,
}

impl ServiceCollection {
    pub async fn build(nats_uri: &str, stream: String, bucket: String, max_age: Duration) -> Result<Arc<Self>, &'static str> {
        let base = NatsBaseServiceCollection::build(nats_uri, bucket, max_age).await?;
        Ok(Arc::new(ServiceCollection{
            publish_service: Arc::new(PublishService::new(base.base_jetstream.clone(), stream)),
            job_persistence: base.job_persistence.clone(),
        }))
    }
}
