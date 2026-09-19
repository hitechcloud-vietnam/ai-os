# Tích hợp Codex (CLI/Cloud)

> Lưu ý: API/CLI của Codex thay đổi khá nhanh — coi tài liệu này là khung thiết kế, cần đối chiếu lại tài liệu chính thức của OpenAI tại thời điểm code thật.

## Nguyên tắc

Codex CLI/Cloud về bản chất vẫn chạy trên model OpenAI với cơ chế tool-calling, nên phần lớn logic dùng lại từ [OPENAI-INTEGRATION.md](OPENAI-INTEGRATION.md). Điểm khác biệt nằm ở **cách khai báo cấu hình môi trường/tool** (thường qua file cấu hình dạng TOML/JSON trong thư mục project, tương tự `.mcp.json` của Claude Code).

## Cấu hình mẫu (khái niệm)

```toml
# codex.toml (ví dụ khái niệm, cần đối chiếu format thật của Codex)
[mcp_servers.hitechcloud]
url = "https://gw.hitechcloud.vn/mcp/v1"
auth_header = "Bearer ${hitechcloud-sk_API_KEY}"
```

Nếu Codex tại thời điểm triển khai **chưa hỗ trợ MCP trực tiếp**, dùng đường vòng:

```
Codex CLI ──gọi shell/tool nội bộ──▶ hitechcloud (CLI của HiTechCloud)
                                            │
                                            ▼
                                     HiTechCloud Gateway
                                            │
                                            ▼
                                       MCP Servers
```

Tức là expose một **CLI mỏng** (`hitechcloud tool call <name> --args '{...}'`) mà Codex có thể gọi như một shell command bình thường — đây là cách "chuyển vị" MCP tool sang môi trường không có MCP native, không lý tưởng bằng nhưng hoạt động được ngay.

## Skills cho Codex

Tương tự OpenAI Integration: nén Skill thành system/developer instructions hoặc file context. Nếu Codex hỗ trợ "custom instructions" ở cấp project (giống `AGENTS.md`/`CODEX.md`), ưu tiên nhúng skill liên quan trực tiếp vào file đó khi cài plugin ở scope `project`.

## Plugin cho Codex

Vì Codex chưa có khái niệm "plugin marketplace" chính thức (tính đến thời điểm viết tài liệu này), HiTechCloud đóng vai trò:
1. Sinh file cấu hình (`codex.toml`/`AGENTS.md`) tương ứng khi user chọn "Install for Codex" trên Marketplace.
2. Cung cấp `hitechcloud` như lớp thực thi tool nếu MCP chưa được hỗ trợ native.

## Việc cần làm khi triển khai thật

- [ ] Xác nhận Codex CLI/Cloud hiện tại đã hỗ trợ MCP native hay chưa (kiểm tra tài liệu OpenAI mới nhất).
- [ ] Nếu có, bỏ qua lớp `hitechcloud` và dùng thẳng MCP endpoint như Claude.
- [ ] Nếu chưa, hoàn thiện `hitechcloud` làm cầu nối tạm thời.
