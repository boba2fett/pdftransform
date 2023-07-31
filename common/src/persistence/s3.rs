use std::collections::HashMap;

use s3::{Bucket, creds::Credentials};
use tokio_util::io::ReaderStream;

use crate::util::stream::{StreamReader, VecReader};

use super::IFileStorage;

pub struct S3FileStorage {
    bucket: Bucket,
    expire_seconds: u32,
}

impl S3FileStorage {
    pub fn build(endpoint: String, region: String, access_key_id: String, secret_access_key: String, bucket: String, expire_seconds: u32) -> Result<Self, &'static str> {
        let credentials = Credentials::new(Some(&access_key_id), Some(&secret_access_key), None, None, None);
        let credentials = credentials.map_err(|_| "error with credentials")?;
        let bucket = Bucket::new(&bucket, region.parse().map_err(|_| "invalid region")?, credentials).map_err(|_| "error with bucket")?;
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
        self.bucket.put_object_stream(&mut vec_reader, key).await.map_err(|_| "could not put blob")?;
        let mut custom_queries = HashMap::new();
        custom_queries.insert(
            "response-content-disposition".into(),
            format!("attachment; filename=\"{}\"", file_name),
        );
        let presigned = self.bucket.presign_get(key, self.expire_seconds, Some(custom_queries)).map_err(|_| "could not get presigned url")?;
        Ok(presigned)
    }
}
