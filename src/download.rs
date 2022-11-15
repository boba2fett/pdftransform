use std::path::PathBuf;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{files::TempJobFileProvider, models::SourceFile, consts::PARALLELISM};


pub struct DownloadedSourceFile {
    pub id: String,
    pub path: PathBuf,
}

pub async fn download_source_files(client: &reqwest::Client, job_id: &str, source_files: Vec<SourceFile>) -> Vec<Result<DownloadedSourceFile, &'static str>> {
    let ref_client = &client;
    let job_files = TempJobFileProvider::build(job_id).await;
    let ref_job_files = &job_files;
    let parallelism = unsafe {PARALLELISM};
    futures::stream::iter(source_files)
    .map(|source_file| {
        async move {
            dowload_source_file(ref_client, ref_job_files, source_file).await
        }
    }).buffer_unordered(parallelism).collect::<Vec<Result<DownloadedSourceFile, &'static str>>>().await
}

async fn dowload_source_file<'a>(client: &reqwest::Client, job_files: &TempJobFileProvider, source_file: SourceFile) -> Result<DownloadedSourceFile, &'static str> {
    let path = job_files.get_path();
    let mut response = client.get(&source_file.uri).send().await.map_err(|_| "Could not load document.")?;
    let mut file = tokio::fs::File::create(&path).await.map_err(|_| "Could not create file.")?;
    while let Some(mut item) = response.chunk().await.map_err(|_| "Could not read response.")? {
        file.write_all_buf(&mut item).await.map_err(|_| "Could not write to file.")?;
    }
    Ok(DownloadedSourceFile { id: source_file.id, path })
}
