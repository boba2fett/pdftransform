use axum::{
    body::StreamBody,
    extract::State,
    response::{AppendHeaders, IntoResponse},
    routing::get,
};
use axum::{
    extract::{Path, Query},
    Router,
};
use common::util::stream::StreamReader;
use reqwest::{header, StatusCode};
use std::collections::HashMap;
use tokio_util::io::ReaderStream;

use crate::state::FileStorageState;

pub fn create_route(storage: FileStorageState) -> Router {
    Router::new().route("/file/:file_id", get(file)).with_state(storage)
}

#[tracing::instrument(skip(params, state))]
pub async fn file(Path(file_id): Path<String>, Query(params): Query<HashMap<String, String>>, State(state): State<FileStorageState>) -> impl IntoResponse {
    let token = params.get("token").map(|token| token as &str).unwrap_or("wrong_token");
    if let Ok(file) = state.get_result_file(&token, &file_id).await {
        let mime = &file.0;
        let file = StreamReader { stream: file.1, buffer: vec![] };
        let stream = ReaderStream::new(file);
        let body = StreamBody::new(stream);
        let headers = AppendHeaders([(header::CONTENT_TYPE, mime.to_string())]);
        Ok((headers, body))
    } else {
        Err((StatusCode::NOT_FOUND, ()))
    }
}

pub fn file_route(file_id: &str, token: &str) -> String {
    format!("/file/{}?token={}", file_id, token)
}
