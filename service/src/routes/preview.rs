use axum::extract::{Path, Query};
use axum::{
    extract::{RawBody, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
};
use axum::{Json, Router};
use bytes::Bytes;
use common::models::CreatePreviewJobModel;
use common::util::random::generate_30_alphanumeric;
use reqwest::{header::CONTENT_TYPE, StatusCode};
use std::{collections::HashMap, sync::Arc};
use tokio_stream::StreamExt;

use crate::state::ServiceCollection;

pub fn create_route(services: Arc<ServiceCollection>) -> Router {
    Router::new()
        .route("/preview/:job_id", get(preview_job))
        .route("/preview", post(create_preview_job))
        .route("/preview/sync", post(preview_sync))
        .with_state(services)
}

#[tracing::instrument(skip(params, services))]
pub async fn preview_job(Path(job_id): Path<String>, Query(params): Query<HashMap<String, String>>, State(services): State<Arc<ServiceCollection>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    match services.preview_persistence.get_preview_job_dto(&job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

#[tracing::instrument(skip(services, create_job))]
pub async fn create_preview_job(State(services): State<Arc<ServiceCollection>>, Json(create_job): Json<CreatePreviewJobModel>) -> impl IntoResponse {
    match services.preview_persistence.create_new_preview_job(create_job).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move { services.convert_service.process_preview_job(job_id, Some(job_model)).await });
            Ok(Json(job_dto))
        }
        Err(e) => Err((StatusCode::CONFLICT, e)),
    }
}

#[tracing::instrument(skip(services, body))]
pub async fn preview_sync(State(services): State<Arc<ServiceCollection>>, headers: HeaderMap, RawBody(body): RawBody) -> impl IntoResponse {
    let content_type = headers.get(CONTENT_TYPE).map(|content_type| content_type.to_str().unwrap_or("application/pdf")).unwrap_or("application/pdf");
    let job_id = generate_30_alphanumeric();
    let token = generate_30_alphanumeric();
    let bytes: Vec<_> = body.collect().await;
    let bytes: Result<Vec<Bytes>, _> = bytes.into_iter().collect();
    match bytes {
        Ok(bytes) => {
            let bytes = bytes.concat();
            let result = services.preview_service.get_preview(&job_id, &token, bytes).await.map(|r| Json(r));
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err((StatusCode::CONFLICT, err)),
            }
        }
        Err(_) => Err((StatusCode::BAD_REQUEST, "")),
    }
}
