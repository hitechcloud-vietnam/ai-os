# Đặc tả Kỹ thuật: OpenTelemetry & Giám sát Hệ thống (Observability)

Hệ thống Observability của **HiTechCloud Agent Platform** được xây dựng chuẩn theo tiêu chuẩn **OpenTelemetry (OTel)** với W3C Trace Context Propagation xuyên suốt toàn bộ chuỗi mắt xích AI: **Client / UI ──▶ Model Router ──▶ A2A Gateway ──▶ MCP Tool Engine ──▶ External APIs**.

---

## 1. Kiến trúc 3 Trụ Cột Observability

```
┌────────────────────────────────────────────────────────────────────────┐
│                        OPENTELEMETRY COLLECTOR                         │
├────────────────────────────────┬───────────────────────────────────────┤
│ 1. Traces (Distributed Spans)  │ Tempo / Jaeger (W3C TraceContext)     │
│ 2. Metrics (Prometheus / OTLP) │ Prometheus & Grafana Dashboard        │
│ 3. Logs (Structured JSON)      │ Vector ──▶ Loki / Elasticsearch       │
└────────────────────────────────┴───────────────────────────────────────┘
```

---

## 2. Distributed Tracing xuyên chuỗi AI Execution

Mỗi request từ người dùng hoặc agent được gán một `traceparent` header (W3C TraceContext standard: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`):

```
[User / VS Code Extension / AG-UI]
  │
  ▼ span: "agui.session.message" (trace_id: 4bf92f35...)
    ├── span: "auth.verify_oidc_token"
    ├── span: "router.select_model" (provider: "anthropic", model: "claude-3-7-sonnet")
    ├── span: "model.generate_content"
    │     └── attributes: gen_ai.usage.input_tokens=1450, output_tokens=320, cost_usd=0.0068
    ├── span: "a2a.delegate_task" (target_agent: "agent.hitechcloud.devops")
    │     ├── span: "a2a.task.inspect_server"
    │     └── span: "mcp.tool_call" (server: "postgres-mcp", tool: "query")
    │           └── span: "sandbox.wasm_isolate.execute" (duration: 12ms)
    └── span: "audit.record_event"
```

### OTel GenAI Semantic Conventions Attributes
Mọi span liên quan đến AI đều tự động đính kèm các thuộc tính chuẩn OpenTelemetry:
- `gen_ai.system`: `"anthropic" | "openai" | "deepseek" | "ollama"`
- `gen_ai.request.model`: `"claude-3-7-sonnet"`
- `gen_ai.usage.input_tokens`: `1450`
- `gen_ai.usage.output_tokens`: `320`
- `gen_ai.usage.cost_usd`: `0.00685`
- `gen_ai.agent.id`: `"agent.hitechcloud.devops"`
- `mcp.server.id`: `"postgres-mcp"`
- `mcp.tool.name`: `"query"`

---

## 3. Danh mục Metrics Prometheus Cốt lõi

```promql
# Throughput & Traffic
hitechcloud_gateway_requests_total{route, status, method}
hitechcloud_gateway_request_duration_seconds_bucket{route, le="0.1"}

# MCP Tool Execution
hitechcloud_mcp_tool_calls_total{server_id, tool_name, status}
hitechcloud_mcp_tool_duration_seconds{server_id, tool_name}

# AI Tokens & Financial Cost Tracking
hitechcloud_ai_tokens_consumed_total{org_id, model, token_type="input|output"}
hitechcloud_ai_cost_usd_total{org_id, model, user_id}

# A2A Coordination
hitechcloud_a2a_tasks_dispatched_total{source_agent, target_agent, status}
hitechcloud_a2a_task_queue_depth{priority}

# Security & Sandboxing
hitechcloud_rate_limit_rejections_total{org_id, client_type}
hitechcloud_sandbox_wasm_isolates_active
hitechcloud_security_signature_verification_failures_total
```

---

## 4. Cảnh báo Tự động (Operational Alerting Rules)

- **AI Token Spike Alert**: Số lượng token tiêu thụ vượt 300% ngưỡng trung bình trong 10 phút → Báo động và tạm áp dụng dynamic throttle.
- **MCP Server High Latency**: `p99(mcp_tool_duration_seconds) > 3.0s` liên tục trong 3 phút.
- **Tool Error Rate**: Tỷ lệ lỗi tool execution `> 5%` trong 5 phút → Gửi thông báo tới kênh webhook Telegram/Slack/Email.
- **Circuit Breaker Trip**: Một upstream MCP server bị timeout liên tục 5 lần → Kích hoạt ngắt mạch (Circuit Breaker) và trả về fallback tool schema.

