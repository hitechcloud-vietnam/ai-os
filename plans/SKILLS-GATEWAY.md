# Skills Gateway

## Skill là gì (trong hệ này)

Một **Skill** là gói hướng dẫn + tài nguyên (không nhất thiết là code chạy được) giúp agent làm tốt hơn một loại việc cụ thể: cách viết docx chuẩn, cách review PR theo playbook công ty, cách vận hành HiTechCloud Panel...

```
Skill
 ├── SKILL.md          # instructions + metadata (frontmatter: name, description, trigger)
 ├── knowledge/         # tài liệu tham khảo, checklist
 ├── scripts/           # script hỗ trợ (optional)
 └── mcp-refs.yaml       # (optional) khai báo MCP tool mà skill này cần dùng
```

Khác với MCP server (cung cấp *tool* để gọi), Skill cung cấp *ngữ cảnh + quy trình*. Một Plugin có thể bundle cả hai.

## Vai trò của Skills Gateway

- **Discovery**: agent hỏi "có skill nào liên quan tới X không?" → Gateway search theo keyword/description, trả về danh sách phù hợp (giống cơ chế `search_skills`/`suggest_skills`).
- **Serving nội dung**: khi skill được kích hoạt, Gateway trả nội dung `SKILL.md` + file liên quan cho agent đọc (qua `view`/tool đọc file của agent, hoặc trả thẳng qua API).
- **Versioning**: mỗi skill có version, agent có thể pin version cụ thể để tránh thay đổi hành vi đột ngột.
- **Scope**: skill có thể ở scope `user` (riêng cá nhân), `org` (dùng chung trong công ty), `public` (marketplace).
- **Trigger matching**: mỗi skill khai báo mô tả/từ khóa kích hoạt để Gateway hoặc agent tự quyết định khi nào nên nạp skill này (tương tự cách Claude quyết định đọc SKILL.md nào).

## Luồng hoạt động

```
Agent: "review PR này theo chuẩn công ty"
   │
   ▼
Skills Gateway.search(keywords=["pr","review","guideline"])
   │
   ▼
Trả về: skill "pr-review-hitechcloud" (org-scope, version 2.3)
   │
   ▼
Agent tải SKILL.md + checklist.md, áp dụng vào review
```

## Định dạng SKILL.md (frontmatter)

```markdown
---
name: pr-review-hitechcloud
description: Review PR theo chuẩn coding & security của HiTechCloud. Dùng khi user nhắc "review PR", "code review", "kiểm tra pull request".
version: 2.3.0
scope: org
requires_mcp: [github]
---

# PR Review Checklist — HiTechCloud

1. Kiểm tra không có secret/API key bị commit...
2. ...
```

## Tương thích Anthropic & OpenAI

- **Anthropic/Claude**: format gần như 1-1 với "Agent Skills" (`SKILL.md` + frontmatter) mà Anthropic đã công bố public — có thể publish thẳng làm Claude Code plugin marketplace entry.
- **OpenAI/Codex**: OpenAI chưa có khái niệm "Skill" chuẩn hóa; Adapter sẽ nén nội dung Skill thành **system/developer message** hoặc **file trong context** khi gọi Responses API, tuỳ giới hạn độ dài mà cắt/tóm tắt.

## Khác biệt Skill vs Plugin vs MCP tool

| | Cung cấp gì | Có gọi API/network không |
|---|---|---|
| MCP tool | Hành động cụ thể (đọc file, gọi API) | Có |
| Skill | Hướng dẫn/ngữ cảnh/quy trình | Không (thường) |
| Plugin | Đóng gói Skill + MCP + Command | Có thể có |

Xem đặc tả đầy đủ: [SKILL-SPEC.md](SKILL-SPEC.md).
