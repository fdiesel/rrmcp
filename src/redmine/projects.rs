use serde::{Deserialize, Serialize};

use super::common::IdName;
use crate::{error::RedmineError, redmine::RedmineClient};

/// A Redmine project as returned by the API.
#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    /// 1 = active, 5 = archived, 9 = closed
    pub status: u32,
    pub is_public: bool,
    pub parent: Option<IdName>,
    pub created_on: String,
    pub updated_on: String,
    // Optional includes
    pub trackers: Option<Vec<IdName>>,
    pub issue_categories: Option<Vec<IdName>>,
    pub enabled_modules: Option<Vec<IdName>>,
    pub time_entry_activities: Option<Vec<IdName>>,
    pub issue_custom_fields: Option<Vec<IdName>>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectResponse {
    pub project: Project,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
    pub total_count: u32,
    pub offset: u32,
    pub limit: u32,
}

// ── Request types ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProjectCreate {
    pub name: String,
    pub identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_members: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_assigned_to_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_version_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_ids: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_module_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_custom_field_ids: Option<Vec<u32>>,
}

#[derive(Debug, Serialize)]
pub struct ProjectUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_members: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_assigned_to_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_version_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_ids: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_module_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_custom_field_ids: Option<Vec<u32>>,
}

impl RedmineClient {
    pub async fn get_projects(&self, api_key: &str) -> Result<ProjectListResponse, RedmineError> {
        let res = self.get("projects.json", api_key).send().await?;
        let text = Self::check_response(res).await?;
        let parsed: ProjectListResponse = serde_json::from_str(&text)
            .map_err(|e| RedmineError::UnexpectedResponse(e.to_string()))?;
        Ok(parsed)
    }
}
