# Roadmap: MVP → Production

## Giai đoạn 0 — Thiết kế (tài liệu này)

- [x] Kiến trúc tổng thể, spec MCP/Skill/Plugin, mô hình bảo mật/permission.

## Giai đoạn 1 — MVP (4-6 tuần)

- [ ] Registry cơ bản: CRUD mcp_servers/skills/plugins, chưa cần signing.
- [ ] MCP Gateway: aggregate tối đa 3-5 MCP server phổ biến (GitHub, filesystem, Postgres), auth bằng API key đơn giản.
- [ ] Skills Gateway: search + serve nội dung `SKILL.md`, scope user/org.
- [ ] Adapter Anthropic: xuất `.mcp.json` cho Claude Code dùng thử nội bộ.
- [ ] CLI tối thiểu: `hitechcloud login`, `mcp publish`, `skill publish`.
- [ ] Audit log cơ bản (chưa cần masking nâng cao, chưa cần dashboard).

**Mục tiêu MVP**: team nội bộ HiTechCloud dùng được Claude Code kết nối qua Gateway thay vì cấu hình MCP server thủ công từng máy.

## Giai đoạn 2 — Beta có kiểm soát (2-3 tháng)

- [ ] RBAC đầy đủ theo permission string.
- [ ] Signing & verification cho package.
- [ ] Sandbox runner cho MCP server chưa verified.
- [ ] Adapter OpenAI (dịch MCP tool → function schema) + thử nghiệm với Codex.
- [ ] Admin Console cơ bản: quản lý member, xem installations, allowlist plugin.
- [ ] Billing/usage tracking (chưa cần tính phí thật, chỉ đo lường).
- [ ] Mời một nhóm khách hàng/đối tác thân thiết dùng thử (private beta).

## Giai đoạn 3 — Marketplace công khai (3-6 tháng)

- [ ] Developer Portal đầy đủ, quy trình review/publish tự động + thủ công.
- [ ] Trust badges (Verified/Signed/Community), quy trình KYC nhẹ cho publisher.
- [ ] Revenue share cho plugin trả phí.
- [ ] Observability đầy đủ (Prometheus/Grafana/Tracing), SLA cho gói Enterprise.
- [ ] Đa vùng/HA cho Gateway (nếu lượng truy cập yêu cầu).
- [ ] Public launch Marketplace.

## Giai đoạn 4 — Mở rộng hệ sinh thái

- [ ] Hỗ trợ thêm agent khác ngoài Claude/OpenAI (Cursor, Cline, Windsurf...) nếu chưa làm ở giai đoạn trước.
- [ ] SSO/SCIM cho khách hàng Enterprise.
- [ ] Compliance nâng cao (SOC2, audit log dài hạn xuất ra kho lưu trữ riêng của khách hàng).
- [ ] Tự động hoá phát hiện thay đổi API của Anthropic/OpenAI để cảnh báo sớm khi Adapter cần cập nhật.

## Rủi ro cần theo dõi xuyên suốt

- API/định dạng MCP, Claude Code plugin, OpenAI tools **thay đổi nhanh** → cần job giám sát thay đổi tài liệu chính thức, không hard-code giả định lâu dài.
- Review permission nhạy cảm cần đủ nhân sự khi marketplace mở công khai — nếu không sẽ thành nút thắt cổ chai hoặc bị bỏ qua vì áp lực tốc độ.
