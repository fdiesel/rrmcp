use anyhow::{Context, Result};

pub struct Config {
    pub redmine_base_url: String,
    pub redmine_api_key: String,
    pub server_host: String,
    pub server_port: u16,
    pub database_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            redmine_base_url: std::env::var("REDMINE_BASE_URL")
                .context("REDMINE_BASE_URL not set")?,
            redmine_api_key: std::env::var("REDMINE_API_KEY")
                .context("REDMINE_API_KEY not set")?,
            server_host: std::env::var("MCP_SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("MCP_SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("MCP_SERVER_PORT must be a valid port number")?,
            database_path: std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "/data/rrmcp.db".to_string()),
        })
    }
}
