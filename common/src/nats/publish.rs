use std::sync::Arc;

use crate::models::{IdModel, RefIdModel};

use super::base::BaseJetstream;

#[async_trait::async_trait]
pub trait IPublishService: Sync + Send {
    async fn publish<'a>(&self, id: &'a str) -> Result<(), &'static str>;
}

pub struct PublishService  {
    base: Arc<BaseJetstream>,
    queue: String,
}

impl PublishService {
    pub async fn build(base: Arc<BaseJetstream>, queue: String) -> Result<Self, &'static str> {
        _ = base.jetstream.get_or_create_stream(async_nats::jetstream::stream::Config {
            name: queue.clone(),
            max_messages: 10_000,
            ..Default::default()
        }).await.map_err(|_| "could not get or create stream")?;
        Ok(PublishService {
            base,
            queue,
        })
    }
}

#[async_trait::async_trait]
impl IPublishService for PublishService {
    async fn publish<'a>(&self, id: &'a str) -> Result<(), &'static str> {
        let content = RefIdModel {
            id,
        };
        let json = serde_json::to_string(&content).map_err(|_| "not valid json")?;
        self.base.jetstream.publish(self.queue.clone(), json.into()).await.map_err(|_| "not published")?;
        Ok(())
    }
}