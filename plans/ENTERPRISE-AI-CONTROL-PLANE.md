# Đặc tả Kỹ thuật: Enterprise AI Control Plane & Operating Gateway

Tài liệu này đặc tả kiến trúc cấp doanh nghiệp (Enterprise-Grade) biến **HiTechCloud Agent Platform** thành một **AI Operating Gateway / AI Control Plane toàn diện**, vượt xa vai trò của một proxy/router thông thường để trở thành trung tâm điều phối **AI, Agent, Tool, Identity, Data, Policy, Marketplace và Billing**.

---

## 1. Kiến trúc Tổng thể: Control Plane & Data Plane

Hệ thống được tách biệt nghiêm ngặt thành hai phân tầng độc lập để đảm bảo khả năng mở rộng quy mô lớn (High Scalability), độ trễ cực thấp (Sub-millisecond latency) và độ tin cậy chuẩn doanh nghiệp:

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 HITECHCLOUD AI CONTROL PLANE                                     │
├──────────────────────────┬──────────────────────────┬─────────────────────────┬──────────────────┤
│    Identity Federation   │    Package Registry      │   AI Policy Engine      │  Billing Engine  │
│  • Entra ID / OIDC / SSO │  • Universal HCP Packages│  • RBAC + ABAC + ReBAC  │  • Token Metering│
│  • SCIM Provisioning     │  • Ed25519 / SBOM / CVE  │  • Risk-Based Auth      │  • Dynamic Sync  │
├──────────────────────────┼──────────────────────────┼─────────────────────────┼──────────────────┤
│    AI Marketplace        │   Agent Registry & Cards │   Workflow / DAG Planner│  Observability   │
│  • MCP Apps & Connectors │  • A2A Capability Cards  │  • Dynamic Skill Engine │  • OpenTelemetry │
│  • 1-Click Provisioning  │  • AI-to-AI Negotiation  │  • Multi-step Workflows │  • Trace Trees   │
└──────────────────────────┴─────────────┬────────────┴─────────────────────────┴──────────────────┘
                                         │ Distributed Sync & Cache Invalidation (gRPC / Redis)
                                         ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   HITECHCLOUD AI DATA PLANE                                      │
├──────────────────────────┬──────────────────────────┬─────────────────────────┬──────────────────┤
│   Multi-AI Federation    │   Multi-Agent Federation │   Universal Tool Fabric │  Transport Hub   │
│  • Claude/GPT/Gemini     │  • A2A Protocol v1.0     │  • MCP, REST, GraphQL   │  • HTTP/2, SSE   │
│  • DeepSeek / Qwen / Kimi│  • Task Dispatcher       │  • gRPC, SQL, SSH, K8s  │  • WebSockets    │
│  • Local GPU / Ollama    │  • Subagent Loops        │  • WASM Sandbox Runner  │  • Stdio Pipes   │
└──────────────────────────┴──────────────────────────┴─────────────────────────┴──────────────────┘
```

---

## 2. Multi-AI Federation & Chiến lược Định tuyến Thông minh

Hệ thống hỗ trợ cơ chế định tuyến linh hoạt theo chính sách (Policy-Driven Routing), không hard-code vào bất kỳ nhà cung cấp nào:

```
                                  Client / Agent Task
                                           │
                                           ▼
                                    AI Task Router
                                           │
                 ┌─────────────────────────┼─────────────────────────┐
                 ▼                         ▼                         ▼
          Claude 3.7 Sonnet             GPT-4o / o1              Gemini 2.0 Pro
          (Coding Specialist)       (Reasoning Specialist)    (Long Context 2M)
                 │                         │                         │
                 ├─────────────────────────┼─────────────────────────┤
                 ▼                         ▼                         ▼
         DeepSeek / Nube.sh           Qwen / GLM               Local GPU / vLLM
          (Cost Optimized)         (Fast Batch Tasks)       (Air-Gapped Confidential)
                 │                         │                         │
                 └─────────────────────────┼─────────────────────────┘
                                           │
                                           ▼
                                Aggregator & Judge Model
                                           │
                                           ▼
                                     Final Result
```

### Các Chiến lược Định tuyến Khả dụng:
1. **Cost-Based Routing**: Tự động chọn model có chi phí $/token thấp nhất thỏa mãn yêu cầu năng lực (Capabilities).
2. **Latency-Based**: Điều hướng đến endpoint có p99 latency thấp nhất tại thời điểm thực thi.
3. **Quality & Capability Matching**:
   - `Coding & Refactoring`: Claude 3.7 Sonnet / Opus.
   - `Deep Logic & Reasoning`: OpenAI o1 / o3-mini hoặc DeepSeek-R1.
   - `Long-Context (1M-2M tokens)`: Gemini 2.0 / Nube.sh Kimi-K2.6.
   - `High-Volume & Background Jobs`: Qwen3.8-27B / GLM-5.3-Flash.
   - `Confidential & On-Prem Data`: Local GPU (vLLM / Ollama private cluster).
4. **Resilience & Fallback**: Tự động thử lại (Circuit breaker) với model thay thế tương đương khi upstream bị 429/500.
5. **Parallel Inference & Consensus/Ensemble**: Gửi đồng thời prompt tới 2-3 model và sử dụng **Judge Model** để tổng hợp kết quả chính xác nhất cho các bài toán bảo mật hoặc pháp lý.

---

## 3. Multi-Agent Federation & AI-to-AI Capability Negotiation

Các Agent tự trị có khả năng tự động khám phá và thương lượng năng lực (Negotiation) thông qua chuẩn **A2A Protocol v1.0**:

```
                         Master Orchestrator Agent
                                     │
                                     │ "Tôi cần rà soát bảo mật & khởi động lại VM"
                                     ▼
                           A2A Agent Registry
                                     │
                 ┌───────────────────┼───────────────────┐
                 ▼                   ▼                   ▼
         Security Agent      Infrastructure Agent   Billing Agent
          (WAF, Scan)          (VM, Reboot, K8s)      (Invoices, Quota)
                 │                   │
                 └─────────┬─────────┘
                           │ Phối hợp thực thi (A2A Tasks)
                           ▼
                  Infrastructure Agent
                           │
                           ▼
                 HiTechCloud VM Tools (803 EPs)
```

- **Cơ chế Thương lượng (Capability Negotiation)**: Agent A gửi yêu cầu *"Tôi cần phân tích tệp dump memory 10GB"*, Registry tìm Agent B có tag `memory-dump-analyzer` với context phù hợp và thiết lập phiên làm việc A2A có ký chữ ký số Ed25519.

---

## 4. Universal Tool Fabric (Hợp nhất Đa Giao thức Công cụ)

HiTechCloud chuẩn hóa toàn bộ các giao thức công cụ khác nhau vào một cấu trúc chuẩn duy nhất (`Canonical Tool Object`):

```
Universal Tool Fabric
├── MCP (Model Context Protocol 2026)
├── REST APIs & OpenAPI 3.1 (803 HiTechCloud Endpoints)
├── Microsoft Graph API (Mail, Teams, SharePoint, OneDrive, Excel)
├── GraphQL Endpoints
├── gRPC Services
├── Webhooks & Event Streams
├── Direct SQL Query Engines (Postgres / MySQL)
├── Secure SSH Shell Commands
├── Kubernetes API (Pod, Deployment, Ingress)
└── Native Rust In-Memory Plugins
```

### Cấu trúc Canonical Tool:
```rust
pub struct CanonicalTool {
    pub id: String,                  // "htc_user.cloud_vm.restart"
    pub namespace: String,           // "hitechcloud"
    pub capability: String,          // "cloud.compute.lifecycle"
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub risk_level: RiskLevel,       // Low, Medium, High, Critical
    pub cost_per_invocation: f64,    // USD
    pub average_latency_ms: u32,
    pub required_permissions: Vec<String>,
    pub execution_handler: ToolHandlerType, // Stdio, RemoteHttp, Grpc, Wasm
}
```

---

## 5. Dynamic Skill Graph & AI Workflow Engine

Skill không chỉ là một prompt đơn lẻ mà là một đồ thị có hướng không chu trình (**DAG - Directed Acyclic Graph**) gồm nhiều bước thực thi tự động:

```
                            User: "Deploy WordPress cho domain abc.com"
                                             │
                                             ▼
                                    Skill DAG Planner
                                             │
                       ┌─────────────────────┴─────────────────────┐
                       ▼                                           ▼
             [Step 1: Check Server]                      [Step 2: DNS Lookup]
             (Tool: htc_user.server)                     (Tool: htc_tool.dns)
                       │                                           │
                       └─────────────────────┬─────────────────────┘
                                             ▼
                                 [Step 3: Create MySQL DB]
                                 (Tool: htc_user.database)
                                             │
                                             ▼
                                [Step 4: Install WordPress]
                                (Tool: htc_user.hosting)
                                             │
                                             ▼
                                  [Step 5: Issue Free SSL]
                                  (Tool: htc_user.ssl)
                                             │
                                             ▼
                                 [Step 6: Configure WAF]
                                 (Tool: htc_user.waf)
                                             │
                                             ▼
                                  [Step 7: Verify Status]
```

---

## 6. AI Policy Engine & Phân quyền Dựa trên Rủi ro (Risk-Based Authorization)

Mọi hành động của AI đều chịu sự kiểm soát chặt chẽ của **Policy Engine** kết hợp đa mô hình phân quyền: **RBAC** (Role-Based), **ABAC** (Attribute-Based), **ReBAC** (Relationship-Based) và **Risk-Based Authorization**:

```
                       Agent Muốn Gọi Tool / Hành Động
                                      │
                                      ▼
                             AI Risk Policy Engine
                                      │
        ┌─────────────────────────────┼─────────────────────────────┐
        ▼                             ▼                             ▼
    [Mức THẤP - Low Risk]      [Mức VỪA - Medium Risk]     [Mức CAO - High Risk]
   (Tra cứu DNS, Đọc log)    (Thêm DNS record, Send Mail)   (Xóa VM, Drop DB, Thanh toán)
        │                             │                             │
        ▼                             ▼                             ▼
   Tự động Thực thi           Xác nhận Người dùng            Phê duyệt Admin
    (Auto Execute)             (User Confirmation)         (Admin Approval / 2FA)
```

- **Phân loại Dữ liệu Bảo mật (Data Classification Policy)**: Dữ liệu được gắn cờ `Confidential` hoặc `Internal Only` sẽ bị chặn chuyển tiếp tới các model AI công cộng và bắt buộc điều hướng về Local Private Model.

---

## 7. Identity Federation & Doanh nghiệp SSO

Tích hợp định danh tập trung với các hệ sinh thái doanh nghiệp lớn:
- **Microsoft Entra ID (Azure AD)**: Hỗ trợ SSO, OIDC, SAML 2.0, OAuth 2.0 On-Behalf-Of (OBO) flow, và Conditional Access policies.
- **SCIM (System for Cross-domain Identity Management)**: Tự động đồng bộ thêm/xóa/phân nhóm người dùng từ hệ thống quản trị nhân sự công ty vào HiTechCloud Agent Platform.
- **Unified Identity Plane**: Một người dùng sở hữu 1 danh tính duy nhất nhưng có hạn mức và quyền truy cập khác nhau trên từng Workspace / Project / Tenant.

---

## 8. Supply Chain Security & An toàn Thực thi Sandbox

Đảm bảo an toàn tuyệt đối cho chuỗi cung ứng phần mềm AI Marketplace:
1. **Chữ ký số Cryptographic Signing (Ed25519 & Sigstore/Cosign)**: Toàn bộ packages (.hcp) đều được ký điện tử và xác thực trước khi cài đặt.
2. **Tự động Sinh SBOM (Software Bill of Materials)**: Hỗ trợ định dạng chuẩn SPDX và CycloneDX cho mọi Plugin/Skill.
3. **Tự động Quét Lỗ Hổng & Mã Độc (Static CVE & Secret Leak Scan)**: Phát hiện khóa API bị hardcode, hàm nguy hiểm (`eval`, `rm -rf`), hoặc CVE đã biết.
4. **Môi trường Cô lập Đa Tầng (Multi-Tier Isolation Sandbox)**:
   - **Tầng 1 (Lightweight)**: WASM Runtime (Wasmtime / WASI) cô lập bộ nhớ và hạn chế syscalls.
   - **Tầng 2 (Container Isolation)**: Linux cgroups v2, seccomp profiles, network namespaces hoặc gVisor/Firecracker microVMs cho các tác vụ cần chạy binary cục bộ.

---

## 9. Canonical AI Protocol Translation trong Rust Core

Rust Core Engine xử lý chuyển đổi chuẩn hóa giữa mọi định dạng giao tiếp qua `Canonical AI Model`:

```rust
pub enum AIRequest {
    Chat(ChatPayload),
    AgentTask(AgentTaskPayload),
    ToolCall(ToolCallPayload),
    A2ATask(A2ATaskPayload),
    MCPCall(MCPCallPayload),
    GraphQuery(GraphQueryPayload),
    WorkflowStep(WorkflowStepPayload),
}

pub trait AIProtocolAdapter: Send + Sync {
    fn to_canonical(&self, raw: &[u8]) -> Result<AIRequest, AdapterError>;
    fn from_canonical(&self, req: &AIRequest) -> Result<Vec<u8>, AdapterError>;
}
```
Nhờ cơ chế này, nền tảng hoạt động như một **Universal Translator** chuyển dịch thông suốt giữa OpenAI, Anthropic, Gemini, DeepSeek, MCP, A2A, OpenAPI và Microsoft Graph.
