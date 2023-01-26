use actix_web::web::{Path, Query, Json, self};
use actix_web::{get, post, HttpResponse};

use crate::consts::MAX_KIBIBYTES;
use crate::convert::process_preview_job;
use crate::models::{CreatePreviewJobDto, PreviewJobDto};
use crate::persistence::{create_new_preview_job, generate_30_alphanumeric, get_preview_job_dto};
use crate::routes::Token;
use crate::{models::PreviewResult, preview::get_preview};

#[get("/preview/{job_id}")]
pub async fn preview_job(job_id: Path<String>, token: Query<Token>) -> HttpResponse {
    match get_preview_job_dto(&job_id, token).await {
        Ok(job_dto) => HttpResponse::Ok().json(job_dto),
        Err(err) => HttpResponse::NotFound().json(err),
    }
}

pub fn preview_job_route(job_id: &str, token: &str) -> String {
    format!("/preview/{}?token={}", &job_id, token)
}

#[post("/preview")]
pub async fn create_preview_job(create_job: Json<CreatePreviewJobDto>) -> HttpResponse {
    match create_new_preview_job(&db_client, create_job.0).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move { process_preview_job(job_id, Some(job_model)).await });
            Ok(Json(job_dto))
        }
        Err(e) => Err(Conflict(Some(e))),
    }
}

#[post("/preview")]
pub async fn preview_sync(data: web::Bytes) -> HttpResponse {
    let job_id = generate_30_alphanumeric();
    let token = generate_30_alphanumeric();
    let max = unsafe { MAX_KIBIBYTES };
    let bytes = data.open(max.kibibytes()).into_bytes().await.map_err(|_| Conflict(Some("Could not get provided file.")))?;
    if !bytes.is_complete() {
        return Err(Conflict(Some("Inputfile to big.")));
    }
    let result = get_preview(&job_id, &token, bytes.value).await.map(|r| Json(r)).map_err(|err| Conflict(Some(err)));
    result
}
