use std::{path::PathBuf, env, str::FromStr, time::Duration};
use bson::{oid::ObjectId, doc, DateTime};
use futures::{AsyncRead, Stream};
use kv_log_macro::warn;
use mongodb::{options::{IndexOptions}, IndexModel, error::Error};
use rocket::fs::FileName;
use tokio::fs;
use mongodb_gridfs::{options::GridFSBucketOptions, GridFSBucket};

use crate::{persistence::{get_job_model, generate_30_alphanumeric, get_preview_model}, consts::{self}};

const FILES_COLLECTION: &str = "fs.files";
const CHUNKS_COLLECTION: &str = "fs.chunks";

fn get_bucket(client: &mongodb::Client) -> GridFSBucket {
    let db = client.database(consts::NAME);
    GridFSBucket::new(db, Some(GridFSBucketOptions::default()))
}

pub async fn get_job_result_file(client: &mongodb::Client, job_id: &str, token: &str, file_id: &str) -> Result<impl Stream<Item = Vec<u8>>, &'static str> {
    _ = get_job_model(client, job_id, token).await?;
    _get_result_file(client, file_id).await
}

pub async fn get_preview_result_file(client: &mongodb::Client, preview_id: &str, token: &str, file_id: &str) -> Result<impl Stream<Item = Vec<u8>>, &'static str> {
    _ = get_preview_model(client, preview_id, token).await?;
    _get_result_file(client, file_id).await
}

async fn _get_result_file(client: &mongodb::Client, file_id: &str) -> Result<impl Stream<Item = Vec<u8>>, &'static str> {
    let bucket = get_bucket(client);
    if let Ok(id) = ObjectId::from_str(file_id) {
        let cursor = bucket.open_download_stream(id).await.map_err(|_| "Could not find file.")?;
        return Ok(cursor)
    }
    Err("Could not find file.")
}

pub async fn store_result_file(client: &mongodb::Client, file_name: &str, source: impl AsyncRead + Unpin) -> Result<String, &'static str> {
    let mut bucket = get_bucket(client);
    let file_id = bucket.upload_from_stream(file_name, source, None).await.map_err(|_| "Could not store result.").map(|id| id.to_string())?;
    let chunks = client.database(consts::NAME).collection::<()>(CHUNKS_COLLECTION);
    let result = chunks.update_many(doc!{ "files_id": ObjectId::from_str(&file_id).unwrap() }, doc!{"$set": {"uploadDate": DateTime::now()}} , None).await;
    if result.is_err() {
        warn!("Could not set uploadDate for chunks of {}.", &file_id, {fileId: &file_id});
    }
    Ok(file_id)
}

pub async fn set_expire_after(client: &mongodb::Client, seconds: u64) -> Result<(), Error> {
    let files = client.database(consts::NAME).collection::<()>(FILES_COLLECTION);
    let chunks = client.database(consts::NAME).collection::<()>(CHUNKS_COLLECTION);

    let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
    let index = IndexModel::builder()
        .keys(doc! {"uploadDate": 1})
        .options(options)
        .build();

    files.create_index(index.clone(), None).await?;
    chunks.create_index(index, None).await?;
    Ok(())
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

    pub async fn clean_up(&self) {
        if let Err(err) = fs::remove_dir_all(&self.job_directory).await {
            warn!("Error occured, while deleting temp job files for {}: {}", &self.job_directory.to_str().unwrap_or("<none>"), &err)
        }
    }

    pub fn get_path(&self) -> PathBuf
    {
        self.job_directory.join(generate_30_alphanumeric())
    }

    pub fn get_one() -> PathBuf {
        env::temp_dir().join(generate_30_alphanumeric())
    }
}
