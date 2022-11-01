#[macro_use] extern crate rocket;

use pdftransform::{models::{RootDto, JobDto, CreateJobDto}, consts::{VERSION, NAME}, persistence::{create_new_job, get_job_dto}, convert::process_job, files::get_job_files};
use rocket::{serde::json::Json, response::status::{Conflict, NotFound}};
use tokio::fs::File;

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![root])
        .mount("/convert", routes![job, create_job, file])
}

#[get("/")]
fn root<'a>() -> Json<RootDto<'a>> {
    Json(RootDto {
        version: VERSION,
        name: NAME,
    })
}

#[get("/<job_id>")]
async fn job<'a>(job_id: String) -> Result<Json<JobDto>, NotFound<&'static str>> {
    match get_job_dto(&job_id).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(NotFound(e)),
    }
}

#[get("/<job_id>/<file_id>")]
async fn file(job_id: String, file_id: String) -> Option<File> {
    File::open(get_job_files(&job_id).get_path(&file_id)).await.ok()
}

#[post("/", format = "json", data="<create_job>")]
async fn create_job(create_job: Json<CreateJobDto>) -> Result<Json<JobDto>, Conflict<&'static str>> {
    match create_new_job(create_job.0).await {
        Ok(job_dto) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move {
                dbg!("asd");
                process_job(job_id).await
            });
            Ok(Json(job_dto))
        },
        Err(e) => Err(Conflict(Some(e))),
    }
}