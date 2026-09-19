# Skill Spec (đặc tả kỹ thuật)

## Frontmatter bắt buộc (`SKILL.md`)

```yaml
---
name: string, bắt buộc, kebab-case, duy nhất trong scope
description: string, bắt buộc, tối đa 300 ký tự — dùng để search/trigger, nên nêu rõ "dùng khi nào"
version: string, tùy chọn nhưng khuyến nghị, semver
scope: user | org | public, bắt buộc
requires_mcp: [array of mcp-server-id, tùy chọn]
tags: [array of string, tùy chọn — phục vụ filter trên marketplace]
---
```

## Nội dung thân bài

Sau frontmatter là nội dung Markdown thuần: hướng dẫn, quy trình, checklist, ví dụ. Có thể tham chiếu file phụ trong cùng thư mục:

```
<skill-id>/
├── SKILL.md
├── knowledge/
│   └── checklist.md
└── scripts/
    └── validate.py       # tùy chọn, script hỗ trợ (không tự chạy — agent tự quyết định có chạy hay không)
```

## Nguyên tắc viết `description` tốt

- Nêu **khi nào nên dùng** skill này, không chỉ tên: "Dùng khi user nhắc 'review PR', 'kiểm tra pull request', hoặc yêu cầu áp chuẩn coding công ty."
- Tránh mô tả chung chung ("hỗ trợ code") vì search/trigger sẽ kém chính xác.

## Quy tắc validate khi submit

1. `name` unique trong đúng `scope` khai báo (2 org khác nhau có thể trùng tên skill scope `org` của riêng họ).
2. Không cho phép nội dung skill chứa hướng dẫn vượt rào an toàn (bypass safety, tạo mã độc...) — quét tự động + review nếu nghi ngờ.
3. Nếu `requires_mcp` tham chiếu server không tồn tại/không public → cảnh báo cho tác giả, không chặn cứng (vì có thể server đó là private trong org khác).

## Tương thích ngược với Anthropic Agent Skills

Định dạng trên cố ý bám sát chuẩn `SKILL.md` public của Anthropic để:
- Có thể copy 1 skill HiTechCloud thẳng vào thư mục skill cục bộ của Claude Code mà không cần chuyển đổi.
- Import skill cộng đồng Anthropic vào HiTechCloud Registry chỉ cần bổ sung field `scope`.

## Ví dụ

Xem [EXAMPLES/example-skill.md](EXAMPLES/example-skill.md).
