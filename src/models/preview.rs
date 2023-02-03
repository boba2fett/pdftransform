use crate::serialize::base64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePreviewJobDto {
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
