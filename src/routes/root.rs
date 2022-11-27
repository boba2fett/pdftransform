use rocket::{get, serde::json::Json};

use crate::{
    consts::{NAME, VERSION},
    models::{RootDto, RootLinks},
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
