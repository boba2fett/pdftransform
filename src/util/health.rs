use std::{fs::read_dir, sync::Arc};

use libc::{sysconf, _SC_OPEN_MAX};

use crate::{models::MetricsDto, persistence::JobsBasePersistence};

pub async fn get_metrics(job_health_provider: Arc<dyn JobsBasePersistence>) -> Result<MetricsDto, &'static str> {
    let job_health = job_health_provider.jobs_health().await?;
    let file_limit = unsafe { sysconf(_SC_OPEN_MAX) } as usize;
    let paths = read_dir("/proc/self/fd").map_err(|_| "Could not determine open files")?;
    let open_files = paths.count();
    Ok(MetricsDto {
        jobs: job_health,
        file_handels: open_files,
        file_handel_limit: file_limit,
    })
}