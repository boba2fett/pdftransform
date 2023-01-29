use axum::routing::get;
use axum::{Router, Json};
use crate::health::get_health;
use crate::{
    consts::{NAME, VERSION},
    models::{RootDto, RootLinks, HealthDto},
};

pub fn create_route() -> Router {
    Router::new()
        .route("/", get(root_links))
        .route("/health", get(health))
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

pub async fn health() -> Result<Json<HealthDto>, &'static str> {
    match get_health().await {
        Ok(health) => Ok(Json(health)),
        Err(err) => Err(err),
    }
}
