# Developer Portal

## Đối tượng dùng

Publisher/nhà phát triển muốn tạo và publish MCP server / Skill / Plugin lên HiTechCloud Marketplace.

## Các màn hình chính

```
/developer
├── /dashboard            # tổng quan: lượt cài, rating, doanh thu (nếu plugin trả phí)
├── /new                   # wizard tạo mới mcp-server/skill/plugin
├── /packages/{id}          # chi tiết 1 package: version history, review status
├── /packages/{id}/versions/new  # submit version mới, kèm CHANGELOG bắt buộc nếu đổi permission
├── /keys                   # quản lý signing key (xem SIGNING-VERIFICATION.md)
├── /payouts                # đối soát doanh thu revenue-share
└── /docs                   # liên kết tới MCP-SPEC / SKILL-SPEC / PLUGIN-SPEC
```

## Wizard tạo Skill (rút gọn)

1. Nhập `name`, `description` (gợi ý: viết rõ "dùng khi nào" để search chính xác hơn).
2. Chọn `scope`: user / org / public.
3. Soạn nội dung `SKILL.md` trực tiếp trên portal (có preview) hoặc upload thư mục.
4. Portal tự validate theo [SKILL-SPEC.md](SKILL-SPEC.md), báo lỗi nếu thiếu frontmatter bắt buộc.
5. Submit → vào hàng đợi scan tự động → publish (public thì cần thêm review thủ công).

## CLI thay thế UI cho publisher quen dùng terminal

```bash
hitechcloud login
hitechcloud skill init my-skill
hitechcloud skill validate ./my-skill
hitechcloud skill publish ./my-skill --version 1.0.0
hitechcloud plugin publish ./my-plugin --version 1.0.0
```

Xem đầy đủ lệnh CLI: [CLI.md](CLI.md).

## Yêu cầu xác minh publisher (KYC nhẹ)

Để đạt badge "Signed"/"Verified", publisher cần xác minh email + (tuỳ chọn) liên kết GitHub org để tăng độ tin cậy hiển thị công khai trên marketplace.

## Chính sách doanh thu (nếu bán plugin trả phí)

- Nền tảng giữ phí platform_fee% (cấu hình được, ví dụ 15-20%), phần còn lại trả publisher theo chu kỳ (vd hàng tháng), có bảng đối soát minh bạch tại `/developer/payouts`.
