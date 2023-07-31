use bytes::Bytes;

use crate::models::ToIdJson;

#[async_trait::async_trait]
pub trait IJobPersistence: Send + Sync {
    async fn get(&self, job_id: &str) -> Result<Option<Bytes>, &'static str>;
    async fn put(&self, job: &dyn ToIdJson) -> Result<(), &'static str>;
}

#[async_trait::async_trait]
pub trait IFileStorage: Send + Sync {
    async fn store_result_file(&self, key: &str, file_name: &str, mime_type: Option<&str>, source: Vec<u8>) -> Result<String, &'static str>;
}