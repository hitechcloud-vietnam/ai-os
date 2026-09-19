<p align="center">
  <img src="https://hitechcloud.vn/wp-content/uploads/2025/01/hitechcloudvn.svg" alt="HiTechCloud" height="60">
</p>

# Kiến trúc Tổng thể: HiTechCloud Agent Platform (AI OS)

## 1. Sơ đồ Kiến trúc 5 Tầng Chuẩn Hóa (5-Layer AI Interoperability Model)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                             TẦNG 5: UI & NGƯỜI DÙNG (AG-UI & A2UI)                               │
├──────────────────────────┬──────────────────────────┬─────────────────────────┬──────────────────┤
│    Developer Dashboard   │   VS Code Extension      │    HiTechCloud CLI      │   AI Coding IDEs │
│  (MCPHub Technical UI)   │ (ActivityBar TreeViews)  │  (Rust Native Binary)   │  & Terminals     │
│  - No AI gradient/color  │ - Interactive Tool Run   │  - Client Exporter (11x)│  - Claude / Goose│
│  - 100% Inside Dashboard │ - .mcp.json Generator    │  - TUI Agent Loop       │  - Cursor / Roo  │
│  - 11 Technical Views    │ - Direct .VSIX download  │  - Direct tool call     │  - OpenCode/Aider│
└────────────┬─────────────┴────────────┬─────────────┴────────────┬────────────┴────────┬─────────┘
             │                          │                          │                     │
             └──────────────────────────┴─────────────┬────────────┴─────────────────────┘
                                                      │ AG-UI Event Stream (SSE/WS) & A2UI Widgets
                                                      ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                         TẦNG 4: AGENTS & ORCHESTRATION (A2A PROTOCOL v1.0)                       │
│  • Agent Discovery & Agent Cards Catalog   • Multi-Agent Task Delegation Engine                  │
│  • Human-In-The-Loop (HITL) Interrupts     • Multi-Model Router (Options: Anthropic/OpenAI/Nube) │
│  • Dynamic Pricing Sync (ai.nube-api.com)  • Prompt Cache Read Optimization                      │
└─────────────────────────────────────────────────────┬────────────────────────────────────────────┘
                                                      │
                                                      ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                            TẦNG 3: TOOLS & CONTEXT (MCP 2026 CORE SPEC)                          │
├──────────────────────────┬──────────────────────────┬─────────────────────────┬──────────────────┤
│       MCP Gateway        │      Skills Gateway      │      Plugin Gateway     │  Adapters Layer  │
│  • stdio / sse / stream  │  • Anthropic SKILL.md    │  • Manifest v1.0 engine │  • Anthropic SDK │
│  • Multi-server merge    │  • iFlyTek SkillHub      │  • Lifecycle hooks      │  • OpenAI Tools  │
│  • Tools OpenAPI JSON    │  • Portal OpenAPI JSON   │  • Sandbox permissions  │  • MS Graph SDK  │
│  • (tools.hitechcloud)   │  • (my.hitechcloud)      │  • Auto JSON Ingest     │  • M365 Copilot  │
└────────────┬─────────────┴────────────┬─────────────┴────────────┬────────────┴────────┬─────────┘
             │                          │                          │                     │
             └──────────────────────────┴─────────────┬────────────┴─────────────────────┘
                                                      │
                                                      ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                            TẦNG 2: APIS & DỮ LIỆU (REST, GRAPHQL & JSON-RPC)                     │
│  • REST API (OpenAPI 3.1)  • GraphQL Engine (Hasura/Async-GraphQL)  • JSON-RPC 2.0 Engine        │
│  • Universal Package Registry (HCP-Spec Manifest for Skill, Plugin, Agent, McpServer)            │
└─────────────────────────────────────────────────────┬────────────────────────────────────────────┘
                                                      │
                                                      ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│               TẦNG 1: BẢO MẬT, GIAO THỨC & GIÁM SÁT (SECURITY, TRANSPORT & OTEL)                 │
├─────────────────────────────────────────────────────┬────────────────────────────────────────────┤
│                  PostgreSQL 16                      │                  Redis 7                   │
│  • Organizations, Users, Roles, API Keys            │  • Token-Bucket Rate Limiter Quotas        │
│  • MCP Server Catalog & Version Registries          │  • Gateway Routing Cache                   │
│  • Skills, Plugins & Workspace Installations        │  • Real-time Pub/Sub Event Broadcast       │
│  • Structured Audit Logs & Telemetry Events         │  • Active Session State Store              │
├─────────────────────────────────────────────────────┴────────────────────────────────────────────┤
│  • OAuth 2.0 / OIDC / BetterAuth Authentication     • Ed25519 Cryptographic Signatures           │
│  • OpenTelemetry W3C Distributed Tracing            • Zero-Trust Linux Sandbox / WASM Isolates   │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Nguyên tắc Thiết kế Cốt lõi

1. **Tuân thủ Toàn diện 5 Tầng AI Interoperability**:
   - Tầng 5: **AG-UI** (Event streaming hai chiều, Human-in-the-loop) & **A2UI** (Giao diện cấu trúc).
   - Tầng 4: **A2A Protocol v1.0** (Agent Cards, discovery, task delegation) & Multi-Model Routing.
   - Tầng 3: **MCP Core 2026** (Tools, Prompts, Resources, Roots, Sampling).
   - Tầng 2: **REST / OpenAPI 3.1**, **GraphQL**, **JSON-RPC 2.0**, và **HCP-Spec Universal Manifest**.
   - Tầng 1: **OAuth 2.0/OIDC**, **Ed25519 Signatures**, **OpenTelemetry W3C Tracing**.
2. **Giao diện Kỹ thuật Chuẩn mực (MCPHub Style)**:
   - **Không AI hoá giao diện**: Không sử dụng màu AI gradient (tím/hồng/cyan gradient), không marketing landing page.
   - 100% ứng dụng hoạt động bên trong **Dashboard** tối giản, tối ưu năng suất cho developer.
3. **Tích hợp 19 Reference Repositories**:
   - Kế thừa toàn bộ tinh hoa từ `mcphub`, `registry`, `crush`, `Roo-Code`, `goose`, `opencode`, `aider`, `leaked-claude-code`, `simcode-cli`, `claude-code-analysis`, `open-claude-code`, `claude-code`, `claude-code-reverse-engineering`, `skills`, `skillhub`, `claude-code-plugin-template`, `template`, `claude-marketplace-template`, `marketplace`.
4. **Bộ Công cụ Đồng hành Độc lập**:
   - **VS Code Extension riêng**: Quản lý server, chạy thử nghiệm tool và tạo cấu hình `.mcp.json`.
   - **Rust CLI riêng**: Tốc độ cao, hỗ trợ lệnh `hitechcloud client export` cho 11 client AI phổ biến (Claude Code, Goose, Aider, OpenCode, Crush, SimCode, Codex, Cursor, Windsurf, Roo Code, Zed).
5. **Zero-Trust Sandbox Isolation**:
   - Các công cụ và plugin từ nguồn thứ ba được cô lập hoàn toàn bằng Linux cgroups/seccomp hoặc WASM isolates, chỉ được truy cập mạng và file khi được cấp quyền rõ ràng trên Dashboard.

---

## 3. Các Phân hệ Cốt lõi (10 Crate Rust Workspace)

| Crate | Tên Crate | Chức năng Chính |
|---|---|---|
| 1 | `hitechcloud-common` | Chứa các cấu trúc dữ liệu dùng chung (Models, Errors, RPC schemas, HCP-Spec) |
| 2 | `hitechcloud-signing` | Xác thực chữ ký số ED25519 và SHA256 checksums cho các package trên Registry |
| 3 | `hitechcloud-sandbox-runner` | Môi trường cô lập thực thi lệnh an toàn cho Tool & Plugin (WASM / Seccomp) |
| 4 | `hitechcloud-adapter-anthropic`| Chuyển đổi định dạng prompt `SKILL.md` và giao thức Claude Code |
| 5 | `hitechcloud-adapter-openai` | Chuyển đổi công cụ sang định dạng OpenAI Function Calling & Responses |
| 6 | `hitechcloud-registry` | Lưu trữ và quản lý phiên bản của MCP Servers, Skills, Plugins, Agents |
| 7 | `hitechcloud-skills-service` | Nạp, phân loại phạm vi (scope) và thực thi chuỗi Agent Skills |
| 8 | `hitechcloud-plugin-service` | Quản lý vòng đời Plugin, manifest v1.0 và lifecycle hooks |
| 9 | `hitechcloud-mcp-gateway` | Router JSON-RPC 2.0, SSE, Stream HTTP, A2A Router và AG-UI Event Broadcaster |
| 10 | `hitechcloud-ai-gateway` & `hitechcloud` (CLI) | Cổng API tập trung, RBAC, Rate Limit, OpenTelemetry Spans và CLI binary |

---

---

## 4. Danh mục Tên miền phụ (Subdomains) & Hạ tầng Mạng

### 4.1. 11 Subdomains MỚI Của Agent Platform (Cần Triển Khai)
1. `api-mcp.hitechcloud.vn`: REST & JSON-RPC 2.0 Gateway Ingress.
2. `gw.hitechcloud.vn`: SSE & WebSocket Real-time Stream Hub.
3. `mcp.hitechcloud.vn`: Developer Web Dashboard (100% inside dashboard, 0 AI gradient).
4. `get-mcp.hitechcloud.vn`: Phân phối CLI & VS Code Extension `.vsix`.
5. `registry-mcp.hitechcloud.vn`: Universal Package Registry & OCI Artifacts.
6. `marketplace-mcp.hitechcloud.vn`: AI Marketplace & Enterprise Catalog.
7. `a2a.hitechcloud.vn`: A2A Protocol & Agent Cards Discovery Gateway.
8. `copilot.hitechcloud.vn`: Microsoft 365 Federated MCP Connector Ingress.
9. `auth-mcp.hitechcloud.vn`: Identity Federation, Entra ID, OIDC & SCIM.
10. `telemetry-mcp.hitechcloud.vn.`: OpenTelemetry OTLP Collector & Trace Ingress.
11. `sandbox.hitechcloud.vn`: Zero-Trust WASM / MicroVM Sandbox Workers.

### 4.2. 4 Hệ thống Hiện hữu Bên ngoài (External Existing Systems)
12. `docs.hitechcloud.vn`: Tài liệu OpenAPI Cổng Khách hàng (`my.hitechcloud.vn` - 348 EPs).
13. `doc-api-tools.hitechcloud.vn`: Tài liệu OpenAPI Trang Công cụ Kỹ thuật (`tools.hitechcloud.vn` - 455 EPs).
14. `api.hitechcloud.vn`: Backend API Cổng Khách hàng.
15. `api-tools.hitechcloud.vn`: Backend API Trang Công cụ Kỹ thuật.

---

## 5. Danh mục Tài liệu Kỹ thuật Chi tiết

- `ENTERPRISE-AI-CONTROL-PLANE.md`: **Đặc tả Master Enterprise AI Control Plane, Multi-AI/Agent Federation, Universal Tool Fabric & Risk Policy**.
- `SUBDOMAINS-AND-NETWORKING.md`: **Quy hoạch Toàn diện 15 Subdomains, Cấu hình Mạng, Nginx Ingress & SSL Wildcard**.
- `AI-CONNECT-STANDARDS.md`: Toàn văn đặc tả 5 tầng AI Interoperability Standards.
- `MICROSOFT-GRAPH-AND-M365-INTEGRATION.md`: Tích hợp Toàn diện Microsoft Graph API & M365 Copilot (Federated Connectors & MCP Apps).
- `NUBE-SH-INTEGRATION.md`: Tích hợp Nube.sh AI Provider, Dual Protocol & Dynamic Pricing.
- `HITECHCLOUD-OPENAPI-TOOL-CATALOG.md`: Tích hợp Kho 803 Endpoints từ 2 Bộ API HiTechCloud thành MCP/Skills/Plugins.
- `A2A-GATEWAY.md`: Đặc tả Agent-to-Agent Protocol v1.0, Agent Cards và Task Delegation.
- `AG-UI-SPEC.md`: Đặc tả luồng sự kiện AG-UI, Human-in-the-Loop và widget A2UI.
- `PACKAGE-MANIFEST-SPEC.md`: Đặc tả Universal `hcp.json` manifest cho 4 loại tài nguyên.
- `MCPHUB-DASHBOARD.md`: Đặc tả Dashboard kỹ thuật lập trình viên (Zero AI marketing/gradient).
- `THIRD-PARTY-CLI-INTEGRATION.md`: Hướng dẫn kết nối 11 CLI/IDE AI bên thứ 3.
- `REPOSITORIES-INTEGRATION.md`: Báo cáo tích hợp tính năng từ 19 repositories mẫu.
- `VSCODE-EXTENSION.md`: Đặc tả VS Code Extension độc lập.
- `CLI.md`: Đặc tả Rust CLI native binary và exporter.
- `OBSERVABILITY.md`: Đặc tả OpenTelemetry, Prometheus và Distributed Tracing.
- `BILLING-USAGE.md`: Đặc tả Hạch toán Token AI, Dynamic Pricing Sync & Budget Limits.
- `REGISTRY.md`: Đặc tả Control Plane Package Registry và chữ ký số Ed25519.
- `MCP-GATEWAY.md`: Đặc tả Router MCP JSON-RPC 2.0.


