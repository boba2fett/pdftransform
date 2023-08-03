use crate::util::serialize::base64;
use serde::{Deserialize, Serialize};

use super::JobModel;

pub type PreviewJobModel = JobModel<PreviewInput, PreviewResult>;

impl PreviewJobModel {
    pub fn from_json_slice<'a>(slice: &'a [u8]) -> Result<Self, &'static str> {
       serde_json::from_slice(slice).map_err(|_| "job is not valid json")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewInput {
    pub source_uri: String,
    pub source_mime_type: Option<String>,
    pub pdf: bool,
    pub png: bool,
    pub attachments: bool,
    pub signatures: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    pub page_count: usize,
    pub pages: Option<Vec<PreviewPageResult>>,
    pub attachments: Option<Vec<PreviewAttachmentResult>>,
    pub signatures: Option<Vec<PreviewSignature>>,
    pub protected: bool,
    pub pdf: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSignature {
    pub signing_date: Option<String>,
    pub reason: Option<String>,
    #[serde(with = "base64")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPageResult {
    pub download_url: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewAttachmentResult {
    pub name: String,
    pub download_url: String,
}
