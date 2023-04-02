use bson::{doc, oid::ObjectId, DateTime};
use futures::Stream;
use mime::Mime;
use mongodb::{error::Error, options::IndexOptions, IndexModel};
use mongodb_gridfs::{options::GridFSBucketOptions, GridFSBucket};
use std::{str::FromStr, sync::Arc, time::Duration};
use tracing::warn;

use crate::{
    models::{DummyJobModel, FileModel},
    util::consts,
};

use super::MongoPersistenceBase;

const FILES_COLLECTION: &str = "fs.files";
const CHUNKS_COLLECTION: &str = "fs.chunks";

#[async_trait::async_trait]
pub trait IFileStorage: Send + Sync {
    async fn get_result_file(&self, token: &str, file_id: &str) -> Result<(Mime, Box<dyn Stream<Item = Vec<u8>> + Unpin + Send>), &'static str>;
    async fn store_result_file(&self, job_id: &str, token: &str, file_name: &str, mime_type: Option<&str>, source: Vec<u8>) -> Result<String, &'static str>;
}

pub struct GridFSFileStorage {
    base: Arc<MongoPersistenceBase>,
}

impl GridFSFileStorage {
    fn get_bucket(&self) -> GridFSBucket {
        let client = self.base.get_mongo_client();
        let db = client.database(&consts::NAME);
        GridFSBucket::new(db, Some(GridFSBucketOptions::default()))
    }

    pub async fn build(base: Arc<MongoPersistenceBase>, expire_seconds: u64) -> Result<Self, &'static str> {
        let oneself = GridFSFileStorage { base };
        oneself.set_expire_after(expire_seconds).await.map_err(|_| "Cloud not set expire time")?;
        Ok(oneself)
    }

    async fn set_expire_after(&self, seconds: u64) -> Result<(), Error> {
        let client = self.base.get_mongo_client();
        let files = client.database(&consts::NAME).collection::<DummyJobModel>(FILES_COLLECTION);
        let chunks = client.database(&consts::NAME).collection::<DummyJobModel>(CHUNKS_COLLECTION);

        let options = IndexOptions::builder().expire_after(Duration::new(seconds, 0)).build();
        let index = IndexModel::builder().keys(doc! {"uploadDate": 1}).options(options).build();

        files.create_index(index.clone(), None).await?;
        chunks.create_index(index, None).await?;
        Ok(())
    }

    async fn validate(&self, token: &str, file_id: &str) -> Result<FileModel, &'static str> {
        let client = self.base.get_mongo_client();
        if let Ok(id) = ObjectId::from_str(file_id) {
            let files = client.database(&consts::NAME).collection::<FileModel>(FILES_COLLECTION);
            let file = files.find_one(Some(doc! { "_id": id, "token": token}), None).await.map_err(|_| "Could not find file.")?.ok_or("Could not find file.")?;
            return Ok(file);
        }
        Err("Could not find file.")
    }
}

#[async_trait::async_trait]
impl IFileStorage for GridFSFileStorage {
    async fn get_result_file(&self, token: &str, file_id: &str) -> Result<(Mime, Box<dyn Stream<Item = Vec<u8>> + Unpin + Send>), &'static str> {
        let file_model = self.validate(token, file_id).await?;
        let mime_type = file_model.get_content_type();
        let bucket = self.get_bucket();
        let cursor = bucket.open_download_stream(file_model.id.unwrap()).await.map_err(|_| "Could not find file.")?;
        Ok((mime_type, Box::new(cursor)))
    }

    async fn store_result_file(&self, job_id: &str, token: &str, file_name: &str, mime_type: Option<&str>, source: Vec<u8>) -> Result<String, &'static str> {
        let client = self.base.get_mongo_client();
        let mut bucket = self.get_bucket();
        let file_id = bucket.upload_from_stream(file_name, &*source, None).await.map_err(|_| "Could not store result.").map(|id| id.to_string())?;
        let chunks = client.database(&consts::NAME).collection::<DummyJobModel>(CHUNKS_COLLECTION);
        let chunks_result = chunks.update_many(doc! { "files_id": ObjectId::from_str(&file_id).unwrap() }, doc! {"$set": {"uploadDate": DateTime::now()}}, None);
        let files = client.database(&consts::NAME).collection::<DummyJobModel>(FILES_COLLECTION);
        let file_result = files.update_one(doc! { "_id": ObjectId::from_str(&file_id).unwrap() }, doc! {"$set": {"token": token, "mimeType": mime_type}}, None);
        if file_result.await.is_err() {
            warn!("Could not set uploadDate for chunks of {}.", &file_id);
        }
        if chunks_result.await.is_err() {
            warn!("Could not set job for file {}.", &file_id);
        }
        Ok(file_id)
    }
}