# Audit Logging

## Mục tiêu

Trả lời được: **ai, làm gì, với tool/dữ liệu nào, lúc nào, kết quả ra sao** — phục vụ compliance, điều tra sự cố, và cảnh báo bất thường theo thời gian thực.

## Schema audit_logs (tóm tắt)

```
audit_logs
 ├── id
 ├── org_id
 ├── actor_type      (user | service_account | plugin)
 ├── actor_id
 ├── action          (tool.call | plugin.install | permission.grant | ...)
 ├── resource_type    (mcp_tool | skill | plugin)
 ├── resource_id
 ├── request_meta      (JSON, đã mask secret)
 ├── result_status     (success | denied | error)
 ├── ip_address
 ├── created_at
```

## Nguyên tắc masking

- Mọi field trùng pattern secret phổ biến (`sk-`, `ghp_`, `AKIA`, chuỗi Base64 dài bất thường trong field tên `token`/`key`/`secret`) được thay bằng `***MASKED***` trước khi ghi log.
- Không log body request/response đầy đủ cho tool có gắn nhãn `sensitive: true` trong manifest — chỉ log tên tool + mã kết quả.

## Retention

| Gói | Thời gian giữ log |
|---|---|
| Free | 7 ngày |
| Team | 90 ngày |
| Enterprise | Tùy chọn tới nhiều năm, xuất được ra S3/kho lưu trữ riêng của khách hàng |

## Cảnh báo bất thường (tùy chọn nâng cao)

- Gọi tool `*.delete`/`*.write` với tần suất bất thường so với baseline của user đó.
- Nhiều lần `403 forbidden` liên tiếp từ cùng 1 API key (dấu hiệu dò quyền).
- Cài đặt plugin mới rồi ngay lập tức gọi permission nhạy cảm nhất mà nó có (dấu hiệu abuse).

## Truy vấn cho khách hàng

```
GET /orgs/{id}/audit-logs?actor_id=&action=&from=&to=&resource_type=
```

Kết quả trả về dạng phân trang, có thể export CSV/JSON để đưa vào SIEM riêng của doanh nghiệp.

Xem thêm giám sát hệ thống (khác với audit nghiệp vụ): [OBSERVABILITY.md](OBSERVABILITY.md).
