use axum::extract::{Path, Query};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use axum::{Json, Router};
use reqwest::StatusCode;
use std::{collections::HashMap, sync::Arc};

use common::models::CreateTransformJobModel;
use crate::state::ServiceCollection;

pub fn create_route(services: Arc<ServiceCollection>) -> Router {
    Router::new().route("/transform/:job_id", get(transform_job)).route("/transform", post(create_transform_job)).with_state(services)
}

#[tracing::instrument(skip(params, services))]
pub async fn transform_job(State(services): State<Arc<ServiceCollection>>, Path(job_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    match services.transform_persistence.get_transform_job_dto(&job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

#[tracing::instrument(skip(services, create_job))]
pub async fn create_transform_job(State(services): State<Arc<ServiceCollection>>, Json(create_job): Json<CreateTransformJobModel>) -> impl IntoResponse {
    match services.transform_persistence.create_new_transform_job(create_job).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move { services.convert_service.process_transform_job(job_id, Some(job_model)).await });
            Ok(Json(job_dto))
        }
        Err(e) => Err((StatusCode::CONFLICT, e)),
    }
}
