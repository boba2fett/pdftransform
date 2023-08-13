use axum::extract::{Path, Query};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use axum::{Json, Router};
use chrono::Utc;
use common::dtos::CreatePreviewJobDto;
use common::models::{PreviewJobModel, PreviewInput, JobStatus};
use common::util::random;
use reqwest::StatusCode;
use std::collections::HashMap;

use crate::state::Services;


pub fn create_route(services: Services) -> Router {
    Router::new()
        .route("/preview/:job_id", get(preview_job))
        .route("/preview", post(create_preview_job))
        .with_state(services)
}

#[tracing::instrument(skip(params, services))]
pub async fn preview_job(State(services): State<Services>, Path(job_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    if let Ok(Some(job)) = services.job_persistence.get(&job_id).await {
        let job = PreviewJobModel::from_json_slice(&job).unwrap();
        if job.token == token {
            return Ok(Json(job.to_dto()))
        }
    }
    Err(StatusCode::NOT_FOUND)
}

pub async fn create_preview_job(State(services): State<Services>, Json(create_job): Json<CreatePreviewJobDto>) -> impl IntoResponse {
    let id = random::generate_30_alphanumeric();
    let token = random::generate_30_alphanumeric();
    let job = PreviewJobModel {
        id: id.clone(),
        token,
        created: Utc::now(),
        status: JobStatus::Pending,
        message: None,
        callback_uri: create_job.callback_uri,
        input: PreviewInput {
            source_uri: create_job.source_uri,
            source_mime_type: create_job.source_mime_type,
            pdf: create_job.pdf.unwrap_or(true),
            png: create_job.png.unwrap_or(true),
            attachments: create_job.attachments.unwrap_or(true),
            signatures: create_job.signatures.unwrap_or(true),
        },
        result: None,
    };
    if let Err(e) = services.job_persistence.put(&job).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e))
    }
    match services.preview_publish_service.publish_job(&job.id).await {
        Ok(_) => Ok(Json(job.to_dto())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
