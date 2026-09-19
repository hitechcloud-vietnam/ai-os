# Security

## Mô hình mối đe dọa (threat model) chính

1. **Plugin/MCP server độc hại** publish lên marketplace nhằm đánh cắp credential hoặc thực thi lệnh trái phép.
2. **Prompt injection** từ dữ liệu trả về bởi MCP server bên thứ ba (vd nội dung trang web, file, issue GitHub chứa lệnh ẩn) khiến agent thực hiện hành động ngoài ý muốn user.
3. **Leak credential** do log/audit ghi nhầm secret dạng plaintext.
4. **Privilege escalation** qua update plugin âm thầm xin thêm quyền.
5. **Supply-chain attack**: publisher hợp lệ bị chiếm tài khoản, đẩy version độc hại.

## Biện pháp tương ứng

| Rủi ro | Biện pháp |
|---|---|
| Plugin độc hại | Review thủ công cho permission nhạy cảm, quét tự động (static scan), sandbox khi test trước khi publish |
| Prompt injection | Coi output MCP server là dữ liệu, không phải lệnh; áp policy lọc trước khi trả agent (xem [SANDBOX.md](SANDBOX.md)) |
| Leak credential | Secret lưu ở Vault/KMS riêng, audit log mask theo pattern (token, key), không bao giờ trả secret qua tool output |
| Privilege escalation | Permission diff bắt buộc xác nhận thủ công khi update (xem [PLUGIN-GATEWAY.md](PLUGIN-GATEWAY.md)) |
| Supply-chain attack | Bắt buộc ký (signing) mọi version, 2FA cho tài khoản publisher, có thể yank version bị compromise ngay lập tức |

## Quét tự động trước khi publish

- Static scan mã nguồn/script trong plugin (nếu có) tìm pattern nguy hiểm: `eval`, network call ra domain lạ, đọc biến môi trường nhạy cảm hàng loạt.
- Diff permission so với version trước.
- Kiểm tra domain gọi ra trong MCP server có nằm trong allowlist đã khai báo không.

## Không log / không lưu

- Không log nội dung request/response đầy đủ nếu chứa secret — chỉ log metadata (tool name, thời gian, mã kết quả, user).
- Không cho phép Skill/Plugin đọc biến môi trường của tiến trình Gateway trực tiếp — chỉ nhận secret được "tiêm" có kiểm soát qua Vault theo đúng key đã khai báo.

## Xử lý sự cố (incident response)

1. Phát hiện (báo cáo từ user, quét tự động, hoặc bất thường trong audit log).
2. **Kill-switch**: Registry đánh dấu version là `revoked`, Gateway ngừng route trong vài giây (qua cơ chế invalidate ở [REGISTRY.md](REGISTRY.md)).
3. Thông báo cho mọi tổ chức đã cài version bị revoke.
4. Xoay vòng (rotate) mọi credential có khả năng bị lộ liên quan.
5. Viết post-mortem công khai mức độ phù hợp.

Xem thêm: [SANDBOX.md](SANDBOX.md), [SIGNING-VERIFICATION.md](SIGNING-VERIFICATION.md), [AUDIT-LOGGING.md](AUDIT-LOGGING.md).
