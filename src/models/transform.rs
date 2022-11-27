use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransformJobDto {
    pub callback_uri: Option<String>,
    pub documents: Vec<Document>,
    pub source_files: Vec<SourceFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceFile {
    pub id: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub source_file: String,
    pub start_page_number: Option<u16>,
    pub end_page_number: Option<u16>,
    pub rotation: Option<Rotation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub source_file: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum Rotation {
    N270 = -270,
    N180 = -180,
    N90 = -90,
    P0 = 0,
    P90 = 90,
    P180 = 180,
    P270 = 270,
}

impl Rotation {
    pub fn as_degrees(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub parts: Vec<Part>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransformDocumentResult {
    pub id: String,
    pub download_url: String,
}
