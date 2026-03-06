use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    tool, tool_router,
};

use super::RedmineServer;
use crate::redmine::projects::{ProjectCreate, ProjectListResponse};

#[tool_router(router = project_tool_router)]
impl RedmineServer {
    pub fn project_tool_router_pub() -> ToolRouter<Self> {
        Self::project_tool_router()
    }

    #[tool(
        description = "List Redmine projects. Returns a JSON array of projects with their IDs and names."
    )]
    pub async fn list_projects(&self) -> String {
        let res = self
            .client
            .read::<ProjectListResponse>("projects.json", &self.api_key)
            .await;
        match res {
            Ok(resp) => serde_json::to_string_pretty(&resp).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error fetching projects: {e}"),
        }
    }
}
