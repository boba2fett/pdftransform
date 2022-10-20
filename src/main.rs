#[macro_use] extern crate rocket;
use pdf::{models::{RootDto, JobDto, CreateJobDto}, consts::{VERSION, NAME}, persistence::{save_job, get_job}};
use rocket::{serde::json::Json, response::status::{Conflict, NotFound}};

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![root])
        .mount("/convert", routes![job, create_job])
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
    match get_job(job_id).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(NotFound(e)),
    }
}

#[post("/", format = "json", data="<create_job>")]
async fn create_job<'a>(create_job: Json<CreateJobDto<'a>>) -> Result<Json<JobDto>, Conflict<&str>> {
    match save_job(create_job.0).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(Conflict(Some(e))),
    }
}