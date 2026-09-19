# MCP Server Spec (đặc tả đăng ký vào Registry)

## Metadata bắt buộc khi đăng ký

```json
{
  "id": "kebab-case, duy nhất",
  "name": "Tên hiển thị",
  "description": "Mô tả server làm gì, tối đa 300 ký tự",
  "transport": "stdio | http | sse",
  "command": "lệnh khởi chạy (nếu transport=stdio)",
  "url": "endpoint (nếu transport=http|sse)",
  "env_schema": {
    "GITHUB_TOKEN": { "required": true, "secret": true }
  },
  "namespace": "tiền tố cho tool, vd 'github'",
  "permissions": ["github.read", "github.write"],
  "allowed_egress_domains": ["api.github.com"],
  "visibility": "public | org | private"
}
```

## Yêu cầu tuân thủ giao thức MCP

Server phải tuân theo MCP spec chuẩn của Anthropic (JSON-RPC 2.0 qua stdio/HTTP/SSE), hỗ trợ tối thiểu:
- `initialize`
- `tools/list`
- `tools/call`
- (tuỳ chọn) `resources/list`, `resources/read`, `prompts/list`

## Khai báo permission cho từng tool

Mỗi tool tự khai báo permission cần thiết trong `tools/list` response (mở rộng field, không phá chuẩn MCP):

```json
{
  "name": "create_issue",
  "description": "Tạo issue mới",
  "inputSchema": {"...": "..."},
  "x-hitechcloud-permission": "github.write"
}
```

Gateway đọc field `x-hitechcloud-permission` để map với RBAC ở [AUTH-RBAC.md](AUTH-RBAC.md) mà không cần publisher tự implement kiểm tra quyền bên trong server.

## Yêu cầu review trước khi public

1. Domain trong `allowed_egress_domains` phải cụ thể (không cho phép wildcard `*` trừ khi có lý do chính đáng và được duyệt riêng).
2. Không được yêu cầu biến môi trường không liên quan tới chức năng khai báo.
3. Server phải trả lỗi rõ ràng (không crash) khi thiếu quyền/credential, tránh lộ stack trace chứa thông tin nhạy cảm.

## Ví dụ

Xem [EXAMPLES/example-mcp-server.md](EXAMPLES/example-mcp-server.md).
