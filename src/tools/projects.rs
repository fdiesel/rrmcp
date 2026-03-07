use rmcp::{handler::server::tool::ToolRouter, tool, tool_router};

use super::RedmineServer;
use crate::{error::RedmineError, redmine::projects::ProjectListResponse};

#[tool_router(router = project_tool_router)]
impl RedmineServer {
    pub fn project_tool_router_pub() -> ToolRouter<Self> {
        Self::project_tool_router()
    }

    #[tool(
        description = "List Redmine projects. Returns a JSON array of projects with their IDs and names.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    pub async fn list_projects(&self) -> Result<String, RedmineError> {
        self.client
            .read::<ProjectListResponse>("projects.json", &self.api_key)
            .await
    }
}
