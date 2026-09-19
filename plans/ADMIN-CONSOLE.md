# Admin Console

## Đối tượng dùng

Admin/owner của một tổ chức (org) trên HiTechCloud — quản lý ai được cài gì, xem chi tiêu, xem audit log.

## Các màn hình chính

```
/admin
├── /overview            # usage vs quota, cảnh báo, hoạt động gần đây
├── /members              # mời/xoá thành viên, gán role
├── /roles                # tạo role tuỳ chỉnh, gán permission
├── /installations         # danh sách mcp/skill/plugin đã cài, theo scope
├── /marketplace-policy     # allowlist/denylist entry nào member được cài
├── /audit-logs             # tra cứu, export
├── /billing                 # gói hiện tại, hoá đơn, phương thức thanh toán
└── /security                # API key, IP allowlist, SSO config
```

## Luồng duyệt plugin nội bộ (org tự publish plugin private)

```
Developer trong org submit plugin (scope: org-private)
        │
        ▼
Admin Console hiển thị permission yêu cầu + diff so với version cũ (nếu có)
        │
        ▼
Admin bấm "Approve" → plugin available cho member cài
        │ (hoặc "Reject" kèm lý do)
        ▼
Ghi vào audit_logs: admin.plugin.approve / admin.plugin.reject
```

## Policy allowlist/denylist mẫu

```json
{
  "mode": "allowlist",
  "allowed_entities": ["hitechcloud-server", "github", "aws-basic"],
  "require_approval_for_permissions": ["*.delete", "billing.*", "admin.*"]
}
```

## Yêu cầu UX quan trọng

- Mọi thay đổi permission/policy phải hỏi xác nhận rõ ràng (không có hành động "âm thầm mở rộng quyền").
- Trang `/audit-logs` cho phép filter theo actor, action, resource, khoảng thời gian và export CSV/JSON ngay trong UI.
- Cảnh báo trực quan (badge đỏ) cho plugin đang ở trạng thái `pending_review` hoặc phiên bản đã bị `deprecated`/`revoked` mà org vẫn đang dùng.
