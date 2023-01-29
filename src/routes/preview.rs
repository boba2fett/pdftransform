use std::collections::HashMap;
use axum::{routing::{get, post}, response::IntoResponse, extract::RawBody};
use bytes::Bytes;
use reqwest::StatusCode;
use axum::{extract::{Path, Query}};
use axum::{Router, Json};
use tokio_stream::StreamExt;

use crate::{preview::get_preview};
use crate::convert::process_preview_job;
use crate::models::{CreatePreviewJobDto,};
use crate::persistence::{create_new_preview_job, generate_30_alphanumeric, get_preview_job_dto};

pub fn create_route() -> Router {
    Router::new()
        .route("/preview/:job_id", get(preview_job))
        .route("/preview", post(create_preview_job))
        .route("/preview/sync", post(preview_sync))
}

pub async fn preview_job(Path(job_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    match get_preview_job_dto(&job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err((StatusCode::NOT_FOUND , e)),
    }
}

pub fn preview_job_route(job_id: &str, token: &str) -> String {
    format!("/preview/{}?token={}", &job_id, token)
}

pub async fn create_preview_job(Json(create_job): Json<CreatePreviewJobDto>) -> impl IntoResponse {
    match create_new_preview_job(create_job).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move { process_preview_job(job_id, Some(job_model)).await });
            Ok(Json(job_dto))
        }
        Err(e) => Err((StatusCode::CONFLICT , e)),
    }
}

pub async fn preview_sync(RawBody(body): RawBody) -> impl IntoResponse {
    let job_id = generate_30_alphanumeric();
    let token = generate_30_alphanumeric();
    let bytes: Vec<_> = body.collect().await;
    let bytes: Result<Vec<Bytes>, _> = bytes.into_iter().collect();
    match bytes {
        Ok(bytes) => {
            let bytes = bytes.concat();
            let result = get_preview(&job_id, &token, bytes).await.map(|r| Json(r));
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err((StatusCode::CONFLICT, err)),
            }
        },
        Err(_) => Err((StatusCode::BAD_REQUEST, "")),
    }
}
