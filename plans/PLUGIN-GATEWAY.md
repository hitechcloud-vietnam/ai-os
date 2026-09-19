# Plugin Gateway

## Plugin là gì

Một Plugin là **đơn vị đóng gói cài đặt được**, gộp:

```
plugin/
├── plugin.json        # metadata + version + permissions
├── skills/             # 0..N skill
│   └── <skill-name>/SKILL.md
├── mcp.json            # 0..N khai báo MCP server cần cài kèm
└── commands/           # 0..N slash-command / CLI command
```

Người dùng/tổ chức chỉ cần "Add"/"Install" 1 plugin, Gateway tự:
1. Cài các MCP server khai báo trong `mcp.json` (nếu org đã approve).
2. Đăng ký các Skill vào Skills Gateway ở đúng scope.
3. Đăng ký command (nếu agent hỗ trợ, ví dụ slash command trong Claude Code).
4. Áp permission tương ứng vào role của người cài (không tự ý cấp permission ngoài những gì user/org đã duyệt).

## plugin.json mẫu

```json
{
  "name": "hitechcloud-server",
  "version": "1.0.0",
  "description": "Quản lý server HiTechCloud: xem trạng thái, restart service, quản lý DNS.",
  "author": "HiTechCloud",
  "skills": ["linux-admin", "hitechcloud-panel", "dns-management"],
  "mcp": ["hitechcloud-api"],
  "commands": ["server-status", "restart-service"],
  "permissions": [
    "server.read",
    "server.service.restart",
    "dns.read"
  ],
  "compatible_agents": ["claude-code", "codex-cli", "cursor"]
}
```

## Vòng đời Plugin

```
draft → submitted → reviewed → signed → published → (deprecated/yanked)
```

- **submitted**: tác giả nộp plugin lên Registry.
- **reviewed**: quét tự động (malware scan, permission diff, secret scan) + review thủ công nếu plugin xin permission nhạy cảm (vd `server.delete`, `*.write`).
- **signed**: Registry ký plugin (xem [SIGNING-VERIFICATION.md](SIGNING-VERIFICATION.md)) để Gateway/agent xác thực tính toàn vẹn khi cài.
- **published**: hiển thị trên Marketplace, cài được.
- **deprecated/yanked**: gỡ khỏi marketplace nhưng vẫn track cho ai đã cài (để cảnh báo).

## Cài đặt (installation) theo scope

| Scope | Ý nghĩa |
|---|---|
| `user` | chỉ user đó dùng, trên mọi máy đăng nhập |
| `project` | chỉ trong 1 repo/project cụ thể (commit `plugin-lock.json` vào repo) |
| `org` | toàn bộ tổ chức, admin cài, member không tự gỡ được |

## Permission diffing khi update

Khi 1 version mới của plugin xin thêm permission so với version cũ đang cài, Gateway **không tự động** nâng cấp permission — phải có xác nhận rõ ràng từ user/admin, tương tự cách trình duyệt cảnh báo khi extension xin thêm quyền.

## Tương thích Anthropic & OpenAI

- **Claude Code**: gần khớp với khái niệm "plugin marketplace" mà Claude Code đã hỗ trợ (bundle skill + command + MCP). Có thể publish plugin HiTechCloud thẳng vào Claude Code marketplace bằng cách trỏ registry URL.
- **OpenAI/Codex**: Codex CLI có khái niệm extension/tool riêng; Adapter cần tách plugin thành (a) tool schema cho Codex, (b) nội dung skill nhúng vào system prompt/context.

Xem đặc tả đầy đủ: [PLUGIN-SPEC.md](PLUGIN-SPEC.md).
