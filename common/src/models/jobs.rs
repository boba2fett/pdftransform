use bson::oid::ObjectId;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum JobStatus {
    InProgress = 0,
    Finished = 1,
    Error = 2,
}

pub type BaseJobModel = JobModel<()>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobModel<DataType> {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub token: String,
    pub created: DateTime,
    pub status: JobStatus,
    pub message: Option<String>,
    pub callback_uri: Option<String>,
    pub data: DataType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BaseJobDto {
    pub id: String,
    pub callback_uri: Option<String>,
    pub created: DateTime,
    pub status: JobStatus,
    pub message: Option<String>,
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
pub struct IdModel {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DummyJobModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
}
