# Đặc tả Kỹ thuật: A2A (Agent2Agent) Gateway

Tài liệu này đặc tả kiến trúc phân hệ **A2A Gateway** của **HiTechCloud Agent Platform**, tuân thủ tiêu chuẩn **A2A Protocol v1.0** (Linux Foundation), phục vụ bài toán khám phá, phối hợp và giao việc giữa các Agent AI tự trị.

---

## 1. Vấn đề A2A Giải quyết

Trong khi **MCP** chuẩn hóa cách một **Agent sử dụng Tool/Data**, thì **A2A** chuẩn hóa cách một **Agent giao tiếp và phối hợp với Agent khác**:

```
MCP  : Agent ──▶ Tool / Data
A2A  : Agent A ──▶ Agent B ──▶ Agent C
```

Ví dụ trong môi trường doanh nghiệp của HiTechCloud:
1. `Triage Agent` tiếp nhận yêu cầu từ lập trình viên: *"Hạ tầng web đang quá tải, hãy kiểm tra và tối ưu database"*.
2. `Triage Agent` sử dụng **A2A Discovery** để tìm các agent có năng lực phù hợp.
3. Giao task cho `Infrastructure Agent` kiểm tra CPU/RAM máy chủ.
4. Giao task cho `Database Optimizer Agent` phân tích slow query log và tạo index.
5. Thu thập kết quả từ cả hai agent qua luồng A2A streaming và phản hồi kết quả tổng hợp cho người dùng.

---

## 2. Kiến trúc A2A Gateway trong Rust Core

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          HITECHCLOUD A2A GATEWAY                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ 1. Agent Discovery Service  : Tìm kiếm agent theo năng lực (Capabilities)   │
│ 2. Agent Card Registry      : Quản lý metadata & endpoint của từng Agent    │
│ 3. Task Dispatcher & Queue  : Điều phối nhiệm vụ, quản lý trạng thái task   │
│ 4. A2A Auth & Policy Engine : Xác thực token mTLS/OAuth2 giữa các Agent     │
│ 5. Event Stream Multiplexer : Quản lý luồng streaming kết quả A2A           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Đặc tả Chuẩn Giao tiếp A2A v1.0

### 3.1. Cấu trúc Agent Card (`AgentCard`)
Agent Card là định dạng tự mô tả của mỗi Agent, được lưu trữ trên Registry:
```json
{
  "$schema": "https://a2a-protocol.org/v1/agent-card.schema.json",
  "id": "agent.hitechcloud.devops",
  "version": "1.0.0",
  "name": "DevOps Automation Agent",
  "description": "Specialized agent for Kubernetes, Nginx and server operations",
  "publisher": {
    "name": "HiTechCloud Infrastructure Team",
    "verified": true,
    "signature": "ed25519:..."
  },
  "capabilities": [
    {
      "name": "server.inspect",
      "description": "Inspect CPU, memory, disk and running systemd services",
      "inputSchema": {
        "type": "object",
        "properties": { "serverId": { "type": "string" } },
        "required": ["serverId"]
      }
    },
    {
      "name": "service.restart",
      "description": "Restart system service (e.g. nginx, postgresql)",
      "inputSchema": {
        "type": "object",
        "properties": { "serviceName": { "type": "string" } },
        "required": ["serviceName"]
      }
    }
  ],
  "endpoints": {
    "taskDispatch": "https://api-mcp.hitechcloud.vn/a2a/v1/tasks",
    "taskStream": "https://gw.hitechcloud.vn/a2a/v1/stream"
  },
  "auth": {
    "type": "oauth2_bearer",
    "scopes": ["agent:delegate", "agent:read"]
  }
}
```

### 3.2. Vòng đời Task A2A (Task Lifecycle)

```
[Created] ──▶ [Assigned] ──▶ [In_Progress] ──┬──▶ [Completed]
                                             ├──▶ [Requires_Input / HITL]
                                             └──▶ [Failed]
```

- **Khởi tạo Task (Dispatch)**:
  `POST /a2a/v1/tasks`
  ```json
  {
    "taskId": "task_991823ab",
    "sourceAgentId": "agent.hitechcloud.triage",
    "targetAgentId": "agent.hitechcloud.devops",
    "action": "server.inspect",
    "input": { "serverId": "srv-master-01" },
    "timeoutMs": 30000,
    "callbackUrl": "https://api-mcp.hitechcloud.vn/a2a/v1/callbacks"
  }
  ```

- **Phản hồi Streaming (SSE Stream)**:
  `GET /a2a/v1/stream?taskId=task_991823ab`
  ```
  event: a2a.task.progress
  data: {"percentage": 50, "step": "Connecting to server metrics agent..."}

  event: a2a.task.completed
  data: {"status": "SUCCESS", "result": {"cpu": 12.4, "ram": 45.1, "services": "all running"}}
  ```

---

## 4. Bảo mật & Ủy quyền trong A2A

- **Zero-Trust Delegation**: Agent A khi giao việc cho Agent B không được truyền toàn bộ master token của người dùng, mà phải dùng **Scoped Delegation Token** (JWT có thời hạn ngắn, chỉ chứa quyền cần thiết để thực hiện task đó).
- **Audit Logging**: A2A Gateway ghi nhận nguồn gốc giao việc: `Originating User -> Agent A -> Agent B -> Tool Executed`.
