use serde::{Deserialize, Serialize};

use super::JobStatus;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootDto {
    pub version: &'static str,
    pub name: &'static str,
    #[serde(rename = "_links")]
    pub _links: RootLinks,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootLinks {
    pub transform: &'static str,
    pub preview: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvgTimeModel {
    status: JobStatus,
    avg_time_millis: f64,
    count: usize,
}
