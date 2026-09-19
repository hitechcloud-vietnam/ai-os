# Rust Implementation

## Workspace layout

```
hitechcloud-agent-platform/
├── Cargo.toml               # workspace root
├── crates/
│   ├── ai-gateway/           # entrypoint chung: auth, RBAC, rate-limit, routing
│   ├── mcp-gateway/          # aggregation + proxy MCP servers
│   ├── skills-service/       # CRUD + search skill
│   ├── plugin-service/       # CRUD + cài đặt plugin
│   ├── registry/              # control plane API
│   ├── signing/                # ký/verify package (ed25519)
│   ├── sandbox-runner/         # chạy MCP server chưa verified trong sandbox
│   ├── adapter-anthropic/      # dịch sang/định dạng Claude
│   ├── adapter-openai/         # dịch sang/định dạng OpenAI function schema
│   ├── hitechcloud/                 # CLI cho publisher & admin
│   └── common/                  # types, error, telemetry dùng chung
```

## Thư viện gợi ý

| Nhu cầu | Crate |
|---|---|
| HTTP server | `axum` |
| Async runtime | `tokio` |
| DB (Postgres) | `sqlx` (compile-time checked query) |
| Cache/rate-limit | `redis` (crate `redis-rs`) |
| Ký/verify | `ed25519-dalek` |
| JSON Schema validate | `jsonschema` |
| OpenAPI codegen | `utoipa` (sinh OpenAPI từ code) hoặc ngược lại `openapi-generator` |
| Observability | `tracing` + `tracing-opentelemetry` |
| Config | `figment` hoặc `config` crate |

## Ví dụ route Axum cho MCP Gateway (rút gọn)

```rust
use axum::{routing::post, Router, Json};
use serde_json::Value;

async fn mcp_rpc_handler(Json(payload): Json<Value>) -> Json<Value> {
    // 1. xác thực (middleware đã chạy trước)
    // 2. xác định method: tools/list | tools/call
    // 3. route tới MCP server tương ứng theo namespace trong payload
    // 4. ghi audit log (async, không block response)
    Json(serde_json::json!({ "jsonrpc": "2.0", "result": {} }))
}

pub fn router() -> Router {
    Router::new().route("/mcp/v1", post(mcp_rpc_handler))
}
```

## Middleware pipeline gợi ý

```
Request
  → tracing span
  → auth middleware (verify Bearer token)
  → RBAC middleware (check permission theo route/tool)
  → rate-limit middleware (redis token bucket)
  → handler
  → audit-log middleware (ghi async sau khi có response)
```

## Testing

- Unit test cho từng crate (`cargo test`).
- Integration test dựng docker-compose tạm (Postgres + Redis thật) chạy trong CI.
- Contract test cho Adapter (Anthropic/OpenAI) — snapshot test đảm bảo format output không đổi ngoài ý muốn khi refactor.

Xem thêm chiến lược test tổng thể: [TESTING.md](TESTING.md).
