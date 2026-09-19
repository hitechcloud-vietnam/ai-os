# Đặc tả Tích hợp & Tương thích Toàn diện với các CLI Bên thứ ba (3rd-Party CLIs)

Tài liệu này đặc tả cơ chế tương thích 100%, tự động thích ứng giao thức và kết nối hai chiều giữa **HiTechCloud Agent Platform** với tất cả các công cụ dòng lệnh (CLIs) và tác tử lập trình (AI Coding Agents) của bên thứ ba.

---

## 1. Mục tiêu & Triết lý Tương thích Toàn diện (Full Interoperability)

HiTechCloud Agent Platform được thiết kế theo nguyên tắc **Universal Agent Gateway**:
- **Không khóa chặt nhà cung cấp (Zero Vendor Lock-in)**: Lập trình viên có thể tự do sử dụng bất kỳ CLI nào mà họ yêu thích (Claude Code, Goose, Aider, OpenCode, Crush, SimCode, Cursor CLI, Roo Code, Codex...).
- **Hỗ trợ Đa Giao thức Trong suốt (Transparent Multi-Protocol Support)**: Tự động chuyển tiếp và dịch giao thức giữa `stdio` (local process piping), `SSE` (Server-Sent Events), `WebSocket` và REST `JSON-RPC 2.0`.
- **Đồng bộ Cấu hình Tự động (1-Click Auto-Configuration)**: Tự động phát hiện và sinh các tệp cấu hình phù hợp với từng CLI tương ứng (`.claude.json`, `.cursor/mcp.json`, `config.yaml`, `.aider.conf.yml`, `opencode.config.json`, `mcpSettings.json`).
- **Phân quyền & Kiểm soát Tập trung (Centralized Governance)**: Dù gọi từ bất kỳ CLI nào, mọi request đều đi qua lớp bảo mật RBAC, giới hạn tần suất Redis Token-Bucket, và được ghi nhật ký đầy đủ trong Audit Log của Gateway.

---

## 2. Danh mục CLI Bên thứ ba được Hỗ trợ Đầy đủ

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   3RD-PARTY CLI ECOSYSTEM MATRIX                                │
├─────────────────────────┬─────────────────────────┬────────────────────┬────────────────────────┤
│ CLI Tool / Project      │ Giao thức Kết nối       │ Tệp Cấu hình       │ Lệnh Tích hợp 1-Click   │
├─────────────────────────┼─────────────────────────┼────────────────────┼────────────────────────┤
│ Claude Code CLI         │ Stdio Proxy / HTTP SSE  │ ~/.claude.json     │ claude mcp add ...     │
│ Block Goose CLI         │ Stdio Subprocess / YAML │ config.yaml        │ goose session          │
│ Aider AI CLI            │ Stdio Proxy / YAML      │ .aider.conf.yml    │ aider --mcp-server ... │
│ Anomaly OpenCode        │ JSON-RPC Stream         │ opencode.config    │ opencode mcp-serve     │
│ Charm Crush CLI         │ Stdio Pipe / Lip Gloss  │ crush.json         │ crush serve --mcp      │
│ SimCode CLI             │ Sandbox Daemon          │ simcode.toml       │ simcode daemon         │
│ OpenAI Codex CLI        │ REST / Responses API    │ codex.toml         │ hitechcloud target ... │
│ Roo Code (VS Code & CLI)│ Stdio / JSON Settings   │ mcpSettings.json   │ roo-code --mcp         │
│ Windsurf IDE CLI        │ HTTP Stream             │ mcp_config.json    │ codeium sync           │
│ Zed Editor CLI          │ Stdio Context Server    │ settings.json      │ zed --context-server   │
│ Cline / Continue.dev    │ JSON-RPC 2.0 Stdio      │ config.json        │ cline mcp connect      │
└─────────────────────────┴─────────────────────────┴────────────────────┴────────────────────────┘
```

---

## 3. Chi tiết Tích hợp Từng CLI Bên thứ ba

### 3.1. Anthropic Claude Code CLI (`claude`)
Claude Code là công cụ terminal agent hàng đầu của Anthropic với khả năng chạy subagents và tự động gọi tools.

- **Phương thức 1: Kết nối Direct Remote MCP qua SSE**
  ```bash
  claude mcp add hitechcloud -- https://api-mcp.hitechcloud.vn/mcp/v1 \
    -H "Authorization: Bearer htc_live_d8f7e2a4b1c90123456789abcdef01"
  ```
- **Phương thức 2: Kết nối Cục bộ qua HiTechCloud Stdio Proxy (`hitechcloud mcp serve`)**
  Tự động thêm vào `~/.claude.json` hoặc `.mcp.json` của workspace:
  ```json
  {
    "mcpServers": {
      "hitechcloud": {
        "command": "hitechcloud",
        "args": ["mcp", "serve"],
        "env": {
          "HITECHCLOUD_API_KEY": "htc_live_d8f7e2a4b1c90123456789abcdef01",
          "HITECHCLOUD_GATEWAY_URL": "https://api-mcp.hitechcloud.vn/mcp/v1"
        }
      }
    }
  }
  ```
- **Hỗ trợ Plugin & Marketplace Claude Code**:
  - Tự động sinh `claude-marketplace.json` tại `https://api-mcp.hitechcloud.vn/claude-marketplace.json`.
  - Hỗ trợ đầy đủ lệnh:
    ```bash
    claude plugin marketplace add https://api-mcp.hitechcloud.vn/claude-marketplace.json
    claude plugin install hitechcloud-server
    ```

---

### 3.2. Block Goose Developer Agent CLI (`goose`)
Goose là on-machine agent mã nguồn mở mạnh mẽ từ Block (Square), hỗ trợ tự động hóa refactoring và shell execution.

- **Vị trí Cấu hình**: `~/.config/goose/config.yaml`
- **Tự động sinh bởi CLI**: `hitechcloud client export --client goose --write`
- **Cấu hình YAML tương thích**:
  ```yaml
  extensions:
    hitechcloud:
      type: stdio
      cmd: hitechcloud
      args:
        - mcp
        - serve
      envs:
        HITECHCLOUD_API_KEY: "htc_live_d8f7e2a4b1c90123456789abcdef01"
        HITECHCLOUD_GATEWAY_URL: "https://api-mcp.hitechcloud.vn/mcp/v1"
      enabled: true
      timeout: 300
  ```
- **Khởi chạy Goose với toàn bộ công cụ HiTechCloud**:
  ```bash
  goose session --with-extension hitechcloud
  ```

---

### 3.3. Aider AI Pair Programmer CLI (`aider`)
Aider là terminal coding assistant hàng đầu với giải thuật Repo Map AST (Tree-sitter) và tự động tạo git commits.

- **Vị trí Cấu hình**: `.aider.conf.yml` trong thư mục dự án.
- **Tự động sinh bởi CLI**: `hitechcloud client export --client aider --write`
- **Cấu hình Tương thích**:
  ```yaml
  mcp-servers:
    - name: hitechcloud
      cmd: hitechcloud mcp serve
      env:
        HITECHCLOUD_API_KEY: "htc_live_d8f7e2a4b1c90123456789abcdef01"
        HITECHCLOUD_GATEWAY_URL: "https://api-mcp.hitechcloud.vn/mcp/v1"
      auto-approve:
        - read_file
        - github_search_issues
        - execute_sql
  ```
- **Khởi chạy Aider**:
  ```bash
  aider --mcp-server hitechcloud
  ```

---

### 3.4. Charm Crush CLI (`crush`)
Crush là terminal coding companion hiện đại từ Charmbracelet (sử dụng Lip Gloss & Bubble Tea).

- **Khởi chạy MCP Server tương thích Crush**:
  ```bash
  crush serve --mcp --port 8081
  ```
- **Cấu hình trong Crush**:
  ```json
  {
    "mcp": {
      "servers": [
        {
          "name": "hitechcloud",
          "url": "https://api-mcp.hitechcloud.vn/mcp/v1",
          "auth": "Bearer htc_live_d8f7e2a4b1c90123456789abcdef01"
        }
      ]
    }
  }
  ```

---

### 3.5. Anomaly OpenCode CLI (`opencode`)
OpenCode là terminal AI assistant mã nguồn mở hỗ trợ multi-provider model routing và streaming diff.

- **Vị trí Cấu hình**: `opencode.config.json`
- **Tự động sinh bởi CLI**: `hitechcloud client export --client opencode --write`
- **Cấu hình Tương thích**:
  ```json
  {
    "plugins": {
      "mcp": {
        "hitechcloud": {
          "endpoint": "https://api-mcp.hitechcloud.vn/mcp/v1",
          "authHeader": "Bearer htc_live_d8f7e2a4b1c90123456789abcdef01"
        }
      }
    }
  }
  ```

---

### 3.6. SimCode CLI (`simcode-cli`)
SimCode CLI cung cấp cơ chế thực thi mô phỏng và kiểm thử an toàn các thay đổi code trước khi áp dụng vào codebase.

- **Vị trí Cấu hình**: `simcode.toml`
- **Cấu hình Tương thích**:
  ```toml
  [mcp.hitechcloud]
  command = "hitechcloud"
  args = ["mcp", "serve"]
  sandbox_mode = "strict"
  timeout_seconds = 60
  ```

---

### 3.7. OpenAI Codex & OpenAI Tools API (`codex` / `openai`)
Đối với các agent sử dụng OpenAI Function Calling hoặc Responses API tools:

- **Adapter Layer (`hitechcloud-adapter-openai`)**:
  - Tự động chuyển đổi toàn bộ công cụ MCP sang định dạng JSON Schema của OpenAI:
    ```json
    {
      "type": "function",
      "function": {
        "name": "hitechcloud__execute_sql",
        "description": "Execute read-only SQL queries",
        "parameters": { ... }
      }
    }
    ```
- **Cổng Thực thi Trực tiếp**: `POST https://api-mcp.hitechcloud.vn/tool-execute`
  ```json
  {
    "tool": "hitechcloud.server_status",
    "arguments": {}
  }
  ```

---

### 3.8. Roo Code, Cursor, Windsurf, Zed & Cline
- **Roo Code**: `mcpSettings.json` hỗ trợ tự động cấp quyền `alwaysAllow` cho các công cụ an toàn.
- **Cursor IDE**: `.cursor/mcp.json` kết nối trực tiếp endpoint HTTPS không cần node/python runtime cục bộ.
- **Windsurf**: `~/.codeium/windsurf/mcp_config.json` hỗ trợ luồng agentic flow.
- **Zed Editor**: `~/.config/zed/settings.json` khai báo trong trường `context_servers`.

---

### 3.9. Cấu hình Nube.sh Provider cho các 3rd-Party CLIs

Khi lập trình viên muốn sử dụng backend suy luận giá rẻ hoặc reasoning models từ **Nube.sh** cùng với các MCP tools của HiTechCloud:

- **Claude Code với Nube.sh (Anthropic Protocol)**:
  ```bash
  export ANTHROPIC_BASE_URL="https://ai.nube-api.com"
  export ANTHROPIC_API_KEY="nb_live_xxxxxxxxxxxxxxxx"
  claude --model "Nube-Choice"
  ```
- **Aider / OpenCode / Goose với Nube.sh (OpenAI Protocol)**:
  ```bash
  export OPENAI_BASE_URL="https://ai.nube-api.com/v1"
  export OPENAI_API_KEY="nb_live_xxxxxxxxxxxxxxxx"
  aider --model "openai/DeepSeek-V4.1-Flash"
  ```

---

## 4. Kiến trúc Cầu nối Đa Giao thức (Multi-Protocol Bridge Architecture)

Để đảm bảo mọi CLI bên thứ ba đều hoạt động mượt mà không gặp lỗi không tương thích:

```
                      ┌───────────────────────────────────────┐
                      │          3RD-PARTY CLI AGENTS         │
                      │  Claude · Goose · Aider · OpenCode    │
                      │  Crush · SimCode · Codex · Roo · Zed  │
                      └───────────────────┬───────────────────┘
                                          │
                  ┌───────────────────────┼───────────────────────┐
                  ▼                       ▼                       ▼
            STDIO PIPES              SSE STREAMS             REST RPC 2.0
        (hitechcloud serve)     (gw.hitechcloud.vn)     (api-mcp.hitechcloud.vn)
                  │                       │                       │
                  └───────────────────────┼───────────────────────┘
                                          ▼
                      ┌───────────────────────────────────────┐
                      │    HITECHCLOUD PROTOCOL MULTIPLEXER   │
                      │  • Header Injection (Bearer Token)    │
                      │  • JSON-RPC 2.0 Harmonizer            │
                      │  • Stream Chunk Buffer & Keep-Alive   │
                      │  • Error Code Normalization           │
                      └───────────────────┬───────────────────┘
                                          │
                                          ▼
                      ┌───────────────────────────────────────┐
                      │       RUST CORE MCP GATEWAY & RBAC    │
                      │        (Port 8080 / Tokio Async)      │
                      └───────────────────────────────────────┘
```

### 4.1. Chuẩn hóa Mã Lỗi (JSON-RPC Error Code Normalization)
Các CLI bên thứ ba tuân thủ tiêu chuẩn JSON-RPC 2.0 nghiêm ngặt. Gateway chuẩn hóa toàn bộ mã lỗi:
- `-32700`: Parse Error (Payload JSON không đúng định dạng).
- `-32600`: Invalid Request (Cấu trúc bản tin không hợp lệ).
- `-32601`: Method Not Found (Không tìm thấy method hoặc tool tương ứng).
- `-32602`: Invalid Params (Tham số không khớp với InputSchema).
- `-32603`: Internal RPC Error (Lỗi nội bộ server/sandbox).
- `42900`: Rate Limit Exceeded (Vượt quá hạn ngạch Token-Bucket).
- `40100`: Unauthorized (API Key không hợp lệ hoặc hết hạn).

### 4.2. Cơ chế Heartbeat & Keep-Alive
Đối với kết nối SSE và Stdio của các CLI chạy lâu (long-running sessions như Goose và Aider), Gateway tự động gửi bản tin Ping/Keep-alive định kỳ mỗi 15 giây để ngăn chặn việc ngắt kết nối do timeout của proxy/firewall.

---

## 5. Lệnh CLI Đồng bộ Toàn bộ Cấu hình

Lập trình viên chỉ cần chạy lệnh sau để tự động phát hiện các CLI đang cài trên máy và sinh cấu hình đồng loạt:

```bash
# Liệt kê trạng thái tích hợp của tất cả các CLI trên máy
hitechcloud client list

# Tự động xuất cấu hình cho toàn bộ các CLI đã phát hiện
hitechcloud client export --all --write
```
