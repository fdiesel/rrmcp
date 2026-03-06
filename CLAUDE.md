# CLAUDE.md — rrmcp

## Project Overview

**rrmcp** is an open-source Redmine MCP (Model Context Protocol) server written in Rust.
It exposes the full Redmine REST API as MCP tools, allowing AI assistants (e.g. Claude.ai) to
interact with Redmine projects, issues, wiki pages, and more.

Deployment target: Docker container alongside Redmine, behind an Nginx reverse proxy.

## Tech Stack

| Concern         | Crate / Tool                                     |
| --------------- | ------------------------------------------------ |
| MCP framework   | `rmcp`                                           |
| Async runtime   | `tokio`                                          |
| HTTP client     | `reqwest`                                        |
| Serialization   | `serde` + `serde_json`                           |
| Error handling  | `thiserror` (own types) + `anyhow` (propagation) |
| Logging/tracing | `tracing` + `tracing-subscriber`                 |
| Config          | Environment variables (Docker-optimized)         |
| Database        | `sqlx` + SQLite (user registry)                  |
| CLI             | `clap` (user management subcommands)             |

## Architecture Summary

- **Transport**: HTTP with SSE (Server-Sent Events) — no stdio
- **Auth**: OAuth2 Client Credentials flow (per-user client_id/client_secret)
  - Each user registers a client_id + client_secret with the MCP server
  - The server maps those credentials to a Redmine API key
  - All Redmine API calls use the resolved per-user API key
- **User registry**: SQLite database (via `sqlx`), persisted as a Docker volume
- **Redmine connection**: Single self-hosted Redmine instance via REST API + API key per user

@docs/architecture.md
@docs/redmine-api.md

## Build & Run

```bash
# Build
cargo build

# Run server (requires env vars — see .env.example)
cargo run -- serve

# Run tests
cargo test
```

## Configuration (Environment Variables)

All configuration is via environment variables for Docker compatibility.

| Variable           | Description                                            | Required |
| ------------------ | ------------------------------------------------------ | -------- |
| `REDMINE_BASE_URL` | Base URL of the Redmine instance                       | Yes      |
| `MCP_SERVER_HOST`  | Host to bind the HTTP server (default: 0.0.0.0)        | No       |
| `MCP_SERVER_PORT`  | Port to bind the HTTP server (default: 3000)           | No       |
| `DATABASE_PATH`    | Path to SQLite database file (default: /data/rrmcp.db) | No       |
| `RUST_LOG`         | Log level (e.g. `info`, `debug`)                       | No       |

## User Registry (SQLite)

Database persisted via Docker volume at `/data/rrmcp.db` (managed by `sqlx` migrations).

Schema:

```sql
CREATE TABLE users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id       TEXT NOT NULL UNIQUE,
    client_secret   TEXT NOT NULL,  -- hashed (argon2)
    redmine_api_key TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Users are managed via CLI subcommand (run inside the container):

```bash
rrmcp user add --client-id alice --client-secret s3cr3t --redmine-api-key abc123
rrmcp user remove --client-id alice
rrmcp user list
```

## Code Conventions

- Use `thiserror` to define domain error enums; use `anyhow::Result` for internal propagation
- All async code uses `tokio`
- Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`) — not `println!` or `eprintln!`
- Keep Redmine API logic in a dedicated `redmine` module
- Keep MCP tool definitions in a dedicated `tools` module
- Keep auth/OAuth logic in a dedicated `auth` module
- Entry point dispatches via `clap`: `rrmcp serve` starts the server, `rrmcp user *` manages users
- No `unwrap()` or `expect()` in non-test code — propagate errors properly
- Format with `cargo fmt` before committing
- Lint with `cargo clippy -- -D warnings`

## Redmine API Coverage

Target: full Redmine stable REST API (Redmine stable release). See @docs/redmine-api.md.

Resources covered:

- Issues, Projects, Project Memberships, Users, Time Entries
- News, Issue Relations, Versions, Wiki Pages, Queries
- Attachments, Issue Statuses, Trackers, Enumerations, Issue Categories
- Roles, Groups, Custom Fields, Search, Files, My Account, Journals

## Docker

The server is designed to run as a container alongside Redmine and Nginx.

- Expose port `3000` internally
- Nginx terminates TLS and reverse-proxies to the MCP server
- SQLite database persisted via a named Docker volume at `/data/rrmcp.db`
- All secrets passed as environment variables (never baked into the image)
