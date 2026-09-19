# Ví dụ: Plugin tối giản "weather-lookup"

Ví dụ này minh hoạ 1 plugin đơn giản, không nhạy cảm, để làm quen với cấu trúc trước khi xem ví dụ phức tạp hơn ([example-hitechcloud-plugin.md](example-hitechcloud-plugin.md)).

## Cấu trúc

```
weather-lookup/
├── plugin.json
├── mcp.json
└── skills/
    └── weather-explainer/SKILL.md
```

## `plugin.json`

```json
{
  "name": "weather-lookup",
  "version": "1.0.0",
  "description": "Tra cứu thời tiết và giải thích chỉ số dễ hiểu cho người dùng phổ thông.",
  "author": "community-dev",
  "license": "MIT",
  "skills": ["weather-explainer"],
  "mcp": ["weather-api"],
  "commands": [],
  "permissions": ["weather.read"],
  "compatible_agents": ["claude-code", "claude-desktop"]
}
```

## `mcp.json`

```json
{
  "mcpServers": {
    "weather-api": {
      "url": "https://mcp.weatherexample.com/v1",
      "headers": { "Authorization": "Bearer ${WEATHER_API_KEY}" }
    }
  }
}
```

## `skills/weather-explainer/SKILL.md`

```markdown
---
name: weather-explainer
description: Giải thích chỉ số thời tiết (UV, chất lượng không khí) dễ hiểu. Dùng khi user hỏi "thời tiết hôm nay", "chỉ số UV", "có nên mang ô không".
scope: public
requires_mcp: [weather-api]
---

Khi trả lời, luôn kèm gợi ý hành động cụ thể (mang ô, tránh ra ngoài giờ nắng gắt...)
thay vì chỉ liệt kê số liệu thô.
```

Vì plugin này chỉ xin permission `weather.read` (không nhạy cảm), nó có thể được **auto-approve** khi submit, không cần review thủ công — minh hoạ nhánh nhanh trong vòng đời ở [PLUGIN-SPEC.md](PLUGIN-SPEC.md).
