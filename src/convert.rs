use futures::Future;
use serde::Serialize;
use tracing::{info, span, Level};

use crate::{
    download::{download_source_bytes, download_source_files, DownloadedSourceFile},
    files::TempJobFileProvider,
    models::{PreviewJobModel, TransformJobDto, TransformJobModel},
    persistence::{_get_preview_job_dto, _get_transform_job_dto, _get_transform_job_model, set_error, set_ready},
    preview::get_preview,
    transform::get_transformation,
};

#[tracing::instrument(skip(job_model))]
pub async fn process_transform_job(job_id: String, job_model: Option<TransformJobModel>) {
    info!("Starting job");
    let job_model = job_model.ok_or(_get_transform_job_model(&job_id).await);
    if let Ok(job_model) = job_model {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        let job_files = TempJobFileProvider::build(&job_id).await;
        let source_files = download_source_files(&client, job_model.source_files, &job_files).await;
        info!("Downloaded all files for job");

        let failed = source_files.iter().find(|source_file| source_file.is_err());

        match failed {
            None => {
                let source_files: Vec<&DownloadedSourceFile> = source_files.iter().map(|source_file| source_file.as_ref().unwrap()).collect();
                let results: Result<_, &str> = get_transformation(&job_id, &job_model.token, &job_model.documents, source_files, &job_files).await;
                match results {
                    Ok(results) => ready(&job_id, &job_model.callback_uri, &client, results, |job_id| _get_transform_job_dto(job_id)).await,
                    Err(err) => error( &job_id, &job_model.callback_uri, &client, err).await,
                };
            }
            Some(err) => {
                error(&job_id, &job_model.callback_uri, &client, err.as_ref().err().unwrap()).await;
            }
        }
        job_files.clean_up().await;
    }
}

#[tracing::instrument(skip(job_model))]
pub async fn process_preview_job(job_id: String, job_model: Option<PreviewJobModel>) {
    info!("Starting job");
    let job_model = job_model.ok_or(_get_transform_job_model(&job_id).await);
    if let Ok(job_model) = job_model {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        let job_files = TempJobFileProvider::build(&job_id).await;
        let source_file = download_source_bytes(&client, &job_model.source_uri.unwrap()).await;
        info!("Downloaded file for job");

        match source_file {
            Ok(source_file) => {
                let result: Result<_, &str> = get_preview(&job_id, &job_model.token, source_file.to_vec(), &job_model.source_mime_type).await;
                match result {
                    Ok(result) => ready(&job_id, &job_model.callback_uri, &client, result, |job_id| _get_preview_job_dto(job_id)).await,
                    Err(err) => error(&job_id, &job_model.callback_uri, &client, err).await,
                };
            }
            Err(err) => {
                error(&job_id, &job_model.callback_uri, &client, err).await;
            }
        }
        job_files.clean_up().await;
    }
}

async fn ready<'a, 'b, ResultType: Serialize, JobType: Serialize + Sized, F, Fut>(
    job_id: &'b str, callback_uri: &Option<String>, client: &reqwest::Client, result: ResultType, job_fn: F,
) where
    F: Send + 'static,
    F: FnOnce(&'b str) -> Fut,
    Fut: Future<Output = Result<JobType, &'static str>> + Send,
{
    info!("Finished job");
    let result = set_ready(job_id, result).await;
    if let Err(err) = result {
        error(job_id, callback_uri, client, err).await;
        return;
    }
    if let Some(callback_uri) = callback_uri {
        let dto = job_fn(&job_id).await;
        if let Ok(dto) = dto {
            let result = client.post(callback_uri).json::<JobType>(&dto).send().await;
            if let Err(err) = result {
                info!("Error sending callback '{}' to '{}', because of {}", &job_id, callback_uri, err);
            }
        }
    }
}

async fn error(job_id: &str, callback_uri: &Option<String>, client: &reqwest::Client, err: &str) {
    info!("Finished job with error {}", err);
    let result = set_error(job_id, err).await;
    if result.is_err() {
        return;
    }
    if let Some(callback_uri) = callback_uri {
        let dto = _get_transform_job_dto(&job_id).await;
        if let Ok(dto) = dto {
            let result = client.post(callback_uri).json::<TransformJobDto>(&dto).send().await;
            if let Err(err) = result {
                info!("Error sending error callback to '{}', because of {}", callback_uri, err);
            }
        }
    }
}
