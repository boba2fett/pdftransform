use crate::{files::get_result_file, persistence::DbClient};
use futures::StreamExt;
use rocket::{
    get,
    http::ContentType,
    response::{status::NotFound, stream::ByteStream},
};

#[get("/file/<file_id>?<token>")]
pub async fn file(db_client: &DbClient, file_id: String, token: String) -> Result<(ContentType, ByteStream![Vec<u8>]), NotFound<String>> {
    let file = get_result_file(&db_client.0, &token, &file_id).await.map_err(|e| NotFound(e.to_string()))?;
    let content_type = file.0;
    let mut stream = file.1;
    Ok((
        content_type,
        ByteStream! {
            while let Some(bytes) = stream.next().await {
                yield bytes;
            }
        },
    ))
}

pub fn file_route(file_id: &str, token: &str) -> String {
    format!("/file/{}?token={}", file_id, token)
}
