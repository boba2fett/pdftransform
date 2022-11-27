use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootDto<'a> {
    pub version: &'a str,
    pub name: &'a str,
    #[serde(rename = "_links")]
    pub _links: RootLinks<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootLinks<'a> {
    pub transform: &'a str,
    pub preview: &'a str,
}