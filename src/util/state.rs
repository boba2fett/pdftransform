use std::sync::Arc;

use crate::{files::FileStorage, persistence::{PreviewPersistence, TransformPersistence}};

pub struct ServiceCollection {
    // pub file_storage: Arc<dyn FileStorage + Sync + Send>,
    // pub preview_persistence: Arc<dyn PreviewPersistence + Sync + Send>,
    // pub transform_persistence: Arc<dyn TransformPersistence + Sync + Send>,
}