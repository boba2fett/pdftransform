use rocket::{response::status::Conflict, serde::json::Json, post};
use rocket::fs::TempFile;
use crate::{files::TempJobFileProvider, persistence::DbClient, preview::get_preview, models::PreviewResult};

#[post("/preview", format = "pdf", data="<file>")]
pub async fn preview_sync(db_client: &DbClient, mut file: TempFile<'_>) -> Result<Json<PreviewResult>, Conflict<&'static str>> {
    let path = TempJobFileProvider::get_one();
    file.persist_to(&path).await.map_err(|_| Conflict(Some("Could not get provided file.")))?;
    get_preview(&db_client, path).await.map(|r| Json(r)).map_err(|err| Conflict(Some(err)))
}