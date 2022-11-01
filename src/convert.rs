use std::path::PathBuf;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{persistence::{get_job_model, set_ready, set_error}, models::{DocumentResult, Document}, transform::{add_page, get_pdfium}, files::{JobFileProvider, get_job_files}};

pub async fn process_job(job_id: String) -> () {
    let job_model = get_job_model(&job_id).await.unwrap();
    let client = reqwest::Client::new();
    let ref_client = &client;
    
    let job_files = get_job_files(&job_id).await;

    let source_files = futures::stream::iter(job_model.source_files)
    .map(|source_file| {
        let path = job_files.get_path(&source_file.source_file_id);
        async move {
            dowload_source_file(ref_client, &source_file.source_uri, path).await
        }
    }).buffer_unordered(10).collect::<Vec<PathBuf>>().await;

    dbg!(&source_files);

    let results: Result<_, &str> = process(&job_id, job_model.documents, source_files, job_files);
    match results {
        Ok(results) => set_ready(&job_id, results).await.unwrap(),
        Err(_) => set_error(&job_id).await.unwrap(),
    }
}

fn process(job_id: &String, documents: Vec<Document>, source_files: Vec<PathBuf>, job_files: JobFileProvider) -> Result<Vec<DocumentResult>, &'static str> {
    {
        let mut results = Vec::with_capacity(documents.len());
        let pdfium = get_pdfium();
        for document in documents {
            let mut new_doc = pdfium.create_new_pdf().unwrap();
            for part in document.binaries {
                let source_path = source_files.iter().find(|path| path.ends_with(&part.source_file_id)).unwrap();
                let source_doc = pdfium.load_pdf_from_file(source_path, None).unwrap();
                // if source_doc.pages().len() <= part.start_page_number.unwrap_or_else(|| u16::MIN) || source_doc.pages().len() <= part.end_page_number.unwrap_or_else(|| u16::MAX) {
                //     return Err("pages do not line up.")
                // }
                add_page(&mut new_doc, source_doc, &part);
            }
            let path = job_files.get_path(&document.id);
            new_doc.save_to_file(&path).unwrap();

            results.push(DocumentResult {
                download_url: format!("/convert/{}/{}", job_id, document.id),
                id: document.id.to_string(),
            });
        }
        Ok(results)
    }
}

async fn dowload_source_file(client: &reqwest::Client, source_file_url: &str, path: PathBuf) -> PathBuf {
    let mut response = client.get(source_file_url).send().await.unwrap();
    let mut file = tokio::fs::File::create(&path).await.unwrap();
    while let Some(mut item) = response.chunk().await.unwrap() {
        file.write_all_buf(&mut item).await.unwrap();
    }
    path
}
