use axum::routing::get;
use axum::{Router, Json};
use crate::util::health::get_metrics;
use crate::{
    util::consts::{NAME, VERSION},
    models::{RootDto, RootLinks, MetricsDto},
};
use axum::http::StatusCode;

pub fn create_route() -> Router {
    Router::new()
        .route("/", get(root_links))
        .route("/health", get(health))
        .route("/metrics", get(health))
}

#[tracing::instrument]
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

#[tracing::instrument]
pub async fn metrics() -> Result<Json<MetricsDto>, &'static str> {
    match get_metrics().await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(err) => Err(err),
    }
}
