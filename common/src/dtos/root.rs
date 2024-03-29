use serde::{Deserialize, Serialize};

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
