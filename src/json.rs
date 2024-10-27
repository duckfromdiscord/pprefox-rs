use serde_derive::*;
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct ExtensionRequest {
    pub uuid: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_id: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Theme {
    pub name: String,
    pub id: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct ExtensionResponse {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub themes: Option<Vec<Theme>>,
}
