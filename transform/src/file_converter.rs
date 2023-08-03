use std::sync::Arc;

use common::{nats::publish::IPublishService, download::DownloadedSourceFile};

#[async_trait::async_trait]
pub trait IFileConverterService: Send + Sync {
    async fn convert<'a>(&self, job_id: &str, source_file: &DownloadedSourceFile) -> Result<(), &'static str>;
}

pub struct FileConvertService {
    pub convert_publish_service: Arc<dyn IPublishService>,
}

#[async_trait::async_trait]
impl IFileConverterService for FileConvertService {
    async fn convert<'a>(&self, job_id: &str, source_file: &DownloadedSourceFile) -> Result<(), &'static str> {
        self.convert_publish_service.publish()
    }
}