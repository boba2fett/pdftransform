use serde::{Deserialize, Serialize};

use crate::models::{TransformResult, Document, SourceFile, TransformJobModel};

use super::{JobDto, GetSelfRoute};

pub type TransformJobDto = JobDto<TransformResult>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransformJobDto {
    pub callback_uri: Option<String>,
    pub documents: Vec<Document>,
    pub source_files: Vec<SourceFile>,
}

impl GetSelfRoute for TransformJobModel {
    fn get_self_route(&self) -> String {
        format!("/transform/{}?token={}", self.id, self.token)
    }
}