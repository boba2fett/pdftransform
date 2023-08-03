use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{stream::{Stream, Source}, AckKind};
use futures::StreamExt;
use tracing::{error, info};

use crate::{models::{IdModel, MessageDLQModel}, util::serialize::base64};

use super::base::BaseJetStream;

#[async_trait::async_trait]
pub trait IDLQSubscribeService: Sync + Send {
    async fn subscribe(&self) -> Result<(), &'static str>;
}

#[async_trait::async_trait]
pub trait IDLQWorkerService: Sync + Send {
    async fn work(&self, id: &str) -> ();
}

pub struct DLQSubscribeService<DLQWorker>  {
    dlq_stream: Stream,
    mirror_stream: Stream,
    worker: DLQWorker,
    consumer: String,
}

impl<Worker> DLQSubscribeService<Worker> {
    pub async fn build(base: Arc<BaseJetStream>, stream: String, worker: Worker, consumer: String, max_age_mirror: Duration) -> Result<Self, &'static str> {
        let mirror_stream = base.jetstream.get_or_create_stream(async_nats::jetstream::stream::Config {
            name: format!("{}-mirror", stream),
            max_messages: 10_000,
            mirror: Some(Source {
                name: stream.clone(),
                ..Default::default()
            }),
            max_age: max_age_mirror,
            ..Default::default()
        }).await.map_err(|_| "could not get or create stream")?;

        let dlq_stream = base.jetstream.get_or_create_stream(async_nats::jetstream::stream::Config {
            name: format!("{}-dlq", stream),
            subjects: vec![
                format!("$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.{}.{}", stream, consumer),
                format!("$JS.EVENT.ADVISORY.CONSUMER.MSG_TERMINATED.{}.{}", stream, consumer),
            ],
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            max_messages: 10_000,
            ..Default::default()
        }).await.map_err(|_| "could not get or create stream")?;
        Ok(DLQSubscribeService {
            dlq_stream: dlq_stream,
            mirror_stream: mirror_stream,
            worker,
            consumer,
        })
    }
}

#[async_trait::async_trait]
impl<Worker> IDLQSubscribeService for DLQSubscribeService<Worker> where Worker: IDLQWorkerService {
    async fn subscribe(&self) -> Result<(), &'static str> {
        let consumer = self.dlq_stream.get_or_create_consumer(&format!("{}-dlq", self.consumer), async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(self.consumer.to_string()),
            ..Default::default()
        }).await.map_err(|_| "could not get or create consumer")?;
        let mut messages = consumer.messages().await.map_err(|_| "could not get messages")?;
        while let Some(Ok(msg)) = messages.next().await {
            info!("procressing next message");
            let work: Result<(), &'static str> = {
                let dlq_model: MessageDLQModel = serde_json::from_slice(&msg.payload).map_err(|_| "not valid json from dlq")?;
                msg.ack_with(AckKind::Progress).await.map_err(|_| "could not progress")?;
                let orig_message = self.mirror_stream.get_raw_message(dlq_model.stream_seq).await.map_err(|_| "could not get message")?;
                let payload = String::from_utf8(base64::deserialize_str(orig_message.payload)?).map_err(|_| "error in base64 json")?;
                let content: IdModel = serde_json::from_str(&payload).map_err(|_| "not valid json")?;
                
                info!("## start: {}", &content.id);
                self.worker.work(&content.id).await;
                info!("## end: {}", &content.id);
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
