use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct IdName {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomField {
    pub id: u32,
    pub name: String,
    pub value: Option<serde_json::Value>,
}
