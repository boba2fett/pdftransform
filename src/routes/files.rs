use rocket::{get, response::{stream::ByteStream, status::NotFound}};
use futures::StreamExt;
use crate::{files::{get_result_file}, persistence::DbClient};

#[get("/file/<file_id>?<token>")]
pub async fn file(db_client: &DbClient, file_id: String, token: String) -> Result<ByteStream![Vec<u8>], NotFound<String>> {
    let mut stream = get_result_file(&db_client.0, &token, &file_id).await.map_err(|e| NotFound(e.to_string()))?;
    Ok(ByteStream!{
        while let Some(bytes) = stream.next().await {
            yield bytes;
        }
    })
}

pub fn file_route(file_id: &str, token: &str) -> String {
    format!("/file/{}?token={}", file_id, token)
}
