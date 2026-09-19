# Authentication & RBAC

## Xác thực

- **API Key** dạng `hitechcloud-sk_live_xxxxx` / `hitechcloud-sk_test_xxxxx`, gắn với 1 user hoặc 1 service account trong org.
- **OAuth2 / OIDC** cho luồng đăng nhập giao diện Marketplace/Admin Console (hỗ trợ SSO cho khách hàng doanh nghiệp).
- **mTLS tùy chọn** cho MCP server nội bộ nhạy cảm (vd kết nối tới hạ tầng production).
- Mọi API key có thể giới hạn theo: scope (org/project/user), thời hạn, IP allowlist, và **danh sách tool được phép gọi**.

## Mô hình RBAC

```
User ──belongs_to──▶ Organization
User ──has many──▶ Role (per org)
Role ──has many──▶ Permission
Permission ──applies to──▶ Resource (mcp_server | skill | plugin | tool)
```

## Permission mẫu

```
github.read
github.write
server.read
server.service.restart
server.delete
dns.read
dns.write
billing.read
billing.write
admin.marketplace.approve
```

Quy ước đặt tên: `resource.action`, action con dùng `.` để phân cấp (`server.service.restart` hẹp hơn `server.write`).

## Role mặc định

| Role | Mô tả |
|---|---|
| `owner` | Toàn quyền, kể cả billing và xóa org |
| `admin` | Quản lý plugin/MCP/skill được org duyệt, quản lý member |
| `developer` | Cài & dùng plugin đã duyệt, không được approve plugin mới |
| `viewer` | Chỉ xem, không cài/gọi tool ghi dữ liệu |

## Kiểm tra quyền tại Gateway

```
Request → xác thực API key → resolve user + role
        → tra permission cần cho tool đích (vd server.service.restart)
        → role có permission đó? → cho qua : 403 Forbidden (ghi audit log)
```

## Nguyên tắc

- **Least privilege theo mặc định**: plugin mới cài chỉ có permission nó khai báo và org đã duyệt, không tự "thừa kế" quyền rộng hơn.
- **Permission tường minh, không suy diễn**: không suy ra "có quyền `server.write` thì chắc cũng có `server.delete`" — mỗi permission phải liệt kê rõ.
- **Session ngắn hạn cho tác vụ nhạy cảm**: các action như `server.delete`, `billing.write` có thể yêu cầu xác thực lại (step-up auth) dù đã đăng nhập.
