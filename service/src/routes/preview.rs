use axum::extract::{Path, Query};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use axum::{Json, Router};
use common::models::CreatePreviewJobModel;
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
    match services.preview_persistence.get_preview_job_dto(&job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

#[tracing::instrument(skip(services, create_job))]
pub async fn create_preview_job(State(services): State<Services>, Json(create_job): Json<CreatePreviewJobModel>) -> impl IntoResponse {
    match services.preview_persistence.create_new_preview_job(create_job).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move { services.convert_service.process_preview_job(job_id, Some(job_model)).await });
            Ok(Json(job_dto))
        }
        Err(e) => Err((StatusCode::CONFLICT, e)),
    }
}
