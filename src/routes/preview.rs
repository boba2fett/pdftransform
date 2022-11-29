use crate::consts::MAX_KIBIBYTES;
use crate::convert::process_preview_job;
use crate::models::{CreatePreviewJobDto, PreviewJobDto};
use crate::persistence::{create_new_preview_job, generate_30_alphanumeric, get_preview_job_dto};
use crate::{models::PreviewResult, persistence::DbClient, preview::get_preview};

use rocket::data::{Data, ToByteUnit};
use rocket::get;
use rocket::response::status::NotFound;
use rocket::{post, response::status::Conflict, serde::json::Json};

#[get("/preview/<job_id>?<token>")]
pub async fn preview_job(
    job_id: String,
    token: String,
    client: &DbClient,
) -> Result<Json<PreviewJobDto>, NotFound<&'static str>> {
    match get_preview_job_dto(client, &job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(NotFound(e)),
    }
}

pub fn preview_job_route(job_id: &str, token: &str) -> String {
    format!("/preview/{}?token={}", &job_id, token)
}

#[post("/preview", format = "json", data = "<create_job>")]
pub async fn create_preview_job(
    db_client: &DbClient,
    create_job: Json<CreatePreviewJobDto>,
) -> Result<Json<PreviewJobDto>, Conflict<&'static str>> {
    match create_new_preview_job(&db_client, create_job.0).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            let db_client_ref = db_client.0.clone();
            tokio::spawn(async move {
                process_preview_job(&db_client_ref, job_id, Some(job_model)).await
            });
            Ok(Json(job_dto))
        }
        Err(e) => Err(Conflict(Some(e))),
    }
}

#[post("/preview", format = "pdf", data = "<data>")]
pub async fn preview_sync(
    db_client: &DbClient,
    data: Data<'_>,
) -> Result<Json<PreviewResult>, Conflict<&'static str>> {
    let job_id = generate_30_alphanumeric();
    let token = generate_30_alphanumeric();
    let max = unsafe { MAX_KIBIBYTES };
    let bytes = data.open(max.kibibytes()).into_bytes().await.map_err(|_| Conflict(Some("Could not get provided file.")))?;
    if !bytes.is_complete() {
        return Err(Conflict(Some("Inputfile to big.")))
    }
    let result = get_preview(&db_client, &job_id, &token, &bytes)
        .await
        .map(|r| Json(r))
        .map_err(|err| Conflict(Some(err)));
    result
}
