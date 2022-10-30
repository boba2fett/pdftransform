use std::{path::PathBuf, thread::JoinHandle};
use tokio::{io::copy, task::JoinError};
use tempfile::Builder;

use crate::{persistence::{get_job_model, set_error}, models::SourceFile, transform::{add_page, get_pdfium}};

pub async fn process_job(job_id: String) -> () {
    let job_model = get_job_model(&job_id).await.unwrap();
    let client = reqwest::Client::new();
    let ref_client = &client;
    //let source_files = job_model.source_files;
    
    let tmp_dir = Builder::new().prefix(&job_id).tempdir().unwrap();

    // let source_files = futures::stream::iter(source_files)
    // .map(|source_file| {
    //     let path = tmp_dir.path().join(&source_file.source_file_id);
    //     async move {
    //         dowload_source_file(ref_client, &source_file.source_uri, path).await
    //     }
    // }).buffer_unordered(10).collect::<Vec<PathBuf>>().await;
    // let source_file_handles = source_files.iter().map(|source_file| {
    //     let path = tmp_dir.path().join(&source_file.source_file_id);
    //     tokio::spawn(async {
    //         dowload_source_file(ref_client, &source_file.source_uri, path).await
    //     })
    // }).collect::<Vec<tokio::task::JoinHandle<PathBuf>>>();
    // let source_file_results: Vec<Result<PathBuf, JoinError>> = futures::future::join_all(source_file_handles).await;
    // let source_files: Vec<PathBuf> = source_file_results.into_iter().flatten().collect();
    let mut source_files: Vec<PathBuf> = Vec::with_capacity(job_model.source_files.len());
    for source_file in job_model.source_files {
        let path = tmp_dir.path().join(&source_file.source_file_id);
        source_files.push(dowload_source_file(ref_client, &source_file.source_uri, path).await);
    }

    let pdfium = get_pdfium();

    for document in job_model.documents {
        let mut new_doc = pdfium.create_new_pdf().unwrap();
        for part in document.binaries {
            let source_path = source_files.iter().find(|path| path.ends_with(&part.source_file_id)).unwrap();
            let source_doc = pdfium.load_pdf_from_file(source_path, None).unwrap();
            if source_doc.pages().len() <= part.start_page_number.unwrap_or_else(|| u16::MAX) && source_doc.pages().len() <= part.end_page_number.unwrap_or_else(|| u16::MAX) {
                _ = set_error(&job_id).await;
                return ()
            }
            add_page(&mut new_doc, source_doc, &part);
        }
    }
    ()
}

async fn dowload_source_file(client: &reqwest::Client, source_file_url: &str, path: PathBuf) -> PathBuf {
    let response = client.get(source_file_url).send().await;
    let content = response.unwrap().text().await.unwrap();
    let mut file = tokio::fs::File::create(&path).await.unwrap();
    copy(&mut content.as_bytes(), &mut file).await.unwrap();
    path
}


