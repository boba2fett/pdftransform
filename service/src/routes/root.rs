use common::dtos::{RootDto, RootLinks};
use common::util::consts::{NAME, VERSION};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

pub fn create_route() -> Router {
    Router::new().route("/", get(root_links)).route("/health", get(health))
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
