use serde_derive::*;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone, Default)]
pub struct TabQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "currentWindow")]
    pub current_window: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "windowId")]
    pub window_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "windowType")]
    pub window_type: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct ExtensionRequest {
    pub uuid: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<TabQuery>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u16>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Theme {
    pub name: String,
    pub id: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Tab {
    pub active: bool,
    pub id: u16,
    pub index: u16,
    pub pinned: bool,
    pub title: String,
    pub url: String,
    #[serde(rename = "windowId")]
    pub window_id: u16,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct ExtensionResponse {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub themes: Option<Vec<Theme>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<Tab>>,
}
