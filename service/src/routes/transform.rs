use axum::extract::{Path, Query};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use axum::{Json, Router};
use reqwest::StatusCode;
use std::collections::HashMap;

use common::models::CreateTransformJobModel;
use crate::state::Services;

pub fn create_route(services: Services) -> Router {
    Router::new().route("/transform/:job_id", get(transform_job)).route("/transform", post(create_transform_job)).with_state(services)
}

#[tracing::instrument(skip(params, services))]
pub async fn transform_job(State(services): State<Services>, Path(job_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    match services.persistence.transform_persistence.get_transform_job_dto(&job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

#[tracing::instrument(skip(services, create_job))]
pub async fn create_transform_job(State(services): State<Services>, Json(create_job): Json<CreateTransformJobModel>) -> impl IntoResponse {
    match services.persistence.transform_persistence.create_new_transform_job(create_job).await {
        Ok((job_dto, job_model)) => {
            match services.transform_starter.publish(&job_dto.id).await {
                Ok(_) => Ok(Json(job_dto)),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
            }
        }
        Err(e) => Err((StatusCode::CONFLICT, e)),
    }
}
