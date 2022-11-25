use rocket::{get, serde::json::Json};

use crate::{consts::{VERSION, NAME}, models::{RootLinks, RootDto}};

#[get("/")]
pub fn root<'a>() -> Json<RootDto<'a>> {
    Json(RootDto {
        version: VERSION,
        name: NAME,
        _links: RootLinks {
            convert: "/convert",
            preview: "/preview",
        }
    })
}