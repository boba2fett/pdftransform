use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootDto<'a> {
    pub version: &'a str,
    pub name: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDto {
    pub id: String,
    pub status: JobStatus,
    pub results: Vec<DocumentResult>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub status: JobStatus,
    pub callback_uri: String,
    pub source_files: Vec<SourceFile>,
    pub documents: Vec<Document>,
    pub results: Vec<DocumentResult>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum JobStatus {
    InProgress = 0,
    Finished = 1,
    Error = 2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobDto {
    pub callback_uri: String,
    pub documents: Vec<Document>,
    pub source_files: Vec<SourceFile>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFile {
    pub source_file_id: String,
    pub source_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub source_file_id: String,
    pub start_page_number: Option<u16>,
    pub end_page_number: Option<u16>,
    pub rotation: Rotation,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub binaries: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResult {
    pub id: String,
    pub download_url: String,
}
