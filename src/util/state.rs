use pdfium_render::prelude::Pdfium;
use std::sync::Arc;

use crate::{
    convert::{
        preview::pdfium::{PdfiumPreviewService, PreviewService},
        transform::PdfiumLibreTransformService,
        ConvertService, ConvertServiceImpl,
    },
    download::DownloadServiceImpl,
    persistence::{
        files::{FileStorage, GridFSFileStorage},
        JobsBasePersistence, MongoPersistenceBase, MongoPreviewPersistence, MongoTransformPersistence,
    },
    persistence::{PreviewPersistence, TransformPersistence},
};

pub type FileStorageState = Arc<dyn FileStorage + Sync + Send>;
pub type PreviewPersistenceState = Arc<dyn PreviewPersistence + Sync + Send>;
pub type TransformPersistenceState = Arc<dyn TransformPersistence + Sync + Send>;
pub type ConvertServiceState = Arc<dyn ConvertService + Sync + Send>;
pub type PreviewServiceState = Arc<dyn PreviewService + Sync + Send>;
pub type JobsBasePersistenceState = Arc<dyn JobsBasePersistence + Sync + Send>;

pub struct ServiceCollection {
    pub file_storage: FileStorageState,
    pub preview_persistence: PreviewPersistenceState,
    pub transform_persistence: TransformPersistenceState,
    pub convert_service: ConvertServiceState,
    pub preview_service: PreviewServiceState,
    pub jobs_base_peristence: JobsBasePersistenceState,
}

impl ServiceCollection {
    pub async fn build(mongo_uri: &str, expire_seconds: u64, parallelism: usize, pdfium: Pdfium) -> Result<Self, &'static str> {
        let jobs_base_peristence = Arc::new(MongoPersistenceBase::build(mongo_uri, expire_seconds).await?);
        let file_storage = Arc::new(GridFSFileStorage::build(jobs_base_peristence.clone(), expire_seconds).await?);
        let preview_persistence = Arc::new(MongoPreviewPersistence { base: jobs_base_peristence.clone() });
        let transform_persistence = Arc::new(MongoTransformPersistence { base: jobs_base_peristence.clone() });
        let transform_persistence = Arc::new(MongoTransformPersistence { base: jobs_base_peristence.clone() });
        let pdfium = Arc::new(pdfium);
        let preview_service = Arc::new(PdfiumPreviewService {
            storage: file_storage.clone(),
            pdfium: pdfium.clone(),
        });
        let transform_service = Arc::new(PdfiumLibreTransformService {
            storage: file_storage.clone(),
            pdfium: pdfium.clone(),
        });
        let download_service = Arc::new(DownloadServiceImpl { parallelism });
        let convert_service = Arc::new(ConvertServiceImpl {
            base_persistence: jobs_base_peristence.clone(),
            transform_persistence: transform_persistence.clone(),
            preview_persistence: preview_persistence.clone(),
            preview_service: preview_service.clone(),
            transform_service: transform_service.clone(),
            download_service: download_service.clone(),
        });
        Ok(ServiceCollection {
            jobs_base_peristence,
            file_storage,
            preview_persistence,
            transform_persistence,
            preview_service,
            convert_service,
        })
    }
}
