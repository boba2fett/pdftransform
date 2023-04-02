use std::sync::Arc;

use crate::{models::MetricsDto, persistence::JobsBasePersistence};

pub async fn get_metrics(job_health_provider: Arc<dyn JobsBasePersistence>) -> Result<MetricsDto, &'static str> {
    Ok(MetricsDto {
        jobs: job_health_provider.jobs_health().await?,
    })
}
