use std::sync::Arc;

use common::models::{ConversionRequestRef, SourceFile};
use common::nats::publish::IPublishService;

#[async_trait::async_trait]
pub trait IFileConverterService: Send + Sync {
    async fn convert<'a>(&self, job_id: &str, source_file: &SourceFile) -> Result<(), &'static str>;
}

pub struct FileConvertService {
    pub convert_publish_service: Arc<dyn IPublishService>,
    pub response_subject: String,
}

#[async_trait::async_trait]
impl IFileConverterService for FileConvertService {
    async fn convert<'a>(&self, job_id: &str, source_file: &SourceFile) -> Result<(), &'static str> {
        let subject = format!("{}.{}", self.convert_publish_service.get_stream(), &source_file.content_type.as_ref().unwrap());
        self.convert_publish_service.publish_conversion(subject, &ConversionRequestRef
        {
            job_id: &job_id,
            source_uri: &source_file.uri,
            response_subject: &self.response_subject,
        }).await?;
        Ok(())
    }
}