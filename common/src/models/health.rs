use serde::{Deserialize, Serialize};

use super::JobStatus;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvgTimeModel {
    pub status: JobStatus,
    pub avg_time_millis: f64,
    pub count: usize,
}
