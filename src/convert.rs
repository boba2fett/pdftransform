use kv_log_macro::info;
use serde::Serialize;

use crate::{persistence::{set_ready, set_error, _get_job_model, _get_job_dto}, models::{TransformJobModel, TransformJobDto, PreviewJobModel}, transform::get_transformation, files::TempJobFileProvider, download::{download_source_files, download_source, DownloadedSourceFile}, preview::get_preview};

pub async fn process_transform_job(db_client: &mongodb::Client, job_id: String, job_model: Option<TransformJobModel>) {
    info!("Starting job '{}'", &job_id, {jobId: job_id});
    let job_model = job_model.ok_or(_get_job_model(db_client, &job_id).await);
    if let Ok(job_model) = job_model {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        let job_files = TempJobFileProvider::build(&job_id).await;
        let source_files = download_source_files(&client, job_model.source_files, &job_files).await;
        info!("Downloaded all files for job '{}'", &job_id, {jobId: job_id});

        let failed = source_files.iter().find(|source_file| source_file.is_err());

        match failed {
            None => {
                let source_files: Vec<&DownloadedSourceFile> = source_files.iter().map(|source_file| source_file.as_ref().unwrap()).collect();
                let results: Result<_, &str> = get_transformation(db_client, &job_id, &job_model.token, &job_model.documents, source_files).await;
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

pub async fn process_preview_job(db_client: &mongodb::Client, job_id: String, job_model: Option<PreviewJobModel>) {
    info!("Starting job '{}'", &job_id, {jobId: job_id});
    let job_model = job_model.ok_or(_get_job_model(db_client, &job_id).await);
    if let Ok(job_model) = job_model {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        let job_files = TempJobFileProvider::build(&job_id).await;
        let source_file = download_source(&client, &job_model.source_uri.unwrap(), &job_files).await;
        info!("Downloaded file for job '{}'", &job_id, {jobId: job_id});

        match source_file {
            Ok(source_file) => {
                let result: Result<_, &str> = get_preview(db_client, &job_id, &job_model.token, &source_file, &job_files).await;
                match result {
                    Ok(result) => ready(db_client, &job_id, &job_model.callback_uri, &client, result).await,
                    Err(err) => error(db_client, &job_id, &job_model.callback_uri, &client, err).await,
                };
            }
            Err(err) => {
                error(db_client, &job_id, &job_model.callback_uri, &client, err).await;
            }
        }
        job_files.clean_up().await;
    }
}

async fn ready<ResultType: Serialize>(db_client: &mongodb::Client, job_id: &str, callback_uri: &Option<String>, client: &reqwest::Client, result: ResultType) {
    info!("Finished job '{}'", &job_id, {jobId: job_id});
    let result = set_ready(db_client, job_id, result).await;
    if let Err(err) = result {
        error(db_client, job_id, callback_uri, client, err).await;
        return;
    }
    if let Some(callback_uri) = callback_uri {
        let dto = _get_job_dto(db_client, &job_id).await;
        if let Ok(dto) = dto {
            let result = client.post(callback_uri).json::<TransformJobDto>(&dto).send().await;
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
            let result = client.post(callback_uri).json::<TransformJobDto>(&dto).send().await;
            if let Err(err) = result {
                info!("Error sending error callback '{}' to '{}', because of {}", &job_id, callback_uri, err, {jobId: job_id});
            }
        }
    }
}
