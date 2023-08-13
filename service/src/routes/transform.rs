use axum::extract::{Path, Query};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use axum::{Json, Router};
use chrono::Utc;
use common::dtos::CreateTransformJobDto;
use common::models::{TransformJobModel, JobStatus, TransformInput};
use common::util::random;
use reqwest::StatusCode;
use std::collections::HashMap;

use crate::state::Services;


pub fn create_route(services: Services) -> Router {
    Router::new().route("/transform/:job_id", get(transform_job)).route("/transform", post(create_transform_job)).with_state(services)
}

#[tracing::instrument(skip(params, services))]
pub async fn transform_job(State(services): State<Services>, Path(job_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    if let Ok(Some(job)) = services.job_persistence.get(&job_id).await {
        let job = TransformJobModel::from_json_slice(&job).unwrap();
        if job.token == token {
            return Ok(Json(job.to_dto()))
        }
    }
    Err(StatusCode::NOT_FOUND)
}

#[tracing::instrument(skip(services, create_job))]
pub async fn create_transform_job(State(services): State<Services>, Json(create_job): Json<CreateTransformJobDto>) -> impl IntoResponse {
    let id = random::generate_30_alphanumeric();
    let token = random::generate_30_alphanumeric();
    let job = TransformJobModel {
        id: id.clone(),
        token,
        created: Utc::now(),
        status: JobStatus::Pending,
        message: None,
        callback_uri: create_job.callback_uri,
        input: TransformInput {
            source_files: create_job.source_files,
            documents: create_job.documents,
        },
        result: None,
    };
    if let Err(e) = services.job_persistence.put(&job).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e))
    }
    match services.transform_publish_service.publish_job(&job.id).await {
        Ok(_) => Ok(Json(job.to_dto())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
