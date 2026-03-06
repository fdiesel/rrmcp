# rrmcp

A Redmine MCP (Model Context Protocol) server written in Rust. Exposes the full Redmine REST API as MCP tools, allowing AI assistants like Claude to interact with your Redmine instance — reading and managing issues, projects, wiki pages, time entries, and more.

> **WARNING: Unstable, untested software.**
>
> This project is in early development. It has not been tested, is not feature-complete, and
> **must not be used in production or exposed publicly**. There are no security guarantees.
> The author makes no recommendations and takes no responsibility for any use of this software.
> Use entirely at your own risk.

## Status

**MCP tool implementation comes first.** OAuth 2.1 authentication (via Ory Hydra) is part of
the planned architecture but will be implemented in a later phase. The server may initially
require no auth or a simple API key until the OAuth layer is added.

## Features

- **Full Redmine API coverage** — Issues, Projects, Users, Wiki, Time Entries, Attachments, Search, and all other Redmine resources
- **Per-user authentication** — Planned: each user authenticates via OAuth 2.1 (Ory Hydra), mapped to their personal Redmine API key
- **HTTP transport** — Runs as an HTTP server with SSE, compatible with Claude.ai custom connectors
- **Docker-native** — Designed to run alongside Redmine and Nginx in Docker Compose
- **Zero-downtime user management** — Add, remove, and list users via CLI without restarting the server

## How It Works

```
Claude.ai ──(OAuth 2.1)──► Nginx (TLS) ──► Ory Hydra ──► Login/Consent App
                                               │
                                               ▼
                                            rrmcp ──(API key)──► Redmine
```

1. Each user authenticates via OAuth 2.1 (Authorization Code + PKCE) through Ory Hydra
2. rrmcp validates the bearer token via Hydra introspection and resolves the user's Redmine API key
3. All Redmine calls are made with that user's API key — proper Redmine permissions apply

> **Note:** OAuth is planned for a later phase. MCP tools are the current implementation priority.

## Quick Start

### Prerequisites

- Docker and Docker Compose
- A running Redmine instance
- A Redmine API key per user (found in Redmine under *My account*)

### 1. Configure Claude.ai

In Claude.ai, add a custom MCP connector:
- **MCP Server URL**: `https://your-domain.com/mcp`
- **OAuth Client ID**: your assigned `client_id`
- **OAuth Client Secret**: your assigned `client_secret`

### 2. Deploy with Docker Compose

```yaml
services:
  redmine:
    image: redmine:latest
    # ... your existing Redmine config

  rrmcp:
    image: ghcr.io/your-org/rrmcp:latest
    environment:
      REDMINE_BASE_URL: http://redmine:3000
      JWT_SECRET: your-random-secret-here
    volumes:
      - rrmcp_data:/data
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    # ... TLS termination, proxy to rrmcp:3000

volumes:
  rrmcp_data:
```

### 3. Add Users

```bash
docker compose exec rrmcp rrmcp user add alice@example.com \
  --api-key "your-redmine-api-key"
```

## Configuration

All configuration is via environment variables.

| Variable          | Description                               | Default           | Required |
|-------------------|-------------------------------------------|-------------------|----------|
| `REDMINE_BASE_URL`| Base URL of your Redmine instance         | —                 | Yes      |
| `JWT_SECRET`      | Secret for signing JWT bearer tokens      | —                 | Yes      |
| `MCP_SERVER_HOST` | Host to bind                              | `0.0.0.0`         | No       |
| `MCP_SERVER_PORT` | Port to bind                              | `3000`            | No       |
| `DATABASE_PATH`   | Path to SQLite database                   | `/data/rrmcp.db`  | No       |
| `RUST_LOG`        | Log level (`info`, `debug`, `warn`)       | `info`            | No       |

## User Management

Users are managed via the `rrmcp user` CLI subcommand. Run these inside the container:

```bash
# Add a user
rrmcp user add alice@example.com --api-key "abc123"

# List all users
rrmcp user list

# Remove a user
rrmcp user remove alice@example.com

# Update a user's API key
rrmcp user set-api-key alice@example.com "new-api-key"
```

Passwords are hashed with argon2 and Redmine API keys are encrypted at rest — never stored in plaintext.

## Redmine API Coverage

| Resource            | Status    |
|---------------------|-----------|
| Issues              | Stable    |
| Projects            | Stable    |
| Users               | Stable    |
| Time Entries        | Stable    |
| Wiki Pages          | Alpha     |
| Attachments         | Beta      |
| Search              | Alpha     |
| Project Memberships | Alpha     |
| Issue Relations     | Alpha     |
| Versions            | Alpha     |
| Groups              | Alpha     |
| Roles               | Alpha     |
| Custom Fields       | Alpha     |
| Enumerations        | Alpha     |
| Issue Categories    | Alpha     |
| Issue Statuses      | Alpha     |
| Trackers            | Alpha     |
| Queries             | Alpha     |
| Files               | Alpha     |
| News                | Prototype |
| My Account          | Alpha     |
| Journals            | Alpha     |

## Building from Source

```bash
# Clone
git clone https://github.com/your-org/rrmcp.git
cd rrmcp

# Build
cargo build --release

# Run
REDMINE_BASE_URL=http://localhost:3000 JWT_SECRET=dev-secret cargo run -- serve
```

## Security

- OAuth 2.1 Authorization Code + PKCE via Ory Hydra (planned)
- User passwords hashed with argon2; Redmine API keys encrypted with AES-256-GCM at rest
- Bearer token introspection on every MCP request
- API keys never logged or exposed in URLs
- Designed to sit behind Nginx — the server itself speaks plain HTTP internally
- **Not audited. Do not expose publicly.**

## License

MIT
