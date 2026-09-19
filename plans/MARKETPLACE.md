# Marketplace

## Vai trò

Lớp trải nghiệm (UX + API công khai) đặt trên Registry, để người dùng cuối và tổ chức **tìm, xem, cài, đánh giá** MCP server / Skill / Plugin — giống App Store nhưng cho agent tooling.

## Tính năng chính

- **Catalog** phân loại theo category: DevOps, Security, Database, Coding, Marketing, HiTechCloud-native...
- **Trang chi tiết** mỗi entry: mô tả, permission yêu cầu, changelog, tác giả, trạng thái đã ký hay chưa, số lượt cài, đánh giá/review.
- **Cài đặt 1 click** — nút "Install" gọi tới Plugin Gateway/Registry để tạo `installation` record và đẩy config xuống agent tương ứng (Claude Code, Codex CLI...).
- **Quản lý theo tổ chức**: admin org duyệt entry nào member được phép cài (allowlist/denylist), đặc biệt với plugin xin permission nhạy cảm.
- **Review & rating** có kiểm duyệt để tránh spam/thao túng.
- **Trust badge**: "Verified Publisher" (HiTechCloud tự ký), "Community" (chưa review kỹ), "Deprecated".

## Luồng cài đặt điển hình

```
User tìm "dns management" trên Marketplace
        │
        ▼
Chọn plugin "hitechcloud-server" (Verified)
        │
        ▼
Xem permission yêu cầu: server.read, dns.read, server.service.restart
        │
        ▼
Nhấn Install → chọn scope (user/project/org)
        │
        ▼
Registry tạo installation → Gateway nạp MCP + Skill + Command
        │
        ▼
Agent (Claude Code / Codex) thấy plugin sẵn sàng dùng ngay
```

## Trang danh mục gợi ý

```
/marketplace
├── /mcp-servers
├── /skills
├── /plugins
├── /featured          # HiTechCloud tự chọn nổi bật
├── /verified           # đã audit + ký
└── /category/{devops|security|database|coding|...}
```

## Chính sách nội dung

- Không cho phép plugin/skill chứa hướng dẫn tạo mã độc, vượt rào bảo mật, hay thu thập dữ liệu người dùng trái phép.
- Plugin xin quyền `*.write`, `*.delete`, truy cập secret cần qua review thủ công trước khi publish public (org-private thì chủ org tự chịu trách nhiệm, review nhẹ hơn).
- Có cơ chế báo cáo (report) + gỡ khẩn cấp (kill-switch) khi phát hiện plugin độc hại sau khi đã publish.

Xem thêm phần dành cho nhà phát triển: [DEVELOPER-PORTAL.md](DEVELOPER-PORTAL.md).
