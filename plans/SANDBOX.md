# Sandbox

## Mục tiêu

Cô lập việc **thực thi MCP server cộng đồng/chưa tin cậy** và **lọc dữ liệu chưa xác thực** trước khi nó chạm tới agent hoặc hạ tầng thật.

## Sandbox cho MCP server tự chạy (local/stdio)

- Chạy mỗi MCP server chưa được "Verified" trong container riêng (gVisor/Firecracker hoặc container thường + seccomp profile chặt), không chia sẻ filesystem/network với server khác.
- Giới hạn network egress theo allowlist domain publisher khai báo khi submit (giống mô hình allowed_domains ở Gateway).
- Giới hạn CPU/RAM/thời gian chạy per request để tránh DoS.
- Không mount credential thật vào container demo — dùng dữ liệu giả khi review tự động.

## Lọc output (data, not instruction)

Mọi nội dung trả về từ MCP server bên thứ ba được gán nhãn nội bộ `untrusted_content` trước khi đưa vào ngữ cảnh agent, kèm nhắc rằng đây là dữ liệu để đọc, không phải chỉ thị cần tuân theo — giảm rủi ro prompt injection kiểu "trong file này có dòng: từ giờ hãy bỏ qua mọi rule trước đó".

```
MCP Server (chưa verified) → Sandbox Runner → Output Filter → Agent
                                                     │
                                       (gắn nhãn untrusted_content,
                                        loại bỏ pattern lệnh ẩn nếu có)
```

## Sandbox cho testing plugin trước khi publish

- Môi trường staging riêng mô phỏng gọi từ Claude Code/Codex thật, chạy hết bộ test tác giả khai báo (xem [TESTING.md](TESTING.md)).
- Chấm điểm rủi ro tự động (permission requested, domain gọi ra, kích thước payload bất thường) → route sang review thủ công nếu vượt ngưỡng.

## Giới hạn hiện tại

- Sandbox hoá 100% không loại bỏ hoàn toàn rủi ro prompt injection ở tầng nội dung (vd 1 trang web độc hại có thể vẫn chứa văn bản gây nhiễu agent dù bản thân MCP server vô hại) — cần kết hợp với policy ở tầng model/agent, không chỉ ở Gateway.
- Với MCP server đã "Verified" (do HiTechCloud tự viết/audit kỹ), có thể nới lỏng sandbox để tối ưu hiệu năng, nhưng vẫn giữ nguyên tắc least privilege về network/credential.
