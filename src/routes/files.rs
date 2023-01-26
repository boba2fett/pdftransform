use crate::{files::get_result_file, routes::Token};
use actix_web::{web::{Path, Query}, HttpResponse, get};


pub fn create_route() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/authenticate", post(authenticate_user))
}

#[get("/file/{file_id}")]
pub async fn file(file_id: Path<String>, token: Query<Token>) -> HttpResponse {
    let file_id = file_id.into_inner();
    let token = token.into_inner().token;
    match get_result_file(&token, &file_id).await {
        Ok(file) => HttpResponse::Ok().content_type(file.0).async_stream(file.1),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

pub fn file_route(file_id: &str, token: &str) -> String {
    format!("/file/{}?token={}", file_id, token)
}
