use std::sync::Arc;

use crate::persistence::{JobsBasePersistence, files::{IFileStorage, GridFSFileStorage}, PreviewPersistence, TransformPersistence, MongoPersistenceBase, MongoPreviewPersistence, MongoTransformPersistence};

pub type FileStorageState = Arc<dyn IFileStorage>;
pub type JobsBasePersistenceState = Arc<dyn JobsBasePersistence>;
pub type PreviewPersistenceState = Arc<dyn PreviewPersistence>;
pub type TransformPersistenceState = Arc<dyn TransformPersistence>;

pub struct PersistenceServiceCollection {
    pub file_storage: FileStorageState,
    pub jobs_base_peristence: JobsBasePersistenceState,
    pub preview_persistence: PreviewPersistenceState,
    pub transform_persistence: TransformPersistenceState,
}

impl PersistenceServiceCollection {
    pub async fn build(mongo_uri: &str, expire_seconds: u64) -> Result<Self, &'static str> {
        let jobs_base_peristence = Arc::new(MongoPersistenceBase::build(mongo_uri, expire_seconds).await?);
        let file_storage = Arc::new(GridFSFileStorage::build(jobs_base_peristence.clone(), expire_seconds).await?);
        let preview_persistence = Arc::new(MongoPreviewPersistence { base: jobs_base_peristence.clone() });
        let transform_persistence = Arc::new(MongoTransformPersistence { base: jobs_base_peristence.clone() });

        Ok(PersistenceServiceCollection {
            file_storage,
            jobs_base_peristence,
            preview_persistence,
            transform_persistence,
        })
    }
}