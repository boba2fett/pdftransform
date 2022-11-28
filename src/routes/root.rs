use rocket::{get, response::status::Conflict, serde::json::Json};

use crate::{
    consts::{NAME, VERSION},
    models::{AvgTimeModel, RootDto, RootLinks},
    persistence::{jobs_health, DbClient},
};

#[get("/")]
pub fn root_links<'a>() -> Json<RootDto<'a>> {
    Json(RootDto {
        version: VERSION,
        name: NAME,
        _links: RootLinks {
            transform: "/transform",
            preview: "/preview",
        },
    })
}

#[get("/health")]
pub async fn health(client: &DbClient) -> Result<Json<Vec<AvgTimeModel>>, Conflict<&'static str>> {
    match jobs_health(client).await {
        Ok(health) => Ok(Json(health)),
        Err(err) => Err(Conflict(Some(err))),
    }
}
