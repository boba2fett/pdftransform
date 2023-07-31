use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use chrono::serde::ts_seconds;

use crate::util::serialize::Serializable;

use super::ToIdJson;

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum JobStatus {
    Pending = 0,
    InProgress = 1,
    Finished = 2,
    Error = 3,
}

pub type BaseJobModel = JobModel<(), ()>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobModel<InputType, ResultType> {
    pub id: String,
    pub token: String,
    #[serde(with = "ts_seconds")]
    pub created: DateTime<Utc>,
    pub status: JobStatus,
    pub message: Option<String>,
    pub callback_uri: Option<String>,
    pub input: InputType,
    pub result: Option<ResultType>,
}

impl<InputType, ResultType> ToIdJson for JobModel<InputType, ResultType> where InputType: Serializable, ResultType: Serializable {
    fn to_json(&self) -> Result<String, &'static str> {
        serde_json::to_string(self).map_err(|_| "job is not valid json")
    }
    fn get_id(&self) -> &str {
        &self.id
    }
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
pub struct RefIdModel<'a> {
    pub id: &'a str,
}
