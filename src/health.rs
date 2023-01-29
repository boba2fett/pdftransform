use std::fs::read_dir;

use libc::{sysconf, _SC_OPEN_MAX};

use crate::{models::HealthDto, persistence::jobs_health};

pub async fn get_health() -> Result<HealthDto, &'static str> {
    let job_health = jobs_health().await?;
    let file_limit = unsafe { sysconf(_SC_OPEN_MAX) } as usize;
    let paths = read_dir("/proc/self/fd").map_err(|_| "Could not determine open files")?;
    let open_files = paths.count();
    Ok(HealthDto {
        jobs: job_health,
        file_handels: open_files,
        file_handel_limit: file_limit,
    })
}