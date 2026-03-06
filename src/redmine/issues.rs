use serde::{Deserialize, Serialize};

use super::RedmineClient;
use super::common::{CustomField, IdName};
use crate::error::RedmineError;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Journal {
    pub id: u32,
    pub user: IdName,
    pub notes: Option<String>,
    pub created_on: String,
    pub details: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attachment {
    pub id: u32,
    pub filename: String,
    pub filesize: u64,
    pub content_type: Option<String>,
    pub content_url: String,
    pub author: IdName,
    pub created_on: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Issue {
    pub id: u32,
    pub project: IdName,
    pub tracker: IdName,
    pub status: IdName,
    pub priority: IdName,
    pub author: IdName,
    pub assigned_to: Option<IdName>,
    pub parent: Option<serde_json::Value>,
    pub subject: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub done_ratio: u32,
    pub estimated_hours: Option<f64>,
    pub custom_fields: Option<Vec<CustomField>>,
    pub created_on: String,
    pub updated_on: String,
    pub closed_on: Option<String>,
    pub journals: Option<Vec<Journal>>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Deserialize)]
pub struct IssueResponse {
    pub issue: Issue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IssueListResponse {
    pub issues: Vec<Issue>,
    pub total_count: u32,
    pub offset: u32,
    pub limit: u32,
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct IssueCreate {
    pub project_id: u32,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_issue_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct IssueUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_ratio: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f64>,
}

// ── API methods ───────────────────────────────────────────────────────────────

impl RedmineClient {
    /// GET /issues.json — list issues with optional filters.
    pub async fn list_issues(
        &self,
        api_key: &str,
        project_id: Option<&str>,
        status_id: Option<&str>,
        tracker_id: Option<u32>,
        assigned_to_id: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<IssueListResponse, RedmineError> {
        let mut req = self.get("/issues.json", api_key);

        if let Some(v) = project_id {
            req = req.query(&[("project_id", v)]);
        }
        if let Some(v) = status_id {
            req = req.query(&[("status_id", v)]);
        }
        if let Some(v) = tracker_id {
            req = req.query(&[("tracker_id", v.to_string().as_str())]);
        }
        if let Some(v) = assigned_to_id {
            req = req.query(&[("assigned_to_id", v)]);
        }
        req = req.query(&[("limit", limit.unwrap_or(25).to_string())]);
        if let Some(v) = offset {
            req = req.query(&[("offset", v.to_string())]);
        }

        let resp = req.send().await?;
        let body = Self::check_response(resp).await?;
        serde_json::from_str(&body).map_err(|e| RedmineError::UnexpectedResponse(e.to_string()))
    }

    /// GET /issues/{id}.json — get a single issue.
    pub async fn get_issue(
        &self,
        api_key: &str,
        id: u32,
        include: Option<&str>,
    ) -> Result<Issue, RedmineError> {
        let mut req = self.get(&format!("/issues/{id}.json"), api_key);
        if let Some(inc) = include {
            req = req.query(&[("include", inc)]);
        }

        let resp = req.send().await?;
        let body = Self::check_response(resp).await?;
        let parsed: IssueResponse = serde_json::from_str(&body)
            .map_err(|e| RedmineError::UnexpectedResponse(e.to_string()))?;
        Ok(parsed.issue)
    }

    /// POST /issues.json — create a new issue.
    pub async fn create_issue(
        &self,
        api_key: &str,
        issue: IssueCreate,
    ) -> Result<Issue, RedmineError> {
        let body = serde_json::json!({ "issue": issue });
        let resp = self
            .post("/issues.json", api_key)
            .json(&body)
            .send()
            .await?;
        let text = Self::check_response(resp).await?;
        let parsed: IssueResponse = serde_json::from_str(&text)
            .map_err(|e| RedmineError::UnexpectedResponse(e.to_string()))?;
        Ok(parsed.issue)
    }

    /// PUT /issues/{id}.json — update an issue.
    pub async fn update_issue(
        &self,
        api_key: &str,
        id: u32,
        issue: IssueUpdate,
    ) -> Result<(), RedmineError> {
        let body = serde_json::json!({ "issue": issue });
        let resp = self
            .put(&format!("/issues/{id}.json"), api_key)
            .json(&body)
            .send()
            .await?;
        Self::check_response(resp).await?;
        Ok(())
    }

    /// DELETE /issues/{id}.json — delete an issue.
    pub async fn delete_issue(&self, api_key: &str, id: u32) -> Result<(), RedmineError> {
        let resp = self
            .delete(&format!("/issues/{id}.json"), api_key)
            .send()
            .await?;
        Self::check_response(resp).await?;
        Ok(())
    }
}
