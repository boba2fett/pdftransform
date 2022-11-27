use std::{path::Path, str::FromStr};

use bson::oid::ObjectId;
use rocket::http::ContentType;
use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use mongodb::bson::DateTime;

use super::{transform::{SourceFile, Document, TransformDocumentResult}, PreviewResult};

pub type TransformJobDto = JobDto<Vec<TransformDocumentResult>>;
pub type PreviewJobDto = JobDto<Option<PreviewResult>>;

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum JobStatus {
    InProgress = 0,
    Finished = 1,
    Error = 2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDto<ResultType> {
    pub id: String,
    pub status: JobStatus,
    pub message: Option<String>,
    pub result: ResultType,
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
    pub source_uri: Option<String>,
    pub result: Option<PreviewResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DummyModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
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
    pub result: Vec<TransformDocumentResult>,
}

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
    pub fn get_content_type(&self) -> ContentType {
        if let Some(mime_type) = &self.mime_type {
            if let Some(content_type) = ContentType::from_str(mime_type).ok() {
                return content_type;
            }
        }
        if let Some(extension) = Path::new(&self.filename).extension() {
            if let Some(extension) = extension.to_str() {
                if let Some(extension_content_type) = ContentType::from_extension(extension) {
                    return extension_content_type;
                }
            }
        }
        ContentType::Any
    }
}