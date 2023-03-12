use axum::extract::State;
use axum::routing::get;
use axum::{Router, Json};
use crate::util::health::get_metrics;
use crate::util::state::JobsBasePersistenceState;
use crate::{
    util::consts::{NAME, VERSION},
    models::{RootDto, RootLinks, MetricsDto},
};
use axum::http::StatusCode;

pub fn create_route(jobs_base_persistence: JobsBasePersistenceState) -> Router {
    Router::new()
        .route("/", get(root_links))
        .route("/health", get(health))
        .route("/metrics", get(health))
        .with_state(jobs_base_persistence)
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

pub async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn metrics(State(base_presistence): State<JobsBasePersistenceState>) -> Result<Json<MetricsDto>, &'static str> {
    match get_metrics(base_presistence).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(err) => Err(err),
    }
}
