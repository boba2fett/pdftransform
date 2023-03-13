use crate::util::health::get_metrics;
use crate::util::state::JobsBasePersistenceState;
use crate::{
    models::{MetricsDto, RootDto, RootLinks},
    util::consts::{NAME, VERSION},
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

pub fn create_route(jobs_base_persistence: JobsBasePersistenceState) -> Router {
    Router::new().route("/", get(root_links)).route("/health", get(health)).route("/metrics", get(health)).with_state(jobs_base_persistence)
}

pub async fn root_links() -> Result<Json<RootDto>, &'static str> {
    Ok(Json(RootDto {
        version: VERSION,
        name: NAME,
        _links: RootLinks {
            transform: "/transform",
            preview: "/preview",
        },
    }))
}

#[tracing::instrument]
pub async fn health() -> StatusCode {
    StatusCode::OK
}

#[tracing::instrument(skip(base_persistence))]
pub async fn metrics(State(base_persistence): State<JobsBasePersistenceState>) -> Result<Json<MetricsDto>, &'static str> {
    match get_metrics(base_persistence).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(err) => Err(err),
    }
}
