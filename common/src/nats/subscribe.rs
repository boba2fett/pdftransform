use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{stream::{Stream, RetentionPolicy}, AckKind};
use futures::StreamExt;
use tracing::{error, info};

use crate::models::IdModel;

use super::base::BaseJetStream;

#[async_trait::async_trait]
pub trait ISubscribeService: Sync + Send {
    async fn subscribe(&self) -> Result<(), &'static str>;
}

#[async_trait::async_trait]
pub trait IWorkerService: Sync + Send {
    async fn work(&self, id: &str) -> Result<(), WorkError>;
}

pub struct SubscribeService<Worker>  {
    stream: Stream,
    worker: Worker,
    consumer: String,
    max_deliver: i64,
    ack_wait: Duration,
}

impl<Worker> SubscribeService<Worker> {
    pub async fn build(base: Arc<BaseJetStream>, stream: String, worker: Worker, consumer: String, max_deliver: i64, ack_wait: Duration) -> Result<Self, &'static str> {
        let stream = base.jetstream.get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream.clone(),
            subjects: vec![format!("{}.*", stream)],
            max_messages: 10_000,
            retention: RetentionPolicy::Interest,
            ..Default::default()
        }).await.map_err(|_| "could not get or create stream")?;
        Ok(SubscribeService {
            stream,
            worker,
            consumer,
            max_deliver,
            ack_wait,
        })
    }
}

#[derive(Debug, Clone)]
pub enum WorkError {
    NoRetry,
    Retry,
}

#[async_trait::async_trait]
impl<Worker> ISubscribeService for SubscribeService<Worker> where Worker: IWorkerService {
    async fn subscribe(&self) -> Result<(), &'static str> {
        let consumer = self.stream.get_or_create_consumer(&self.consumer, async_nats::jetstream::consumer::pull::Config {
            name: Some(self.consumer.clone()),
            filter_subject: format!("newJob.{}", self.consumer),
            durable_name: Some(self.consumer.clone()),
            max_deliver: self.max_deliver,
            ack_wait: self.ack_wait,
            ..Default::default()
        }).await.map_err(|_| "could not get or create consumer")?;
        let mut messages = consumer.messages().await.map_err(|_| "could not get messages")?;
        while let Some(Ok(msg)) = messages.next().await {
            info!("procressing next message");
            let work: Result<(), &'static str> = {
                let content: IdModel = serde_json::from_slice(&msg.payload).map_err(|_| "not valid json")?;
                msg.ack_with(AckKind::Progress).await.map_err(|_| "could not progress")?;
                info!("## start: {}", &content.id);
                let result = self.worker.work(&content.id).await;
                info!("## end: {} with {:?}", &content.id, &result);
                match result {
                    Ok(()) => msg.ack().await.map_err(|_| "could not ack")?,
                    Err(WorkError::NoRetry) => msg.ack_with(AckKind::Term).await.map_err(|_| "could not term")?,
                    Err(WorkError::Retry) => msg.ack_with(AckKind::Nak(None)).await.map_err(|_| "could not nak")?,
                };
                Ok(())
            };
            if let Err(err) = work {
                error!("Error occured processing message {err}");
            }
        }
        Ok(())
    }
}
