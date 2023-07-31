use std::sync::Arc;

use crate::models::RefIdModel;

use super::base::BaseJetStream;

#[async_trait::async_trait]
pub trait IPublishService: Sync + Send {
    async fn publish<'a>(&self, id: &'a str) -> Result<(), &'static str>;
}

pub struct PublishService  {
    base: Arc<BaseJetStream>,
    stream: String,
}

impl PublishService {
    pub fn new(base: Arc<BaseJetStream>, stream: String) -> Self {
        PublishService {
            base,
            stream,
        }
    }
}

#[async_trait::async_trait]
impl IPublishService for PublishService {
    async fn publish<'a>(&self, id: &'a str) -> Result<(), &'static str> {
        let content = RefIdModel {
            id,
        };
        let json = serde_json::to_string(&content).map_err(|_| "not valid json")?;
        self.base.jetstream.publish(self.stream.clone(), json.into()).await.map_err(|_| "not published")?;
        Ok(())
    }
}