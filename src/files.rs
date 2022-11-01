use std::{path::PathBuf, env};
use rocket::fs::FileName;
use tokio::fs;

pub async fn get_job_files(job_id: &str) -> JobFileProvider {
    let file_name = FileName::new(job_id);
    let dir = env::temp_dir().join(file_name.as_str().unwrap());
    fs::create_dir_all(&dir).await;
    JobFileProvider {job_directory: dir}
}

pub struct JobFileProvider {
    job_directory: PathBuf,
}

impl JobFileProvider {
    pub fn get_path(&self, id: &str) -> PathBuf
    {
        let file_name = FileName::new(id);
        self.job_directory.join(file_name.as_str().unwrap())
    }
}