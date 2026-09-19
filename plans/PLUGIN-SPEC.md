# Plugin Spec (đặc tả kỹ thuật)

## Cấu trúc thư mục chuẩn

```
<plugin-name>/
├── plugin.json          # bắt buộc
├── skills/               # tùy chọn, 0..N
│   └── <skill-id>/SKILL.md
├── mcp.json              # tùy chọn
├── commands/             # tùy chọn
│   └── <command>.md
├── icon.png              # tùy chọn, hiển thị trên Marketplace
└── CHANGELOG.md          # khuyến nghị
```

## Schema `plugin.json`

```json
{
  "$schema": "https://schema.hitechcloud.vn/plugin/v1.json",
  "name": "string, bắt buộc, kebab-case, duy nhất trong registry",
  "version": "string, bắt buộc, semver",
  "description": "string, bắt buộc, tối đa 200 ký tự",
  "author": "string, bắt buộc",
  "license": "string, tùy chọn (vd MIT, Apache-2.0)",
  "homepage": "url, tùy chọn",
  "skills": ["array of skill-id, tùy chọn"],
  "mcp": ["array of mcp-server-id, tùy chọn"],
  "commands": ["array of command-id, tùy chọn"],
  "permissions": ["array of permission string, bắt buộc nếu >0 skill/mcp cần quyền"],
  "compatible_agents": ["claude-code", "claude-desktop", "codex-cli", "cursor", "..."],
  "min_platform_version": "string, tùy chọn"
}
```

## Quy tắc validate khi submit

1. `name` phải unique toàn registry, chỉ chứa `a-z0-9-`.
2. `version` phải tăng dần so với version trước (semver hợp lệ).
3. Mọi `permission` khai báo phải nằm trong tập permission hệ thống đã định nghĩa (không tự bịa permission string tùy ý).
4. Mọi `skills`/`mcp` tham chiếu phải tồn tại trong Registry (hoặc được submit kèm trong cùng release).
5. Nếu `permissions` chứa bất kỳ quyền nào thuộc nhóm nhạy cảm (`*.delete`, `*.write`, `billing.*`, `admin.*`) → tự động chuyển trạng thái `pending_review` thay vì auto-publish.

## Vòng đời version

```
draft → submitted → scanned → (pending_review | approved) → signed → published
```

## Ví dụ đầy đủ

Xem [EXAMPLES/example-plugin.md](EXAMPLES/example-plugin.md) và [EXAMPLES/example-hitechcloud-plugin.md](EXAMPLES/example-hitechcloud-plugin.md).
