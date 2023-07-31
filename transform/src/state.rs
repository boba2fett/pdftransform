use std::sync::Arc;

use common::{nats::{subscribe::{ISubscribeService, SubscribeService}, base::BaseJetstream}, state::PersistenceServiceCollection, convert::BaseConvertService, download::DownloadService};
use pdfium_render::prelude::Pdfium;

use crate::{convert::ConvertService, transform::TransformService};

pub struct Services {
    pub subscriber: Arc<dyn ISubscribeService>,
}

impl Services {
    pub async fn build(mongo_uri: &str, expire_seconds: u64, parallelism: usize, nats_uri: &str, pdfium: Pdfium) -> Result<Self, &'static str> {
        let persistence = Arc::new(PersistenceServiceCollection::build(mongo_uri, expire_seconds).await?);
        let download_service = Arc::new(DownloadService { parallelism });
        let transform = Arc::new(TransformService {
            storage: persistence.file_storage.clone(),
            pdfium,
        });
        let worker = ConvertService {
            base: Arc::new(BaseConvertService {
                job_persistence: persistence.jobs_base_peristence.clone(),
                preview_persistence: persistence.preview_persistence.clone(),
                transform_persistence: persistence.transform_persistence.clone(),
            }),
            transform_service: transform,
            download_service: download_service,
        };
        let base_nats = Arc::new(BaseJetstream::build(nats_uri).await?);
        let subscriber = Arc::new(SubscribeService::build(base_nats.clone(), "transform".to_string(), worker).await?);
        Ok(Services {
            subscriber
        })
    }
}