# Đặc tả Kỹ thuật: Billing, Usage & Dynamic Token Cost Accounting

Hệ thống thanh toán và đo lường tài nguyên của **HiTechCloud Agent Platform** quản lý việc hạch toán chi phí đa chiều: từ **lượt gọi MCP Tools**, **băng thông Gateway**, đến **chi phí suy luận mô hình AI (AI Tokens)** tích hợp tự động với **Dynamic Pricing API của Nube.sh**.

---

## 1. Đơn vị Tính phí & Hạch toán Tài nguyên

| Hạng mục | Đơn vị Tính | Nguồn Giá & Đo đạc |
|---|---|---|
| **AI Inference Tokens (Input/Output)** | USD / Triệu Tokens | Đồng bộ từ `GET https://ai.nube-api.com/v1/models/pricing` |
| **AI Prompt Cache Read** | USD / Triệu Tokens | Đồng bộ giá Cache Hit từ Nube.sh ($0.0027 - $0.035 / M tokens) |
| **Tool Execution qua MCP Gateway** | Lượt gọi (Calls) | Ghi nhận tại Gateway Router |
| **Hạ tầng Cloud Sandbox / WASM** | CPU-second / RAM-MB | Đo đạc từ Sandbox Controller |
| **Seat / Thành viên Tổ chức (Org)** | Tháng / Năm | Phân cấp theo gói Free / Team / Enterprise |

---

## 2. Đồng bộ Dynamic Pricing Thời gian thực (Nube.sh Pricing Sync)

Vì các nhà cung cấp như Nube.sh không gửi email thông báo thay đổi giá mà cập nhật trực tiếp qua API, hệ thống triển khai worker tự động:

- **Endpoint**: `GET https://ai.nube-api.com/v1/models/pricing`
- **Tần suất đồng bộ**: Mỗi 60 phút hoặc kích hoạt thủ công từ Dashboard.
- **Dữ liệu nạp vào Redis Cache**:
  ```json
  {
    "Nube-Choice": { "input": 0.135, "output": 0.54, "cache_read": 0.0027 },
    "DeepSeek-V4.1-Flash": { "input": 0.135, "output": 0.54, "cache_read": 0.0027 },
    "Qwen3.8-27B": { "input": 0.175, "output": 1.05, "cache_read": 0.035 },
    "GLM-5.3": { "input": 0.63, "output": 1.98, "cache_read": 0.117 },
    "GLM-5.3-Flash": { "input": 0.0825, "output": 0.275, "cache_read": 0.0165 },
    "Kimi-K2.6": { "input": 0.4275, "output": 1.80, "cache_read": 0.072 }
  }
  ```

---

## 3. Cấu trúc Bảng `usage_records` Mở rộng

```sql
CREATE TABLE usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id),
    user_id UUID REFERENCES users(id),
    session_id UUID,
    provider VARCHAR(64) NOT NULL,          -- 'nube', 'anthropic', 'openai', 'hitechcloud'
    model_name VARCHAR(128),                -- 'Nube-Choice', 'claude-3-7-sonnet'
    resource_type VARCHAR(64) NOT NULL,     -- 'ai_tokens', 'mcp_call', 'bandwidth'
    input_tokens INT DEFAULT 0,
    output_tokens INT DEFAULT 0,
    cached_tokens INT DEFAULT 0,
    effective_unit_price NUMERIC(12, 6),    -- Giá tại thời điểm thực thi ($/M tokens)
    total_cost_usd NUMERIC(12, 6) NOT NULL, -- Chi phí tính toán tức thời
    occurred_at TIMESTAMPTZ DEFAULT NOW(),
    billing_period VARCHAR(7) NOT NULL      -- '2026-09'
);

CREATE INDEX idx_usage_org_period ON usage_records(org_id, billing_period);
```

---

## 4. Kiểm soát Ngân sách & Hạn mức (Budget Quotas & Hard Limits)

- **Soft Limit**: Khi mức chi phí đạt 80% hạn mức tháng, gửi cảnh báo qua Email/Webhook/Telegram.
- **Hard Limit**: Khi đạt 100% ngân sách được duyệt, Gateway tự động chuyển sang chế độ an toàn:
  - Tạm dừng các mô hình đắt tiền, chuyển fallback sang mô hình chi phí thấp (`GLM-5.3-Flash` hoặc DeepSeek nhỏ).
  - Yêu cầu Admin phê duyệt để tiếp tục mở rộng quota.

