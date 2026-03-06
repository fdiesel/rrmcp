use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::redmine::{
    RedmineClient,
    issues::{IssueCreate, IssueUpdate},
};

// ── Server struct ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct RedmineServer {
    client: Arc<RedmineClient>,
    api_key: String,
    tool_router: ToolRouter<Self>,
}

impl RedmineServer {
    pub fn new(client: RedmineClient, api_key: String) -> Self {
        let mut router = Self::tool_router();
        router.merge(Self::project_tool_router_pub());
        Self {
            client: Arc::new(client),
            api_key,
            tool_router: router,
        }
    }
}

// ── Tool input parameter types ────────────────────────────────────────────────

/// Parameters for listing Redmine issues.
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListIssuesInput {
    /// Filter by project identifier or numeric ID (e.g. "my-project" or "42").
    pub project_id: Option<String>,
    /// Filter by status: "open", "closed", "*", or a numeric status ID.
    pub status_id: Option<String>,
    /// Filter by tracker ID.
    pub tracker_id: Option<u32>,
    /// Filter by assignee: numeric user ID or "me".
    pub assigned_to_id: Option<String>,
    /// Maximum number of results to return (default 25, max 100).
    pub limit: Option<u32>,
    /// Number of results to skip for pagination.
    pub offset: Option<u32>,
}

/// Parameters for retrieving a single Redmine issue.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssueInput {
    /// Numeric issue ID.
    pub issue_id: u32,
    /// Comma-separated list of associated data to include.
    /// Valid values: journals, attachments, changesets, watchers, children, relations.
    pub include: Option<String>,
}

/// Parameters for creating a Redmine issue.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueInput {
    /// ID of the project to create the issue in.
    pub project_id: u32,
    /// Issue subject/title.
    pub subject: String,
    /// Issue description (Markdown supported if Redmine is configured for it).
    pub description: Option<String>,
    /// Tracker ID (Bug, Feature, etc.).
    pub tracker_id: Option<u32>,
    /// Status ID.
    pub status_id: Option<u32>,
    /// Priority ID.
    pub priority_id: Option<u32>,
    /// User ID to assign the issue to.
    pub assigned_to_id: Option<u32>,
    /// Parent issue ID for creating a sub-task.
    pub parent_issue_id: Option<u32>,
    /// Start date in YYYY-MM-DD format.
    pub start_date: Option<String>,
    /// Due date in YYYY-MM-DD format.
    pub due_date: Option<String>,
    /// Estimated hours.
    pub estimated_hours: Option<f64>,
}

/// Parameters for updating a Redmine issue.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateIssueInput {
    /// Numeric issue ID to update.
    pub issue_id: u32,
    /// New subject/title.
    pub subject: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New tracker ID.
    pub tracker_id: Option<u32>,
    /// New status ID.
    pub status_id: Option<u32>,
    /// New priority ID.
    pub priority_id: Option<u32>,
    /// New assignee user ID.
    pub assigned_to_id: Option<u32>,
    /// Journal note to add with this update.
    pub notes: Option<String>,
    /// New done ratio (0–100).
    pub done_ratio: Option<u32>,
    /// New start date in YYYY-MM-DD format.
    pub start_date: Option<String>,
    /// New due date in YYYY-MM-DD format.
    pub due_date: Option<String>,
    /// New estimated hours.
    pub estimated_hours: Option<f64>,
}

/// Parameters for deleting a Redmine issue.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteIssueInput {
    /// Numeric issue ID to delete.
    pub issue_id: u32,
}

// ── Tool implementations ──────────────────────────────────────────────────────

mod projects;

#[tool_router(router = tool_router)]
impl RedmineServer {
    /// List Redmine issues with optional filters. Returns paginated results.
    /// Stable Redmine API (since 1.0).
    #[tool(
        description = "List Redmine issues with optional filters for project, status, tracker, and assignee. Returns paginated JSON results."
    )]
    pub async fn list_issues(&self, Parameters(input): Parameters<ListIssuesInput>) -> String {
        let result = self
            .client
            .list_issues(
                &self.api_key,
                input.project_id.as_deref(),
                input.status_id.as_deref(),
                input.tracker_id,
                input.assigned_to_id.as_deref(),
                input.limit,
                input.offset,
            )
            .await;

        match result {
            Ok(resp) => serde_json::to_string_pretty(&resp).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Get a single Redmine issue by numeric ID.
    /// Stable Redmine API (since 1.0).
    #[tool(
        description = "Get a single Redmine issue by ID. Optionally include journals (comments), attachments, watchers, relations, children, or changesets."
    )]
    pub async fn get_issue(&self, Parameters(input): Parameters<GetIssueInput>) -> String {
        let result = self
            .client
            .get_issue(&self.api_key, input.issue_id, input.include.as_deref())
            .await;

        match result {
            Ok(issue) => serde_json::to_string_pretty(&issue).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Create a new Redmine issue.
    /// Stable Redmine API (since 1.0).
    #[tool(
        description = "Create a new Redmine issue in the specified project. Returns the created issue as JSON."
    )]
    pub async fn create_issue(&self, Parameters(input): Parameters<CreateIssueInput>) -> String {
        let create = IssueCreate {
            project_id: input.project_id,
            subject: input.subject,
            description: input.description,
            tracker_id: input.tracker_id,
            status_id: input.status_id,
            priority_id: input.priority_id,
            assigned_to_id: input.assigned_to_id,
            parent_issue_id: input.parent_issue_id,
            start_date: input.start_date,
            due_date: input.due_date,
            estimated_hours: input.estimated_hours,
        };

        match self.client.create_issue(&self.api_key, create).await {
            Ok(issue) => serde_json::to_string_pretty(&issue).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Update an existing Redmine issue.
    /// Stable Redmine API (since 1.0).
    #[tool(
        description = "Update fields of an existing Redmine issue. Only provided fields are changed. Use 'notes' to add a journal comment."
    )]
    pub async fn update_issue(&self, Parameters(input): Parameters<UpdateIssueInput>) -> String {
        let issue_id = input.issue_id;
        let update = IssueUpdate {
            subject: input.subject,
            description: input.description,
            tracker_id: input.tracker_id,
            status_id: input.status_id,
            priority_id: input.priority_id,
            assigned_to_id: input.assigned_to_id,
            notes: input.notes,
            done_ratio: input.done_ratio,
            start_date: input.start_date,
            due_date: input.due_date,
            estimated_hours: input.estimated_hours,
        };

        match self
            .client
            .update_issue(&self.api_key, issue_id, update)
            .await
        {
            Ok(()) => format!("Issue #{issue_id} updated successfully."),
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Delete a Redmine issue permanently.
    /// Stable Redmine API (since 1.0).
    #[tool(description = "Permanently delete a Redmine issue by ID. This action is irreversible.")]
    pub async fn delete_issue(&self, Parameters(input): Parameters<DeleteIssueInput>) -> String {
        let issue_id = input.issue_id;
        match self.client.delete_issue(&self.api_key, issue_id).await {
            Ok(()) => format!("Issue #{issue_id} deleted successfully."),
            Err(e) => format!("Error: {e}"),
        }
    }
}

// ── ServerHandler ─────────────────────────────────────────────────────────────

#[tool_handler(router = self.tool_router)]
impl ServerHandler for RedmineServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("rrmcp", env!("CARGO_PKG_VERSION")))
            .with_instructions("Redmine MCP server — exposes the Redmine REST API as MCP tools.")
    }
}
