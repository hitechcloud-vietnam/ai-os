# Đặc tả Kỹ thuật: Tích hợp Toàn diện Microsoft Graph & Hệ sinh thái Microsoft 365 Copilot

Tài liệu này đặc tả kiến trúc tích hợp sâu giữa **HiTechCloud Agent Platform** và **Hệ sinh thái Microsoft 365 / Microsoft Graph**, thiết kế phân hệ Microsoft như một **Ecosystem Adapter hạng nhất (First-Class Ecosystem Adapter)** ngang hàng với Anthropic, OpenAI và Google, tận dụng lợi thế thành viên **Microsoft AI Cloud Partner Program & CSP Indirect Reseller** của HiTechCloud.

---

## 1. Vị trí Kiến trúc & Mô hình Tổng thể

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   MICROSOFT 365 USER WORKSPACE                                   │
│  • Microsoft 365 Copilot Chat      • Microsoft Teams         • Outlook Web/Desktop               │
│  • SharePoint Intranet Portal      • OneDrive for Business   • Excel / Planner                   │
└─────────────────────────────────┬────────────────────────────────────────────────────────────────┘
                                  │
         ┌────────────────────────┴────────────────────────┐
         │ (1) M365 Copilot Extensibility & MCP Apps       │ (2) Microsoft Graph REST / Delta API
         ▼                                                 ▼
┌─────────────────────────────────┐               ┌────────────────────────────────────────────────┐
│   M365 COPILOT AGENTS & MCP     │               │           MICROSOFT GRAPH ADAPTER LAYER        │
│  • Declarative Agents (ATK)     │               │  • Outlook Mail       • Calendar / Schedule    │
│  • Custom Engine Agents (CEA)   │               │  • Teams Channels     • OneDrive & Files       │
│  • Remote Federated MCP Server  │               │  • SharePoint Sites   • Excel Workbooks        │
│  • Interactive A2UI Widgets     │               │  • Users & Groups     • Planner & Tasks        │
└────────────────┬────────────────┘               └───────────────────────┬────────────────────────┘
                 │                                                        │
                 └────────────────────────┬───────────────────────────────┘
                                          │ OAuth 2.0 / OBO (On-Behalf-Of) & Bearer Auth
                                          ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                         HITECHCLOUD AI CORE GATEWAY (RUST WORKSPACE)                             │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│  • Multi-Model Router (Anthropic, OpenAI, Nube.sh, DeepSeek, M365 Copilot APIs)                  │
│  • MCP Gateway & Registry (803 Official HiTechCloud Tools + M365 Graph Tools)                    │
│  • A2A Protocol v1.0 Dispatcher (Agent-to-Agent Delegation)                                      │
│  • Zero-Trust Linux / WASM Sandbox & RBAC Policy Engine                                          │
└─────────────────────────────────┬────────────────────────────────────────────────────────────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         ▼                        ▼                        ▼
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ my.hitechcloud.vn│    │tools.hitechcloud │    │ 3rd-Party Clouds │
│ (348 User APIs)  │    │(455 Tool APIs)   │    │ (AWS/Azure/GCP)  │
└──────────────────┘    └──────────────────┘    └──────────────────┘
```

---

## 2. Phân biệt Rõ Ràng: Microsoft Graph APIs vs. Copilot APIs

| Tiêu chí | **Microsoft Graph APIs** | **Microsoft 365 Copilot APIs** |
|---|---|---|
| **Bản chất** | Lớp API nền tảng CRUD & Data Access | Lớp AI Reasoning & Semantic Intelligence |
| **Chức năng chính** | Đọc/Ghi dữ liệu Mail, Teams, Calendar, OneDrive, Users, SharePoint | Đặt câu hỏi tự nhiên, suy luận ngữ cảnh doanh nghiệp, sinh nội dung |
| **Namespace** | `https://graph.microsoft.com/v1.0/...` | Graph Copilot namespace (`/beta/copilot/...`) |
| **Quyền hạn (RBAC)** | Scopes Granular (vd: `Mail.ReadWrite`, `Files.Read.All`) | Tuân thủ Conditional Access & Semantic Permission Trimming |
| **Cách dùng trong Agent** | Agent gọi Graph làm **Tool/Action** để thao tác dữ liệu | Agent gọi Copilot API để nhờ Copilot phân tích dữ liệu M365 |

---

## 3. Chi tiết Tích hợp Microsoft Graph API (Data Access Layer)

Adapter `providers/microsoft/graph/` chuyển đổi các endpoint Graph thành bộ công cụ **MCP Tools**:

```
HiTechCloud Agent
       │
       ▼
Microsoft Graph Adapter
       │
 ┌─────┼─────────────────────────────────────────┐
 ▼     ▼          ▼           ▼         ▼        ▼
Mail Calendar  Teams/Chat  OneDrive  SharePoint Users
Files Excel     Planner     Groups    Devices    Security
```

### Danh mục Công cụ MCP Microsoft Graph (`pkg:mcp/microsoft/graph`):

1. **Outlook Mail (`msgraph.mail.*`)**:
   - `list_messages`: Lọc email theo người gửi, tiêu đề, trạng thái đọc (`$filter`, `$search`).
   - `send_mail`: Soạn và gửi email, đính kèm tệp từ OneDrive hoặc tệp cục bộ.
   - `create_draft`: Tạo bản nháp email cho người dùng duyệt trước khi gửi.
2. **Calendar & Meeting (`msgraph.calendar.*`)**:
   - `list_events`: Lấy lịch họp theo khoảng thời gian.
   - `create_event`: Đặt lịch hẹn, tự động tạo phòng họp Teams online (`isOnlineMeeting: true`).
   - `find_meeting_times`: Tự động tìm khoảng thời gian trống chung giữa nhiều người tham gia.
3. **Microsoft Teams (`msgraph.teams.*`)**:
   - `send_chat_message`: Gửi tin nhắn vào kênh (Channel) hoặc chat 1:1.
   - `list_channel_messages`: Đọc lịch sử trao đổi trong kênh dự án.
   - `post_adaptive_card`: Bắn thẻ tương tác (Adaptive Card / A2UI) vào Teams.
4. **OneDrive & SharePoint (`msgraph.storage.*`)**:
   - `search_files`: Tìm kiếm tài liệu theo từ khóa hoặc nội dung ngữ nghĩa trong SharePoint/OneDrive.
   - `upload_file` / `download_file`: Tải lên/tải xuống tệp tin dự án.
   - `share_link`: Sinh link chia sẻ tài liệu với phân quyền (`view`, `edit`, `org-only`).
5. **Excel Workbooks (`msgraph.excel.*`)**:
   - `read_range`: Đọc dữ liệu bảng tính theo dải ô (Range / Table).
   - `append_row`: Thêm dòng dữ liệu mới vào bảng Excel (ví dụ: ghi log hóa đơn hoặc báo cáo).
6. **Planner & ToDo (`msgraph.planner.*`)**:
   - `create_task`: Tạo task công việc trong Planner bucket, gán người phụ trách và hạn chót.
   - `list_tasks`: Tra cứu tiến độ công việc của team.
7. **Users & Directory (`msgraph.directory.*`)**:
   - `get_user_profile`: Lấy thông tin phòng ban, chức danh, quản lý trực tiếp.
   - `list_group_members`: Liệt kê thành viên nhóm bảo mật hoặc M365 Group.

---

## 4. Microsoft 365 Copilot Connectors

HiTechCloud hỗ trợ cả hai mô hình Connector của Microsoft:

### 4.1. Synced Connector (Knowledge Index Ingestion)
- **Cơ chế**: Đồng bộ tài liệu kỹ thuật, kiến thức hướng dẫn, catalog dịch vụ của HiTechCloud vào **Microsoft Graph Search Index**.
- **Luồng dữ liệu**:
  ```
  HiTechCloud Docs & Knowledgebase ──▶ Microsoft Graph Connector ──▶ Semantic Index ──▶ M365 Copilot
  ```
- **Lợi ích**: Khi nhân viên công ty hỏi Copilot trong Word/Teams: *"Chính sách backup máy chủ của HiTechCloud như thế nào?"*, Copilot tự động trích dẫn tài liệu từ HiTechCloud.

### 4.2. Federated Connector qua Remote MCP Server (Realtime Live Data)
- **Cơ chế**: Kết nối thời gian thực sử dụng chuẩn **Remote MCP Server qua HTTPS**.
- **Luồng dữ liệu**:
  ```
  User hỏi M365 Copilot ──▶ M365 Copilot (Federated MCP Call) ──▶ HiTechCloud Core Gateway ──▶ Live Data
  ```
- **Lợi ích**:
  - Dữ liệu luôn tươi mới 100% thời gian thực (Trạng thái WAF, CPU VM, số dư ví, sự cố mạng).
  - Không cần đồng bộ dữ liệu nhạy cảm ra khỏi hệ thống nguồn HiTechCloud.
  - Phù hợp cho 803 công cụ quản lý hạ tầng và tra cứu kỹ thuật của HiTechCloud.

---

## 5. Microsoft 365 Copilot Agents & MCP Apps (Interactive UI Widgets)

### 5.1. Các Agents Chuyên Biệt trong M365 Copilot
HiTechCloud xuất bản bộ **Declarative Agents & Custom Engine Agents (CEA)** sẵn sàng cài đặt trên M365:
- `HiTechCloud Cloud Agent`: Điều khiển VM, GPU, kiểm tra tài nguyên.
- `HiTechCloud Security Agent`: Quản lý WAF, quét lỗ hổng, phát hiện IP Blacklist.
- `HiTechCloud Billing Agent`: Tra cứu hóa đơn, nạp tiền ví, gia hạn dịch vụ tự động.
- `HiTechCloud Support Agent`: Tiếp nhận và giải quyết ticket hỗ trợ khách hàng.
- `HiTechCloud DevOps Agent`: Scale Kubernetes, reload Nginx, kiểm tra DNS/SSL.

### 5.2. MCP Apps: Hiển thị Widget Tương tác Sống động (A2UI / AG-UI trong Copilot)
Thay vì chỉ trả về text thô, HiTechCloud MCP Server trả về **Interactive UI Widgets** kết hợp React/Fluent UI hoặc Adaptive Cards:

```
┌──────────────────────────────────────────────────────────────────┐
│              HiTechCloud Cloud Infrastructure Monitor            │
├──────────────────────────────────────────────────────────────────┤
│ Instance: srv-production-master-01                               │
│ Status  : ● RUNNING (Uptime: 45 days)                            │
│                                                                  │
│ CPU: [████████░░░░░░] 54%    RAM: [████████████░░] 78%           │
│ Disk: 120GB / 500GB (SSD)    Network IO: 420 Mbps                │
│                                                                  │
│ [ Restart Service ]    [ View Live Metrics ]    [ Open VM Panel ]│
└──────────────────────────────────────────────────────────────────┘
```

---

## 6. Cấu trúc Cây Thư mục Ecosystem Adapter (`providers/microsoft`)

Trong mã nguồn của nền tảng, Microsoft được thiết kế như một hệ sinh thái ngang hàng:

```
providers/
├── anthropic/             # Claude 3.7 Sonnet / Haiku / Opus
├── openai/                # GPT-4o / o1 / o3-mini / Codex
├── nube/                  # Nube.sh External Inference Option
├── google/                # Gemini 2.0 Flash / Pro
└── microsoft/             # FIRST-CLASS MICROSOFT 365 ECOSYSTEM
    ├── graph/             # Microsoft Graph CRUD APIs (Mail, Teams, SharePoint, OneDrive, Excel)
    ├── copilot/           # M365 Copilot Reasoning APIs & Declarative Agent Runtimes
    ├── teams/             # Teams Bot Adapter, Adaptive Cards & Message Extensions
    ├── outlook/           # Outlook Add-in & Actionable Messages
    ├── sharepoint/        # SharePoint Synced Connectors & Index Ingestion
    ├── onedrive/          # OneDrive Large File Chunked Streaming
    └── connectors/        # Remote Federated MCP Connectors for M365 Connectors Gallery
```

---

## 7. Lợi thế Thương mại & Kênh Phân phối (Partner Strategy)

Tận dụng tư cách **Microsoft AI Cloud Partner Program & CSP Indirect Reseller** của HiTechCloud:
1. **Tenant Sideloading & Organizational Catalog**: Doanh nghiệp khách hàng của HiTechCloud có thể cài đặt 1-click bộ Agent và MCP Server vào tenant Microsoft 365 của họ.
2. **Microsoft Commercial Marketplace**: Đăng tải chính thức lên Azure Marketplace và M365 AppSource dưới dạng Verified Enterprise SaaS.
3. **Microsoft Connectors Gallery Review**: Đệ trình Remote MCP Server của HiTechCloud để Microsoft đánh giá và đưa vào danh mục Connectors Gallery toàn cầu của M365 Copilot.
