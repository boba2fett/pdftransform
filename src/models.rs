use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use mongodb::bson::DateTime;
use crate::serialize::base64;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootDto<'a> {
    pub version: &'a str,
    pub name: &'a str,
    #[serde(rename = "_links")]
    pub _links: RootLinks<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootLinks<'a> {
    pub convert: &'a str,
    pub preview: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    pub page_count: usize,
    pub pages: Vec<PreviewPageResult>,
    pub signatures: Vec<Signature>,
    pub protected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub signing_date: Option<String>,
    pub reason: Option<String>,
    #[serde(with="base64")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPageResult {
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewAttachmentResult {
    pub name: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub token: String,
    pub created: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDto {
    pub id: String,
    pub status: JobStatus,
    pub message: Option<String>,
    pub results: Vec<DocumentResult>,
    #[serde(rename = "_links")]
    pub _links: ConvertLinks,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertLinks {
    #[serde(rename = "self")]
    pub _self: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub status: JobStatus,
    pub callback_uri: Option<String>,
    pub source_files: Vec<SourceFile>,
    pub documents: Vec<Document>,
    pub results: Vec<DocumentResult>,
    pub message: Option<String>,
    pub token: String,
    pub created: DateTime,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum JobStatus {
    InProgress = 0,
    Finished = 1,
    Error = 2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobDto {
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
    pub fn as_degrees(&self) -> i32
    {
        *self as i32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub binaries: Vec<Part>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResult {
    pub id: String,
    pub download_url: String,
}
