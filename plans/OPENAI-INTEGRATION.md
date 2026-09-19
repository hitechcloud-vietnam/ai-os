# Tích hợp OpenAI

## Các điểm tích hợp

| Sản phẩm OpenAI | Cách HiTechCloud tích hợp |
|---|---|
| Responses API / Chat Completions | Dịch MCP tool → `tools` (function-calling JSON Schema) |
| Assistants-style agents | Nạp Skill dưới dạng file/instructions trong context |
| Codex CLI / Codex Cloud | Đăng ký MCP server qua config của Codex (nếu hỗ trợ MCP), hoặc bọc qua tool adapter riêng |

## Dịch MCP tool → OpenAI function schema

MCP tool định nghĩa theo JSON Schema tương tự OpenAI function, nên việc dịch chủ yếu là đổi field name:

```json
// MCP tool definition (rút gọn)
{
  "name": "github.create_issue",
  "description": "Tạo issue mới trên GitHub repo",
  "inputSchema": {
    "type": "object",
    "properties": {
      "repo": {"type": "string"},
      "title": {"type": "string"},
      "body": {"type": "string"}
    },
    "required": ["repo", "title"]
  }
}
```

```json
// OpenAI tools[] tương ứng
{
  "type": "function",
  "function": {
    "name": "github__create_issue",
    "description": "Tạo issue mới trên GitHub repo",
    "parameters": {
      "type": "object",
      "properties": {
        "repo": {"type": "string"},
        "title": {"type": "string"},
        "body": {"type": "string"}
      },
      "required": ["repo", "title"]
    }
  }
}
```

Gateway giữ bảng ánh xạ 2 chiều `mcp_tool_name <-> openai_function_name` (thay `.` bằng `__` vì OpenAI giới hạn ký tự tên function) để khi model gọi function, Gateway biết forward về đúng MCP server nào.

## Vòng lặp tool-calling

```
App gọi OpenAI API kèm tools[] (đã dịch từ MCP)
        │
        ▼
Model trả tool_call: { name: "github__create_issue", arguments: {...} }
        │
        ▼
App gửi tool_call này cho HiTechCloud Gateway (endpoint /v1/tool-execute)
        │
        ▼
Gateway forward tới MCP server "github", nhận kết quả
        │
        ▼
App gửi kết quả về lại OpenAI API dưới dạng tool result, tiếp tục hội thoại
```

## Skill cho OpenAI

Vì Responses API không có khái niệm "skill" runtime, Adapter nén nội dung Skill thành:
- **Developer/system message** (nếu ngắn, dưới vài nghìn token), hoặc
- **File đính kèm** qua Files API rồi trỏ trong context (nếu dài) — model tự đọc khi cần, tương tự cách Claude đọc SKILL.md.

## Codex CLI/Cloud

Nếu Codex hỗ trợ MCP trực tiếp, dùng lại nguyên cấu hình như phần Claude (endpoint HTTP/SSE của HiTechCloud Gateway). Nếu Codex dùng cơ chế tool riêng, cần một adapter mỏng dịch theo cấu hình tool của Codex tại thời điểm triển khai (kiểm tra tài liệu Codex hiện hành trước khi code cứng, vì định dạng có thể thay đổi).

## Lưu ý

- Giới hạn số lượng tool gửi kèm 1 request (OpenAI khuyến nghị không nên để quá nhiều tool cùng lúc vì ảnh hưởng độ chính xác chọn tool) — Gateway nên hỗ trợ **tool subset theo ngữ cảnh** (chỉ gửi tool liên quan tới plugin đang active) thay vì luôn gửi toàn bộ tool đã cài.
