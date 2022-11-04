use std::path::PathBuf;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{persistence::{get_job_model, set_ready, set_error}, models::{DocumentResult, Document}, transform::{add_page, init_pdfium}, files::{JobFileProvider, get_job_files}};

pub async fn process_job(job_id: String) -> () {
    let job_model = get_job_model(&job_id).await;
    if let Ok(job_model) = job_model {
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

        let results: Result<_, &str> = process(&job_id, job_model.documents, source_files, job_files);
        _ = match results {
            Ok(results) => set_ready(&job_id, results).await,
            Err(err) => set_error(&job_id, err).await,
        };
    }
}

fn process(job_id: &String, documents: Vec<Document>, source_files: Vec<PathBuf>, job_files: JobFileProvider) -> Result<Vec<DocumentResult>, &'static str> {
    {
        let mut results = Vec::with_capacity(documents.len());
        for document in documents {
            let pdfium = init_pdfium();
            let mut new_doc = pdfium.create_new_pdf().map_err(|_| "Could not create document")?;
            for part in document.binaries {
                let source_path = source_files.iter().find(|path| path.ends_with(&part.source_file_id)).ok_or("Could not find corresponding source file.")?;
                let mut source_doc = pdfium.load_pdf_from_file(source_path, None).map_err(|_| "Could not create document")?;
                add_page(&mut new_doc, &mut source_doc, &part).map_err(|_| "Error while converting, were the page numbers correct?")?;
            }
            let path = job_files.get_path(&document.id);
            new_doc.save_to_file(&path).map_err(|_| "Could not save file.")?;

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
