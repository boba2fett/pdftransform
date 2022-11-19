use kv_log_macro::info;
use pdfium_render::prelude::PdfDocument;

use crate::{persistence::{set_ready, set_error, _get_job_model, _get_job_dto}, models::{DocumentResult, Document, JobDto}, transform::{add_part, init_pdfium}, files::{store_job_result_file, TempJobFileProvider}, routes::file_route, download::{download_source_files, DownloadedSourceFile}};

pub async fn process_job(db_client: &mongodb::Client, job_id: String) {
    info!("Starting job '{}'", &job_id, {jobId: job_id});
    let job_model = _get_job_model(db_client, &job_id).await;
    if let Ok(job_model) = job_model {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        let job_files = TempJobFileProvider::build(&job_id).await;
        let source_files = download_source_files(&client, job_model.source_files, &job_files).await;
        info!("Downloaded all files for job '{}'", &job_id, {jobId: job_id});

        let failed = source_files.iter().find(|source_file| source_file.is_err());

        match failed {
            None => {
                let source_files: Vec<&DownloadedSourceFile> = source_files.iter().map(|source_file| source_file.as_ref().unwrap()).collect();
                let results: Result<_, &str> = process(db_client, &job_id, &job_model.token, &job_model.documents, source_files).await;
                match results {
                    Ok(results) => ready(db_client, &job_id, &job_model.callback_uri, &client, results).await,
                    Err(err) => error(db_client, &job_id, &job_model.callback_uri, &client, err).await,
                };
            }
            Some(err) => {
                error(db_client, &job_id, &job_model.callback_uri, &client, err.as_ref().err().unwrap()).await;
            }
        }
        job_files.clean_up().await;
    }
}

async fn ready(db_client: &mongodb::Client, job_id: &str, callback_uri: &Option<String>, client: &reqwest::Client, results: Vec<DocumentResult>) {
    info!("Finished job '{}'", &job_id);
    let result = set_ready(db_client, job_id, results).await;
    if let Err(err) = result {
        error(db_client, job_id, callback_uri, client, err).await;
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
    if result.is_err() {
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

async fn process<'a>(db_client: &mongodb::Client, job_id: &str, job_token: &str, documents: &Vec<Document>, source_files: Vec<&DownloadedSourceFile>) -> Result<Vec<DocumentResult>, &'static str> {
    {
        let mut results = Vec::with_capacity(documents.len());
        for document in documents {
            let bytes = {
                let pdfium = init_pdfium();
                let mut new_doc = pdfium.create_new_pdf().map_err(|_| "Could not create document.")?;
                for part in &document.binaries {
                    let source_file = source_files.iter().find(|source_file| source_file.id.eq(&part.source_file)).ok_or("Could not find corresponding source file.")?;
                    let source_doc = pdfium.load_pdf_from_file(&source_file.path, None).map_err(|_| "Could not create document.")?;
                    add_part(&mut new_doc, &source_doc, part)?;
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
