use crate::models::{PreviewResult, PreviewJobModel};
use serde::{Deserialize, Serialize};

use super::{JobDto, GetSelfRoute};

pub type PreviewJobDto = JobDto<PreviewResult>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePreviewJobDto {
    pub callback_uri: Option<String>,
    pub source_uri: String,
    pub source_mime_type: Option<String>,
    pub pdf: Option<bool>,
    pub png: Option<bool>,
    pub text: Option<bool>,
    pub attachments: Option<bool>,
    pub signatures: Option<bool>,
    pub start_page_number: Option<u16>,
    pub end_page_number: Option<u16>,
}

impl GetSelfRoute for PreviewJobModel {
    fn get_self_route(&self) -> String {
        format!("/preview/{}?token={}", self.id, self.token)
    }
}