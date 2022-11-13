use std::{path::PathBuf, env, str::FromStr};
use bson::oid::ObjectId;
use futures::{AsyncRead, Stream};
use rocket::fs::FileName;
use tokio::fs;
use mongodb_gridfs::{options::GridFSBucketOptions, GridFSBucket};

use crate::{persistence::{get_job_model, generate_30_alphanumeric}, consts};

pub async fn get_job_result_file(db_client: &mongodb::Client, job_id: &str, token: &str, file_id: &str) -> Result<impl Stream<Item = Vec<u8>>, &'static str> {
    _ = get_job_model(db_client, job_id, token).await?;
    let db = db_client.database(consts::NAME);
    let bucket = GridFSBucket::new(db, Some(GridFSBucketOptions::default()));
    if let Ok(id) = ObjectId::from_str(file_id) {
        let cursor = bucket.open_download_stream(id).await.map_err(|_| "Could not find file.")?;
        return Ok(cursor)
    }
    Err("Could not find file.")
}

pub async fn store_job_result_file(db_client: &mongodb::Client, file_name: &str, source: impl AsyncRead + Unpin) -> Result<String, &'static str> {
    let db = db_client.database(consts::NAME);
    let mut bucket = GridFSBucket::new(db, Some(GridFSBucketOptions::default()));
    bucket.upload_from_stream(file_name, source, None).await.map_err(|_| "Could not store result.").map(|id| id.to_string())
}

#[derive(Debug)]
pub struct TempJobFileProvider {
    job_directory: PathBuf,
}

impl TempJobFileProvider {
    pub async fn build(job_id: &str) -> TempJobFileProvider {
        let file_name = FileName::new(job_id);
        let dir = env::temp_dir().join(file_name.as_str().unwrap());
        fs::create_dir_all(&dir).await.unwrap();
        TempJobFileProvider {job_directory: dir}
    }

    pub fn get_path(&self) -> PathBuf
    {
        self.job_directory.join(generate_30_alphanumeric())
    }
}
