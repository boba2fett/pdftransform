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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub status: JobStatus,
    pub source_uri: String,
    pub callback_uri: String,
    pub source_mime_type: String,
    pub destination_mime_type: String,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum JobStatus {
    WaitingForFile = 0,
    InProgress = 1,
    Finished = 2,
    Error = 3,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobDto<'a> {
    pub source_uri: &'a str,
    pub callback_uri: &'a str,
    pub source_mime_type: &'a str,
    pub destination_mime_type: &'a str,
}