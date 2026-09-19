# HiTechCloud AI OS

> **AI Interoperability Platform** — 5-Layer Architecture: AG-UI → A2A → MCP → REST → Security/OTEL
>
> **License: MIT + Non-Commercial Clause** — See [LICENSE](LICENSE)

## Project Structure (SaaS Standard)

```
HiTechCloud AI OS/
├── crates/                    # Rust workspace crates
│   ├── common/                #   Shared models, errors, RPC schemas
│   ├── signing/               #   Ed25519 + SHA-256 package signing
│   ├── sandbox/               #   Sandboxed command execution
│   ├── adapter-anthropic/     #   SKILL.md parser, Claude Code format
│   ├── adapter-openai/        #   MCP → OpenAI function schema
│   ├── registry/              #   Package registry CRUD
│   ├── skills-service/        #   Skills management
│   ├── plugin-service/        #   Plugin management
│   ├── mcp-gateway/           #   MCP JSON-RPC 2.0 + SSE
│   ├── ai-gateway/            #   Main API (RBAC, Billing, Auth)
│   ├── a2a-gateway/           #   Agent-to-Agent Protocol v1.0
│   └── cli/                   #   hitechcloud CLI tool
├── services/                  # Go microservices
│   ├── admin-svc/             #   SaaS admin: Org/User/API Key
│   ├── plugin-svc/            #   Plugin lifecycle
│   └── skills-svc/            #   Skills CRUD + search
├── deployments/               # Deployment configs (systemd, etc.)
├── scripts/                   # DevOps scripts (setup, seed)
├── configs/                   # Environment configs
├── docs/                      # Documentation (OpenAPI spec)
├── migrations/                # Database migrations (SQL)
├── plans/                     # Architecture plans & specs
├── .github/workflows/         # CI/CD pipeline
├── Cargo.toml                 # Rust workspace root
├── go.mod                     # Go module root
├── Makefile                   # Build & run commands
└── README.md
```

## Architecture

```
Port 8080  ── AI Gateway      (Rust)  │  REST API, RBAC, Billing, Auth
Port 8081  ── MCP Gateway     (Rust)  │  JSON-RPC 2.0, SSE, Tool Routing
Port 8082  ── Skills Service  (Go)    │  SKILL.md CRUD, full-text search
Port 8083  ── Plugin Service  (Go)    │  Plugin lifecycle, manifest validation
Port 8084  ── A2A Gateway     (Rust)  │  Agent-to-Agent Protocol v1.0
Port 8085  ── Admin Service   (Go)    │  SaaS admin: Orgs, Users, API Keys, Audit
```

All services are **pure Linux binaries** — no Docker, no containers.

## 5-Layer Architecture

```
┌─────────────────────────────────────────────────────┐
│  Layer 1: AG-UI    — Agent-User Interface (CLI)      │
│  Layer 2: A2A      — Agent-to-Agent Protocol v1.0    │
│  Layer 3: MCP      — Model Context Protocol (JSON-RPC)│
│  Layer 4: REST     — Standard HTTP API               │
│  Layer 5: Security — RBAC, Signing, OTEL, Billing    │
└─────────────────────────────────────────────────────┘
```

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
POST /v1/billing/usage                      Record usage (auto cost calc)
GET  /v1/billing/usage/{org_id}             Usage summary per billing period
GET  /v1/billing/pricing                    Dynamic model pricing (Nube/Anthropic/OpenAI)
```

### A2A Gateway (8084) — Agent-to-Agent Protocol v1.0
```
GET  /health                                Health check
POST /a2a/v1/agents                         Register Agent Card
GET  /a2a/v1/agents                         List active agents
GET  /a2a/v1/agents/{agent_id}              Get agent card
DELETE /a2a/v1/agents/{agent_id}            Deactivate agent
GET  /a2a/v1/discover?capability=           Discover agents by capability
POST /a2a/v1/tasks                          Dispatch task to agent
GET  /a2a/v1/tasks                          List tasks (filter: status, agent_id)
GET  /a2a/v1/tasks/{task_id}                Get task status
PUT  /a2a/v1/tasks/{task_id}                Update task progress/result
GET  /a2a/v1/stream/{task_id}               SSE stream for task progress
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

## Rust Workspace (12 Crates)

| Crate | Path | Purpose |
|-------|------|---------|
| `common` | `crates/common` | Shared models, errors, RPC schemas |
| `signing` | `crates/signing` | Ed25519 + SHA-256 package signing |
| `sandbox` | `crates/sandbox` | Sandboxed command execution |
| `adapter-anthropic` | `crates/adapter-anthropic` | SKILL.md parser, Claude Code format |
| `adapter-openai` | `crates/adapter-openai` | MCP → OpenAI function schema |
| `registry` | `crates/registry` | Package registry CRUD (all entities) |
| `skills-service` | `crates/skills-service` | Skills management |
| `plugin-service` | `crates/plugin-service` | Plugin management |
| `mcp-gateway` | `crates/mcp-gateway` | MCP JSON-RPC 2.0 + SSE router |
| `ai-gateway` | `crates/ai-gateway` | Main API gateway (RBAC, Billing, Auth) |
| `a2a-gateway` | `crates/a2a-gateway` | A2A Protocol v1.0 (agents, tasks, streaming) |
| `cli` | `crates/cli` | `hitechcloud` CLI tool |

## Go Services

| Service | Path | Port | Purpose |
|---------|------|------|---------|
| `admin-svc` | `services/admin-svc` | 8085 | SaaS admin: Org/User/API Key/Audit |
| `plugin-svc` | `services/plugin-svc` | 8083 | Plugin lifecycle management |
| `skills-svc` | `services/skills-svc` | 8082 | Skills CRUD + full-text search |

## CLI

```bash
hitechcloud login --key <API_KEY>           Login with API key
hitechcloud config show                     Show current config
hitechcloud status                          Check all services
hitechcloud mcp list                        List MCP servers
hitechcloud mcp publish --manifest mcp.json Publish MCP server
hitechcloud mcp serve --server <name>       Stdio MCP proxy
hitechcloud skill list                      List skills
hitechcloud skill search <query>            Search skills
hitechcloud skill publish --file SKILL.md   Publish skill
hitechcloud plugin list                     List plugins
hitechcloud plugin publish --manifest p.json Publish plugin
hitechcloud client export --target claude-code  Export for Claude Code
hitechcloud client export --target cursor       Export for Cursor
hitechcloud client export --target vscode       Export for VS Code
hitechcloud client export --target windsurf     Export for Windsurf
hitechcloud client export --target aider        Export for Aider
hitechcloud client export --target codex        Export for OpenAI Codex
hitechcloud client export --target gemini       Export for Gemini CLI
hitechcloud client export --target amazon-q     Export for Amazon Q
```

## RBAC (Role-Based Access Control)

| Role | Permissions |
|------|------------|
| `owner` | All permissions (full control) |
| `admin` | CRUD all resources, manage members, approve plugins |
| `developer` | Read all, write own skills/plugins, create installations |
| `viewer` | Read-only access to all resources |

Permission format: `resource.action` (e.g., `server.write`, `skill.publish`, `plugin.approve`)

## Billing & Usage Tracking

- **Dynamic pricing** from Nube.sh, Anthropic, OpenAI (synced to `model_pricing` table)
- **Auto cost calculation**: `(input_tokens + output_tokens) × price_per_million / 1M`
- **Usage summary** per org per billing period
- **Budget quotas** with soft/hard limits

## Security

- Bearer token auth via API keys (SHA-256 hashed, never stored in plaintext)
- RBAC: 4 roles (Owner/Admin/Developer/Viewer) with granular permissions
- Ed25519 package signing + SHA-256 checksums
- Audit logging for all operations
- Budget quotas with hard/soft limits
- Non-commercial license (see LICENSE)

## OpenAPI Spec

Full OpenAPI 3.1 specification: [`docs/openapi.yaml`](docs/openapi.yaml)

## CI/CD

GitHub Actions pipeline (`.github/workflows/ci.yml`):
- Rust build + test + clippy + fmt
- Go build + test
- Integration tests with real Postgres + Redis
- License compliance check
- Security audit (cargo-audit + TruffleHog)

## Database

- **PostgreSQL 18** — 18+ tables (organizations, users, api_keys, mcp_servers, skills, plugins, installations, audit_logs, usage_records, budget_quotas, model_pricing, a2a_agent_cards, a2a_tasks, ...)
- **Redis 8** — Caching, rate limiting, session store

## Roadmap

- ✅ **Phase 1 (MVP)**: Registry, MCP Gateway, Skills, Plugins, Admin SaaS, CLI
- ✅ **Phase 2 (Beta)**: RBAC, Signing, Sandbox, OpenAI adapter, Billing/Usage, A2A Gateway
- ⬜ **Phase 3 (Marketplace)**: Developer Portal, Trust badges, Revenue share
- ⬜ **Phase 4 (Enterprise)**: SSO/SCIM, Compliance, Multi-region HA
