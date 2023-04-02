use std::sync::Arc;

use common::{state::PersistenceServiceCollection, nats::{publish::{IPublishService, PublishService}, base::BaseJetstream}};

pub type Services = Arc<ServiceCollection>;
pub type PublishServiceState = Arc<dyn IPublishService + Sync + Send>;

pub struct ServiceCollection {
    pub persistence: Arc<PersistenceServiceCollection>,
    pub preview_starter: PublishServiceState,
    pub transform_starter: PublishServiceState,
}

impl ServiceCollection {
    pub async fn build(mongo_uri: &str, expire_seconds: u64, nats_uri: &str) -> Result<Self, &'static str> {
        let persistence = Arc::new(PersistenceServiceCollection::build(mongo_uri, expire_seconds).await?);
        let base_nats = Arc::new(BaseJetstream::build(nats_uri).await?);
        let preview_starter = Arc::new(PublishService::build(base_nats.clone(), "preview".to_string()).await?);
        let transform_starter = Arc::new(PublishService::build(base_nats.clone(), "transform".to_string()).await?);
        Ok(ServiceCollection {
            persistence,
            preview_starter,
            transform_starter,
        })
    }
}
