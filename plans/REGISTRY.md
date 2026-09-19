# Đặc tả Kỹ thuật: HiTechCloud Package Registry

Registry là **Control Plane** trung tâm — nguồn sự thật duy nhất (single source of truth) quản lý toàn bộ vòng đời của mọi gói tài nguyên (**McpServer**, **Skill**, **Plugin**, **Agent**) trong hệ sinh thái HiTechCloud AI Platform.

---

## 1. Kiến trúc Tổng quan

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    HITECHCLOUD PACKAGE REGISTRY                         │
├─────────────────────────────────────────────────────────────────────────┤
│ 1. Package Ingestion API     : Upload .hcp bundle, validate JSON schema │
│ 2. Cryptographic Verifier    : Kiểm tra chữ ký Ed25519 & SHA256 hashes  │
│ 3. Security & CVE Scanner    : Quét mã độc, regex secret leak & CVEs    │
│ 4. Dependency Resolver Engine: Giải quyết version SemVer & lockfile     │
│ 5. Artifact Storage          : S3-compatible Object Storage (MinIO)     │
│ 6. Realtime Sync & Webhooks  : Đồng bộ tức thời xuống Gateway (SSE/MQ) │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Quy trình Phát hành Gói (Package Publishing Pipeline)

```
Developer CLI (`hitechcloud package publish`)
    │
    ├── 1. Tạo bundle (.hcp) kèm checksums.sha256 & chữ ký Ed25519
    │
    ▼
Registry Ingestion API (`POST /v1/registry/packages/publish`)
    │
    ├── 2. Xác thực Token / API Key của Publisher
    ├── 3. Kiểm tra tính hợp lệ của manifest `hcp.json` theo JSON Schema
    ├── 4. Xác minh chữ ký số Ed25519 với Public Key của Publisher
    ├── 5. Automated Security & Vulnerability Scan (Static analysis)
    │      └── Kiểm tra: Hardcoded API keys, dangerous shell exec, known CVEs
    │
    ├── 6. Lưu Artifact vào MinIO S3 & ghi metadata vào PostgreSQL
    │
    ▼
Real-time Gateway Notification
    │
    └── 7. Bắn Webhook / Redis PubSub thông báo Gateways cập nhật cache
```

---

## 3. Quản lý Phiên bản & Phụ thuộc (SemVer & Dependency Resolution)

- **Semantic Versioning 2.0.0**: Bắt buộc định dạng `MAJOR.MINOR.PATCH` (kèm prerelease nếu có: `-alpha.1`, `-beta.2`).
- **Dependency Specifiers**:
  - Exact: `1.2.3`
  - Caret: `^1.2.0` (tương thích mọi bản `1.x.x >= 1.2.0`)
  - Tilde: `~1.2.0` (tương thích mọi bản `1.2.x >= 1.2.0`)
- **Package Lockfile (`hcp.lock`)**: Khi cài đặt trong môi trường chạy, hệ thống sinh file lock chứa cây phụ thuộc đã resolve kèm SHA256 integrity hash nhằm đảm bảo tính tái lập (deterministic builds).

---

## 4. Bảo mật & Xác minh Tác giả (Publisher Verification)

1. **Chữ ký số Ed25519**: Mỗi gói phát hành bắt buộc phải ký bằng private key của tác giả. Registry công khai public key trên trang hồ sơ tác giả.
2. **Huy hiệu Xác thực (Verified Badge)**:
   - `Verified Organization`: Dành cho các tổ chức doanh nghiệp đã xác minh tên miền DNS / OIDC.
   - `Official`: Các gói cốt lõi do chính HiTechCloud phát triển và bảo trì.
   - `Community`: Các gói do cộng đồng đóng góp.

---

## 5. Thu hồi, Khai tử & Rollback (Deprecation, Yanking & Rollback)

- **Yank Package (`hitechcloud package yank <name>@<version>`)**:
  - Gói bị yank sẽ không thể được tải mới bởi các dự án mới.
  - Các dự án đang có sẵn `hcp.lock` trỏ đích danh tới version này vẫn tải được để tránh làm sập hệ thống (build break).
  - Registry gửi tín hiệu cảnh báo bảo mật tới Dashboard.
- **Deprecate Package**:
  - Đánh dấu gói không còn được bảo trì, kèm lời nhắn gợi ý gói thay thế.
- **Rollback tức thì**:
  - Quản trị viên Gateway có thể chuyển active version của một MCP Server/Skill về phiên bản trước đó chỉ với 1 click trên Dashboard trong vòng `<100ms`.

---

## 6. REST & GraphQL API của Registry

### 6.1. REST Endpoints
```http
POST   /v1/registry/packages/publish       # Upload và publish package bundle
GET    /v1/registry/packages/search        # Tìm kiếm full-text với bộ lọc category
GET    /v1/registry/packages/{id}          # Lấy chi tiết metadata và các versions
GET    /v1/registry/packages/{id}/download # Tải về file .hcp bundle
POST   /v1/registry/packages/{id}/yank     # Thu hồi một phiên bản có lỗ hổng
POST   /v1/registry/packages/{id}/deprecate# Đánh dấu khai tử một package
```

### 6.2. GraphQL Query Example
```graphql
query SearchComponents($query: String!, $kind: PackageKind) {
  searchPackages(query: $query, kind: $kind, limit: 10) {
    id
    name
    version
    displayName
    description
    publisher {
      name
      isVerified
    }
    security {
      vulnerabilitiesCount
      isSigned
    }
  }
}
```

