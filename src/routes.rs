use crate::{models::{RootDto, JobDto, CreateJobDto, RootLinks}, consts::{VERSION, NAME}, persistence::{create_new_job, get_job_dto, DbClient}, convert::process_job, files::get_job_result_file};
use rocket::{serde::json::Json, response::{status::{Conflict, NotFound}, stream::ByteStream}, http::ContentType, get, post};
use futures::StreamExt;

#[get("/")]
pub fn root<'a>() -> Json<RootDto<'a>> {
    Json(RootDto {
        version: VERSION,
        name: NAME,
        _links: RootLinks {
            convert: "/convert"
        }
    })
}

#[get("/convert/<job_id>?<token>")]
pub async fn job<'a>(job_id: String, token: String, client: &DbClient) -> Result<Json<JobDto>, NotFound<&'static str>> {
    match get_job_dto(client, &job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(NotFound(e)),
    }
}

pub fn job_route(job_id: &str, token: &str) -> String {
    format!("/convert/{}?token={}", &job_id, token)
}

#[get("/convert/<job_id>/<file_id>?<token>")]
pub async fn file(db_client: &DbClient,job_id: String, file_id: String, token: String) -> Result<(ContentType, ByteStream![Vec<u8>]), NotFound<String>> {
    let mut stream = get_job_result_file(&db_client.0, &job_id, &token, &file_id).await.map_err(|e| NotFound(e.to_string()))?;
    Ok((ContentType::PDF, ByteStream!{
        while let Some(bytes) = stream.next().await {
            yield bytes;
        }
    }))
}

pub fn file_route(job_id: &str, file_id: &str, token: &str) -> String {
    format!("/convert/{}/{}?token={}", job_id, file_id, token)
}

#[post("/convert", format = "json", data="<create_job>")]
pub async fn create_job(db_client: &DbClient, create_job: Json<CreateJobDto>) -> Result<Json<JobDto>, Conflict<&'static str>> {
    match create_new_job(&db_client, create_job.0).await {
        Ok(job_dto) => {
            let job_id = job_dto.id.clone();
            let db_client_ref = db_client.0.clone();
            tokio::spawn(async move {
                process_job(&db_client_ref, job_id).await
            });
            Ok(Json(job_dto))
        },
        Err(e) => Err(Conflict(Some(e))),
    }
}