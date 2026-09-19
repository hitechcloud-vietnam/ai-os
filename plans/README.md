<p align="center">
  <img src="https://hitechcloud.vn/wp-content/uploads/2025/01/hitechcloudvn.svg" alt="HiTechCloud" height="60">
</p>

# HiTechCloud Agent Platform (AI OS)

Nền tảng trung tâm chuẩn hóa kết nối **5 Tầng AI Interoperability** để quản lý **MCP Servers**, **Skills**, **Plugins**, **Agents** và **UI/Widgets** dùng chung cho nhiều nhà cung cấp AI (Anthropic/Claude, OpenAI/Codex, Cursor, Roo Code, Goose, Windsurf, Aider, OpenCode, Zed).

## 1. Mục tiêu & Định vị

- **Chuẩn hóa 5 Tầng AI Connect (2026 Standards)**:
  - **Tầng 5 (UI/User)**: **AG-UI** (Event streaming hai chiều, Human-in-the-loop interrupt) & **A2UI** (Dynamic structured widgets).
  - **Tầng 4 (Agents)**: **A2A Protocol v1.0** (Agent Cards, discovery, task delegation) & Multi-Model Routing (Anthropic/OpenAI/DeepSeek).
  - **Tầng 3 (Tools & Data)**: **MCP Core Spec 2026** (Tools, Prompts, Resources, Roots, Sampling) & OpenAI Tool Calling.
  - **Tầng 2 (APIs & Registry)**: **REST / OpenAPI 3.1**, **GraphQL Engine**, **JSON-RPC 2.0**, và **HCP-Spec Universal Manifest**.
  - **Tầng 1 (Security & Observability)**: **OAuth 2.0 / OIDC**, **Ed25519 Signatures**, **OpenTelemetry W3C Tracing**.
- Một **Package Registry** duy nhất cho mọi tài nguyên (Skill, Plugin, Agent, McpServer) với SemVer 2.0.0 và kiểm tra chữ ký Ed25519.
- Một **AI Gateway** duy nhất bằng Rust Tokio hiệu năng cao xác thực RBAC, giới hạn tần suất Token-Bucket, sandbox bảo mật WASM/Linux và trace OpenTelemetry.
- **Giao diện Kỹ thuật Chuẩn mực (MCPHub Style)**: 100% ứng dụng vận hành trong Dashboard, không trang chủ marketing, không dùng màu AI gradient.
- **Hệ sinh thái Công cụ Riêng biệt**: Kèm **VS Code Extension riêng** (`.vsix`) và **Rust CLI riêng** (`hitechcloud`) hỗ trợ xuất cấu hình cho 11 client IDE/Agent phổ biến.
- Có thể tự host (self-hosted) trên hạ tầng HiTechCloud (Rust, Docker, Kubernetes).

---

## 2. Danh mục Tài liệu Thiết kế Toàn diện

### 2.1. Kiến trúc Cốt lõi & Enterprise Control Plane
- [ENTERPRISE-AI-CONTROL-PLANE.md](ENTERPRISE-AI-CONTROL-PLANE.md) — **Đặc tả Master Enterprise AI Operating Gateway & Control Plane** (Multi-AI/Agent Federation, Universal Tool Fabric, Dynamic Skill DAG, Risk Policy & Supply Chain Security).
- [SUBDOMAINS-AND-NETWORKING.md](SUBDOMAINS-AND-NETWORKING.md) — **Quy hoạch Toàn diện 15 Subdomains, Cấu hình Mạng, Nginx Ingress & SSL Wildcard**.
- [ARCHITECTURE.md](ARCHITECTURE.md) — Kiến trúc tổng thể 5 tầng của toàn bộ nền tảng.

### 2.2. Chuẩn Kết Nối AI (AI Connect Standards) & Tương tác UI/Agent
- [AI-CONNECT-STANDARDS.md](AI-CONNECT-STANDARDS.md) — **Đặc tả Master 5 Tầng AI Interoperability Standards** (MCP, A2A, AG-UI, A2UI, OpenAI, OTel, OAuth2, GraphQL, REST).
- [A2A-GATEWAY.md](A2A-GATEWAY.md) — Đặc tả Agent-to-Agent Protocol v1.0, Agent Cards Catalog, và Task Delegation.
- [AG-UI-SPEC.md](AG-UI-SPEC.md) — Đặc tả Luồng sự kiện AG-UI hai chiều, Human-in-the-Loop Interrupt & Component A2UI.
- [PACKAGE-MANIFEST-SPEC.md](PACKAGE-MANIFEST-SPEC.md) — Đặc tả Universal `hcp.json` Package Manifest cho Skill, Plugin, Agent, McpServer.
- [HITECHCLOUD-OPENAPI-TOOL-CATALOG.md](HITECHCLOUD-OPENAPI-TOOL-CATALOG.md) — **Đặc tả Nạp Tự động 803 Endpoints từ 2 Bộ API HiTechCloud (`my.hitechcloud.vn` & `tools.hitechcloud.vn`) thành MCP/Skills/Plugins**.

### 2.3. Thiết kế Giao diện, CLIs, Branding & Tích hợp 19 Repositories
- [MCPHUB-DASHBOARD.md](MCPHUB-DASHBOARD.md) — Đặc tả Giao diện Dashboard chuẩn MCPHub (không AI gradient, 11 phân hệ kỹ thuật).
- [BRANDING-AND-ASSETS.md](BRANDING-AND-ASSETS.md) — **Đặc tả Tài nguyên Nhận diện Thương hiệu & Biểu tượng Chính thức** (Favicons, Icons 32x32, 192x192, 270x270).
- [THIRD-PARTY-CLI-INTEGRATION.md](THIRD-PARTY-CLI-INTEGRATION.md) — **Đặc tả Tích hợp & Tương thích Toàn diện với 11 CLI/IDE Bên thứ ba** (Claude Code, Goose, Aider, OpenCode, Crush, SimCode, Roo, Cursor, Windsurf, Zed, Codex).
- [REPOSITORIES-INTEGRATION.md](REPOSITORIES-INTEGRATION.md) — Kế hoạch tích hợp và trích xuất tính năng từ **19 Reference Repositories**.
- [VSCODE-EXTENSION.md](VSCODE-EXTENSION.md) — Đặc tả Tiện ích mở rộng VS Code riêng biệt (`hitechcloud-agent-platform`).
- [CLI.md](CLI.md) — Đặc tả Công cụ Dòng lệnh Rust riêng biệt (`hitechcloud`).

### 2.4. Gateways & Package Registry
- [MCP-GATEWAY.md](MCP-GATEWAY.md) — Gateway đa giao thức (stdio, SSE, stream HTTP), gộp server.
- [SKILLS-GATEWAY.md](SKILLS-GATEWAY.md) — Nạp, discover, phục vụ Skills cho agent (chuẩn Anthropic & SkillHub).
- [PLUGIN-GATEWAY.md](PLUGIN-GATEWAY.md) — Đóng gói Skill + MCP + Command thành 1 plugin cài đặt được.
- [REGISTRY.md](REGISTRY.md) — Kho lưu metadata, version SemVer, chữ ký Ed25519 và kiểm tra lỗ hổng.
- [MARKETPLACE.md](MARKETPLACE.md) — Giao diện & API để tìm kiếm, cài đặt 1-click, đánh giá.

### 2.5. Tích hợp AI Providers, Coding Agents & Hệ sinh thái Microsoft 365
- [MICROSOFT-GRAPH-AND-M365-INTEGRATION.md](MICROSOFT-GRAPH-AND-M365-INTEGRATION.md) — **Đặc tả Tích hợp Toàn diện Microsoft Graph API, M365 Copilot Connectors (Synced & Federated MCP) và MCP Apps (A2UI Widgets)**.
- [NUBE-SH-INTEGRATION.md](NUBE-SH-INTEGRATION.md) — **Đặc tả Tùy chọn Kết nối API Ngoại vi Nube.sh (OpenAI & Anthropic) & Dynamic Pricing**.
- [THIRD-PARTY-CLI-INTEGRATION.md](THIRD-PARTY-CLI-INTEGRATION.md) · [ANTHROPIC-INTEGRATION.md](ANTHROPIC-INTEGRATION.md) · [CLAUDE-CODE-INTEGRATION.md](CLAUDE-CODE-INTEGRATION.md)
- [OPENAI-INTEGRATION.md](OPENAI-INTEGRATION.md) · [CODEX-INTEGRATION.md](CODEX-INTEGRATION.md)

### 2.6. Đặc tả (Spec)
- [MCP-SPEC.md](MCP-SPEC.md) · [SKILL-SPEC.md](SKILL-SPEC.md) · [PLUGIN-SPEC.md](PLUGIN-SPEC.md) · [PACKAGE-MANIFEST-SPEC.md](PACKAGE-MANIFEST-SPEC.md)

### 2.7. Vận hành & Bảo mật
- [AUTH-RBAC.md](AUTH-RBAC.md) · [SECURITY.md](SECURITY.md) · [SANDBOX.md](SANDBOX.md)
- [SIGNING-VERIFICATION.md](SIGNING-VERIFICATION.md) · [VERSIONING.md](VERSIONING.md)
- [BILLING-USAGE.md](BILLING-USAGE.md) · [AUDIT-LOGGING.md](AUDIT-LOGGING.md) · [OBSERVABILITY.md](OBSERVABILITY.md)

### 2.8. Triển khai & Hạ tầng
- [DEPLOYMENT.md](DEPLOYMENT.md) · [DOCKER.md](DOCKER.md) · [KUBERNETES.md](KUBERNETES.md)
- [RUST-IMPLEMENTATION.md](RUST-IMPLEMENTATION.md) · [DATABASE.md](DATABASE.md) · [REDIS.md](REDIS.md)

### 2.8. Quản trị & Cổng Lập trình viên
- [ADMIN-CONSOLE.md](ADMIN-CONSOLE.md) · [DEVELOPER-PORTAL.md](DEVELOPER-PORTAL.md)

### 2.9. Tài liệu Khác
- [API.md](API.md) · [OPENAPI.md](OPENAPI.md) · [TESTING.md](TESTING.md) · [CONTRIBUTING.md](CONTRIBUTING.md) · [ROADMAP.md](ROADMAP.md)
- Ví dụ mẫu: [EXAMPLES/](EXAMPLES/)

---

## 3. Trạng thái Bộ Tài liệu

Bộ tài liệu này là bản thiết kế hệ thống hoàn chỉnh (Comprehensive Architecture Design Documents), chuẩn bị sẵn sàng cho quy trình triển khai chi tiết với Claude Code, Codex và các công cụ lập trình tự trị.
