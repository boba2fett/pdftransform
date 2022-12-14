use futures::StreamExt;
use rocket::http::ContentType;
use std::{path::PathBuf, str::FromStr};
use tokio::io::AsyncWriteExt;
use reqwest::{header::CONTENT_TYPE, Response};

use crate::{consts::PARALLELISM, files::TempJobFileProvider, models::SourceFile};

pub struct DownloadedSourceFile {
    pub id: String,
    pub path: PathBuf,
    pub content_type: ContentType,
}

pub async fn download_source_files(client: &reqwest::Client, source_files: Vec<SourceFile>, job_files: &TempJobFileProvider) -> Vec<Result<DownloadedSourceFile, &'static str>> {
    let ref_client = &client;
    let ref_job_files = &job_files;
    let parallelism = unsafe { PARALLELISM };
    futures::stream::iter(source_files)
        .map(|source_file| async move { download_source_file(ref_client, source_file, ref_job_files).await })
        .buffer_unordered(parallelism)
        .collect::<Vec<Result<DownloadedSourceFile, &'static str>>>()
        .await
}

async fn download_source_file(client: &reqwest::Client, source_file: SourceFile, job_files: &TempJobFileProvider) -> Result<DownloadedSourceFile, &'static str> {
    let (path, content_type) = download_source(client, &source_file.uri, job_files, &source_file.content_type).await?;
    Ok(DownloadedSourceFile {
        id: source_file.id,
        path,
        content_type,
    })
}

pub async fn download_source(client: &reqwest::Client, source_uri: &str, job_files: &TempJobFileProvider, content_type: &Option<String>) -> Result<(PathBuf, ContentType), &'static str> {
    let path = job_files.get_path();
    let mut response = client.get(source_uri).send().await.map_err(|_| "Could not load document.")?;
    let content_type = determine_content_type(&response, content_type)?;
    let mut file = tokio::fs::File::create(&path).await.map_err(|_| "Could not create file.")?;
    while let Some(mut item) = response.chunk().await.map_err(|_| "Could not read response.")? {
        file.write_all_buf(&mut item).await.map_err(|_| "Could not write to file.")?;
    }
    Ok((path, content_type))
}

fn determine_content_type(response: &Response, force_content_type: &Option<String>) -> Result<ContentType, &'static str> {
    match force_content_type {
        Some(content_type) => ContentType::from_str(content_type).map_err(|_| "Content-Type is not recognized."),
        None => {
            let content_type = response.headers().get(CONTENT_TYPE);
            let content_type = match content_type {
                Some(content_type) => ContentType::from_str(content_type.to_str().map_err(|_| "Could not load Content-Type")?).map_err(|_| "Could not load Content-Type")?,
                None => ContentType::PDF,
            };
            Ok(content_type)
        },
    }
}

pub async fn download_source_bytes(client: &reqwest::Client, source_uri: &str) -> Result<rocket::http::hyper::body::Bytes, &'static str> {
    let response = client.get(source_uri).send().await.map_err(|_| "Could not load document.")?;
    match response.error_for_status() {
        Ok(response) => response.bytes().await.map_err(|_| "Could not read source."),
        Err(_) => Err("Could not read source."),
    }
}
