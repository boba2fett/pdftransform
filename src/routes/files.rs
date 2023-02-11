use std::collections::HashMap;
use axum::{routing::get, body::{StreamBody}, response::{AppendHeaders, IntoResponse}};
use reqwest::{StatusCode, header};
use tokio_util::io::{ReaderStream};
use crate::{files::get_result_file, stream::StreamReader};
use axum::{Router, extract::{Path, Query}};

pub fn create_route() -> Router {
    Router::new()
        .route("/file/:file_id", get(file))
}

pub async fn file(Path(file_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    if let Ok(file) = get_result_file(&token, &file_id).await {
        let mime = &file.0;
        let file = StreamReader {
            stream: file.1,
            buffer: vec![]
        };
        let stream = ReaderStream::new(file);
        let body = StreamBody::new(stream);
        let headers = AppendHeaders([
            (header::CONTENT_TYPE, mime.to_string()),
        ]);
        Ok((headers, body))
    }
    else {
        Err((StatusCode::NOT_FOUND, ()))
    }
}

pub fn file_route(file_id: &str, token: &str) -> String {
    format!("/file/{}?token={}", file_id, token)
}
