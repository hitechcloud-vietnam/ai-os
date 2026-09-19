# Đặc tả Kỹ thuật: VS Code Extension Riêng biệt

Tài liệu này đặc tả toàn bộ kiến trúc, chức năng và quy trình phát triển tiện ích mở rộng riêng biệt dành cho Visual Studio Code mang tên **HiTechCloud Agent Platform** (`hitechcloud-agent-platform`).

---

## 1. Mục tiêu & Phạm vi

Tiện ích mở rộng VS Code đóng vai trò là cầu nối trực tiếp giữa môi trường lập trình của lập trình viên và cụm máy chủ **HiTechCloud Agent Gateway**:
- Khám phá và quản lý danh sách **MCP Servers**, **Tools** và **Skills** ngay trên thanh Activity Bar của VS Code.
- Cho phép chạy thử nghiệm và kiểm tra công cụ MCP trực tiếp với giao diện nhập tham số thân thiện và xem kết quả JSON trong editor.
- Tự động tạo và cập nhật tệp cấu hình `.mcp.json` hoặc cấu hình cho các client khác trong workspace.
- Hiển thị trạng thái Gateway và các chỉ số hoạt động theo thời gian thực trên Status Bar.
- Đóng gói thành tệp `.vsix` độc lập để phân phối trực tiếp từ Dashboard mà không cần xuất bản lên Visual Studio Marketplace công khai.

---

## 2. Cấu trúc Dự án Extension

```
vscode-extension/
├── package.json               # Manifest khai báo contribution points, views, commands
├── tsconfig.json              # Cấu hình biên dịch TypeScript sang CommonJS
├── .vscodeignore              # Loại bỏ source files và node_modules khi đóng gói VSIX
├── media/
│   └── icon.png               # High-res 270x270 extension brand icon
├── resources/
│   ├── icon.svg               # Biểu tượng thương hiệu HiTechCloud trên Activity Bar
│   └── icon-32.png            # 32x32 raster icon cho Activity Bar
├── src/
│   ├── api.ts                 # HTTP & JSON-RPC Client giao tiếp với Gateway
│   ├── extension.ts           # Điểm khởi chạy (activation lifecycle) & đăng ký commands
│   ├── treeViews/
│   │   └── providers.ts       # TreeDataProviders cho Servers, Tools, Skills
│   └── utils/
│       └── configHelper.ts    # Tiện ích đọc/ghi cấu hình .mcp.json vào workspace
└── dist/
    └── extension.js           # Bundle mã nguồn sau khi biên dịch
```

> **Tài nguyên biểu tượng chính thức**:
> - Extension Marketplace Icon (`media/icon.png`): Lấy từ `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-270x270.png`.
> - Activity Bar Container Icon (`resources/icon-32.png`): Lấy từ `https://hitechcloud.vn/wp-content/uploads/2025/01/cropped-hitechcloud500-32x32.png?v=2&v=3`.

---

## 3. Khai báo Contribution Points (`package.json`)

### 3.1. Views Containers & Views (Activity Bar)
Extension đăng ký một icon riêng biệt trên thanh Activity Bar bên trái của VS Code:
- **Container ID**: `hitechcloud-explorer` (Title: `HiTechCloud MCP`)
- **3 Phân hệ TreeView**:
  1. `hitechcloud.serversView` (MCP Servers): Danh sách các server đang chạy và phương thức truyền tải (`stdio`, `sse`, `stream`).
  2. `hitechcloud.toolsView` (Available Tools): Danh sách toàn bộ công cụ mà AI agent có thể gọi kèm mô tả ngắn.
  3. `hitechcloud.skillsView` (Agent Skills): Danh sách các kỹ năng tự trị đã được tải vào hệ thống.

### 3.2. Danh sách Commands (Lệnh điều khiển)
- `hitechcloud.login`: Hiển thị input box yêu cầu người dùng nhập Master API Key (`htc_live_...`) và lưu vào cấu hình Global của VS Code.
- `hitechcloud.refresh`: Làm mới dữ liệu từ Gateway cho cả 3 bảng TreeView.
- `hitechcloud.runTool`: Mở hộp thoại nhập tham số JSON và thực thi công cụ được chọn, sau đó mở tài liệu JSON mới hiển thị output trả về.
- `hitechcloud.openDashboard`: Mở trình duyệt web trỏ tới Dashboard tại `https://mcp.hitechcloud.vn`.
- `hitechcloud.generateClientConfig`: Tự động tạo tệp `.mcp.json` chuẩn trong thư mục gốc của dự án hiện tại.

### 3.3. Cấu hình Extension Settings (`hitechcloud.*`)
- `hitechcloud.apiUrl`: Địa chỉ endpoint của Gateway (Mặc định: `https://api-mcp.hitechcloud.vn`).
- `hitechcloud.apiKey`: Khóa xác thực Bearer Token của người dùng.

---

## 4. Chi tiết Kiến trúc Thành phần

### 4.1. API Client (`src/api.ts`)
- Sử dụng module `http`/`https` tích hợp của Node.js để tương thích hoàn toàn với môi trường VS Code extension host.
- Tự động gắn header `Authorization: Bearer <API_KEY>` và `Content-Type: application/json`.
- Các phương thức cốt lõi:
  - `listServers()`: Gửi `GET /v1/registry/mcp-servers`.
  - `listSkills()`: Gửi `GET /v1/registry/skills`.
  - `listTools()`: Gửi JSON-RPC 2.0 `tools/list` tới `/mcp/v1`.
  - `callTool(name, args)`: Gửi JSON-RPC 2.0 `tools/call` tới `/mcp/v1`.

### 4.2. Tree Data Providers (`src/treeViews/providers.ts`)
- Triển khai interface `vscode.TreeDataProvider<T>` với cơ chế phát tín hiệu sự kiện thay đổi dữ liệu `EventEmitter`.
- Tự động gán ThemeIcon tương ứng:
  - Server: `pass-filled` (Running) / `circle-slash` (Stopped).
  - Tool: `tools`.
  - Skill: `sparkle`.
- Khi nhấp đúp vào Tool, tự động gọi command `hitechcloud.runTool` với tham số là tool tương ứng.

### 4.3. Interactive Tool Runner (`src/extension.ts`)
- Khi người dùng kích hoạt thực thi tool:
  1. Hiển thị hộp thoại `vscode.window.showInputBox` với giá trị mặc định là `{}` hoặc schema mẫu.
  2. Hiển thị thanh tiến trình không chặn `vscode.ProgressLocation.Notification` ("Executing {tool_name}...").
  3. Sau khi có phản hồi, tạo một TextDocument tạm thời với ngôn ngữ `json` và mở trên Editor tab để lập trình viên phân tích và sao chép.

---

## 5. Quy trình Đóng gói & Phân phối (.VSIX)

1. **Biên dịch mã nguồn**:
   ```bash
   cd vscode-extension && npm run compile
   ```
2. **Đóng gói VSIX bằng `@vscode/vsce`**:
   ```bash
   npx --yes @vscode/vsce package --allow-missing-repository
   ```
   Kết quả tạo ra tệp: `hitechcloud-agent-platform-1.0.0.vsix`.
3. **Phân phối trên Dashboard & Nginx**:
   - Sao chép tệp `.vsix` vào thư mục `frontend/public/downloads/`.
   - Nginx phục vụ tại URL: `https://mcp.hitechcloud.vn/downloads/hitechcloud-agent-platform-1.0.0.vsix`.
   - Lập trình viên có thể tải về và cài đặt bằng 1 lệnh:
     ```bash
     code --install-extension hitechcloud-agent-platform-1.0.0.vsix
     ```
