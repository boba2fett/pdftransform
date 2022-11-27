use rocket::{response::status::Conflict, serde::json::Json, post};
use rocket::fs::TempFile;
use crate::persistence::generate_30_alphanumeric;
use crate::{files::TempJobFileProvider, persistence::DbClient, preview::get_preview, models::PreviewResult};

#[post("/preview", format = "pdf", data="<file>")]
pub async fn preview_sync(db_client: &DbClient, mut file: TempFile<'_>) -> Result<Json<PreviewResult>, Conflict<&'static str>> {
    let job_id = generate_30_alphanumeric();
    let token = generate_30_alphanumeric();
    let file_provider = TempJobFileProvider::build(&job_id).await;
    let path = file_provider.get_path();
    file.persist_to(&path).await.map_err(|_| Conflict(Some("Could not get provided file.")))?;
    let result = get_preview(&db_client, &job_id, &token ,&path, &file_provider).await.map(|r| Json(r)).map_err(|err| Conflict(Some(err)));
    file_provider.clean_up().await;
    result
}