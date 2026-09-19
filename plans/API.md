# API tổng quan

> Đặc tả OpenAPI đầy đủ: [OPENAPI.md](OPENAPI.md)

## Base URL

```
https://api-mcp.hitechcloud.vn/v1
```

## Nhóm endpoint

### Registry (control plane)
```
POST   /registry/mcp-servers
GET    /registry/mcp-servers/{id}
POST   /registry/skills
GET    /registry/skills/search?q=
POST   /registry/plugins
GET    /registry/plugins/{id}
GET    /registry/plugins/{id}/versions
```

### Installations
```
POST   /installations                # cài 1 mcp/skill/plugin
GET    /installations?scope=org
DELETE /installations/{id}
```

### Gateway (data plane, agent gọi trực tiếp)
```
POST   /mcp/v1                       # JSON-RPC MCP chuẩn (list_tools, call_tool...)
GET    /mcp/v1/sse                   # kênh SSE cho MCP streaming
POST   /tool-execute                 # dùng cho adapter OpenAI/Codex (không native MCP)
```

### Marketplace
```
GET    /marketplace/search?q=&category=
GET    /marketplace/featured
POST   /marketplace/{entity}/{id}/reviews
```

### Admin / Org
```
GET    /orgs/{id}/members
POST   /orgs/{id}/roles
POST   /orgs/{id}/policies            # allowlist/denylist plugin
GET    /orgs/{id}/audit-logs
GET    /orgs/{id}/usage
```

## Xác thực

```
Authorization: Bearer hitechcloud-sk_live_xxxxxxxxxxxx
```

## Mã lỗi chuẩn hoá

| HTTP | Mã nội bộ | Ý nghĩa |
|---|---|---|
| 401 | `unauthorized` | API key sai/hết hạn |
| 403 | `forbidden` | Thiếu permission cho tool/resource |
| 404 | `not_found` | Entity không tồn tại hoặc đã bị yank |
| 409 | `version_conflict` | Cài version không khớp dependency |
| 422 | `permission_diff_required` | Cần xác nhận thủ công vì version mới xin thêm quyền |
| 429 | `rate_limited` | Vượt quota |
| 502 | `upstream_mcp_error` | MCP server đích lỗi/timeout |

## Phân trang & lọc

Chuẩn `?page=`, `?per_page=` (mặc định 20, tối đa 100), lọc theo `?category=`, `?scope=`, `?status=published|deprecated`.

## Webhook (cho Registry → hệ thống ngoài)

```
POST <callback_url>
{
  "event": "plugin.published" | "plugin.revoked" | "installation.created",
  "entity_id": "...",
  "version": "...",
  "occurred_at": "2026-09-19T10:00:00Z"
}
```
