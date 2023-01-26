use rocket::{
    get, post,
    response::status::{Conflict, NotFound},
    serde::json::Json,
};

use crate::{
    convert::process_transform_job,
    models::{CreateTransformJobDto, TransformJobDto},
    persistence::{create_new_transform_job, get_transform_job_dto},
};

#[get("/transform/<job_id>?<token>")]
pub async fn transform_job(job_id: String, token: String) -> Result<Json<TransformJobDto>, NotFound<&'static str>> {
    match get_transform_job_dto(&job_id, token).await {
        Ok(job_dto) => Ok(Json(job_dto)),
        Err(e) => Err(NotFound(e)),
    }
}

pub fn transform_job_route(job_id: &str, token: &str) -> String {
    format!("/transform/{}?token={}", &job_id, token)
}

#[post("/transform", format = "json", data = "<create_job>")]
pub async fn create_transform_job(create_job: Json<CreateTransformJobDto>) -> Result<Json<TransformJobDto>, Conflict<&'static str>> {
    match create_new_transform_job(create_job.0).await {
        Ok((job_dto, job_model)) => {
            let job_id = job_dto.id.clone();
            tokio::spawn(async move { process_transform_job(job_id, Some(job_model)).await });
            Ok(Json(job_dto))
        }
        Err(e) => Err(Conflict(Some(e))),
    }
}
