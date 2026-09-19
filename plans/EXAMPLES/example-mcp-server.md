# Ví dụ: MCP Server "hitechcloud-api"

## Đăng ký vào Registry

```json
{
  "id": "hitechcloud-api",
  "name": "HiTechCloud Panel API",
  "description": "Quản lý server, dịch vụ, DNS trên HiTechCloud Panel.",
  "transport": "http",
  "url": "https://mcp.hitechcloud.vn/panel",
  "namespace": "hitechcloud",
  "permissions": ["server.read", "server.service.restart", "dns.read", "dns.write"],
  "allowed_egress_domains": ["control-mcp.hitechcloud.vn"],
  "visibility": "public"
}
```

## Danh sách tool (rút gọn)

```json
[
  {
    "name": "server_status",
    "description": "Xem trạng thái CPU/RAM/disk/service của 1 server",
    "inputSchema": {
      "type": "object",
      "properties": { "server_id": { "type": "string" } },
      "required": ["server_id"]
    },
    "x-hitechcloud-permission": "server.read"
  },
  {
    "name": "restart_service",
    "description": "Khởi động lại 1 service trên server",
    "inputSchema": {
      "type": "object",
      "properties": {
        "server_id": { "type": "string" },
        "service_name": { "type": "string" }
      },
      "required": ["server_id", "service_name"]
    },
    "x-hitechcloud-permission": "server.service.restart"
  }
]
```

## Gọi thử qua Gateway (namespace `hitechcloud`)

```bash
hitechcloud tool call hitechcloud.server_status --args '{"server_id":"srv-01"}'
```

Kết quả (ví dụ):

```json
{
  "server_id": "srv-01",
  "cpu_percent": 12,
  "ram_percent": 44,
  "disk_percent": 61,
  "services": [{ "name": "nginx", "status": "running" }]
}
```
