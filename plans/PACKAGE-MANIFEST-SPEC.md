# Đặc tả Kỹ thuật: Universal Package Manifest (`hcp.json` / `hcp.yaml`)

Tài liệu này đặc tả quy chuẩn manifest thống nhất **HCP-Spec** (HiTechCloud Package Specification) cho cả 4 loại tài nguyên: **Skill**, **Plugin**, **Agent**, và **McpServer**.

---

## 1. Mục đích Thiết kế

Thay vì mỗi loại tài nguyên có một định dạng file cấu hình rời rạc, **HCP-Spec** tạo ra một cấu trúc schema thống nhất, dễ đọc, tương thích tuyệt đối với JSON Schema validation, đồng thời cho phép build, kiểm tra chữ ký số và phát hành lên **HiTechCloud Package Registry**.

---

## 2. Cấu trúc Schema Chung của `hcp.json`

```json
{
  "$schema": "https://specs.hitechcloud.vn/schemas/hcp-v1.json",
  "hcpVersion": "1.0.0",
  "kind": "Skill | Plugin | Agent | McpServer",
  "metadata": {
    "name": "string (lowercase, kebab-case)",
    "namespace": "string (e.g. hitechcloud, community, acme)",
    "version": "string (SemVer, e.g. 1.2.0)",
    "displayName": "string",
    "description": "string (1-2 sentences)",
    "icon": "string (relative path or svg/png url)",
    "category": "database | devops | security | ai | analytics",
    "tags": ["string"],
    "license": "Apache-2.0 | MIT | Proprietary",
    "authors": [
      { "name": "HiTechCloud Team", "email": "dev@hitechcloud.vn" }
    ],
    "repository": "https://github.com/hitechcloud/..."
  },
  "runtime": {
    "type": "binary | node | python | docker | wasm | remote_http",
    "entrypoint": "bin/service_binary or index.js",
    "minPlatformVersion": ">=1.0.0"
  },
  "security": {
    "permissions": ["network:outbound", "storage:read", "env:read"],
    "signature": {
      "algorithm": "ed25519",
      "publicKey": "base64_encoded_key",
      "digest": "sha256:..."
    }
  },
  "spec": {
    /* Khối đặc tả chi tiết tùy theo 'kind' */
  }
}
```

---

## 3. Đặc tả Chi tiết theo Từng Loại `kind`

### 3.1. Kind: `McpServer`
```json
{
  "kind": "McpServer",
  "spec": {
    "transport": "stdio | sse | streamable_http",
    "tools": [
      {
        "name": "query_logs",
        "description": "Query server logs with regex filtering",
        "parameters": {
          "type": "object",
          "properties": {
            "query": { "type": "string" },
            "limit": { "type": "integer", "default": 50 }
          },
          "required": ["query"]
        }
      }
    ],
    "prompts": [
      {
        "name": "analyze_error",
        "description": "Pre-configured prompt to diagnose an error code"
      }
    ],
    "resources": [
      {
        "uri": "postgres://schema/public",
        "name": "Public DB Schema",
        "mimeType": "application/json"
      }
    ]
  }
}
```

### 3.2. Kind: `Skill`
```json
{
  "kind": "Skill",
  "spec": {
    "instructions": "SYSTEM INSTRUCTION PROMPT CONTENT...",
    "tools": [
      { "ref": "pkg:mcp/hitechcloud/postgres-tool@^1.0.0#query" },
      { "ref": "pkg:mcp/hitechcloud/redis-tool@^1.0.0#get" }
    ],
    "fallbackStrategy": "ask_human | retry | return_empty"
  }
}
```

### 3.3. Kind: `Agent`
```json
{
  "kind": "Agent",
  "spec": {
    "model": {
      "provider": "anthropic | openai | deepseek | local_ollama",
      "modelName": "claude-3-7-sonnet | gpt-4o | deepseek-r1",
      "temperature": 0.2
    },
    "skills": [
      "pkg:skill/hitechcloud/k8s-diagnostics@^1.0.0",
      "pkg:skill/hitechcloud/db-triage@^1.0.0"
    ],
    "a2a": {
      "canDelegate": true,
      "allowedDelegates": ["agent.hitechcloud.devops", "agent.hitechcloud.billing"]
    },
    "agui": {
      "streaming": true,
      "hitlActions": ["k8s.delete_pod", "db.drop_table"]
    }
  }
}
```

### 3.4. Kind: `Plugin`
```json
{
  "kind": "Plugin",
  "spec": {
    "components": {
      "mcpServers": ["servers/db-server.json"],
      "skills": ["skills/triage.json"],
      "uiWidgets": ["dist/widget.js"]
    }
  }
}
```

---

## 4. Quá trình Đóng gói (`.hcp` Package Bundle)

Lệnh CLI `hitechcloud package build` sẽ tạo ra tệp nén `.hcp` (chuẩn tar.zst) có cấu trúc:
```
my-package-1.0.0.hcp
├── hcp.json               (Manifest chính)
├── checksums.sha256       (Bảng băm SHA256 từng file)
├── signature.sig          (Chữ ký Ed25519)
├── README.md              (Hướng dẫn sử dụng)
├── dist/                  (Binary/JavaScript đã build)
└── assets/                (Icon, schemas)
```
- Registry kiểm tra chữ ký `signature.sig` và `checksums.sha256` trước khi chấp thuận xuất bản.
