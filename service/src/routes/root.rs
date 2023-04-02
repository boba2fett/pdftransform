use crate::dtos::root::{MetricsDto, RootDto, RootLinks};
use crate::state::JobsBasePersistenceState;
use common::util::consts::{NAME, VERSION};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

pub fn create_route(jobs_base_persistence: JobsBasePersistenceState) -> Router {
    Router::new().route("/", get(root_links)).route("/health", get(health)).route("/metrics", get(metrics)).with_state(jobs_base_persistence)
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
    match base_persistence.jobs_health().await {
        Ok(jobs_health) => Ok(Json(
            MetricsDto {
                jobs: jobs_health
            }
        )),
        Err(err) => Err(err),
    }
}
