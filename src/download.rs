use std::path::PathBuf;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{files::TempJobFileProvider, models::SourceFile, consts::PARALLELISM};


pub struct DownloadedSourceFile {
    pub id: String,
    pub path: PathBuf,
}

pub async fn download_source_files(client: &reqwest::Client, source_files: Vec<SourceFile>, job_files: &TempJobFileProvider) -> Vec<Result<DownloadedSourceFile, &'static str>> {
    let ref_client = &client;
    let ref_job_files = &job_files;
    let parallelism = unsafe {PARALLELISM};
    futures::stream::iter(source_files)
    .map(|source_file| {
        async move {
            download_source_file(ref_client, source_file, ref_job_files).await
        }
    }).buffer_unordered(parallelism).collect::<Vec<Result<DownloadedSourceFile, &'static str>>>().await
}

async fn download_source_file(client: &reqwest::Client, source_file: SourceFile, job_files: &TempJobFileProvider) -> Result<DownloadedSourceFile, &'static str> {
    Ok(DownloadedSourceFile { id: source_file.id, path: download_source(client, &source_file.uri, job_files).await? })
}

pub async fn download_source(client: &reqwest::Client, source_uri: &str, job_files: &TempJobFileProvider) -> Result<PathBuf, &'static str> {
    let path = job_files.get_path();
    let mut response = client.get(source_uri).send().await.map_err(|_| "Could not load document.")?;
    let mut file = tokio::fs::File::create(&path).await.map_err(|_| "Could not create file.")?;
    while let Some(mut item) = response.chunk().await.map_err(|_| "Could not read response.")? {
        file.write_all_buf(&mut item).await.map_err(|_| "Could not write to file.")?;
    }
    Ok(path)
}
