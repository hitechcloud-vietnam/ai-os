# Đặc tả Kỹ thuật: AG-UI & A2UI (Agent-to-UI Interaction Layer)

Tài liệu này đặc tả giao thức tương tác giữa **Agent Runtime** và **Giao diện Người dùng (UI)**, bao gồm hai tiêu chuẩn cốt lõi: **AG-UI** (Giao thức truyền phát sự kiện Agent ↔ UI) và **A2UI** (Giao diện cấu trúc do Agent sinh ra).

---

## 1. Mục tiêu & Vị trí Kiến trúc

```
Agent Runtime (Rust Tokio Core)
             │
             ▼  AG-UI Event Stream (SSE / WebSocket)
     Frontend Adapter
             │
 ┌───────────┼───────────┐
 ▼           ▼           ▼
React     Next.js     Mobile / VS Code
             │
             ▼  A2UI Structured Component Rendering
Dynamic Interactive Widgets & Forms
```

- **AG-UI**: Chuẩn hóa luồng sự kiện truyền tải hai chiều giữa backend Agent và client web/extension.
- **A2UI**: Chuẩn hóa cấu trúc JSON mô tả các thành phần giao diện (Thẻ thông tin, bảng biểu, form nhập liệu, nút bấm hành động) để frontend render tương tác.

---

## 2. Đặc tả Giao thức Sự kiện AG-UI (AG-UI Events)

Toàn bộ các sự kiện được truyền qua kênh SSE (`gw.hitechcloud.vn/events`) hoặc WebSocket (`gw.hitechcloud.vn/ws/session`):

### 2.1. Danh mục Sự kiện Chuẩn (Event Catalog)

| Event Name | Hướng Truyền | Ý nghĩa |
|---|---|---|
| `agent.session.start` | Server → Client | Khởi tạo phiên làm việc của Agent |
| `agent.text.delta` | Server → Client | Chunk văn bản streaming (chữ chạy thời gian thực) |
| `agent.thought.delta` | Server → Client | Luồng suy luận nội bộ của reasoning models (o1/o3/DeepSeek R1) |
| `agent.tool.calling` | Server → Client | Thông báo Agent bắt đầu gọi một tool |
| `agent.tool.executed` | Server → Client | Kết quả thực thi tool kèm thời gian đo đạc |
| `agent.state.changed` | Server → Client | Chuyển đổi trạng thái (`idle`, `thinking`, `executing`, `interrupted`) |
| `agent.hitl.request` | Server → Client | Yêu cầu con người phê duyệt hành động (Human-In-The-Loop) |
| `client.hitl.response`| Client → Server | Phản hồi phê duyệt hoặc từ chối của người dùng |
| `client.interrupt` | Client → Server | Người dùng bấm nút "Dừng" (Stop generation) |

### 2.2. Chi tiết Cơ chế Human-in-the-Loop (HITL Interrupt & Approval)

Khi Agent thực hiện một hành động có độ rủi ro cao (ví dụ: thực thi lệnh drop database hoặc khởi động lại dịch vụ sản xuất), Gateway tự động phát sự kiện `agent.hitl.request`:

```json
{
  "event": "agent.hitl.request",
  "requestId": "hitl_req_01a",
  "agentId": "agent.hitechcloud.devops",
  "severity": "HIGH",
  "action": {
    "tool": "hitechcloud.restart_service",
    "parameters": { "service": "nginx", "graceful": false },
    "explanation": "Restarting Nginx will briefly terminate active HTTP connections for 200ms."
  },
  "timeoutSeconds": 60
}
```

Frontend hiển thị hộp thoại xác nhận cho lập trình viên. Khi bấm "Phê duyệt" hoặc "Hủy", client gửi lại:

```json
{
  "event": "client.hitl.response",
  "requestId": "hitl_req_01a",
  "approved": true,
  "modifiedParameters": null
}
```

---

## 3. Đặc tả Giao diện Cấu trúc A2UI (A2UI Schema)

Thay vì chỉ hiển thị text Markdown thông thường, Agent có thể trả về cấu trúc A2UI để Dashboard render thành widget tương tác sống động:

```json
{
  "a2uiVersion": "1.0.0",
  "components": [
    {
      "component": "Card",
      "props": {
        "title": "PostgreSQL Performance Alert",
        "badge": { "text": "High CPU", "color": "amber" }
      },
      "children": [
        {
          "component": "MetricGrid",
          "props": {
            "items": [
              { "label": "Active Queries", "value": "142" },
              { "label": "Cache Hit Ratio", "value": "89.2%" },
              { "label": "Disk IOPS", "value": "4,200" }
            ]
          }
        },
        {
          "component": "ActionGroup",
          "props": {
            "actions": [
              {
                "label": "Run EXPLAIN ANALYZE",
                "tool": "postgres.explain",
                "args": { "queryId": "q_99182" }
              },
              {
                "label": "Terminate Blocking Locks",
                "tool": "postgres.kill_locks",
                "args": {},
                "requiresConfirmation": true
              }
            ]
          }
        }
      ]
    }
  ]
}
```

---

## 4. Tích hợp SDK trong Rust và React

- **Rust Core SDK (`hitechcloud-agui-sdk`)**:
  Cung cấp `AguiStreamSender` để bắn sự kiện nhanh qua async mpsc channels của Tokio:
  ```rust
  sender.emit_tool_calling("postgres.explain", &args).await?;
  let output = runner.execute(&args).await?;
  sender.emit_tool_executed("postgres.explain", &output, duration_ms).await?;
  ```

- **React Hook (`useAguiStream`)**:
  ```typescript
  const { messages, state, isWaitingApproval, approveAction } = useAguiStream({
    endpoint: "https://gw.hitechcloud.vn/events",
    sessionId: currentSessionId,
  });
  ```
