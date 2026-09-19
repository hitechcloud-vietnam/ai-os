# Đặc tả Toàn diện: Các Chuẩn AI Connect (AI Interoperability Standards 2026)

Tài liệu này đặc tả toàn diện 5 tầng chuẩn kết nối của **HiTechCloud Agent Platform** (cập nhật đến 09/2026), kết nối AI với **Tools**, **Data**, **Agents**, **UI/User**, **APIs**, **Security** và **Observability**.

---

## 1. Sơ đồ 5 Tầng AI Connect (5-Layer AI Interoperability Model)

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ LAYER 5 ─ USER INTERACTION & UI STREAMING                                                       │
│   • AG-UI  : Agent ↔ UI Event Protocol (Streaming text, tool calls, HITL interrupt, subagents)   │
│   • A2UI   : Agent → Structured UI (Declarative JSON/React widgets, forms, interactive cards)     │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ LAYER 4 ─ AGENT-TO-AGENT COLLABORATION & DISCOVERY                                              │
│   • A2A    : Agent ↔ Agent Protocol v1.0 (Linux Foundation / Agent Cards, Task delegation)       │
│   • OpenAI Responses / Agents Protocol (Orchestration & subagent message flow)                   │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ LAYER 3 ─ TOOLS, CONTEXT & SERVICES (MCP SPEC 2026)                                              │
│   • MCP    : Model Context Protocol (Tools, Resources, Prompts, Tasks, MCP Apps, stdio/HTTP-SSE)  │
│   • OpenAI Function / Tool Calling Schema (JSON Schema parameter mapping)                        │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ LAYER 2 ─ DATA, APIS & STRUCTURED RPC                                                            │
│   • REST & OpenAPI 3.1 : Chuẩn giao tiếp RESTful API & tự động sinh SDK                          │
│   • GraphQL            : Truy vấn dữ liệu linh hoạt, graph entity fetching                       │
│   • JSON-RPC 2.0       : Giao thức truyền tin có trạng thái cho MCP và Tool execution            │
│   • JSON Schema        : Đặc tả kiểu dữ liệu đầu vào/đầu ra có cấu trúc                           │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ LAYER 1 ─ SECURITY, TRANSPORT & OBSERVABILITY                                                    │
│   • OAuth 2.0 / OIDC   : Xác thực doanh nghiệp, Service Accounts, mTLS, Token Vault              │
│   • Transport          : HTTPS, Server-Sent Events (SSE), WebSockets, stdio pipe                 │
│   • OpenTelemetry      : Distributed Traces (Model → Gateway → Tool), Metrics, Logs & Cost      │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Bảng Ma trận Tổng hợp Các Chuẩn AI Connect

| Chuẩn / Protocol | Đối tượng Kết nối | Mục đích Chính | Trạng thái & Tiêu chuẩn |
|---|---|---|---|
| **MCP** | AI ↔ Tools / Data / Services | Tool calling, Filesystem, DB, Resources, Prompts, Tasks | Chuẩn quan trọng nhất (Spec 2026-07-28) |
| **A2A** | Agent ↔ Agent | Khám phá Agent, Agent Cards, giao việc, hợp tác đa tác tử | v1.0 Production-Ready (Linux Foundation) |
| **AG-UI** | Agent ↔ UI / User | Truyền phát sự kiện, trạng thái, Tool Events, Human-in-the-Loop | Đang phát triển mạnh (Rust/TS SDK) |
| **A2UI** | Agent → UI | Agent tạo và đề xuất giao diện cấu trúc động (Widgets/Forms) | Tiêu chuẩn UI động mới nổi |
| **OpenAI Responses / Agents** | App ↔ OpenAI Agents | Điều phối quy trình và vòng lặp Agent của OpenAI | Chuẩn API nhà cung cấp |
| **OpenAI Tool Calling** | Model ↔ Application | Định dạng gọi hàm/tool có schema tham số JSON | Rất phổ biến |
| **JSON Schema** | AI ↔ Structured Data | Định nghĩa kiểu, validation dữ liệu đầu vào/ra | Nền tảng cốt lõi |
| **JSON-RPC 2.0** | Client ↔ Server | Truyền tin RPC đóng gói; MCP sử dụng làm message layer | Nền tảng |
| **OAuth 2.0 / OIDC** | AI ↔ Service / SaaS | Xác thực phân quyền, ủy quyền token, Enterprise SSO | Tiêu chuẩn Enterprise |
| **SSE / WebSocket / stdio** | Transport Layer | Truyền phát thời gian thực, duplex stream, local pipe | Tiêu chuẩn truyền tải |
| **REST / OpenAPI 3.1** | AI ↔ API | Khả năng tương tác API, tự sinh tool definitions | Cực kỳ phổ biến |
| **GraphQL** | AI ↔ Data / Graph | Truy vấn quan hệ phức tạp, dữ liệu phân cấp | Phổ biến |
| **OpenTelemetry** | Agent ↔ Observability | Truy vết toàn chuỗi (Distributed Trace), Token, Cost | Rất quan trọng cho Production |

---

## 3. Đặc tả Chi tiết Từng Tầng Chuẩn

### 3.1. Layer 5 — AG-UI & A2UI (Giao tiếp Agent ↔ Frontend & Giao diện Động)

#### AG-UI (Agent ↔ UI Event Protocol)
AG-UI chuẩn hóa toàn bộ luồng sự kiện (event-driven stream) giữa Agent Runtime (Rust Gateway) và Frontend (Dashboard / React / Mobile):
- **Event Types**:
  - `agent.text.stream`: Chunk ký tự văn bản AI trả về theo thời gian thực.
  - `agent.tool.call`: Tín hiệu Agent bắt đầu gọi công cụ (tên tool, tham số JSON).
  - `agent.tool.result`: Kết quả công cụ trả về (output data, execution time).
  - `agent.state.update`: Cập nhật trạng thái Agent (Thinking, Searching, Executing, Idle).
  - `agent.hitl.interrupt`: Tín hiệu tạm dừng yêu cầu con người phê duyệt (Human-in-the-Loop) cho các hành động nhạy cảm (ví dụ: Xóa DB, Chạy lệnh bash phá hủy).
  - `agent.subagent.spawn`: Sự kiện khởi tạo subagent mới và luồng xử lý độc lập.

#### A2UI (Agent → Structured UI)
Cho phép Agent không chỉ trả về text mà còn sinh ra các khối giao diện tương tác có cấu trúc:
```json
{
  "type": "a2ui.card",
  "component": "ServerResourceSummary",
  "props": {
    "server": "market.hitechcloud.vn",
    "cpu": 45.2,
    "memory": 68.0,
    "actions": [
      { "label": "Restart Nginx", "tool": "hitechcloud.restart_service", "args": { "name": "nginx" } },
      { "label": "Clear Cache", "tool": "redis.flush_db", "args": {} }
    ]
  }
}
```

---

### 3.2. Layer 4 — A2A (Agent2Agent Protocol v1.0) & Multi-Agent Collaboration

A2A là tiêu chuẩn do Linux Foundation quản lý (hơn 150 tổ chức tham gia), cho phép các Agent tự trị tìm kiếm và ủy quyền công việc cho nhau:

```
                          ┌──────────────────────────┐
                          │   Support Triage Agent   │
                          └─────────────┬────────────┘
                                        │
                         A2A Task Dispatch / Delegation
                                        │
          ┌─────────────────────────────┼─────────────────────────────┐
          ▼                             ▼                             ▼
┌───────────────────┐         ┌───────────────────┐         ┌───────────────────┐
│   Billing Agent   │         │   DevOps Agent    │         │  Security Agent   │
│  (Card: A2A-v1.0) │         │  (Card: A2A-v1.0) │         │  (Card: A2A-v1.0) │
└───────────────────┘         └───────────────────┘         └───────────────────┘
```

#### Định dạng Agent Card (`agent-card.json`)
Mỗi Agent xuất bản một Agent Card mô tả năng lực:
```json
{
  "a2aVersion": "1.0.0",
  "agentId": "agent-devops-01",
  "name": "HiTechCloud DevOps Agent",
  "description": "Handles infrastructure deployments, Kubernetes rollouts and container monitoring",
  "endpoints": {
    "task": "https://api-mcp.hitechcloud.vn/a2a/v1/tasks",
    "status": "https://api-mcp.hitechcloud.vn/a2a/v1/status"
  },
  "capabilities": [
    "kubernetes.deploy",
    "nginx.reload",
    "dns.configure"
  ],
  "authentication": {
    "type": "oauth2_bearer",
    "issuer": "https://api-mcp.hitechcloud.vn/oauth"
  }
}
```

---

### 3.3. Layer 3 — MCP (Model Context Protocol Spec 2026-07-28)

Kế thừa toàn bộ phiên bản mới nhất của MCP:
- **Stateless Core**: Cho phép Gateway mở rộng ngang không giới hạn (horizontal scale), session context được cache trong Redis.
- **Multi-Round-Trip Requests**: Hỗ trợ chuỗi tương tác nhiều bước phức tạp giữa model và server.
- **Authorization & Scoped Grants**: Gắn token định danh cho từng tool invocation.
- **Tasks & MCP Apps**: Hỗ trợ các tác vụ bất đồng bộ kéo dài (Long-running background tasks) và ứng dụng widget tương tác.
- **Registry Ba Chiều**:
  1. `Tool Registry`: Danh mục công cụ thực thi (`tools/list`, `tools/call`).
  2. `Resource Registry`: Danh mục tài nguyên dữ liệu (`resources/list`, `resources/read`, URI subscriptions).
  3. `Prompt Registry`: Danh mục prompt templates (`prompts/list`, `prompts/get`).

---

### 3.4. Layer 2 — Data & API Interoperability (REST, OpenAPI, GraphQL, JSON Schema)

- **REST / OpenAPI 3.1 & Automated Tool Ingestion**:
  - Gateway tự động sinh đặc tả `/openapi.json` cho toàn bộ công cụ và endpoint quản trị.
  - Tích hợp pipeline tự động nạp từ **2 tệp OpenAPI JSON Specification**:
    1. **API của Cổng Dịch vụ Khách hàng (`my.hitechcloud.vn`)**: `https://docs.hitechcloud.vn/endpoint/login?env=production` (Tra cứu dịch vụ, gia hạn cước, quản lý VPS/Cloud, ticket).
    2. **API của Trang Công cụ Kỹ thuật (`tools.hitechcloud.vn`)**: `https://doc-api-tools.hitechcloud.vn/` (Công cụ DNS, kiểm tra SSL, quét mạng, ping, port check, tiện ích hạ tầng).
  - Tự động chuyển đổi schema JSON thành kho MCP Servers, Skills và Plugins chính thức trên Registry.
- **Microsoft Graph API Integration (Enterprise CRUD Layer)**:
  - Tích hợp toàn diện `https://graph.microsoft.com/v1.0` biến dữ liệu và nghiệp vụ M365 thành các công cụ MCP:
    ```
    HiTechCloud Agent ──▶ Microsoft Graph ──▶ Mail, Calendar, Teams, OneDrive, SharePoint, Users, Excel, Planner
    ```
  - Hỗ trợ On-Behalf-Of (OBO) Token flow và Delegated/Application Permissions.
- **GraphQL Engine**: Cung cấp GraphQL query endpoint cho phép agent truy vấn các thực thể liên kết phức tạp (User → Organizations → MCP Servers → Tools → Audit Logs) trong 1 request duy nhất.
- **JSON Schema Validation**: Mọi payload `tools/call` đều được kiểm tra chặt chẽ qua engine JSON Schema trước khi chuyển tới backend.

---

### 3.5. Microsoft 365 Copilot Connectors & MCP Apps Ecosystem

- **Federated MCP Connector (Live Real-time Data)**:
  - M365 Copilot gọi Remote MCP Server của HiTechCloud qua HTTPS để truy vấn live data thời gian thực từ Panel, Billing, WAF, Cloud Server mà không cần sync dữ liệu nhạy cảm ra ngoài.
- **Synced Connector**:
  - Đồng bộ tài liệu và catalog dịch vụ của HiTechCloud vào Microsoft Graph Semantic Index.
- **Interactive MCP Apps (A2UI in Copilot)**:
  - Trả về Interactive Widgets (Fluent UI / Adaptive Cards) trực tiếp trong Copilot Chat và Teams.
- **Declarative & Custom Engine Agents (CEA)**:
  - `HiTechCloud Cloud Agent`, `HiTechCloud Security Agent`, `HiTechCloud Billing Agent`, `HiTechCloud DevOps Agent`.

---

### 3.6. Multi-Model Inference & Tùy chọn Kết nối API Ngoài (Nube.sh Option)

- **Tùy chọn Kết nối Nube.sh API (External Provider Option)**:
  - Nube.sh là một **Option kết nối ngoại vi** (không phải core platform), cung cấp thêm lựa chọn mô hình suy luận giá rẻ hoặc reasoning models cho lập trình viên.
  - Giao thức kép: OpenAI-compatible (`https://ai.nube-api.com/v1`) và Anthropic-compatible (`https://ai.nube-api.com`).
  - Hỗ trợ các mô hình: `Nube-Choice` (DeepSeek-V4.1-Flash), `Kimi-K2.6`, `GLM-5.3`, `GLM-5.3-Flash`, `Qwen3.8-27B` với context window 1.000.000 tokens và Quantization-Aware Training (QAT, MXFP4, NVFP4).
  - Tự động cập nhật bảng giá thời gian thực từ endpoint công khai: `GET https://ai.nube-api.com/v1/models/pricing`.
  - Tối ưu chi phí bằng prompt cache read ($0.0027/triệu tokens) và cost-aware model routing.

---

### 3.7. Layer 1 — Security, Transport & Observability

- **OAuth 2.0 / OIDC & Token Vault**:
  - Tích hợp BetterAuth và OAuth 2.0 Authorization Server.
  - Secret Vault: Lưu trữ GitHub Token, AWS Secret Key, DB Connection string trong kho mã hóa AES-256; Agent chỉ nhận tham chiếu `secret://<key>` an toàn.
- **OpenTelemetry (OTel) Distributed Tracing**:
  - Mỗi request từ Client/Agent được gắn `traceparent` header chuẩn W3C.
  - Truy vết toàn diện chuỗi xử lý:
    ```
    Trace ID: 4bf92f3577b34da6a3ce929d0e0e4736
    ├── Span 1: Ingress Authentication (2ms)
    ├── Span 2: Redis Rate Limit Check (1ms)
    ├── Span 3: Model Routing (Claude 3.7 / GPT-4o) (240ms)
    │   ├── Span 3.1: Tool Call: read_file (8ms)
    │   └── Span 3.2: Tool Call: execute_sql (14ms)
    └── Span 4: Response Assembly & Audit Log Commit (3ms)
    ```
  - Thu thập đầy đủ: Token usage (Prompt tokens, Completion tokens), Latency p50/p95/p99, Tool success rate, Chi phí USD theo thời gian thực.
