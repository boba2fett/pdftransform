use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::{JobModel, ToIdJson};

pub type TransformResult = Vec<TransformDocumentResult>;
pub type TransformJobModel = JobModel<TransformInput, TransformResult>;

impl TransformJobModel {
    pub fn from_json_slice<'a>(slice: &'a [u8]) -> Result<Self, &'static str> {
       serde_json::from_slice(slice).map_err(|_| "job is not valid json")
    }
}

// impl ToIdJson for TransformJobModel {
//     fn to_json(&self) -> Result<String, &'static str> {
//         serde_json::to_string(self).map_err(|_| "job is not valid json")
//     }
//     fn get_id(&self) -> &str {
//         &self.id
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransformInput {
    pub source_files: Vec<SourceFile>,
    pub documents: Vec<Document>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceFile {
    pub id: String,
    pub uri: String,
    pub content_type: Option<String>,
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
