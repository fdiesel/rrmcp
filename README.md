# rrmcp

A Redmine MCP (Model Context Protocol) server written in Rust. Exposes the full Redmine REST API as MCP tools, allowing AI assistants like Claude to interact with your Redmine instance — reading and managing issues, projects, wiki pages, time entries, and more.

## Features

- **Full Redmine API coverage** — Issues, Projects, Users, Wiki, Time Entries, Attachments, Search, and all other Redmine resources
- **Per-user authentication** — Each user authenticates with their own OAuth2 credentials, mapped to their personal Redmine API key
- **HTTP transport** — Runs as an HTTP server with SSE, compatible with Claude.ai custom connectors
- **Docker-native** — Designed to run alongside Redmine and Nginx in Docker Compose
- **Zero-downtime user management** — Add, remove, and list users via CLI without restarting the server

## How It Works

```
Claude.ai ──(OAuth2)──► Nginx (TLS) ──► rrmcp ──(API key)──► Redmine
```

1. Each user gets their own `client_id` and `client_secret`, configured in Claude.ai
2. rrmcp validates the credentials and resolves the user's Redmine API key
3. All Redmine calls are made with that user's API key — proper Redmine permissions apply

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
docker compose exec rrmcp rrmcp user add \
  --client-id alice \
  --client-secret "s3cur3p@ss" \
  --redmine-api-key "your-redmine-api-key"
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
rrmcp user add --client-id alice --client-secret "s3cr3t" --redmine-api-key "abc123"

# List all users
rrmcp user list

# Remove a user
rrmcp user remove --client-id alice
```

Client secrets are stored as argon2 hashes — never in plaintext.

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

- OAuth2 Client Credentials flow — no browser login required
- Client secrets hashed with argon2 at rest
- JWT bearer tokens for session auth (short-lived, stateless)
- API keys never logged or exposed in URLs
- Designed to sit behind Nginx — the server itself speaks plain HTTP internally

## License

MIT
