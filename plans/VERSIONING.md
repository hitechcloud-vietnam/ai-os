# Versioning

## Semantic Versioning cho mọi entity

`MAJOR.MINOR.PATCH`

- **MAJOR**: thay đổi phá vỡ tương thích (đổi tên tool, xóa permission cũ theo cách không tương thích, đổi schema input bắt buộc).
- **MINOR**: thêm tính năng/tool mới, không phá vỡ cái cũ.
- **PATCH**: sửa lỗi, không đổi hành vi công khai.

## Version range khi khai báo dependency (trong Plugin)

```
"mcp": [
  { "id": "github", "version": "^2.0.0" },
  { "id": "hitechcloud-api", "version": "~1.4.0" }
]
```

- `^2.0.0`: chấp nhận `>=2.0.0 <3.0.0`
- `~1.4.0`: chấp nhận `>=1.4.0 <1.5.0`
- Ghim cứng (`=1.4.2`) khi publisher biết có rủi ro tương thích cao.

## Chính sách yank (thu hồi)

- Version bị lỗi bảo mật nghiêm trọng → `yanked` ngay, chặn cài mới, nhưng vẫn giữ metadata để ai đã cài biết mà gỡ.
- Không xoá vĩnh viễn version đã publish (trừ yêu cầu pháp lý) — đảm bảo build cũ vẫn resolve được nếu ghim version cụ thể.

## Deprecation

```
published → deprecated (vẫn dùng được, cảnh báo) → removed (sau thời hạn thông báo, tối thiểu 90 ngày với entity public)
```

## Changelog bắt buộc

Mỗi version publish cần kèm `CHANGELOG.md` mô tả thay đổi, đặc biệt bắt buộc nếu có thay đổi permission — hiển thị trực tiếp cho user khi họ được hỏi "có muốn cập nhật không".

## Version cho chính API nền tảng

API HiTechCloud version hoá theo path (`/v1`, `/v2`), duy trì tối thiểu 1 version cũ song song ít nhất 12 tháng sau khi ra version mới, có thông báo trước cho khách hàng Enterprise.
