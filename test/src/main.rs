use axum::{Router, routing::post, Json, response::IntoResponse, http::StatusCode};
use common::models::{TransformJobDto, PreviewJobDto};
use std::net::{SocketAddr, IpAddr, Ipv6Addr};

#[tokio::main]
async fn main() {
    let app = Router::new()
    .route("/transform-callback", post(transform_callback))
    .route("/previewcallback", post(preview_callback));

    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 8001);
    println!("listening on {}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn transform_callback(Json(content): Json<TransformJobDto>) -> impl IntoResponse {
    println!("{:?}", content);
    StatusCode::OK
}

pub async fn preview_callback(Json(content): Json<PreviewJobDto>) -> impl IntoResponse {
    println!("{:?}", content);
    StatusCode::OK
}