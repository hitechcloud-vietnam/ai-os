# Thiết kế Giao diện Dashboard (MCPHub Style)

Tài liệu này đặc tả toàn bộ kiến trúc, thiết kế giao diện và tính năng của hệ thống Dashboard quản trị cho **HiTechCloud Agent Platform**, dựa trên mã nguồn và triết lý thiết kế của **MCPHub** (https://github.com/samanhappy/mcphub).

---

## 1. Triết lý Thiết kế & Yêu cầu Cốt lõi

Theo yêu cầu nghiêm ngặt của dự án:
- **KHÔNG AI hoá giao diện**: Không sử dụng các hình vẽ trừu tượng mang tính marketing, không dùng robot/sparkles vô nghĩa.
- **KHÔNG sử dụng màu AI Gradient**: Tuyệt đối không dùng gradient tím-hồng-xanh neon kiểu AI marketing (ví dụ: purple/violet to pink/cyan gradient).
- **KHÔNG có trang chủ (No Landing Page)**: Hệ thống là một công cụ kỹ thuật thực thụ dành cho kỹ sư và lập trình viên. 100% ứng dụng vận hành bên trong **Dashboard**, không có trang giới thiệu/marketing landing page.
- **Màu sắc Kỹ thuật chuẩn mực (Dark Technical Theme)**:
  - Background chính: Dark Slate / Neutral Navy (`#090d16`, `#0b0f19`, `#0f172a`).
  - Panel & Card: Surface Dark (`#111827`, `#1e293b`).
  - Đường viền: Border mờ tinh tế (`#1e293b`, `#334155`).
  - Màu nhấn thao tác (Action/Primary): Kỹ thuật Xanh Dương Đậm (`#2563eb`, `#3b82f6`).
  - Trạng thái hệ thống: Xanh lá cây kỹ thuật (Success/Running: `#10b981`), Đỏ (Error/Stopped: `#ef4444`), Vàng (Warning/Rate Limited: `#f59e0b`).
  - Typography: `Inter` cho text tiêu chuẩn và `JetBrains Mono` cho mã nguồn, identifiers, JSON và CLI commands.

---

## 2. Kiến trúc Phân hệ Dashboard (Kế thừa từ MCPHub)

Dashboard bao gồm 11 phân hệ kỹ thuật chính nằm trên thanh Sidebar trái cố định:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ HiTechCloud MCP Hub & Gateway                                 [Key] [Prod]  │
├───────────────┬─────────────────────────────────────────────────────────────┤
│ PLATFORM      │                                                             │
│ • Dashboard   │                      MAIN WORKSPACE                         │
│ • Servers     │                                                             │
│ • Marketplace │  (Hiển thị nội dung chi tiết của từng tab chức năng)       │
│ • Skills      │                                                             │
│ • Plugins     │                                                             │
│ • Inspector   │                                                             │
│               │                                                             │
│ INTEGRATIONS  │                                                             │
│ • Clients     │                                                             │
│ • VS Code/CLI │                                                             │
│               │                                                             │
│ MANAGEMENT    │                                                             │
│ • Logs        │                                                             │
│ • API Keys    │                                                             │
│ • Settings    │                                                             │
├───────────────┴─────────────────────────────────────────────────────────────┤
│ Gateway: Online (12ms) | PostgreSQL: Connected | Redis: Ready               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Chi tiết 11 Phân hệ Chức năng

### 3.1. Dashboard Overview (Tổng quan Hệ thống)
- **Top Metrics Grid**:
  - Số lượng MCP Server đã đăng ký và trạng thái hoạt động (Active / Stopped).
  - Tổng số lượng Tool khả dụng qua giao thức JSON-RPC `tools/list`.
  - Độ trễ Gateway thời gian thực (p99 latency tính bằng ms).
  - Trạng thái bảo mật Sandbox & RBAC Token-Bucket Rate Limiter.
- **Infrastructure Health Matrix**:
  - Rust AI Gateway (Port 8080 / systemd status).
  - PostgreSQL 16 (Port 5432 / DB connection pool).
  - Redis 7 (Port 6379 / Cache & Rate Limiting).
  - Nginx Ingress (Port 443 / Let's Encrypt TLS).
- **Active Servers Quick View**: Danh sách các server đang chạy kèm nút Inspect nhanh.
- **Client Quick Connect Widget**: Lệnh copy nhanh 1-click cho Claude Code, Cursor và tải VS Code Extension.

### 3.2. Servers Manager (Quản lý MCP Server)
- **Quản lý Vòng đời Server**:
  - Start, Stop, Restart các tiến trình server.
  - Hỗ trợ đa giao thức: `stdio` (local process spawn qua npx, python, binary), `sse` (Server-Sent Events URL), `http` (Streaming JSON-RPC endpoint).
- **Modal Chi tiết & Công cụ của Server**:
  - Xem danh sách toàn bộ Tool do Server cung cấp.
  - Xem và chỉnh sửa biến môi trường (`ENV`), tham số dòng lệnh (`args`).
  - Xuất cấu hình JSON cho Server tương ứng.
- **Đăng ký Server Mới (Add Server Form)**:
  - Hỗ trợ form trực quan nhập tên định danh, giao thức, lệnh thực thi hoặc URL endpoint.

### 3.3. MCP Marketplace & Catalog (Kho Ứng dụng & Server)
- Kế thừa toàn bộ catalog từ `modelcontextprotocol/registry`, `mcphub`, và các repo cộng đồng (Crush, Goose, Roo, Aider, OpenCode, Anthropic Skills).
- **Tích hợp Tự động Kho OpenAPI HiTechCloud**:
  - Tự động nạp và chuyển đổi danh mục công cụ từ **HiTechCloud Core API** (`https://docs.hitechcloud.vn/endpoint/login?env=production`) và **HiTechCloud Tools API** (`https://doc-api-tools.hitechcloud.vn/`).
  - Hỗ trợ cài đặt 1-click các bộ công cụ chính thức: `HiTechCloud Infrastructure Tools`, `K8s DevOps Bundle`, `DBA Toolkit`, `SSL & DNS Automation`.
- **Tính năng**:
  - Tìm kiếm theo tên, tag, danh mục (Coding, Database, DevOps, Web, Knowledge, Agent).
  - Phân loại theo độ tin cậy và huy hiệu `Verified: HiTechCloud Official`.
  - Cài đặt / Gỡ cài đặt 1-click: Tự động ghi cấu hình vào hệ thống và kích hoạt worker process.

### 3.4. Skills & Prompts (Kỹ năng & Chuỗi Prompt Tự trị)
- Tích hợp chuẩn **Anthropic Skills** (`SKILL.md`) và **iFlyTek SkillHub**.
- Tự động sinh Skills từ kho công cụ OpenAPI HiTechCloud (`k8s-pod-auto-healing`, `database-performance-triage`, `ssl-expiry-monitor-and-renew`).
- **Tính năng**:
  - Quản lý prompt templates theo các phạm vi: `global`, `agent`, `session`, `project`.
  - Trình soạn thảo Prompt có syntax highlighting cho tham số biến.
  - **Trình Test Run trực tiếp**: Nhập tham số JSON đầu vào, gọi qua adapter AI để kiểm tra output trả về ngay trên giao diện.

### 3.5. Plugins & Sandboxes (Plugin & Chính sách Cô lập)
- Tích hợp chuẩn Plugin từ `claude-code-plugin-template` và `ai-plugin-marketplace`.
- **Ma trận Phân quyền Sandbox (Zero-Trust Security Matrix)**:
  - Cho phép / Chặn truy cập mạng (`allowNetwork`).
  - Cho phép / Chặn thao tác tệp hệ thống (`allowFileSystem`).
  - Cho phép / Chặn chạy tiến trình con (`allowProcessExec`).
  - Quản lý trạng thái Bật / Tắt từng bundle plugin độc lập.

### 3.6. Tool Inspector & Playground (Trình Kiểm thử JSON-RPC)
- Kế thừa bộ công cụ kiểm thử của MCPHub:
  - Cột trái: Tìm kiếm và chọn nhanh công cụ từ danh sách `tools/list`.
  - Cột phải: Trình soạn thảo tham số JSON (`params.arguments`) với schema tự động fill mẫu.
  - Nút **Execute Tool**: Gửi payload JSON-RPC 2.0 tới `/mcp/v1`.
  - Hiển thị kết quả trả về có định dạng JSON, đo đạc thời gian thực thi (latency ms), và nút Copy 1-click.

### 3.7. Client Config Exporter (Trình Xuất Cấu hình Client)
- Trình tạo và tải file cấu hình tự động cho **9 môi trường phát triển phổ biến**:
  1. **Claude Code CLI**: `~/.claude.json` hoặc lệnh `claude mcp add`.
  2. **Cursor IDE**: `.cursor/mcp.json`.
  3. **Windsurf IDE**: `~/.codeium/windsurf/mcp_config.json`.
  4. **Roo Code**: `mcpSettings.json`.
  5. **Block Goose**: `~/.config/goose/config.yaml`.
  6. **Aider AI**: `.aider.conf.yml`.
  7. **OpenCode**: `opencode.config.json`.
  8. **Zed Editor**: `~/.config/zed/settings.json`.
  9. **VS Code**: `.vscode/mcp.json`.
- Mỗi client đều có lệnh dòng lệnh, nội dung file xem trước, nút copy và nút tải file trực tiếp.

### 3.8. VS Code Extension & CLI Tooling
- Trang chuyên biệt phục vụ phân phối công cụ lập trình viên:
  - **VS Code Extension Package**: Link tải trực tiếp gói `.vsix`, hướng dẫn cài đặt qua giao diện và lệnh `code --install-extension`.
  - **HiTechCloud CLI (Rust)**: Lệnh cài đặt 1 dòng qua script `curl -sSL https://get-mcp.hitechcloud.vn/install.sh | bash` (Linux/macOS) và PowerShell (Windows).
  - Bảng tra cứu lệnh CLI tóm tắt (`hitechcloud --help`).

### 3.9. Activity & Audit Logs (Nhật ký Hoạt động & Bảo mật)
- Ghi nhận luồng sự kiện chi tiết:
  - Thời gian (`timestamp`), Loại sự kiện (`eventType`), Tên tool và Server (`toolName`, `serverName`).
  - Mã token người gọi (`callerKey`), Độ trễ xử lý (`durationMs`), Địa chỉ IP.
  - Trạng thái HTTP: `200 OK`, `ERROR`, `429 RATE_LIMITED`.
  - Bộ lọc tìm kiếm và phân loại theo trạng thái.

### 3.10. API Keys & RBAC Quotas (Quản lý Quyền & Khóa Truy cập)
- Tạo và thu hồi API Token (`htc_live_...` / `htc_ro_...`).
- Phân quyền theo vai trò (Role-Based Access Control):
  - `admin`: Toàn quyền quản trị hệ thống, server và cấu hình.
  - `developer`: Quyền thực thi công cụ, gọi API và xuất cấu hình.
  - `readonly`: Chỉ đọc danh sách và kiểm tra trạng thái.
- Thiết lập hạn ngạch Rate Limit (Token-Bucket giới hạn số request / phút qua Redis).

### 3.11. Multi-Model Router & Dynamic Pricing (Định tuyến Mô hình AI)
- Quản lý các nhà cung cấp mô hình suy luận: **Anthropic**, **OpenAI**, và **Nube.sh**.
- **Nube.sh Dynamic Pricing Table**:
  - Tự động nạp giá thời gian thực từ `https://ai.nube-api.com/v1/models/pricing`.
  - Hiển thị chi phí Input, Output, Cache Read ($/M tokens), và context window của từng model (`Nube-Choice`, `DeepSeek-V4.1-Flash`, `Qwen3.8-27B`, `GLM-5.3`, `GLM-5.3-Flash`, `Kimi-K2.6`).
  - Lựa chọn chiến lược định tuyến: **Cost-Optimized** (ưu tiên chi phí thấp nhất) hoặc **Deep Reasoning** (ưu tiên chất lượng).

### 3.12. System Settings (Cấu hình Hạ tầng & Tên miền)
- Bảng trạng thái định tuyến 4 tên miền chính thức:
  - `api-mcp.hitechcloud.vn`: REST & JSON-RPC Gateway Ingress.
  - `gw.hitechcloud.vn`: SSE & WebSocket Transport Hub.
  - `get-mcp.hitechcloud.vn`: CLI & Extension Distribution Hub.
  - `mcp.hitechcloud.vn`: Developer Web Dashboard (Vite SPA).
- Tùy chỉnh cổng Gateway, chuỗi kết nối PostgreSQL và Redis, cùng cơ chế bật/tắt Linux Sandbox cgroups.

---

## 4. Công nghệ Triển khai Frontend & Brand Assets

- **Framework**: React 18 + TypeScript 5.7.
- **Build Tool**: Vite 6.
- **Styling**: TailwindCSS 3.4 (Bộ quy chuẩn màu kỹ thuật, slate/neutral palette).
- **Icons & UI Symbols**: Lucide React.
- **Tài nguyên Thương hiệu Chính thức (Official Brand Assets)**:
  - Official Logo Vector SVG (Sidebar/Header): `https://hitechcloud.vn/wp-content/uploads/2025/01/hitechcloudvn.svg`
  - Favicon Multi-size ICO: `https://hitechcloud.vn/wp-content/themes/hitechcloud-news-2026/favicon/icon-v2.ico?v=2`
  - Favicon 32x32 PNG: `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-32x32.png?v=2&v=3`
  - App / PWA Icon 192x192 PNG: `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-192x192.png?v=2&v=3`
  - Apple Touch & Tile 270x270 PNG: `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png`
- **Trạng thái & Định tuyến**: URL Hash Router hỗ trợ bookmark và điều hướng trực tiếp giữa các tab.
- **Deploy**: Static SPA phân phối qua Nginx Ingress với caching tối ưu cho assets và gzip.
