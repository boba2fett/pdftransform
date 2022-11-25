use rocket::{get, http::ContentType, response::{stream::ByteStream, status::NotFound}};
use futures::StreamExt;
use crate::{files::{get_preview_result_file, get_job_result_file}, persistence::DbClient};

#[get("/convert/<job_id>/<file_id>?<token>")]
pub async fn convert_file(db_client: &DbClient, job_id: String, file_id: String, token: String) -> Result<(ContentType, ByteStream![Vec<u8>]), NotFound<String>> {
    let mut stream = get_job_result_file(&db_client.0, &job_id, &token, &file_id).await.map_err(|e| NotFound(e.to_string()))?;
    Ok((ContentType::PDF, ByteStream!{
        while let Some(bytes) = stream.next().await {
            yield bytes;
        }
    }))
}

pub fn convert_file_route(job_id: &str, file_id: &str, token: &str) -> String {
    format!("/convert/{}/{}?token={}", job_id, file_id, token)
}

#[get("/preview/<job_id>/<file_id>?<token>")]
pub async fn preview_file(db_client: &DbClient, job_id: String, file_id: String, token: String) -> Result<(ContentType, ByteStream![Vec<u8>]), NotFound<String>> {
    let mut stream = get_preview_result_file(&db_client.0, &job_id, &token, &file_id).await.map_err(|e| NotFound(e.to_string()))?;
    Ok((ContentType::JPEG, ByteStream!{
        while let Some(bytes) = stream.next().await {
            yield bytes;
        }
    }))
}

pub fn preview_file_route(job_id: &str, file_id: &str, token: &str) -> String {
    format!("/preview/{}/{}?token={}", job_id, file_id, token)
}
