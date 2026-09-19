# Contributing

## Quy trình đóng góp code (nội bộ HiTechCloud hoặc mã nguồn mở nếu open-source hoá)

1. Fork/branch từ `main`, đặt tên branch theo `feature/<mô-tả>` hoặc `fix/<mô-tả>`.
2. Viết code + test tương ứng (xem [TESTING.md](TESTING.md)) — PR thiếu test cho logic mới sẽ bị yêu cầu bổ sung.
3. Chạy `cargo fmt` + `cargo clippy -- -D warnings` trước khi mở PR.
4. Mô tả PR rõ: vấn đề gì, giải pháp gì, có breaking change không (đặc biệt với API/schema publish ra ngoài).
5. Cần ít nhất 1 reviewer approve; PR đổi schema DB hoặc API public cần thêm review từ người phụ trách kiến trúc.

## Đóng góp Skill/Plugin/MCP server (cho publisher bên ngoài)

- Đọc kỹ [SKILL-SPEC.md](SKILL-SPEC.md) / [PLUGIN-SPEC.md](PLUGIN-SPEC.md) / [MCP-SPEC.md](MCP-SPEC.md) trước khi submit.
- Chạy `hitechcloud <type> validate` local trước khi publish để tránh bị reject vì lỗi format.
- Nếu package xin permission nhạy cảm, chuẩn bị sẵn mô tả rõ **tại sao cần** quyền đó — giúp review nhanh hơn.

## Báo lỗi bảo mật

Không mở issue công khai cho lỗ hổng bảo mật. Gửi riêng tới kênh bảo mật nội bộ (security@hitechcloud, hoặc kênh được công bố chính thức) kèm mô tả tái hiện — xử lý theo quy trình ở [SECURITY.md](SECURITY.md).

## Chuẩn commit message

```
feat(mcp-gateway): thêm hỗ trợ Streamable HTTP transport
fix(registry): sửa lỗi version resolve khi có nhiều pre-release
docs(skill-spec): làm rõ quy tắc đặt tên trigger description
```

## Code of Conduct

Tôn trọng, không công kích cá nhân trong review; tập trung góp ý vào code/thiết kế.
