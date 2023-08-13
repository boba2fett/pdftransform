use std::sync::Arc;

use crate::models::{RefIdModel, ConversionRequestRef};

use super::base::BaseJetStream;

#[async_trait::async_trait]
pub trait IPublishService: Sync + Send {
    async fn publish_job<'a>(&self, id: &'a str) -> Result<(), &'static str>;
    async fn publish_conversion<'a>(&self, stream: String, content: &'a ConversionRequestRef) -> Result<(), &'static str>;
    fn get_stream(&self) -> &str;
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
    async fn publish_job<'a>(&self, id: &'a str) -> Result<(), &'static str> {
        let content = RefIdModel {
            id,
        };
        let json = serde_json::to_string(&content).map_err(|_| "not valid json")?;
        self.base.jetstream.publish(self.stream.clone(), json.into()).await.map_err(|_| "not published")?;
        Ok(())
    }

    async fn publish_conversion<'a>(&self, stream: String, content: &'a ConversionRequestRef) -> Result<(), &'static str> {
        let json = serde_json::to_string(&content).map_err(|_| "not valid json")?;
        self.base.jetstream.publish(stream, json.into()).await.map_err(|_| "not published")?;
        Ok(())
    }

    fn get_stream(&self) -> &str {
        &self.stream
    }
}