use std::collections::HashMap;
use axum::{routing::{get, post}, response::IntoResponse};
use reqwest::StatusCode;
use axum::{extract::{Path, Query}};
use axum::{Router, Json};

use crate::{
    convert::process_transform_job,
    models::{CreateTransformJobDto},
    persistence::{create_new_transform_job, get_transform_job_dto},
};

pub fn create_route() -> Router {
    Router::new()
        .route("/transform/:job_id", get(transform_job))
        .route("/transform", post(create_transform_job))
}

#[tracing::instrument(skip(params))]
pub async fn transform_job(Path(job_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    match get_transform_job_dto(&job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err((StatusCode::NOT_FOUND , e)),
    }
}

pub fn transform_job_route(job_id: &str, token: &str) -> String {
    format!("/transform/{}?token={}", &job_id, token)
}

#[tracing::instrument(skip(create_job))]
pub async fn create_transform_job(Json(create_job): Json<CreateTransformJobDto>) -> impl IntoResponse {
    match create_new_transform_job(create_job).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move { process_transform_job(job_id, Some(job_model)).await });
            Ok(Json(job_dto))
        }
        Err(e) => Err((StatusCode::CONFLICT, e)),
    }
}
