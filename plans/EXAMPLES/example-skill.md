# Ví dụ: Skill "hitechcloud-panel"

## Cấu trúc

```
hitechcloud-panel/
├── SKILL.md
└── knowledge/
    └── troubleshooting.md
```

## `SKILL.md`

```markdown
---
name: hitechcloud-panel
description: >
  Hướng dẫn thao tác trên HiTechCloud Panel: xem trạng thái server,
  khởi động lại service, quản lý bản ghi DNS. Dùng khi user nhắc
  "server của tôi", "restart nginx", "thêm bản ghi DNS", "panel HiTechCloud".
version: 1.2.0
scope: public
requires_mcp: [hitechcloud-api]
tags: [devops, hitechcloud, server, dns]
---

# Vận hành HiTechCloud Panel

## Khi user hỏi về trạng thái server

1. Gọi tool `hitechcloud.server_status` với `server_id` tương ứng.
2. Trình bày kết quả dạng bảng: CPU, RAM, Disk, danh sách service và trạng thái.
3. Nếu CPU/RAM > 85%, chủ động cảnh báo và gợi ý kiểm tra tiến trình ngốn tài nguyên.

## Khi user muốn restart service

1. Xác nhận lại tên service và server_id nếu chưa rõ ràng.
2. Gọi tool `hitechcloud.restart_service`.
3. Sau khi restart, gọi lại `hitechcloud.server_status` để xác nhận service đã "running".

## Khi gặp lỗi

Xem thêm `knowledge/troubleshooting.md` cho các mã lỗi thường gặp và cách xử lý.
```

## `knowledge/troubleshooting.md` (trích)

```markdown
# Troubleshooting HiTechCloud Panel

- Lỗi `503 service_unavailable` khi gọi restart_service: thường do service đang
  trong quá trình restart trước đó, đợi 5-10s rồi thử lại thay vì gọi liên tục.
- Lỗi `403` từ MCP server: kiểm tra API key có quyền `server.service.restart`
  hay chưa trước khi báo lỗi cho user.
```
