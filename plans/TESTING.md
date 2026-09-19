# Testing

## Các lớp test

| Lớp | Phạm vi | Công cụ |
|---|---|---|
| Unit test | Từng function/module trong mỗi crate | `cargo test` |
| Integration test | Gateway ↔ Postgres/Redis thật (docker-compose tạm trong CI) | `cargo test --features integration` |
| Contract test | Adapter Anthropic/OpenAI — đảm bảo output đúng schema mong đợi | snapshot test (`insta` crate) |
| End-to-end test | Giả lập agent thật gọi qua Gateway → MCP server thật (staging) | script Python/TS gọi API |
| Security test | Quét plugin/MCP server submit (static scan, permission diff) | pipeline riêng, xem [SECURITY.md](SECURITY.md) |
| Load test | Đo throughput/latency Gateway dưới tải cao | `k6` hoặc `wrk` |

## Test bắt buộc trước khi publish 1 Plugin (do publisher chạy qua CLI)

```bash
hitechcloud plugin validate ./my-plugin        # kiểm tra schema plugin.json
hitechcloud plugin test ./my-plugin --sandbox   # chạy trong sandbox, giả lập agent gọi từng tool/skill
```

`plugin test` nên kiểm tra:
- Mọi tool khai báo trong `mcp.json` trả lời được `tools/list` hợp lệ.
- Mọi permission dùng trong runtime nằm trong danh sách đã khai báo ở `plugin.json` (không "âm thầm" cần quyền chưa khai báo).
- Mọi Skill có frontmatter hợp lệ theo [SKILL-SPEC.md](SKILL-SPEC.md).

## CI pipeline gợi ý (cho chính codebase HiTechCloud)

```
lint (clippy, eslint) → unit test → integration test (docker-compose) 
   → build image → security scan (trivy) → deploy staging 
   → e2e test staging → (approve thủ công) → deploy production canary
```

## Test dữ liệu nhạy cảm

- Không dùng credential thật trong test — luôn dùng mock/fixture.
- Test riêng cho cơ chế masking log (đảm bảo secret không bao giờ lọt vào `audit_logs` dạng plaintext dù có lỗi code).

## Regression test cho Adapter

Khi Anthropic/OpenAI thay đổi định dạng API (khá thường xuyên), cần bộ **contract test** riêng chạy định kỳ (không chỉ khi có PR) để phát hiện sớm việc Adapter bị lệch chuẩn do nhà cung cấp đổi API.
