# Kế hoạch Tích hợp 19 Reference Repositories

Tài liệu này phân tích chi tiết 19 kho lưu trữ mã nguồn (repositories) tham chiếu, các tính năng cốt lõi được trích xuất từ mỗi repository, và cách tích hợp chúng vào kiến trúc thống nhất của **HiTechCloud Agent Platform**.

---

## 1. Danh sách 19 Repositories Tham chiếu

| STT | Tên Repository | Nguồn | Vai trò trong Hệ thống |
|:---:|---|---|---|
| 1 | `mcphub` | [samanhappy/mcphub](https://github.com/samanhappy/mcphub) | Giao diện Dashboard quản trị, quản lý Server, User, Group, Market |
| 2 | `registry` | [modelcontextprotocol/registry](https://github.com/modelcontextprotocol/registry) | Chuẩn đặc tả Registry, xác thực schema package MCP chính thức |
| 3 | `crush` | [charmbracelet/crush](https://github.com/charmbracelet/crush) | Terminal UI agent, công nghệ xem diff và tương tác Git |
| 4 | `Roo-Code` | [RooCodeInc/Roo-Code](https://github.com/RooCodeInc/Roo-Code) | Kiến trúc Agent tự trị trên VS Code, điều phối đa chế độ (modes) |
| 5 | `goose` | [aaif-goose/goose](https://github.com/aaif-goose/goose) | Bộ công cụ Agent mở rộng cho developer, cơ chế session và extension |
| 6 | `opencode` | [anomalyco/opencode](https://github.com/anomalyco/opencode) | Trợ lý code terminal đa nhà cung cấp model, streaming diffs |
| 7 | `aider` | [Aider-AI/aider](https://github.com/Aider-AI/aider) | Lập bản đồ repo (Repo Map AST), commit Git nguyên tử nhiều file |
| 8 | `leaked-claude-code` | [etherbeing/leaked-claude-code](https://github.com/etherbeing/leaked-claude-code) | Kiến trúc vòng lặp Agent nội bộ Claude Code, chuỗi gọi công cụ |
| 9 | `simcode-cli` | [paud/simcode-cli](https://github.com/paud/simcode-cli) | Mô phỏng thực thi công cụ và kiểm thử an toàn trước khi apply |
| 10 | `claude-code-analysis` | [ComeOnOliver/claude-code-analysis](https://github.com/ComeOnOliver/claude-code-analysis) | Phân tích chi tiết giao thức, tối ưu hóa token và cơ chế subagent |
| 11 | `open-claude-code` | [ruvnet/open-claude-code](https://github.com/ruvnet/open-claude-code) | Bản cài đặt mã nguồn mở của Claude Code CLI, cấu trúc lệnh CLI |
| 12 | `claude-code` (benchmark) | [T-Lab-CUHKSZ/claude-code](https://github.com/T-Lab-CUHKSZ/claude-code) | Bộ đánh giá năng lực agent, heuristics lựa chọn tool và xử lý lỗi |
| 13 | `claude-code-reverse-engineering` | [AnimatorPan/claude-code-reverse-engineering](https://github.com/AnimatorPan/claude-code-reverse-engineering) | Cấu trúc bộ nhớ nhiều tầng (User/Session/Repo) và nén ngữ cảnh |
| 14 | `skills` | [anthropics/skills](https://github.com/anthropics/skills) | Bộ kỹ năng chuẩn của Anthropic (`SKILL.md`), định dạng prompt |
| 15 | `skillhub` | [iflytek/skillhub](https://github.com/iflytek/skillhub) | Thị trường kỹ năng cộng đồng và phân phối động (dynamic distribution) |
| 16 | `claude-code-plugin-template` | [ivan-magda/claude-code-plugin-template](https://github.com/ivan-magda/claude-code-plugin-template) | Mẫu plugin Claude Code, manifest v1.0, vòng đời hooks |
| 17 | `template` (ai-plugin-market) | [ai-plugin-marketplace/template](https://github.com/ai-plugin-marketplace/template) | Cấu trúc phân quyền Plugin, ủy quyền OAuth và định tuyến tool |
| 18 | `claude-marketplace-template` | [stbenjam/claude-marketplace-template](https://github.com/stbenjam/claude-marketplace-template) | Mẫu đóng gói và quy trình kiểm thử tự động cho Marketplace |
| 19 | `marketplace` (claude-market) | [claude-market/marketplace](https://github.com/claude-market/marketplace) | Kho catalog plugin doanh nghiệp, kiểm tra chữ ký và sandbox |

---

## 2. Phân tích & Trích xuất Tính năng từ Từng Nhóm Repository

### Nhóm 1: Dashboard, Quản trị & Giao diện Người dùng
- **Repository**: `mcphub`
- **Tính năng kế thừa**:
  1. Cấu trúc Dashboard SPA sạch sẽ, phân tách rõ ràng giữa các phân hệ kỹ thuật: Servers, Marketplace, Prompts, Resources, Groups, Users, Settings.
  2. Cơ chế xác thực và quản lý quyền người dùng (BetterAuth + Role RBAC).
  3. Trình giám sát tiến trình MCP Server (kiểm tra trạng thái `stdio`, `sse`, kiểm tra stderr/stdout).
  4. Trình kiểm thử JSON-RPC 2.0 trực tiếp (`tools/call`, `tools/list`).
- **Ứng dụng vào HiTechCloud**:
  - Tái hiện toàn bộ giao diện theo chuẩn MCPHub (không landing page, không màu gradient AI).
  - Tích hợp thêm các tab mới: Client Config Exporter, VS Code & CLI Manager, Sandbox Policy Matrix.

### Nhóm 2: MCP Registry & Marketplace Chuẩn Quốc tế
- **Repositories**: `registry` (ModelContextProtocol), `marketplace` (claude-market), `ai-plugin-marketplace/template`, `claude-marketplace-template`
- **Tính năng kế thừa**:
  1. Chuẩn JSON Schema cho metadata của MCP Server và Plugin (`mcp-manifest.json`, `plugin-manifest.json`).
  2. Xác thực tính hợp lệ của package trước khi publish: kiểm tra cấu hình transport (`stdio`, `sse`, `stream`), biến môi trường bắt buộc, danh sách tool cung cấp.
  3. Đóng gói plugin tích hợp bao gồm: MCP Server + Agent Skills + Pre/Post Execution Hooks.
  4. Cơ chế ký số ED25519 để xác thực nguồn gốc tác giả và chống giả mạo package.
- **Ứng dụng vào HiTechCloud**:
  - Nằm trong crate `registry` và `plugin-service` của Gateway.
  - Hiển thị trên tab Marketplace của Dashboard với chức năng 1-Click Install.

### Nhóm 3: Terminal UI, CLI Coding Agents & Git Context
- **Repositories**: `crush` (Charm), `goose` (Block), `opencode` (Anomaly), `aider` (Aider-AI), `simcode-cli`
- **Tính năng kế thừa**:
  1. **Aider**: Giải thuật lập bản đồ kho mã nguồn (Repo Map) sử dụng cây cú pháp Tree-sitter AST để nạp ngữ cảnh tối ưu vào context window; cơ chế tạo commit git nguyên tử (atomic commits) tự động.
  2. **Crush & OpenCode**: Giao diện dòng lệnh đẹp mắt, khả năng streaming diff (xem trước thay đổi từng dòng màu xanh/đỏ trong terminal), hỗ trợ đa nhà cung cấp model (Anthropic, OpenAI, Local LLMs).
  3. **Goose**: Quản lý phiên làm việc bền vững (session persistence), cấu hình extension theo chuẩn YAML/JSON, tự động khôi phục ngữ cảnh làm việc sau khi ngắt kết nối.
  4. **SimCode CLI**: Cơ chế mô phỏng thực thi (Dry-Run simulation) cho phép agent chạy thử nghiệm code trong môi trường cô lập trước khi ghi đè vào file thật của dự án.
- **Ứng dụng vào HiTechCloud**:
  - Tích hợp vào **HiTechCloud CLI** (`crates/hitechcloud`) và mô-đun **Sandbox Runner** (`crates/sandbox-runner`).
  - Hỗ trợ lệnh `hitechcloud client export` cho cả 5 công cụ trên.

### Nhóm 4: Claude Code Architecture, Reverse Engineering & Protocol Insights
- **Repositories**: `leaked-claude-code`, `claude-code-analysis`, `open-claude-code`, `claude-code` (T-Lab), `claude-code-reverse-engineering`, `claude-code-plugin-template`
- **Tính năng kế thừa**:
  1. **Cấu trúc Vòng lặp Agent (Agent Execution Loop)**: Khởi tạo ngữ cảnh -> Khám phá công cụ (Tool Discovery) -> Lập kế hoạch (Planning via Todo List) -> Thực thi công cụ theo chuỗi (Sequential Tool Invocations) -> Đánh giá phản hồi -> Hoàn tất task.
  2. **Hệ thống Quản lý Bộ nhớ 3 tầng (Memory Scopes Hierarchy)**:
     - `User Memory` (`/memories/`): Ghi nhớ dài hạn xuyên suốt các dự án.
     - `Session Memory` (`/memories/session/`): Ghi nhớ tạm thời trong phiên làm việc hiện tại.
     - `Repository Memory` (`/memories/repo/`): Quy ước mã nguồn, lệnh build/test riêng của repo.
  3. **Cơ chế Nén Ngữ cảnh (Context Compaction)**: Tóm tắt lịch sử hội thoại tự động khi token tiến gần tới giới hạn cửa sổ ngữ cảnh, bảo toàn toàn vẹn các trạng thái then chốt.
  4. **Subagent Delegation**: Cho phép agent chính khởi tạo các subagent chuyên biệt (như `Explore` agent để đọc/tìm kiếm file mà không làm tràn context chính).
- **Ứng dụng vào HiTechCloud**:
  - Triển khai trong adapter `adapter-anthropic` và `ai-gateway`.
  - Hỗ trợ tạo cấu hình tương thích 100% với Claude Code CLI.

### Nhóm 5: Agent Skills & Prompt Standards
- **Repositories**: `skills` (Anthropic), `skillhub` (iFlyTek)
- **Tính năng kế thừa**:
  1. Định dạng chuẩn `SKILL.md` với YAML frontmatter (`name`, `description`, `scope`, `triggers`, `requires_mcp`) kết hợp nội dung hướng dẫn (instructions) chi tiết.
  2. Cơ chế gắn thẻ (tags), phân loại theo phạm vi (`global`, `agent`, `session`, `project`).
  3. Quản lý tham số hóa linh hoạt (parametric inputs) và kiểm thử đánh giá benchmark.
- **Ứng dụng vào HiTechCloud**:
  - Triển khai trong crate `skills-service` và tab **Skills & Prompts** trên Dashboard.
  - Cho phép xuất bản và đồng bộ skill giữa các thành viên trong tổ chức.

---

## 3. Ma trận Tích hợp Công nghệ vào HiTechCloud Platform

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           HITECHCLOUD AGENT PLATFORM                        │
├──────────────────────────────────────┬──────────────────────────────────────┤
│ FRONTEND & EXTENSIONS                │ CLI & TERMINAL RUNTIME               │
│ • mcphub Dashboard (Zero AI color)   │ • Rust hitechcloud binary            │
│ • VS Code Dedicated Extension (.vsix)│ • Crush/Goose/Aider/OpenCode engines │
│ • Client Exporter (9 clients)        │ • Repo Map AST & Git Context         │
├──────────────────────────────────────┴──────────────────────────────────────┤
│ CORE RUST WORKSPACE (10 CRATES)                                              │
│ • ai-gateway: Cổng vào bảo mật, RBAC, Rate Limiter Token-Bucket              │
│ • mcp-gateway: Proxy đa giao thức (stdio/sse/stream), gộp nhiều MCP server   │
│ • skills-service: Quản lý SKILL.md (Anthropic) & SkillHub marketplace        │
│ • plugin-service: Đóng gói bundle, manifest v1.0, lifecycle hooks           │
│ • sandbox-runner: Cô lập tiến trình, kiểm soát quyền mạng/file/subprocess   │
│ • adapter-anthropic & adapter-openai: Chuyển đổi định dạng công cụ 2 chiều  │
│ • signing: Ký số gói phần mềm ED25519 & xác thực tính toàn vẹn              │
│ • registry: Quản lý catalog theo chuẩn modelcontextprotocol/registry        │
├─────────────────────────────────────────────────────────────────────────────┤
│ INFRASTRUCTURE LAYER                                                        │
│ • PostgreSQL 16 (Dữ liệu cấu hình, tài khoản, audit logs)                  │
│ • Redis 7 (Cache token, Rate Limiter, Pub/Sub events)                       │
│ • Nginx Ingress + Let's Encrypt TLS (api-mcp, gw, get-mcp, mcp)             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Quy trình Quản lý Mã nguồn khi Clone về Workspace

Để tránh xung đột nested Git repositories trong workspace của dự án:
1. Clone mã nguồn của 19 repo vào thư mục đệm `/home/HiTechCloud_Agent_Platform/integrations_src/`.
2. Chạy lệnh xóa toàn bộ thư mục `.git` đệ quy:
   ```bash
   find /home/HiTechCloud_Agent_Platform/integrations_src/ -name ".git" -type d -prune -exec rm -rf {} +
   ```
3. Giữ lại toàn bộ mã nguồn, cấu hình, template và tài liệu để tham chiếu, tái cấu trúc hoặc tái sử dụng trực tiếp trong các module của nền tảng.
