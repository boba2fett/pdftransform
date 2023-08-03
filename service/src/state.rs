use std::sync::Arc;

use common::{nats::publish::{PublishService, IPublishService}, util::state::{NatsBaseServiceCollection, NatsBaseSettings}, persistence::IJobPersistence};

pub type Services = Arc<ServiceCollection>;

pub struct ServiceCollection {
    pub transform_publish_service: Arc<dyn IPublishService>,
    pub preview_publish_service: Arc<dyn IPublishService>,
    pub job_persistence: Arc<dyn IJobPersistence>,
}

impl ServiceCollection {
    pub async fn build(settings: NatsBaseSettings<'_>, stream: String) -> Result<Arc<Self>, &'static str> {
        let base = NatsBaseServiceCollection::build(&settings).await?;
        Ok(Arc::new(ServiceCollection{
            transform_publish_service: Arc::new(PublishService::new(base.base_jetstream.clone(), format!("{}.transform", &stream))),
            preview_publish_service: Arc::new(PublishService::new(base.base_jetstream.clone(), format!("{}.preview", &stream))),
            job_persistence: base.job_persistence.clone(),
        }))
    }
}
