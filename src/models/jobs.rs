use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use mongodb::bson::DateTime;

use super::{transform::{SourceFile, Document, TransformDocumentResult}, PreviewResult};

pub type TransformJobDto = JobDto<Vec<TransformDocumentResult>>;
pub type PreviewJobDto = JobDto<PreviewResult>;

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum JobStatus {
    InProgress = 0,
    Finished = 1,
    Error = 2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDto<Result> {
    pub id: String,
    pub status: JobStatus,
    pub message: Option<String>,
    pub result: Result,
    #[serde(rename = "_links")]
    pub _links: JobLinks,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLinks {
    #[serde(rename = "self")]
    pub _self: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewJobModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub token: String,
    pub created: DateTime,
    pub status: JobStatus,
    pub message: Option<String>,
    pub callback_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransformJobModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub token: String,
    pub created: DateTime,
    pub status: JobStatus,
    pub message: Option<String>,
    pub callback_uri: Option<String>,
    pub source_files: Vec<SourceFile>,
    pub documents: Vec<Document>,
    pub results: Vec<TransformDocumentResult>,
}
