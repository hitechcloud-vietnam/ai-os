# Signing & Verification

## Mục tiêu

Đảm bảo mọi MCP server/Skill/Plugin agent tải về đúng là bản mà publisher đã submit, chưa bị chỉnh sửa giữa đường (supply-chain integrity).

## Cơ chế

1. Mỗi publisher có 1 cặp khoá (Ed25519), public key đăng ký với Registry khi tạo tài khoản publisher.
2. Khi publish version mới, CI/CLI của publisher (`hitechcloud publish`) tính hash (SHA-256) của gói (tar.gz chứa toàn bộ file plugin/skill), ký hash đó bằng private key.
3. Registry lưu `(package_hash, signature, publisher_public_key_id)` cùng metadata version.
4. Khi Gateway/agent tải gói về, verify:
   - Hash file tải về khớp `package_hash` đã lưu.
   - `signature` verify hợp lệ với `publisher_public_key_id`.
   - `publisher_public_key_id` chưa bị thu hồi (trường hợp lộ khoá).

```
hitechcloud publish
      │
      ▼
Tính SHA-256(package) → sign(hash, private_key) → gửi kèm package lên Registry
      │
      ▼
Registry verify chữ ký bằng public key đã đăng ký trước → nếu hợp lệ → lưu + đánh dấu "signed"
```

## Trust levels hiển thị trên Marketplace

| Badge | Điều kiện |
|---|---|
| **Verified (HiTechCloud)** | Publisher chính thức của HiTechCloud, đã audit code |
| **Signed** | Có chữ ký hợp lệ, publisher đã xác minh danh tính (KYC nhẹ) |
| **Community (unsigned)** | Chưa ký hoặc publisher chưa xác minh — cảnh báo rõ trước khi cài |

## Thu hồi khoá (key revocation)

Nếu publisher báo bị lộ private key:
1. Đánh dấu `public_key_id` là `revoked` trong Registry.
2. Mọi version ký bằng khoá đó chuyển trạng thái `signature_invalid`, Gateway từ chối route mặc định (có thể admin override thủ công nếu tự tin về nguồn gốc).
3. Yêu cầu publisher đăng ký khoá mới, ký lại các version còn muốn giữ published.

## Lưu ý triển khai

- Không tự viết crypto từ đầu — dùng thư viện đã kiểm chứng (vd `ed25519-dalek` nếu Registry viết bằng Rust).
- Toàn bộ quy trình publish/verify nên có test tích hợp trong CI, tránh trường hợp "quên ký" lọt qua do lỗi thao tác thủ công.
