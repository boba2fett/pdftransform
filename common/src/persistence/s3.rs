use std::{collections::HashMap, path::PathBuf};

use s3::{Bucket, creds::Credentials, region::Region};
use tokio::{fs::File, io::AsyncRead};
use tracing::info;

use crate::util::stream::VecReader;

use super::IFileStorage;

pub struct S3FileStorage {
    bucket: Bucket,
    expire_seconds: u32,
}

impl S3FileStorage {
    pub async fn build(endpoint: String, region: String, access_key_id: String, secret_access_key: String, bucket: String, expire_seconds: u32) -> Result<Self, &'static str> {
        let credentials = Credentials::new(Some(&access_key_id), Some(&secret_access_key), None, None, None);
        let credentials = credentials.map_err(|_| "error with credentials")?;
        let bucket = Bucket::new(&bucket, Region::Custom { region, endpoint }, credentials).map_err(|_| "error with bucket")?;
        let bucket = bucket.with_path_style();
        Ok(S3FileStorage {
            bucket,
            expire_seconds,
        })
    }
}

#[async_trait::async_trait]
impl IFileStorage for S3FileStorage {   
    async fn store_result_file(&self, key: &str, file_name: &str, mime_type: Option<&str>, source: Vec<u8>) -> Result<String, &'static str> {
        let mut vec_reader = VecReader {
            vec: source,
        };
        self.store_result(key, file_name, mime_type, &mut vec_reader).await
    }

    async fn store_result_file_path(&self, key: &str, file_name: &str, mime_type: Option<&str>, source: &PathBuf) -> Result<String, &'static str> {
        let mut file = File::open(source).await.map_err(|_| "file not found")?;
        self.store_result(key, file_name, mime_type, &mut file).await
    }
}

impl S3FileStorage {
    async fn store_result<R>(&self, key: &str, file_name: &str, mime_type: Option<&str>, mut source: R) -> Result<String, &'static str> where R: AsyncRead + Unpin, {
        info!("Storing {}", &key);
        if let Some(mime_type) = mime_type {
            self.bucket.put_object_stream_with_content_type(&mut source, key, &mime_type).await.map_err(
                |x| "could not put blob")?;
        }
        else {
            self.bucket.put_object_stream(&mut source, key).await.map_err(|_| "could not put blob")?;
        }
        let mut custom_queries = HashMap::new();
        custom_queries.insert(
            "response-content-disposition".into(),
            format!("attachment; filename=\"{}\"", file_name),
        );
        let presigned = self.bucket.presign_get(key, self.expire_seconds, Some(custom_queries)).map_err(|_| "could not get presigned url")?;
        Ok(presigned)
    }
}
