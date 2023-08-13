use std::{sync::Arc, time::Duration};

use common::{nats::{subscribe::{ISubscribeService, SubscribeService}, publish::PublishService}, convert::BaseConvertService, download::DownloadService, persistence::IJobPersistence, util::state::{NatsBaseSettings, S3BaseSettings, StorageBaseServiceCollection}};
use pdfium_render::prelude::Pdfium;

use crate::{convert::ConvertService, transform::TransformService, file_converter::FileConvertService};

pub struct ServiceCollection {
    pub job_persistence: Arc<dyn IJobPersistence>,
    pub subscribe_service: Arc<dyn ISubscribeService>,
}

impl ServiceCollection {
    pub async fn build(settings: NatsBaseSettings<'_>, stream: String, subjects: Vec<String>, parallelism: usize, pdfium: Pdfium, s3_settings: S3BaseSettings, consumer: String, filter: Vec<String>, max_deliver: i64, consumer_ack_wait: Duration, convert_stream: String, aggregate_stream: String) -> Result<Self, &'static str> {
        let base = StorageBaseServiceCollection::build(&settings, s3_settings).await?;
        let download_service = Arc::new(DownloadService { parallelism });
        let convert_publish_service = Arc::new(PublishService::new(base.base_jetstream.clone(), format!("{}", &convert_stream)));
        let file_converter = Arc::new(FileConvertService {
            response_subject: aggregate_stream,
            convert_publish_service: convert_publish_service,
        });
        let transform = Arc::new(TransformService {
            storage: base.file_storage.clone(),
            pdfium,
        });
        let worker = ConvertService {
            base: Arc::new(BaseConvertService {
                job_persistence: base.job_persistence.clone(),
            }),
            transform_service: transform,
            download_service: download_service,
            file_converter_service: file_converter,
        };
        Ok(ServiceCollection{
            subscribe_service: Arc::new(SubscribeService::build(base.base_jetstream.clone(), stream, subjects, worker, consumer, filter, max_deliver, consumer_ack_wait).await?),
            job_persistence: base.job_persistence.clone(),
        })
    }
}
