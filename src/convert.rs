use std::path::PathBuf;
use futures::StreamExt;
use kv_log_macro::info;
use tokio::io::AsyncWriteExt;

use crate::{persistence::{set_ready, set_error, _get_job_model, _get_job_dto}, models::{DocumentResult, Document, JobDto, SourceFile}, transform::{add_part, init_pdfium}, files::{store_job_result_file, TempJobFileProvider}, routes::file_route};

struct DownloadedSourceFile {
    id: String,
    path: PathBuf,
}

pub async fn process_job(db_client: &mongodb::Client, job_id: String) -> () {
    info!("Starting job '{}'", &job_id, {jobId: job_id});
    let job_model = _get_job_model(db_client, &job_id).await;
    if let Ok(job_model) = job_model {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        let ref_client = &client;

        let job_files = TempJobFileProvider::build(&job_id).await;
        let ref_job_files = &job_files;

        let source_files: Vec<Result<DownloadedSourceFile, &'static str>> = futures::stream::iter(job_model.source_files)
        .map(|source_file| {
            async move {
                dowload_source_file(ref_client, ref_job_files, source_file).await
            }
        }).buffer_unordered(10).collect::<Vec<Result<DownloadedSourceFile, &'static str>>>().await;

        info!("Downloaded all files for job '{}'", &job_id, {jobId: job_id});

        let failed = source_files.iter().find(|source_file| source_file.is_err());

        match failed {
            None => {
                let source_files = source_files.iter().map(|source_file| source_file.as_ref().unwrap()).collect();
                let results: Result<_, &str> = process(db_client, &job_id, &job_model.token, &job_model.documents, source_files).await;
                _ = match results {
                    Ok(results) => ready(db_client, &job_id, &job_model.callback_uri, ref_client, results).await,
                    Err(err) => error(db_client, &job_id, &job_model.callback_uri, ref_client, err).await,
                };
            }
        Some(err) => {
            _ = error(db_client, &job_id, &job_model.callback_uri, ref_client, err.as_ref().err().unwrap()).await;
        }
    }
    }
}

async fn ready(db_client: &mongodb::Client, job_id: &str, callback_uri: &Option<String>, client: &reqwest::Client, results: Vec<DocumentResult>) {
    info!("Finished job '{}'", &job_id);
    let result = set_ready(db_client, job_id, results).await;
    if let Err(err) = result {
        _ = error(db_client, job_id, callback_uri, client, err).await;
        return;
    }
    if let Some(callback_uri) = callback_uri {
        let dto = _get_job_dto(db_client, &job_id).await;
        if let Ok(dto) = dto {
            let result = client.post(callback_uri).json::<JobDto>(&dto).send().await;
            if let Err(err) = result {
                info!("Error sending callback '{}' to '{}', because of {}", &job_id, callback_uri, err);
            }
        }
    }
}

async fn error(db_client: &mongodb::Client, job_id: &str, callback_uri: &Option<String>, client: &reqwest::Client, err: &str) {
    info!("Finished job '{}' with error {}", &job_id, err, {jobId: job_id});
    let result = set_error(db_client, job_id, err).await;
    if let Err(_) = result {
        return;
    }
    if let Some(callback_uri) = callback_uri {
        let dto = _get_job_dto(db_client, &job_id).await;
        if let Ok(dto) = dto {
            let result = client.post(callback_uri).json::<JobDto>(&dto).send().await;
            if let Err(err) = result {
                info!("Error sending error callback '{}' to '{}', because of {}", &job_id, callback_uri, err, {jobId: job_id});
            }
        }
    }
}

async fn process<'a>(db_client: &mongodb::Client, job_id: &String, job_token: &str, documents: &Vec<Document>, source_files: Vec<&DownloadedSourceFile>) -> Result<Vec<DocumentResult>, &'static str> {
    {
            let mut results = Vec::with_capacity(documents.len());
            for document in documents {
            let bytes = {
                let pdfium = init_pdfium();
                let mut new_doc = pdfium.create_new_pdf().map_err(|_| "Could not create document.")?;
                for part in &document.binaries {
                    let source_file = source_files.iter().find(|source_file| source_file.id.eq(&part.source_file)).ok_or("Could not find corresponding source file.")?;
                    let mut source_doc = pdfium.load_pdf_from_file(&source_file.path, None).map_err(|_| "Could not create document.")?;
                    add_part(&mut new_doc, &mut source_doc, part)?;
                }
                new_doc.save_to_bytes().map_err(|_| "Could not save file.")?
            };
            let file_id = store_job_result_file(db_client, &document.id, &*bytes).await?;

            results.push(DocumentResult {
                download_url: file_route(job_id, &file_id, job_token),
                id: document.id.to_string(),
            });
        }
        Ok(results)
    }
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
