mod config;
mod error;
mod redmine;
mod tools;

use anyhow::Result;
use clap::{Parser, Subcommand};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;
use tracing::info;

use config::Config;
use redmine::RedmineClient;
use tools::RedmineServer;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "rrmcp", version, about = "Redmine MCP server")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the MCP HTTP server.
    Serve,
    /// Manage user API key registrations.
    #[command(subcommand)]
    User(UserCommand),
}

#[derive(Debug, Subcommand)]
enum UserCommand {
    /// Register a user with their Redmine API key.
    Add {
        email: String,
        #[arg(long)]
        api_key: String,
    },
    /// Remove a registered user.
    Remove { email: String },
    /// List all registered users.
    List,
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Serve => serve().await,
        Command::User(cmd) => {
            eprintln!("User management not yet implemented. Command: {cmd:?}");
            Ok(())
        }
    }
}

// ── Serve command ─────────────────────────────────────────────────────────────

async fn serve() -> Result<()> {
    let config = Config::from_env()?;
    let client = RedmineClient::new(config.redmine_base_url.clone())?;
    let handler = RedmineServer::new(client, config.redmine_api_key.clone());

    let ct = CancellationToken::new();

    let service: StreamableHttpService<RedmineServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(handler.clone()),
            Default::default(),
            StreamableHttpServerConfig {
                stateful_mode: false,
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("rrmcp listening on http://{addr}/mcp");

    let router = axum::Router::new().nest_service("/mcp", service);

    tokio::select! {
        result = axum::serve(listener, router) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down");
            ct.cancel();
        }
    }

    Ok(())
}
