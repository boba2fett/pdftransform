use bson::oid::ObjectId;
use mime::Mime;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use std::{path::Path, str::FromStr};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub filename: String,
    pub mime_type: Option<String>,
    pub token: String,
    pub upload_date: DateTime,
    pub md5: String,
    pub length: usize,
    pub chunk_size: usize,
}

impl FileModel {
    pub fn get_content_type(&self) -> Mime {
        if let Some(mime_type) = &self.mime_type {
            if let Some(content_type) = Mime::from_str(mime_type).ok() {
                return content_type;
            }
        }
        if let Some(extension) = Path::new(&self.filename).extension() {
            if let Some(extension) = extension.to_str() {
                return match extension {
                    "pdf" => mime::APPLICATION_PDF,
                    "png" => mime::IMAGE_PNG,
                    "jpg" => mime::IMAGE_JPEG,
                    "jpeg" => mime::IMAGE_JPEG,
                    "bmp" => mime::IMAGE_BMP,
                    _ => mime::APPLICATION_OCTET_STREAM,
                };
            }
        }
        mime::APPLICATION_OCTET_STREAM
    }
}
