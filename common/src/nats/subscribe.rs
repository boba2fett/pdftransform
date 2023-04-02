use std::{sync::Arc};

use async_nats::{jetstream::{stream::Stream, AckKind}};
use futures::StreamExt;
use tracing::error;

use crate::models::IdModel;

use super::base::BaseJetstream;

#[async_trait::async_trait]
pub trait ISubscribeService: Sync + Send {
    async fn subscribe(&self) -> Result<(), &'static str>;
}

#[async_trait::async_trait]
pub trait Worker: Sync + Send {
    async fn work(&self, id: String) -> Result<(), &'static str>;
}

pub struct SubscribeService  {
    stream: Stream,
    worker: Arc<dyn Worker>,
}

impl SubscribeService {
    pub async fn build(base: Arc<BaseJetstream>, queue: String, worker: Arc<dyn Worker>) -> Result<Self, &'static str> {
        let stream = base.jetstream.get_or_create_stream(async_nats::jetstream::stream::Config {
            name: queue.clone(),
            max_messages: 10_000,
            ..Default::default()
        }).await.map_err(|_| "could not get or create stream")?;
        Ok(SubscribeService {
            stream,
            worker,
        })
    }
}

#[async_trait::async_trait]
impl ISubscribeService for SubscribeService {
    async fn subscribe(&self) -> Result<(), &'static str> {
        let consumer = self.stream.get_or_create_consumer("pull", async_nats::jetstream::consumer::pull::Config {
            durable_name: Some("pull".to_string()),
            ..Default::default()
        }).await.map_err(|_| "could not get or create consumer")?;
        let mut messages = consumer.messages().await.map_err(|_| "could not get messages")?;
        while let Some(Ok(msg)) = messages.next().await {
            let work: Result<(), &'static str> = {
                let content: IdModel = serde_json::from_slice(&msg.payload).map_err(|_| "not valid json")?;
                msg.ack_with(AckKind::Progress).await.map_err(|_| "could not progress")?;
                self.worker.work(content.id).await?;
                msg.ack().await.map_err(|_| "could not ack")?;
                Ok(())
            };
            if let Err(err) = work {
                error!("Error occured processing message {err}");
            }
        }
        Ok(())
    }
}
