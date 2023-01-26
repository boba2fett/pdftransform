use axum::routing::get;
use axum::{Router, Json, response::Response};
use axum::response::{IntoResponse};
use axum::http::StatusCode;
use crate::{
    consts::{NAME, VERSION},
    models::{AvgTimeModel, RootDto, RootLinks},
    persistence::{jobs_health},
};

pub fn create_route() -> Router {
    Router::new()
        .route("/", get(root_links))
        .route("/health", get(health))
}

pub fn root_links() -> Response {
    (StatusCode::OK , Json(RootDto {
        version: VERSION,
        name: NAME,
        _links: RootLinks {
            transform: "/transform".to_string(),
            preview: "/preview".to_string(),
        },
    })).into_response()
}

pub async fn health() -> Result<Json<Vec<AvgTimeModel>>, &'static str> {
    match jobs_health().await {
        Ok(health) => Ok(Json(health)),
        Err(err) => Err(err),
    }
}
