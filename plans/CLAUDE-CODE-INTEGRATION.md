# Tích hợp Claude Code (chi tiết)

## Cách Claude Code phát hiện HiTechCloud

```bash
# Thêm marketplace HiTechCloud
claude plugin marketplace add https://registry-mcp.hitechcloud.vn/claude-marketplace.json

# Xem danh sách plugin
claude plugin list --marketplace hitechcloud

# Cài 1 plugin
claude plugin install hitechcloud-server

# Cài theo scope project (lưu vào repo, cả team dùng chung)
claude plugin install hitechcloud-server --scope project
```

## File `claude-marketplace.json` (do Adapter tự sinh)

```json
{
  "name": "hitechcloud",
  "owner": "HiTechCloud",
  "plugins": [
    {
      "name": "hitechcloud-server",
      "version": "1.0.0",
      "source": "https://registry-mcp.hitechcloud.vn/plugins/hitechcloud-server/1.0.0.tar.gz",
      "sha256": "…",
      "signature": "…"
    }
  ]
}
```

## Cấu trúc plugin giải nén cho Claude Code

```
hitechcloud-server/
├── .claude-plugin/
│   └── plugin.json
├── skills/
│   ├── linux-admin/SKILL.md
│   ├── hitechcloud-panel/SKILL.md
│   └── dns-management/SKILL.md
├── .mcp.json
└── commands/
    ├── server-status.md
    └── restart-service.md
```

`.mcp.json` sinh ra trỏ thẳng về HiTechCloud MCP Gateway (không cần chạy server cục bộ):

```json
{
  "mcpServers": {
    "hitechcloud": {
      "url": "https://gw.hitechcloud.vn/mcp/v1",
      "headers": { "Authorization": "Bearer ${hitechcloud-sk_API_KEY}" }
    }
  }
}
```

## Slash command mẫu (`commands/server-status.md`)

```markdown
---
description: Kiểm tra trạng thái server HiTechCloud hiện tại
---

Gọi tool `hitechcloud.server_status` với server ID người dùng cung cấp,
trình bày kết quả dạng bảng: CPU, RAM, disk, trạng thái service.
```

## Đồng bộ khi có bản cập nhật

Khi Registry publish version mới:
1. Cập nhật `claude-marketplace.json` (tăng version, sha256 mới).
2. Claude Code tự phát hiện bản mới khi user chạy `claude plugin update hitechcloud-server` (hoặc tự động theo cấu hình org).
3. Nếu version mới xin thêm permission → hiển thị cảnh báo trước khi cập nhật (theo nguyên tắc ở [PLUGIN-GATEWAY.md](PLUGIN-GATEWAY.md)).
