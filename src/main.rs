#[macro_use] extern crate rocket;
use std::{sync::{Arc, atomic::{AtomicPtr, Ordering}}};

use pdfium_render::prelude::{Pdfium, PdfiumLibraryBindings};
use pdftransform::{models::{RootDto, JobDto, CreateJobDto, PdfIum}, consts::{VERSION, NAME}, persistence::{save_job, get_job_dto}, convert::process_job};
use rocket::{serde::json::Json, response::status::{Conflict, NotFound}, State};

#[launch]
async fn rocket() -> _ {
    // let pdfium = Pdfium::new(
    //     Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
    //         .or_else(|_| Pdfium::bind_to_system_library()).unwrap());

    rocket::build()
        // .manage(unsafe {PdfIum { pdfium: AtomicPtr::new(&mut Arc::new(pdfium))}})
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
            tokio::spawn(async move {
                process_job(job_id)
            });
            Ok(Json(job_dto))
        },
        Err(e) => Err(Conflict(Some(e))),
    }
}