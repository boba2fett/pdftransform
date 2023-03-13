use axum::body::Bytes;
use futures::StreamExt;
use mime::Mime;
use reqwest::{header::CONTENT_TYPE, Response};
use std::{path::PathBuf, str::FromStr};
use tokio::io::AsyncWriteExt;

use crate::{models::SourceFile, persistence::files::TempJobFileProvider};

#[async_trait::async_trait]
pub trait DownloadService: Send + Sync {
    async fn download_source_files(&self, client: &reqwest::Client, source_files: Vec<SourceFile>, job_files: &TempJobFileProvider) -> Vec<Result<DownloadedSourceFile, &'static str>>;
    async fn download_source(&self, client: &reqwest::Client, source_uri: &str, job_files: &TempJobFileProvider, content_type: &Option<String>) -> Result<(PathBuf, Mime), &'static str>;
    async fn download_source_bytes(&self, client: &reqwest::Client, source_uri: &str) -> Result<Bytes, &'static str>;
}

pub struct DownloadedSourceFile {
    pub id: String,
    pub path: PathBuf,
    pub content_type: Mime,
}

pub struct DownloadServiceImpl {
    pub parallelism: usize,
}

#[async_trait::async_trait]
impl DownloadService for DownloadServiceImpl {
    async fn download_source_files(&self, client: &reqwest::Client, source_files: Vec<SourceFile>, job_files: &TempJobFileProvider) -> Vec<Result<DownloadedSourceFile, &'static str>> {
        let ref_client = &client;
        let ref_job_files = &job_files;
        futures::stream::iter(source_files)
            .map(|source_file| async move { self.download_source_file(ref_client, source_file, ref_job_files).await })
            .buffer_unordered(self.parallelism)
            .collect::<Vec<Result<DownloadedSourceFile, &'static str>>>()
            .await
    }

    async fn download_source(&self, client: &reqwest::Client, source_uri: &str, job_files: &TempJobFileProvider, content_type: &Option<String>) -> Result<(PathBuf, Mime), &'static str> {
        let path = job_files.get_path();
        let mut response = client.get(source_uri).send().await.map_err(|_| "Could not load document.")?;
        let content_type = self.determine_content_type(&response, &content_type)?;
        let mut file = tokio::fs::File::create(&path).await.map_err(|_| "Could not create file.")?;
        while let Some(mut item) = response.chunk().await.map_err(|_| "Could not read response.")? {
            file.write_all_buf(&mut item).await.map_err(|_| "Could not write to file.")?;
        }
        Ok((path, content_type))
    }

    async fn download_source_bytes(&self, client: &reqwest::Client, source_uri: &str) -> Result<Bytes, &'static str> {
        let response = client.get(source_uri).send().await.map_err(|_| "Could not load document.")?;
        match response.error_for_status() {
            Ok(response) => response.bytes().await.map_err(|_| "Could not read source."),
            Err(_) => Err("Could not read source."),
        }
    }
}

impl DownloadServiceImpl {
    fn determine_content_type(&self, response: &Response, force_content_type: &Option<String>) -> Result<Mime, &'static str> {
        match force_content_type {
            Some(content_type) => Ok(Mime::from_str(content_type).map_err(|_| "Could not get MimeType")?),
            None => {
                let content_type = response.headers().get(CONTENT_TYPE);
                let content_type = match content_type {
                    Some(content_type) => Mime::from_str(content_type.to_str().map_err(|_| "Could not get MimeType")?).map_err(|_| "Could not get MimeType")?,
                    None => mime::APPLICATION_PDF,
                };
                Ok(content_type)
            }
        }
    }

    async fn download_source_file(&self, client: &reqwest::Client, source_file: SourceFile, job_files: &TempJobFileProvider) -> Result<DownloadedSourceFile, &'static str> {
        let (path, content_type) = self.download_source(client, &source_file.uri, job_files, &source_file.content_type).await?;
        Ok(DownloadedSourceFile {
            id: source_file.id,
            path,
            content_type,
        })
    }
}
