# MCP Gateway & Multi-Protocol Multiplexer

## Vai trò

Gom nhiều MCP server (GitHub, AWS, Cloudflare, Database, HiTechCloud Panel, Docker, Filesystem...) thành **một endpoint MCP duy nhất** mà mọi agent và 3rd-party CLI kết nối vào, thay vì phải cấu hình từng server riêng lẻ.

## Chức năng Cốt lõi

- **Aggregation**: expose nhiều MCP server qua 1 URL (`https://api-mcp.hitechcloud.vn/mcp/v1` và `https://gw.hitechcloud.vn/mcp/sse`), tools được namespace theo `server_id.tool_name` để tránh trùng tên.
- **Tương thích Toàn diện CLI Bên thứ ba (3rd-Party CLI Interoperability)**:
  - Hỗ trợ đầy đủ các kết nối từ: **Claude Code CLI**, **Block Goose**, **Aider AI**, **Charm Crush**, **Anomaly OpenCode**, **SimCode CLI**, **Cursor**, **Windsurf**, **Roo Code**, **Zed Editor**.
  - Xem chi tiết tại [THIRD-PARTY-CLI-INTEGRATION.md](THIRD-PARTY-CLI-INTEGRATION.md).
- **Transport Bridging & Multiplexing**:
  - `stdio`: Hỗ trợ cầu nối cục bộ qua lệnh `hitechcloud mcp serve`.
  - `SSE (Server-Sent Events)`: Truyền phát sự kiện thời gian thực trên `gw.hitechcloud.vn`.
  - `Streamable HTTP / JSON-RPC 2.0`: Tiếp nhận request đồng bộ và bất đồng bộ theo chuẩn MCP spec mới nhất.
- **Auth pass-through & vaulting**: Credential thật của từng MCP server (token GitHub, AWS key...) được lưu ở Gateway (Vault/KMS), agent không bao giờ nhìn thấy secret gốc — chỉ dùng API key HiTechCloud của mình.
- **RBAC theo tool**: Một role có thể được cấp `github.read` nhưng không có `github.write`, `server.service.restart` nhưng không có `server.delete`.
- **Rate limit & quota** theo user/org/tool qua Redis Token-Bucket.
- **Health check & failover**: Tự động đánh dấu server lỗi, trả mã lỗi chuẩn JSON-RPC thay vì treo request.
- **Audit**: Log mọi tool call (ai, tool nào, tham số gì — có thể mask dữ liệu nhạy cảm), phục vụ compliance.

## Kiến trúc Route & Protocol Multiplexer

```
      ┌────────────────────────────────────────────────────────────┐
      │               3RD-PARTY CLIS & AGENT INTERFACES            │
      │  Claude Code · Goose · Aider · Crush · OpenCode · SimCode  │
      │  Cursor · Windsurf · Roo Code · Zed Editor · Codex CLI     │
      └─────────────────────────────┬──────────────────────────────┘
                                    │
                ┌───────────────────┼───────────────────┐
                ▼                   ▼                   ▼
         STDIO PROXY PIPES     SSE CHANNELS        REST JSON-RPC
        (hitechcloud serve)  (gw.hitechcloud.vn) (api-mcp.hitechcloud.vn)
                │                   │                   │
                └───────────────────┼───────────────────┘
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                   HITECHCLOUD MCP GATEWAY (RUST WORKSPACE)                │
│  • JSON-RPC 2.0 Router    • Token-Bucket Limiter   • RBAC Policy Engine  │
├─────────────────┬───────────────────┬──────────────────┬─────────────────┤
│  github-mcp     │  postgres-mcp     │  docker-sandbox  │  custom-mcp     │
│  (github.*)     │  (postgres.*)     │  (docker.*)      │  (custom.*)     │
└─────────────────┴───────────────────┴──────────────────┴─────────────────┘
```

## Cấu hình Mẫu (Registry Entry)

```yaml
mcp_server:
  id: github
  transport: stdio   # hoặc http, sse
  command: "npx -y @modelcontextprotocol/server-github"
  env:
    GITHUB_TOKEN: "${vault:github_token}"
  namespace: github
  permissions:
    - github.read
    - github.write
  rate_limit:
    per_minute: 60
  visibility: org   # public | org | private
```

## Tương thích Anthropic & OpenAI

- **Claude / Claude Code / Claude Desktop**: MCP là chuẩn gốc của Anthropic, nên Gateway chỉ cần expose theo `mcp.json` chuẩn để Claude Code kết nối trực tiếp.
- **OpenAI/Codex**: OpenAI chưa dùng MCP làm chuẩn native ở mọi sản phẩm, nên Adapter Layer cần dịch từng tool MCP (tool schema JSON) sang định dạng `tools` của Chat Completions/Responses API. Gateway giữ 1 bảng ánh xạ `mcp_tool -> openai_function_schema` để tự sinh.

## Vấn đề Bảo mật Cần Lưu ý

- Coi kết quả trả về từ MCP server bên thứ ba là **dữ liệu chưa tin cậy** (có thể chứa prompt injection). Không tự động cho tool output override quyền hạn hay policy.
- Không log secret/token vào audit log dạng plaintext.
- Cho phép admin approve từng MCP server mới trước khi org được dùng (đặc biệt server cộng đồng, chưa ký).

Chi tiết đặc tả kỹ thuật: xem [MCP-SPEC.md](MCP-SPEC.md) và [THIRD-PARTY-CLI-INTEGRATION.md](THIRD-PARTY-CLI-INTEGRATION.md).
