# Đặc tả Kỹ thuật: Tùy chọn Kết nối API Ngoại vi Nube.sh & Dynamic Pricing

Tài liệu này đặc tả cơ chế tích hợp **Nube.sh** (`https://nube.sh` / `https://ai.nube-api.com`) dưới dạng **Tùy chọn kết nối API mô hình bên ngoài (External LLM Provider Option)** vào phân hệ Multi-Model Router của **HiTechCloud Agent Platform**.

> **Lưu ý Kiến trúc Quan trọng:**
> - **Nube.sh là một Option (Tùy chọn kết nối API bên thứ ba)**, hoàn toàn KHÔNG PHẢI là hệ thống core của nền tảng.
> - Hệ thống Core của **HiTechCloud Agent Platform** được xây dựng độc lập 100% bằng Rust Tokio Core (Gateway, Registry, Sandbox, A2A, AG-UI, RBAC).
> - Nube.sh đóng vai trò là một trong các đích suy luận (Inference Endpoints) mà người dùng có thể lựa chọn cấu hình API Key để kết nối khi cần tận dụng mô hình chi phí thấp hoặc context window lớn.

---

## 1. Tổng quan Tùy chọn Nube.sh API Option

**Nube.sh** là dịch vụ API suy luận (Inference API Provider) bên ngoài, hỗ trợ các frontier models (DeepSeek-V4.1-Flash, Kimi-K2.6, GLM-5.3, Qwen3.8-27B) với quy mô hàng ngàn tỷ tham số (MoE) được huấn luyện Quantization-Aware Training (QAT, MXFP4, NVFP4).

### Ưu điểm Khi Lựa chọn Option Nube.sh
1. **Giao thức Kép (Dual Protocol Support)**: 1 API Key duy nhất sử dụng được cho cả 2 chuẩn **OpenAI-Compatible** và **Anthropic-Compatible**.
2. **Dynamic Pricing API công khai**: Endpoint công khai không cần API key cập nhật giá tức thời ($/triệu tokens), cho phép Gateway tự động tính toán chi phí và chọn mô hình tiết kiệm nhất.
3. **Context Window Siêu lớn**: Lên đến **1,000,000 tokens** với chi phí Cache Hit cực thấp ($0.0027 - $0.035 / triệu tokens).

---

## 2. Cấu hình Endpoint & Giao thức Kết nối

| Giao thức | Base URL | Mô hình Tương thích | Client Sử dụng |
|---|---|---|---|
| **OpenAI Compatible** | `https://ai.nube-api.com/v1` | `Nube-Choice`, `DeepSeek-V4.1-Flash`, `Qwen3.8-27B`, `GLM-5.3`, `GLM-5.3-Flash`, `Kimi-K2.6` | OpenAI SDK, Codex, Aider, OpenCode, Goose |
| **Anthropic Compatible**| `https://ai.nube-api.com` | `Nube-Choice`, `DeepSeek-V4.1-Flash`, `GLM-5.3`, `Kimi-K2.6` | Anthropic SDK, Claude Code, Roo Code, Cursor |

> **Lưu ý quan trọng khi cấu hình Base URL:**
> - Chuẩn OpenAI: Nhập chính xác `https://ai.nube-api.com/v1` (không thêm `/chat/completions` vào Base URL).
> - Chuẩn Anthropic: Nhập chính xác `https://ai.nube-api.com` (không thêm `/v1` ở cuối).
> - Header xác thực: `Authorization: Bearer <NUBE_API_KEY>`

---

## 3. Dynamic Pricing API & Tự động Cập nhật Bảng giá

Nube.sh cung cấp public endpoint trả về bảng giá thời gian thực của mọi model:

- **Endpoint**: `GET https://ai.nube-api.com/v1/models/pricing`
- **Xác thực**: Không yêu cầu (Public Endpoint)
- **Đơn vị tính**: `USD / 1,000,000 tokens`

### 3.1. Cấu trúc Dữ liệu JSON Response
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "models": [
      {
        "model_name": "Nube-Choice",
        "input_cost_per_million_tokens": 0.135,
        "output_cost_per_million_tokens": 0.54,
        "cache_read_cost_per_million_tokens": 0.0027,
        "currency": "USD",
        "display_order": 10,
        "public_tags": ["text", "image", "reasoning", "tools"],
        "public_context_window": 1000000,
        "is_recommended": true,
        "description": "Nube's preferred model (currently DeepSeek-V4.1-Flash).",
        "notice": "No price emails. See API or pricing page."
      },
      {
        "model_name": "DeepSeek-V4.1-Flash",
        "input_cost_per_million_tokens": 0.135,
        "output_cost_per_million_tokens": 0.54,
        "cache_read_cost_per_million_tokens": 0.0027,
        "currency": "USD",
        "display_order": 15,
        "public_tags": ["text", "image", "reasoning", "tools"],
        "public_context_window": 1000000
      },
      {
        "model_name": "Qwen3.8-27B",
        "input_cost_per_million_tokens": 0.175,
        "output_cost_per_million_tokens": 1.05,
        "cache_read_cost_per_million_tokens": 0.035,
        "currency": "USD",
        "display_order": 30,
        "public_tags": ["text", "image", "video", "reasoning", "tools"],
        "public_context_window": 256000
      },
      {
        "model_name": "GLM-5.3",
        "input_cost_per_million_tokens": 0.63,
        "output_cost_per_million_tokens": 1.98,
        "cache_read_cost_per_million_tokens": 0.117,
        "currency": "USD",
        "display_order": 35,
        "public_tags": ["text", "reasoning", "tools"],
        "public_context_window": 1000000
      },
      {
        "model_name": "GLM-5.3-Flash",
        "input_cost_per_million_tokens": 0.0825,
        "output_cost_per_million_tokens": 0.275,
        "cache_read_cost_per_million_tokens": 0.0165,
        "currency": "USD",
        "display_order": 40,
        "public_tags": ["text", "image", "video", "reasoning", "tools"],
        "public_context_window": 1000000
      },
      {
        "model_name": "Kimi-K2.6",
        "input_cost_per_million_tokens": 0.4275,
        "output_cost_per_million_tokens": 1.80,
        "cache_read_cost_per_million_tokens": 0.072,
        "currency": "USD",
        "display_order": 60,
        "public_tags": ["text", "image", "video", "reasoning", "tools"],
        "public_context_window": 256000
      }
    ]
  }
}
```

### 3.2. Background Price Fetcher Job trong Rust Gateway
Rust Gateway khởi chạy một background worker định kỳ mỗi 1 giờ hoặc khi khởi động:
```rust
pub async fn sync_nube_pricing(redis: &redis::Client) -> Result<(), AppError> {
    let url = "https://ai.nube-api.com/v1/models/pricing";
    let resp = reqwest::get(url).await?.json::<NubePricingResponse>().await?;
    
    for model in resp.data.models {
        let key = format!("pricing:nube:{}", model.model_name);
        let val = serde_json::to_string(&model)?;
        redis.set_ex(key, val, 7200).await?; // Cache 2 giờ
    }
    tracing::info!("Synced {} Nube.sh model pricings to Redis cache", resp.data.models.len());
    Ok(())
}
```

---

## 4. Tự động Định tuyến Theo Chi phí & Năng lực (Cost-Aware Routing)

Multi-Model Router của HiTechCloud hỗ trợ chiến lược định tuyến thông minh:
1. **Cost-Optimized Mode**: Tự động chọn mô hình có chi phí thấp nhất hỗ trợ tag `tools` và `reasoning` (Ví dụ: `GLM-5.3-Flash` chỉ $0.0825/M input tokens cho các tác vụ phân loại hoặc kiểm tra trạng thái nền).
2. **Quality & Reasoning Mode**: Điều hướng các tác vụ suy luận sâu và viết code phức tạp tới `Nube-Choice` (DeepSeek-V4.1-Flash) hoặc `Kimi-K2.6` với context window 1,000,000 tokens.
3. **Prompt Cache Optimization**: Tận dụng cơ chế Cache Read của Nube.sh ($0.0027/M tokens) để giảm 98% chi phí cho các hội thoại dài hoặc tài liệu ngữ cảnh lớn.

---

## 5. Tích hợp trong CLI & Dashboard

### 5.1. Thêm Nube.sh qua Rust CLI
```bash
# Thêm Nube Provider vào cấu hình
hitechcloud provider add nube \
  --api-key "nb_live_xxxxxxxxxxxxxxxx" \
  --protocol openai \
  --default-model "Nube-Choice"

# Xuất cấu hình cho Claude Code sử dụng Nube Anthropic Base URL
hitechcloud client export claude-code \
  --provider nube \
  --base-url "https://ai.nube-api.com" \
  --model "Nube-Choice"
```

### 5.2. Quản lý trên MCPHub Dashboard
- **Model Router View**: Hiển thị bảng giá trực quan cập nhật theo thời gian thực từ `ai.nube-api.com/v1/models/pricing`.
- **API Key Vault**: Quản lý `NUBE_API_KEY` an toàn trong Secrets Store, tự động tiêm vào các luồng gọi MCP tools hoặc Agent loops.
