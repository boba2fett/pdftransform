use actix_web::http::header::ContentType;
use bson::{doc, oid::ObjectId, DateTime};
use futures::{AsyncRead, Stream};
use kv_log_macro::warn;
use mongodb::{error::Error, options::IndexOptions, IndexModel};
use mongodb_gridfs::{options::GridFSBucketOptions, GridFSBucket};
use std::{env, path::PathBuf, str::FromStr, time::Duration};
use tokio::fs;

use crate::{
    consts::{self},
    models::{DummyModel, FileModel},
    persistence::generate_30_alphanumeric,
};

const FILES_COLLECTION: &str = "fs.files";
const CHUNKS_COLLECTION: &str = "fs.chunks";

fn get_bucket(client: &mongodb::Client) -> GridFSBucket {
    let db = client.database(consts::NAME);
    GridFSBucket::new(db, Some(GridFSBucketOptions::default()))
}

pub async fn get_result_file(client: &mongodb::Client, token: &str, file_id: &str) -> Result<(ContentType, impl Stream<Item = Vec<u8>>), &'static str> {
    let file_model = validate(client, token, file_id).await?;
    let mime_type = file_model.get_content_type();
    let bucket = get_bucket(client);
    let cursor = bucket.open_download_stream(file_model.id.unwrap()).await.map_err(|_| "Could not find file.")?;
    Ok((mime_type, cursor))
}

async fn validate(client: &mongodb::Client, token: &str, file_id: &str) -> Result<FileModel, &'static str> {
    if let Ok(id) = ObjectId::from_str(file_id) {
        let files = client.database(consts::NAME).collection::<FileModel>(FILES_COLLECTION);
        let file = files.find_one(Some(doc! { "_id": id, "token": token}), None).await.map_err(|_| "Could not find file.")?.ok_or("Could not find file.")?;
        return Ok(file);
    }
    Err("Could not find file.")
}

pub async fn store_result_file(client: &mongodb::Client, job_id: &str, token: &str, file_name: &str, mime_type: Option<&str>, source: impl AsyncRead + Unpin) -> Result<String, &'static str> {
    let mut bucket = get_bucket(client);
    let file_id = bucket.upload_from_stream(file_name, source, None).await.map_err(|_| "Could not store result.").map(|id| id.to_string())?;
    let chunks = client.database(consts::NAME).collection::<DummyModel>(CHUNKS_COLLECTION);
    let chunks_result = chunks.update_many(doc! { "files_id": ObjectId::from_str(&file_id).unwrap() }, doc! {"$set": {"uploadDate": DateTime::now()}}, None);
    let files = client.database(consts::NAME).collection::<DummyModel>(FILES_COLLECTION);
    let file_result = files.update_one(doc! { "_id": ObjectId::from_str(&file_id).unwrap() }, doc! {"$set": {"token": token, "mimeType": mime_type}}, None);
    if file_result.await.is_err() {
        warn!("Could not set uploadDate for chunks of {}.", &file_id, {fileId: &file_id, jobId: &job_id});
    }
    if chunks_result.await.is_err() {
        warn!("Could not set job for file {}.", &file_id, {fileId: &file_id, jobId: &job_id});
    }
    Ok(file_id)
}

pub async fn set_expire_after(client: &mongodb::Client, seconds: u64) -> Result<(), Error> {
    let files = client.database(consts::NAME).collection::<DummyModel>(FILES_COLLECTION);
    let chunks = client.database(consts::NAME).collection::<DummyModel>(CHUNKS_COLLECTION);

    let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
    let index = IndexModel::builder().keys(doc! {"uploadDate": 1}).options(options).build();

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
