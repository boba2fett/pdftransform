use std::path::PathBuf;

use rocket::fs::FileName;
use tempfile::Builder;

pub fn get_job_files(job_id: &str) -> JobFileProvider {
    let file_name = FileName::new(job_id);
    JobFileProvider {directory: Builder::new().prefix(file_name.as_str().unwrap()).tempdir().unwrap()}
}

pub struct JobFileProvider {
    pub directory: tempfile::TempDir,
}

impl JobFileProvider {
    pub fn get_path(&self, id: &str) -> PathBuf
    {
        let file_name = FileName::new(id);
        dbg!(self.directory.path().join(file_name.as_str().unwrap()))
    }
}