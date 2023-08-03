use std::{env, path::PathBuf};
use tokio::fs;
use tracing::warn;

use crate::util::random::generate_30_alphanumeric;

#[async_trait::async_trait]
pub trait TempJobFileProviderFactory {
    async fn build(job_id: &str) -> TempJobFileProvider;
}

#[derive(Debug)]
pub struct TempJobFileProvider {
    job_directory: PathBuf,
}

impl TempJobFileProvider {
    pub async fn build(job_id: &str) -> TempJobFileProvider {
        let dir = env::temp_dir().join(job_id);
        fs::create_dir_all(&dir).await.unwrap();
        TempJobFileProvider { job_directory: dir }
    }

    pub async fn clean_up(&self) {
        if let Err(err) = fs::remove_dir_all(&self.job_directory).await {
            warn!("Error occured, while deleting temp job files for {}: {}", &self.job_directory.to_str().unwrap_or("<none>"), &err)
        }
    }

    pub fn get_path(&self) -> PathBuf {
        self.job_directory.join(generate_30_alphanumeric())
    }

    pub fn get_one() -> PathBuf {
        env::temp_dir().join(generate_30_alphanumeric())
    }
}