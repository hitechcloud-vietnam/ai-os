# Ví dụ đầy đủ: Plugin "hitechcloud-server"

Đây là ví dụ nhạy cảm hơn (có quyền restart service), minh hoạ nhánh **cần review thủ công** trong vòng đời publish.

## Cấu trúc

```
hitechcloud-server/
├── plugin.json
├── mcp.json
├── skills/
│   ├── hitechcloud-panel/SKILL.md      (xem example-skill.md)
│   ├── linux-admin/SKILL.md
│   └── dns-management/SKILL.md
├── commands/
│   ├── server-status.md
│   └── restart-service.md
└── CHANGELOG.md
```

## `plugin.json`

```json
{
  "name": "hitechcloud-server",
  "version": "1.0.0",
  "description": "Quản lý server HiTechCloud: trạng thái, restart service, DNS.",
  "author": "HiTechCloud",
  "license": "proprietary",
  "skills": ["hitechcloud-panel", "linux-admin", "dns-management"],
  "mcp": ["hitechcloud-api"],
  "commands": ["server-status", "restart-service"],
  "permissions": [
    "server.read",
    "server.service.restart",
    "dns.read",
    "dns.write"
  ],
  "compatible_agents": ["claude-code", "claude-desktop", "codex-cli"]
}
```

## `commands/restart-service.md`

```markdown
---
description: Khởi động lại một service trên server HiTechCloud
argument-hint: "<server_id> <service_name>"
---

Xác nhận với user tên service và server trước khi thực hiện.
Gọi tool `hitechcloud.restart_service`, sau đó gọi `hitechcloud.server_status` để xác nhận
service đã chạy lại bình thường. Nếu thất bại, hiển thị log lỗi rút gọn
và gợi ý các bước kiểm tra tiếp theo (xem knowledge/troubleshooting.md
trong skill hitechcloud-panel).
```

## `CHANGELOG.md`

```markdown
## 1.0.0
- Phát hành lần đầu.
- Quyền yêu cầu: server.read, server.service.restart, dns.read, dns.write.
```

## Vì sao plugin này cần review thủ công

Theo quy tắc ở [PLUGIN-SPEC.md](PLUGIN-SPEC.md), permission `server.service.restart` và `dns.write` thuộc nhóm có thể ảnh hưởng trực tiếp tới hạ tầng đang chạy của khách hàng → tự động chuyển trạng thái `pending_review`, cần admin/reviewer HiTechCloud duyệt trước khi publish public (dù publish nội bộ trong 1 org thì chủ org tự quyết định, không cần review từ HiTechCloud).
