# Architecture — rrmcp

## Overview

rrmcp is an HTTP MCP server that bridges AI assistants (Claude.ai) to a self-hosted Redmine
instance. It is multi-user: each user authenticates via OAuth 2.1, which maps server-side to
their personal Redmine API key.

> **Note:** MCP tool implementation is the primary focus. OAuth 2.1 authentication (via Ory
> Hydra) is planned but may be added in a later phase. The architecture below reflects the
> intended final design.

```
Claude.ai
  │
  │  HTTPS (OAuth 2.1 + PKCE)
  ▼
Nginx (TLS termination + reverse proxy)
  │
  ├──► Ory Hydra (OAuth token endpoints)
  │       │
  │       ├──► Login/Consent App (login UI, user → API key registry)
  │       └──► SQLite (hydra.sqlite — Hydra's own state)
  │
  └──► rrmcp MCP Server
          │  introspects token via Hydra
          │  looks up API key in SQLite
          ▼
       Redmine (HTTPS, per-user API key)
```

## Transport

- **Protocol**: MCP over HTTP with SSE (Server-Sent Events)
- **No stdio transport** — HTTP only
- Bound to `0.0.0.0:3000` by default (configurable via env)

## Authentication & Authorization

> **Status**: OAuth implementation is planned. MCP tools are implemented first and may
> initially run without auth (or with a simpler API key check) until Hydra integration is
> complete.

### Intended Flow (OAuth 2.1 via Ory Hydra)

1. Admin creates an OAuth client in Hydra:
   ```bash
   docker exec hydra hydra create oauth2-client \
     --name "Claude.ai" \
     --grant-type authorization_code,refresh_token \
     --response-type code \
     --redirect-uri "https://claude.ai/oauth/callback"
   ```

2. Admin registers a user with their Redmine API key:
   ```bash
   rrmcp user add alice@example.com --api-key <redmine-api-key>
   ```

3. User configures Claude.ai with:
   - Server URL: `https://your-server.com`
   - Client ID: (from step 1)
   - Client Secret: (from step 1)

4. Claude.ai initiates the Authorization Code + PKCE flow via Hydra. The user logs in through
   the Login/Consent App, which accepts the request via the Hydra Admin API.

5. Claude.ai exchanges the auth code for a bearer token, then sends MCP requests with it.

6. rrmcp introspects the token via Hydra, resolves the user's Redmine API key from SQLite, and
   calls Redmine on their behalf.

### User Registry

Stored in SQLite (via `sqlx`), persisted as a Docker volume. Managed by the Login/Consent App
and the `rrmcp user` CLI.

```sql
CREATE TABLE user_api_keys (
    user_id                  TEXT PRIMARY KEY,       -- email address
    password_hash            TEXT,                   -- argon2 hashed
    redmine_api_key_encrypted TEXT NOT NULL,         -- AES-256-GCM encrypted
    redmine_user_login       TEXT,                   -- optional, for reference
    created_at               DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at               DATETIME
);
```

Note: Hydra manages its own state in a separate SQLite file (`hydra.sqlite`).

### CLI Commands

```bash
# User / API key management (rrmcp CLI)
rrmcp user add <email> --api-key <redmine_api_key> [--password <password>]
rrmcp user list
rrmcp user remove <email>
rrmcp user set-api-key <email> <new_api_key>

# OAuth client management (via Hydra CLI)
docker exec hydra hydra create oauth2-client ...
docker exec hydra hydra list oauth2-clients
docker exec hydra hydra delete oauth2-client <client_id>
```

### Why Ory Hydra

| Aspect        | Ory Hydra           | Build in Rust       | Keycloak            |
| ------------- | ------------------- | ------------------- | ------------------- |
| Image size    | ~5 MB               | —                   | ~400 MB             |
| RAM           | ~200 MB             | ~100 MB             | ~2 GB minimum       |
| Database      | SQLite / PostgreSQL  | SQLite              | PostgreSQL/MySQL    |
| OAuth spec    | OpenID Certified    | DIY                 | OpenID Certified    |
| Security risk | Low (battle-tested) | Higher              | Low                 |

Hydra handles all OAuth 2.1 machinery (PKCE, token introspection, revocation, discovery
metadata) so rrmcp only needs to validate tokens — not issue them.

## Module Structure (Planned)

```
src/
  main.rs           — Entry point: clap dispatch to `serve` or `user` subcommand
  config.rs         — Environment variable config structs
  auth/
    mod.rs          — Bearer token validation middleware (Hydra introspection)
    registry.rs     — User registry (sqlx queries against SQLite)
  redmine/
    mod.rs          — Redmine API client (reqwest-based)
    issues.rs       — Issues API
    projects.rs     — Projects API
    wiki.rs         — Wiki Pages API
    time_entries.rs — Time Entries API
    users.rs        — Users API
    attachments.rs  — Attachments / Uploads API
    search.rs       — Search API
    groups.rs       — Groups API
    versions.rs     — Versions API
    memberships.rs  — Project Memberships API
    relations.rs    — Issue Relations API
    categories.rs   — Issue Categories API
    journals.rs     — Journals API
    enumerations.rs — Enumerations API
    custom_fields.rs— Custom Fields API
    roles.rs        — Roles API
    queries.rs      — Queries API
    news.rs         — News API
    files.rs        — Files API
    my_account.rs   — My Account API
  tools/
    mod.rs          — MCP tool registration with rmcp
    issues.rs       — MCP tools wrapping Redmine issue calls
    projects.rs     — MCP tools wrapping Redmine project calls
    ... (mirrors redmine/ module)
  cli/
    mod.rs          — clap App definition: `serve` + `user` subcommands
    serve.rs        — `rrmcp serve`: HTTP server startup, config loading, router setup
    user.rs         — `rrmcp user add/remove/list/set-api-key` subcommands
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
    # ... your existing Redmine config

  hydra:
    image: oryd/hydra:v2.2.0
    command: serve all
    environment:
      DSN: sqlite:///data/hydra.sqlite?_fk=true
      URLS_SELF_ISSUER: https://your-server.com
      URLS_LOGIN: http://login-app:3001/login
      URLS_CONSENT: http://login-app:3001/consent
      URLS_LOGOUT: http://login-app:3001/logout
      SECRETS_SYSTEM: ${HYDRA_SECRET}
    volumes:
      - hydra-data:/data
    # ports 4444 (public) and 4445 (admin) — internal only, Nginx proxies 4444

  login-app:
    build: ./login-app
    environment:
      HYDRA_ADMIN_URL: http://hydra:4445
      DATABASE_PATH: /data/users.db
    volumes:
      - user-data:/data

  rrmcp:
    build: .
    environment:
      REDMINE_BASE_URL: http://redmine:3000
      MCP_SERVER_PORT: 3000
      DATABASE_PATH: /data/users.db
      HYDRA_PUBLIC_URL: http://hydra:4444
    volumes:
      - user-data:/data
    ports:
      - "3000:3000"  # internal only, Nginx proxies

  nginx:
    image: nginx:alpine
    # TLS termination; proxy /oauth2/* and /.well-known/* to hydra:4444, rest to rrmcp:3000

volumes:
  hydra-data:
  user-data:
```

## Security Considerations

- Never log Redmine API keys, OAuth secrets, or bearer tokens
- Redmine API keys encrypted at rest with AES-256-GCM
- User passwords hashed with argon2
- PKCE required (enforced by Hydra)
- JWT signing secrets loaded from env vars, never hardcoded
- Nginx must enforce HTTPS — all internal services speak plain HTTP
- Token introspection on every MCP request (no stale token reuse)
- Hydra secrets stored as env vars (or secrets manager in production)
