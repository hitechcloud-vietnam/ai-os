# Tích hợp Anthropic (Claude)

## Các điểm tích hợp

| Sản phẩm Claude | Cách HiTechCloud tích hợp |
|---|---|
| Claude.ai / Claude API | Đăng ký HiTechCloud MCP Gateway như một MCP connector (remote MCP server qua HTTP/SSE) |
| Claude Code | Đăng ký HiTechCloud Registry như một **plugin marketplace** (`claude plugin marketplace add hitechcloud`) |
| Claude Desktop | Cấu hình qua `mcp.json` trỏ tới Gateway endpoint |
| Agent Skills (public repo của Anthropic) | Skill của HiTechCloud tuân theo cùng định dạng `SKILL.md` + frontmatter để tương thích ngược |

## MCP — remote server

Claude hỗ trợ kết nối MCP server từ xa qua HTTP/SSE. HiTechCloud Gateway expose 1 endpoint:

```
https://gw.hitechcloud.vn/mcp/v1
Authorization: Bearer <hitechcloud-sk_api_key>
```

Claude/Claude Code coi toàn bộ HiTechCloud Gateway như một MCP server duy nhất, với tool list được Gateway tổng hợp động từ mọi MCP server đã cài (namespaced theo `server_id.tool_name`).

## Claude Code Plugin Marketplace

```bash
claude plugin marketplace add https://registry-mcp.hitechcloud.vn/claude-marketplace.json
claude plugin install hitechcloud-server
```

File `claude-marketplace.json` do Adapter Layer tự sinh từ dữ liệu Registry, mapping:

```
HiTechCloud Plugin  →  Claude Code Plugin
 ├── skills/*        →  skills/*  (giữ nguyên SKILL.md)
 ├── mcp.json        →  .mcp.json
 └── commands/*      →  commands/*.md (slash commands)
```

## Agent Skills tương thích ngược

Vì Anthropic đã công bố public repo cho Agent Skills, HiTechCloud Skill format cố tình giữ **tương thích 1-1** (cùng frontmatter: `name`, `description`, version tùy chọn) để:
- Skill HiTechCloud có thể copy thẳng vào `/mnt/skills/` hoặc thư mục skill của Claude Code mà không cần convert.
- Skill từ cộng đồng Anthropic có thể import ngược vào HiTechCloud Registry.

## Giới hạn cần lưu ý

- Claude Code hiện quản lý plugin/skill theo file trong repo/máy cục bộ; HiTechCloud đóng vai trò **nguồn phân phối** (marketplace), việc "cài" thực chất vẫn là Claude Code tải file về máy qua registry URL — không có runtime riêng nằm giữa.
- Vì API và tính năng của Claude Code/Claude thay đổi khá thường xuyên, Adapter Layer nên kiểm tra `docs.claude.com` định kỳ (hoặc qua CI job) để phát hiện thay đổi định dạng `mcp.json`/plugin manifest.
