#[macro_use] extern crate rocket;
use std::env;

use pdftransform::{models::{RootDto, JobDto, CreateJobDto, RootLinks}, consts::{VERSION, NAME}, persistence::{create_new_job, get_job_dto, DbClient}, convert::process_job, files::get_job_result_file};
use rocket::{serde::json::Json, response::{status::{Conflict, NotFound}, self, stream::ByteStream}, fs::NamedFile, Request, Response, http::ContentType};
use rocket_db_pools::{Database};
use futures::StreamExt;

#[launch]
async fn rocket() -> _ {
    let mongo_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    env::set_var("ROCKET_DATABASES", format!("{{db={{url=\"{mongo_uri}\"}}}}"));
    
    json_env_logger::init();
    rocket::build()
        .attach(DbClient::init())
        .mount("/", routes![root])
        .mount("/convert", routes![job, create_job, file])
}

struct PdfFile(NamedFile);

impl<'r> rocket::response::Responder<'r, 'r> for PdfFile {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Content-Type", "application/pdf")
            .ok()
    }
}

#[get("/")]
fn root<'a>() -> Json<RootDto<'a>> {
    Json(RootDto {
        version: VERSION,
        name: NAME,
        _links: RootLinks {
            convert: "/convert"
        }
    })
}

#[get("/<job_id>?<token>")]
async fn job<'a>(job_id: String, token: String, client: &DbClient) -> Result<Json<JobDto>, NotFound<&'static str>> {
    match get_job_dto(client, &job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(NotFound(e)),
    }
}

#[get("/<job_id>/<file_id>?<token>")]
async fn file(db_client: &DbClient,job_id: String, file_id: String, token: String) -> Result<(ContentType, ByteStream![Vec<u8>]), NotFound<String>> {
    let mut stream = get_job_result_file(&db_client.0, &job_id, &token, &file_id).await.map_err(|e| NotFound(e.to_string()))?;
    Ok((ContentType::PDF, ByteStream!{
        while let Some(bytes) = stream.next().await {
            yield bytes;
        }
    }))
}

#[post("/", format = "json", data="<create_job>")]
async fn create_job(db_client: &DbClient, create_job: Json<CreateJobDto>) -> Result<Json<JobDto>, Conflict<&'static str>> {
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
