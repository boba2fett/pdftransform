use crate::{files::get_result_file, routes::Token};
use actix_web::{web::{Data, Path, Query}, HttpResponse, get};

#[get("/file/{file_id}")]
pub async fn file(db_client: Data<&mongodb::Client>, file_id: Path<String>, token: Query<Token>) -> HttpResponse {
    let file_id = file_id.into_inner();
    let token = token.into_inner().token;
    match get_result_file(&db_client.0, &token, &file_id).await {
        Ok(file) => HttpResponse::Ok().content_type(file.0).stream(file.1),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

pub fn file_route(file_id: &str, token: &str) -> String {
    format!("/file/{}?token={}", file_id, token)
}
