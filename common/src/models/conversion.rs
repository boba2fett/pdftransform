use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRequestRef<'a> {
    pub job_id: &'a str,
    pub source_uri: &'a str,
    pub response_subject: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRequest {
    pub job_id: String,
    pub source_uri: String,
    pub response_stream: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResultRef<'a> {
    pub job_id: &'a str,
    pub result_uri: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResult {
    pub job_id: String,
    pub result_uri: String,
}