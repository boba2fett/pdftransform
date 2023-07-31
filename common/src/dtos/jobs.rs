use serde::{Deserialize, Serialize};

use crate::models::JobStatus;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDto<ResultType> {
    pub id: String,
    pub status: JobStatus,
    pub message: Option<String>,
    pub result: Option<ResultType>,
    #[serde(rename = "_links")]
    pub _links: JobLinks,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLinks {
    #[serde(rename = "self")]
    pub _self: String,
}
