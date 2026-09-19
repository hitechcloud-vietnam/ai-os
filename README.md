# HiTechCloud AI OS

> **AI Interoperability Platform** — 5-Layer Architecture: AG-UI → A2A → MCP → REST → Security/OTEL
>
> **License: MIT + Non-Commercial Clause** — See [LICENSE](LICENSE)

## Architecture

```
Port 8080  ── AI Gateway      (Rust)  │  REST API, RBAC, Rate Limit, Auth
Port 8081  ── MCP Gateway     (Rust)  │  JSON-RPC 2.0, SSE, Tool Routing
Port 8082  ── Skills Service  (Go)    │  SKILL.md CRUD, full-text search
Port 8083  ── Plugin Service  (Go)    │  Plugin lifecycle, manifest validation
Port 8085  ── Admin Service   (Go)    │  SaaS admin: Orgs, Users, API Keys, Audit
```

All services are **pure Linux binaries** — no Docker, no containers.

## Quick Start

```bash
# 1. Copy env template
cp .env.example .env
# Edit .env with your PostgreSQL/Redis connection details

# 2. Build everything
make build

# 3. Start all services
make start

# 4. Health check
make health

# 5. Seed sample data (optional)
make seed
```

## API Endpoints

### AI Gateway (8080) — Main API entry point
```
GET  /health                                Health check
POST /v1/registry/mcp-servers               Create MCP server        ← Admin SaaS
GET  /v1/registry/mcp-servers               List MCP servers
GET  /v1/registry/mcp-servers/{id}          Get MCP server
PUT  /v1/registry/mcp-servers/{id}          Update MCP server        ← Admin SaaS
DELETE /v1/registry/mcp-servers/{id}        Delete MCP server        ← Admin SaaS
POST /v1/registry/skills                    Create skill             ← Admin SaaS
GET  /v1/registry/skills                    List skills
GET  /v1/registry/skills/search?q=          Full-text search skills
POST /v1/registry/plugins                   Create plugin            ← Admin SaaS
GET  /v1/registry/plugins                   List plugins
GET  /v1/admin/orgs                         List organizations
GET  /v1/admin/orgs/{id}/members            List org members
GET  /v1/admin/orgs/{id}/audit-logs         Get audit logs
```

### MCP Gateway (8081) — MCP protocol handler
```
GET  /health                                Health check
POST /mcp/v1                                JSON-RPC 2.0 endpoint
GET  /mcp/v1/sse                            SSE streaming
POST /tool-execute                          OpenAI-compatible tool call
GET  /tools                                 List available tools
```

### Skills Service (8082)
```
GET  /skills                                List all skills
GET  /skills/{id}                           Get skill detail
POST /skills                                Create skill
PUT  /skills/{id}                           Update skill
DELETE /skills/{id}                         Delete skill
```

### Plugin Service (8083)
```
GET  /plugins                               List all plugins
GET  /plugins/{id}                          Get plugin detail
POST /plugins                               Create plugin
PUT  /plugins/{id}                          Update plugin
DELETE /plugins/{id}                        Delete plugin
```

### Admin Service (8085) — SaaS management
```
GET  /health                                Health check
GET  /orgs                                  List organizations
POST /orgs                                  Create organization
GET  /users                                 List users
POST /users                                 Create user
POST /api-keys                              Generate API key
GET  /mcp-servers                           List MCP servers (admin view)
POST /mcp-servers                           Create MCP server
DELETE /mcp-servers/{id}                    Delete MCP server
GET  /skills                                List skills (admin view)
POST /skills                                Create skill
DELETE /skills/{id}                         Delete skill
GET  /plugins                               List plugins (admin view)
POST /plugins                               Create plugin
DELETE /plugins/{id}                        Delete plugin
GET  /audit-logs                            View audit logs
```

## SaaS Admin Workflow

```
1. Create Org:        POST /orgs {"name":"Acme","plan":"pro"}
2. Create User:       POST /users {"email":"dev@acme.com","org_id":"..."}
3. Generate API Key:  POST /api-keys {"email":"dev@acme.com","scope":"org"}
4. Create MCP Server: POST /mcp-servers {"name":"GitHub","transport":"stdio","namespace":"github"}
5. Create Skill:      POST /skills {"name":"review-pr","scope":"public","description":"..."}
6. Create Plugin:     POST /plugins {"name":"devtools","description":"..."}
```

## Rust Workspace (10 Crates)

| Crate | Purpose |
|-------|---------|
| `hitechcloud-common` | Shared models, errors, RPC schemas |
| `hitechcloud-signing` | Ed25519 + SHA-256 package signing |
| `hitechcloud-sandbox-runner` | Sandboxed command execution |
| `hitechcloud-adapter-anthropic` | SKILL.md parser, Claude Code format |
| `hitechcloud-adapter-openai` | MCP → OpenAI function schema |
| `hitechcloud-registry` | Package registry CRUD (all entities) |
| `hitechcloud-skills-service` | Skills management |
| `hitechcloud-plugin-service` | Plugin management |
| `hitechcloud-mcp-gateway` | MCP JSON-RPC 2.0 + SSE router |
| `hitechcloud-ai-gateway` | Main API gateway binary |

## Go Services

| Service | Port | Purpose |
|---------|------|---------|
| `skills-svc` | 8082 | Skills CRUD + full-text search |
| `plugin-svc` | 8083 | Plugin lifecycle management |
| `admin-svc` | 8085 | SaaS admin: Org/User/API Key/Audit |

## Security

- Bearer token auth via API keys (SHA-256 hashed, never stored in plaintext)
- RBAC per tool/namespace
- Ed25519 package signing
- Audit logging for all operations
- Non-commercial license (see LICENSE)

## CI/CD

GitHub Actions pipeline (`.github/workflows/ci.yml`):
- Rust build + test + clippy + fmt
- Go build + test
- Integration tests with real Postgres + Redis
- License compliance check
- Security audit (cargo-audit + TruffleHog)

## Roadmap

- **Phase 1 (MVP)** ← Current: Registry, MCP Gateway, Skills, Admin SaaS
- **Phase 2 (Beta)**: RBAC, Signing, Sandbox, OpenAI adapter, Billing
- **Phase 3 (Marketplace)**: Developer Portal, Trust badges, Revenue share
- **Phase 4 (Enterprise)**: SSO/SCIM, Compliance, Multi-region HA
