use std::{sync::Arc, time::Duration};

use common::{nats::{subscribe::{ISubscribeService, SubscribeService}}, convert::BaseConvertService, download::DownloadService, persistence::IJobPersistence, util::state::{NatsBaseSettings, S3BaseSettings, NatsBaseServiceCollection, StorageBaseServiceCollection}};
use pdfium_render::prelude::Pdfium;

use crate::{preview::PreviewService, convert::ConvertService};

pub struct ServiceCollection {
    pub job_persistence: Arc<dyn IJobPersistence>,
    pub subscribe_service: Arc<dyn ISubscribeService>,
}

impl ServiceCollection {
    pub async fn build(settings: NatsBaseSettings<'_>, stream: String, parallelism: usize, pdfium: Pdfium, s3_settings: S3BaseSettings, consumer: String, max_deliver: i64, consumer_ack_wait: Duration) -> Result<Self, &'static str> {
        let base = StorageBaseServiceCollection::build(&settings, s3_settings).await?;
        let download_service = Arc::new(DownloadService { parallelism });
        let preview = Arc::new(PreviewService {
            storage: base.file_storage.clone(),
            pdfium,
        });
        let worker = ConvertService {
            base: Arc::new(BaseConvertService {
                job_persistence: base.job_persistence.clone(),
            }),
            preview_service: preview,
            download_service: download_service,
        };
        Ok(ServiceCollection{
            subscribe_service: Arc::new(SubscribeService::build(base.base_jetstream.clone(), stream, worker, consumer, max_deliver, consumer_ack_wait).await?),
            job_persistence: base.job_persistence.clone(),
        })
    }
}
