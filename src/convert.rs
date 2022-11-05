use std::path::PathBuf;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{persistence::{get_job_model, set_ready, set_error}, models::{DocumentResult, Document}, transform::{add_page, create_new_pdf, load_pdf_from_file}, files::{JobFileProvider, get_job_files}};

pub async fn process_job(job_id: String) -> () {
    let job_model = get_job_model(&job_id).await;
    if let Ok(job_model) = job_model {
        let client = reqwest::Client::new();
        let ref_client = &client;
        
        let job_files = get_job_files(&job_id).await;

        let source_files: Vec<Result<PathBuf, &'static str>> = futures::stream::iter(job_model.source_files)
        .map(|source_file| {
            let path = job_files.get_path(&source_file.source_file_id);
            async move {
                dowload_source_file(ref_client, &source_file.source_uri, path).await
            }
        }).buffer_unordered(10).collect::<Vec<Result<PathBuf, &'static str>>>().await;

        let failed = source_files.iter().find(|source_file| source_file.is_err());

        if failed.is_none() {
            let source_files = source_files.iter().map(|source_file| source_file.as_ref().unwrap()).collect();
            let results: Result<_, &str> = process(&job_id, job_model.documents, source_files, job_files);
            _ = match results {
                Ok(results) => set_ready(&job_id, results).await,
                Err(err) => set_error(&job_id, err).await,
            };
        }
        else {
            _ = set_error(&job_id, failed.as_ref().unwrap().as_ref().err().unwrap()).await;
        }
    }
}

fn process(job_id: &String, documents: Vec<Document>, source_files: Vec<&PathBuf>, job_files: JobFileProvider) -> Result<Vec<DocumentResult>, &'static str> {
    {
        let mut results = Vec::with_capacity(documents.len());
        for document in documents {
            let mut new_doc = create_new_pdf()?;
            for part in document.binaries {
                let source_path = source_files.iter().find(|path| path.ends_with(&part.source_file_id)).ok_or("Could not find corresponding source file.")?;
                let mut source_doc = load_pdf_from_file(source_path)?;
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

async fn dowload_source_file(client: &reqwest::Client, source_file_url: &str, path: PathBuf) -> Result<PathBuf, &'static str> {
    let mut response = client.get(source_file_url).send().await.map_err(|_| "Could not download document.")?;
    let mut file = tokio::fs::File::create(&path).await.map_err(|_| "Could not create file.")?;
    while let Some(mut item) = response.chunk().await.map_err(|_| "Could read response.")? {
        file.write_all_buf(&mut item).await.map_err(|_| "Could write to file.")?;
    }
    Ok(path)
}
