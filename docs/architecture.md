# Architecture — rrmcp

## Overview

rrmcp is a stateless HTTP MCP server that bridges AI assistants (Claude.ai) to a self-hosted
Redmine instance. It is multi-user: each user authenticates with their own OAuth2 credentials,
which map server-side to their personal Redmine API key.

```
Claude.ai
  │
  │  HTTPS (OAuth2 Client Credentials)
  ▼
Nginx (TLS termination + reverse proxy)
  │
  │  HTTP
  ▼
rrmcp MCP Server (Docker container, port 3000)
  │
  │  HTTPS (Redmine REST API + per-user API key)
  ▼
Redmine (Docker container)
```

## Transport

- **Protocol**: MCP over HTTP with SSE (Server-Sent Events)
- **No stdio transport** — HTTP only
- Bound to `0.0.0.0:3000` by default (configurable via env)

## Authentication & Authorization

### Flow

1. User configures Claude.ai with:
   - MCP Server URL (e.g. `https://mcp.example.com`)
   - OAuth Client ID (their personal `client_id`)
   - OAuth Client Secret (their personal `client_secret`)

2. Claude.ai sends an OAuth2 **Client Credentials** grant request to the MCP server:
   ```
   POST /oauth/token
   grant_type=client_credentials&client_id=...&client_secret=...
   ```

3. The server validates credentials against the user registry, issues a short-lived JWT or
   opaque bearer token, and stores the association: token → redmine_api_key.

4. Subsequent MCP requests include the bearer token. The server resolves the Redmine API key
   and uses it for all Redmine REST calls on behalf of that user.

### User Registry

Stored in SQLite (via `sqlx`), persisted as a Docker volume at `/data/rrmcp.db`.
Migrations run automatically at startup.

```sql
CREATE TABLE users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id       TEXT NOT NULL UNIQUE,
    client_secret   TEXT NOT NULL,  -- hashed with argon2
    redmine_api_key TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Users are managed via CLI subcommand (run inside the container — no restart needed):

```bash
rrmcp user add --client-id alice --client-secret s3cr3t --redmine-api-key abc123
rrmcp user remove --client-id alice
rrmcp user list
```

### Token Issuance

- Issue short-lived bearer tokens (JWT recommended, signed with a server secret)
- Token contains: `user_id` (or `client_id`), `exp`
- Server resolves Redmine API key from user registry using `client_id` from token claims
- No persistent token storage needed if using stateless JWT

## Module Structure (Planned)

```
src/
  main.rs           — Entry point: clap dispatch to `serve` or `user` subcommand
  config.rs         — Environment variable config structs
  auth/
    mod.rs          — OAuth2 token endpoint handler
    token.rs        — JWT issuance and validation
    registry.rs     — User registry (sqlx queries against SQLite)
  redmine/
    mod.rs          — Redmine API client (reqwest-based)
    models.rs       — Serde structs for all Redmine resources
    issues.rs       — Issues API
    projects.rs     — Projects API
    wiki.rs         — Wiki Pages API
    ... (one file per major resource group)
  tools/
    mod.rs          — MCP tool registration with rmcp
    issues.rs       — MCP tools wrapping Redmine issue calls
    projects.rs     — MCP tools wrapping Redmine project calls
    ... (mirrors redmine/ module)
  cli/
    mod.rs          — clap App definition: `serve` + `user` subcommands
    serve.rs        — `rrmcp serve`: HTTP server startup, config loading, router setup
    user.rs         — `rrmcp user add/remove/list` subcommands
  error.rs          — thiserror error types
```

## Redmine API Client

- Uses `reqwest` with a shared `Client` (connection pooling)
- `base_url` stored on the client struct; API key passed as `&str` per method call
- API key sent via `X-Redmine-API-Key` header — never in the URL
- All responses deserialized into typed structs via `serde`
- Redmine API errors mapped to typed `RedmineError` variants

## MCP Tool Design

- Each Redmine resource group = one or more MCP tools
- Tool inputs use JSON schema (via `rmcp` derive macros or manual schema)
- Tools return structured JSON responses
- Tool descriptions should include Redmine API stability status (Stable/Alpha/Beta)

## Docker Compose (Intended Layout)

```yaml
services:
  redmine:
    image: redmine:latest
    # ...

  rrmcp:
    build: .
    environment:
      REDMINE_BASE_URL: http://redmine:3000
      MCP_SERVER_PORT: 3000
      DATABASE_PATH: /data/rrmcp.db
    volumes:
      - rrmcp_data:/data
    ports:
      - "3000:3000"  # internal only, Nginx proxies

  nginx:
    image: nginx:alpine
    # TLS termination, proxy to rrmcp:3000
```

## Security Considerations

- Never log Redmine API keys or OAuth secrets
- Client secrets stored as argon2 hashes in SQLite — never in plaintext
- JWT signing secret loaded from env var, never hardcoded
- Nginx must enforce HTTPS — the MCP server itself speaks plain HTTP internally
- Users file mounted read-only in Docker
