#[macro_use] extern crate rocket;
use std::sync::Arc;

use pdfium_render::prelude::Pdfium;
use pdftransform::{models::{RootDto, JobDto, CreateJobDto}, consts::{VERSION, NAME}, persistence::{save_job, get_job_dto}, convert::process_job, transform::{PDFIUM_PTR, get_pdfium}};
use rocket::{serde::json::Json, response::status::{Conflict, NotFound}};

#[launch]
async fn rocket() -> _ {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library()).unwrap());
    let mut pdfium_arc = Arc::new(pdfium);
    unsafe{
        let pdfium_arc = &mut pdfium_arc as *mut Arc<Pdfium>;
        PDFIUM_PTR.store(pdfium_arc, std::sync::atomic::Ordering::Relaxed);
    }
    
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
    match get_job_dto(&job_id).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(NotFound(e)),
    }
}

#[post("/", format = "json", data="<create_job>")]
async fn create_job(create_job: Json<CreateJobDto>) -> Result<Json<JobDto>, Conflict<&'static str>> {
    match save_job(create_job.0).await {
        Ok(job_dto) => {
            let job_id = job_dto.id.clone();
            let pdfium = Arc::clone(get_pdfium());
            tokio::spawn(async {
                async {
                    process_job(job_id, pdfium).await
                }
            });
            Ok(Json(job_dto))
        },
        Err(e) => Err(Conflict(Some(e))),
    }
}