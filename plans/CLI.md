# Đặc tả Kỹ thuật: HiTechCloud CLI Riêng biệt

Tài liệu này đặc tả toàn bộ kiến trúc, hệ thống lệnh dòng lệnh và tính năng của công cụ dòng lệnh **HiTechCloud CLI** (`hitechcloud`), được phát triển bằng ngôn ngữ Rust để đạt hiệu năng tối đa và tiêu thụ tài nguyên tối thiểu.

---

## 1. Mục tiêu & Định vị

HiTechCloud CLI là công cụ đồng hành thiết yếu cho lập trình viên trên terminal:
- Quản lý toàn diện các tài nguyên trên Platform: MCP Servers, Agent Skills, Plugin Bundles, API Keys.
- Kiểm tra tính hợp lệ và đóng gói package theo chuẩn ModelContextProtocol Registry trước khi xuất bản.
- **Client Configuration Generator**: Tự động sinh và ghi cấu hình kết nối cho **9 công cụ AI và IDE** khác nhau chỉ với một lệnh.
- **Direct Tool Calling & Debugging**: Gọi thực thi các công cụ MCP trực tiếp từ terminal mà không cần mở giao diện web.
- **Local MCP Proxy Bridge (`mcp serve`)**: Cung cấp tiến trình proxy giao tiếp qua `stdio` để các IDE (như Cursor, Claude Code, Windsurf) có thể kết nối gián tiếp qua Gateway đám mây.

---

## 2. Cài đặt & Phân phối

### 2.1. Cài đặt tự động 1 dòng (Linux & macOS)
```bash
curl -sSL https://get-mcp.hitechcloud.vn/install.sh | bash
```

### 2.2. Cài đặt trên Windows (PowerShell)
```powershell
iwr -useb https://get-mcp.hitechcloud.vn/install.ps1 | iex
```

### 2.3. Cài đặt từ Mã nguồn (Cargo)
```bash
cargo install --path crates/hitechcloud
```

---

## 3. Bảng Cây Lệnh Toàn diện (Command Tree)

```
hitechcloud
├── login                        # Lưu Master API Key & Gateway URL vào ~/.hitechcloud/config.json
├── config
│   ├── set <key> <value>        # Thiết lập thông số cấu hình cục bộ
│   ├── get <key>                # Đọc giá trị cấu hình
│   └── list                     # Liệt kê toàn bộ cấu hình hiện tại
├── status                       # Kiểm tra tình trạng kết nối tới cụm Gateway
├── provider
│   ├── add <name> [--api-key] [--protocol] # Thêm provider AI (nube, anthropic, openai)
│   ├── list                     # Liệt kê các provider đã cấu hình
│   └── sync-pricing             # Kích hoạt đồng bộ bảng giá thời gian thực từ Nube.sh
├── openapi
│   ├── import <url> [--kind]    # Nạp OpenAPI endpoint và tự động sinh MCP/Skill
│   └── sync                     # Đồng bộ lại schema từ docs.hitechcloud.vn & doc-api-tools
├── package
│   ├── init <name> [--kind]     # Khởi tạo universal manifest hcp.json (Skill/Plugin/Agent/McpServer)
│   ├── build <path>             # Đóng gói thành .hcp bundle kèm SHA256 & Ed25519 signature
│   ├── publish <file.hcp>       # Xuất bản package lên Registry
│   ├── yank <name@version>      # Thu hồi phiên bản có lỗ hổng
│   └── deprecate <name>         # Đánh dấu khai tử một package
├── mcp
│   ├── init <name> [--transport] # Khởi tạo mẫu dự án MCP Server (mcp-manifest.json)
│   ├── validate <path>          # Kiểm tra tính hợp lệ của manifest MCP
│   ├── publish <path> [--version]# Xuất bản MCP Server lên Registry
│   ├── list [--mine]            # Liệt kê danh sách MCP Server trên hệ thống
│   └── serve                    # Khởi chạy STDIO proxy bridge cho các client IDE
├── skill
│   ├── init <name> [--scope]    # Khởi tạo tệp kỹ năng chuẩn SKILL.md (Anthropic spec)
│   ├── validate <path>          # Xác thực cú pháp YAML frontmatter và instructions
│   ├── publish <path> [--version]# Xuất bản Skill lên SkillHub
│   ├── list                     # Liệt kê các Skill khả dụng
│   └── run <name> [params_json] # Chạy thử nghiệm kỹ năng qua pipeline AI adapter
├── plugin
│   ├── init <name>              # Khởi tạo plugin bundle (plugin-manifest.json)
│   ├── validate <path>          # Kiểm tra cấu trúc bundle và permissions
│   ├── publish <path> [--version]# Xuất bản plugin lên Marketplace
│   ├── install <name> [--scope] # Cài đặt plugin vào workspace hoặc user global
│   └── list                     # Liệt kê các plugin đã cài đặt
├── client
│   ├── list                     # Danh sách 11 client AI & IDE được hỗ trợ
│   └── export [--client <type> | --all] [--write] # Xuất cấu hình (hoặc tự ghi vào file)
└── tool
    ├── list                     # Liệt kê toàn bộ tool khả dụng qua RPC tools/list
    ├── inspect <tool_name>      # Xem chi tiết schema tham số đầu vào
    └── call <tool_name> <args>  # Thực thi tool với tham số JSON trực tiếp
```

---

## 4. Chi tiết các Phân hệ Lệnh Cốt lõi

### 4.1. Lệnh Quản lý Client Configurations (`client export`)
Cho phép lập trình viên kết nối editor của mình với HiTechCloud chỉ trong vài giây:

```bash
# Xem danh sách các client được hỗ trợ
hitechcloud client list

# Xuất cấu hình cho Cursor IDE
hitechcloud client export --client cursor

# Tự động ghi file .cursor/mcp.json vào dự án hiện tại
hitechcloud client export --client cursor --write

# Xuất cấu hình cho Claude Code CLI
hitechcloud client export --client claude --write

# Xuất cấu hình cho Block Goose (~/.config/goose/config.yaml)
hitechcloud client export --client goose

# Xuất cấu hình cho Windsurf, Roo Code, Aider, OpenCode, Zed, VS Code
hitechcloud client export --client windsurf --write
hitechcloud client export --client roo --write
hitechcloud client export --client aider --write
hitechcloud client export --client opencode --write
hitechcloud client export --client zed --write
hitechcloud client export --client vscode --write
```

### 4.2. Lệnh Kiểm thử và Gọi Tool Trực tiếp (`tool call`)
Giúp kỹ sư debug và kiểm tra phản hồi từ các server MCP mà không phụ thuộc vào LLM:

```bash
# Liệt kê tất cả công cụ đã đăng ký
hitechcloud tool list

# Gọi công cụ kiểm tra trạng thái máy chủ
hitechcloud tool call hitechcloud.server_status '{}'

# Gọi công cụ đọc file trong sandbox
hitechcloud tool call read_file '{"filePath": "/home/workspace/README.md"}'

# Gọi công cụ truy vấn cơ sở dữ liệu
hitechcloud tool call execute_sql '{"query": "SELECT count(*) FROM mcp_servers;"}'
```

### 4.3. Chế độ STDIO Proxy Bridge (`mcp serve`)
Khi một công cụ client chỉ hỗ trợ kết nối `stdio` (như Claude Code cục bộ, Zed hoặc Roo Code), lệnh `hitechcloud mcp serve` sẽ:
1. Lắng nghe các bản tin JSON-RPC 2.0 gửi vào `stdin`.
2. Gắn kèm Bearer Token xác thực của người dùng.
3. Chuyển tiếp (forward) request tới HTTPS endpoint của Gateway (`https://api-mcp.hitechcloud.vn/mcp/v1`).
4. Nhận response và ghi trực tiếp ra `stdout`.

---

## 5. CI/CD Pipeline Automation

```yaml
# Ví dụ cấu hình GitHub Actions với HiTechCloud CLI
- name: Publish Plugin to HiTechCloud Marketplace
  run: |
    hitechcloud login ${{ secrets.HITECHCLOUD_MASTER_API_KEY }}
    hitechcloud plugin validate ./my-plugin
    hitechcloud plugin publish ./my-plugin --version ${{ github.ref_name }}
```

## 6. Bảng Mã Thoát (Exit Codes)

| Code | Tên Định danh | Ý nghĩa |
|---|---|---|
| 0 | `SUCCESS` | Lệnh hoàn tất thành công |
| 1 | `VALIDATION_ERROR` | Lỗi cú pháp JSON / YAML manifest không hợp lệ |
| 2 | `AUTH_ERROR` | Khóa API Key không hợp lệ hoặc không đủ quyền RBAC |
| 3 | `CONFLICT_ERROR` | Phiên bản package (version) đã tồn tại trên Registry |
| 4 | `NETWORK_ERROR` | Không thể kết nối tới cụm Gateway / Hết thời gian chờ |

