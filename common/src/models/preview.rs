use crate::util::serialize::base64;
use serde::{Deserialize, Serialize};

use super::{JobDto, JobModel};

pub type PreviewJobDto = JobDto<Option<PreviewResult>>;
pub type PreviewJobModel = JobModel<PreviewData>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewData {
    pub source_uri: Option<String>,
    pub source_mime_type: String,
    pub result: Option<PreviewResult>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePreviewJobModel {
    pub callback_uri: Option<String>,
    pub source_uri: String,
    pub source_mime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    pub page_count: usize,
    pub pages: Vec<PreviewPageResult>,
    pub attachments: Vec<PreviewAttachmentResult>,
    pub signatures: Vec<PreviewSignature>,
    pub protected: bool,
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
