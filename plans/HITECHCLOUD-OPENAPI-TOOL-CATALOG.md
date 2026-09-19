# Đặc tả Kỹ thuật: Tự động Chuyển đổi 2 Bộ API HiTechCloud (803 Endpoints) thành MCP Servers, Skills & Plugins

Tài liệu này đặc tả cơ chế nạp (Ingestion Engine) và chuyển đổi tự động **803 endpoints** từ **2 tệp API Specification chính thức của HiTechCloud** thành kho công cụ **MCP Servers**, **Agent Skills** và **Plugins** trên **HiTechCloud Agent Platform**:

1. **Bộ 1: HitechCloud User API — Chính thức (`my.hitechcloud.vn`)**
   - Tệp nguồn: `plans/hitechcloud-user-api-production-2026-09-19.postman_collection.json`
   - Quy mô: **348 endpoints** (37 nhóm module điều khiển tài khoản, dịch vụ cloud, máy chủ ảo, tên miền, DNS, SSL, billing, AI factory, Proxmox, Ceph S3, VNeID eKYC).
   - Base URL: `https://api.hitechcloud.vn`
   - Xác thực: `Authorization: Bearer {{token}}` (lấy từ `POST /api/login`).

2. **Bộ 2: HitechCloud API Tools (`tools.hitechcloud.vn`)**
   - Tệp nguồn: `plans/hitechcloud-api-tools-2026-09-19.postman_collection.json`
   - Quy mô: **455 endpoints** (15 danh mục tra cứu dữ liệu doanh nghiệp & thuế, tên miền & SSL, mạng & hạ tầng, mã hoá, tài chính & tỷ giá, dữ liệu 63 tỉnh thành Việt Nam, logistics XNK, pháp lý).
   - Base URL: `https://api-tools.hitechcloud.vn`
   - Xác thực: Header `X-API-Key: {{apiKey}}`.

---

## 1. Sơ đồ Kiến trúc Ingestion Pipeline (803 Endpoints)

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                      2 TỆP SPECIFICATION CHÍNH THỨC CỦA HITECHCLOUD (803 ENDPOINTS)              │
├────────────────────────────────────────────────┬────────────────────────────────────────────────┤
│  1. User API (`my.hitechcloud.vn`) - 348 EPs   │  2. API Tools (`tools.hitechcloud.vn`) - 455 EPs│
│  • Base URL: https://api.hitechcloud.vn        │  • Base URL: https://api-tools.hitechcloud.vn  │
│  • Auth: Bearer {{token}}                      │  • Auth: Header X-API-Key: {{apiKey}}          │
│  • File: hitechcloud-user-api-...json          │  • File: hitechcloud-api-tools-...json         │
└────────────────────────────────────────────────┴────────────────────────────────────────────────┘
                                                 │
                                                 ▼ (Rust Ingestion Worker & Parser Engine)
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                           POSTMAN / OPENAPI TO MCP GENERATOR ENGINE                             │
│  1. Trích xuất: Tên action, HTTP Method, Path, Headers, Query Params, Request Body JSON Schema   │
│  2. Chuẩn hóa Định danh MCP:                                                                    │
│     - User API ──▶ `htc_user.<module>.<action>` (vd: `htc_user.cloud_instance.restart`)         │
│     - Tools API ──▶ `htc_tool.<category>.<action>` (vd: `htc_tool.dns_ssl.whois_lookup`)        │
│  3. Gắn Metadata Bảo mật:                                                                       │
│     - Gắn cờ `requiresApproval: true` (HITL) cho các tác vụ thay đổi hạ tầng / thanh toán       │
│     - Tự động tiêm Token từ Secrets Store (`vault://htc_user_token`, `vault://htc_tools_key`)   │
│  4. Đóng gói Manifest Universal `hcp.json` và xuất bản vào Registry                             │
└────────────────────────────────────────────────┬────────────────────────────────────────────────┘
                                                 │
                                                 ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                    HITECHCLOUD OFFICIAL PACKAGE REGISTRY (803 TOOLS KHẢ DỤNG)                   │
├────────────────────────────────────────────────┬────────────────────────────────────────────────┤
│  MCP Server 1: `hitechcloud-user-portal-mcp`   │  MCP Server 2: `hitechcloud-tools-suite-mcp`   │
│  (348 Tools quản lý dịch vụ cloud & tài khoản) │  (455 Tools tra cứu dữ liệu & tiện ích hạ tầng)│
├────────────────────────────────────────────────┴────────────────────────────────────────────────┤
│  Agent Skills Chuyên Nghiệp:                                                                    │
│  • `cloud-ops-autopilot`       : Tự động quản lý Cloud VM, GPU, Proxmox, Ceph S3                │
│  • `domain-ssl-lifecycle`      : Quản lý vòng đời tên miền, DNSSEC, cấp phát & gia hạn SSL     │
│  • `enterprise-data-inspector` : Tra cứu MST, DKKD, báo cáo tài chính, tỷ giá ngân hàng, VNeID │
│  • `network-security-triage`   : Quét IP Blacklist, Port check, DNS Propagation, BGP ASN        │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Chi tiết Bộ 1: User API — Cổng Khách hàng (`my.hitechcloud.vn`)

Bộ User API bao gồm **348 endpoints** được phân chia thành **37 nhóm module** nghiệp vụ:

| STT | Nhóm Module | Số EPs | Mô tả Chức năng | MCP Tools Tiêu biểu |
|---|---|---|---|---|
| 1 | **Account Authentication** | 6 | Đăng nhập, cấp mới token, hủy phiên, đổi mật khẩu | `htc_user.auth.login`, `htc_user.auth.refresh` |
| 2 | **User Profile** | 3 | Thông tin tài khoản, cập nhật hồ sơ, nhật ký thao tác | `htc_user.profile.details`, `htc_user.profile.logs` |
| 3 | **Billing & Contracts** | 6 | Số dư ví, danh sách hóa đơn, chi tiết hóa đơn, áp credit | `htc_user.billing.balance`, `htc_user.billing.invoices` |
| 4 | **Support & Tickets** | 13 | Quản lý ticket hỗ trợ, trả lời ticket, đính kèm file, tin tức | `htc_user.support.create_ticket`, `htc_user.support.reply` |
| 5 | **Contacts** | 5 | Danh sách liên hệ phụ, phân quyền truy cập dịch vụ | `htc_user.contacts.list`, `htc_user.contacts.add` |
| 6 | **Domains** | 25 | WHOIS, danh sách tên miền, nameservers, EPP code, khóa domain, gia hạn | `htc_user.domains.nameservers`, `htc_user.domains.renew` |
| 7 | **DNS Manage** | 10 | Quản lý bản ghi DNS, DNSSEC keys, nameserver đăng ký | `htc_user.dns_manage.records`, `htc_user.dns_manage.dnssec` |
| 8 | **SSL Certificates** | 6 | Quản lý chứng chỉ SSL, tải file cert X.509, đặt mua chứng chỉ | `htc_user.ssl.download`, `htc_user.ssl.order` |
| 9 | **Services** | 9 | Danh sách dịch vụ đang chạy, hủy dịch vụ, đổi chu kỳ thanh toán | `htc_user.services.list`, `htc_user.services.renew` |
| 10 | **Cart & Ordering** | 6 | Bảng giá sản phẩm, cấu hình addon, đặt hàng dịch vụ mới | `htc_user.cart.order_service`, `htc_user.cart.quote` |
| 11 | **DNS Zones** | 8 | Thêm/xóa zone DNS, sửa đổi bản ghi trong zone | `htc_user.dns.add_zone`, `htc_user.dns.edit_record` |
| 12 | **Notifications** | 3 | Danh sách thông báo hệ thống, đánh dấu đã đọc | `htc_user.notifications.list`, `htc_user.notifications.ack` |
| 13 | **Virtualizor Services** | 4 | Tạm dừng/mở lại VM, rebuild OS template, đổi SSH Key | `htc_user.virtualizor.rebuild`, `htc_user.virtualizor.sshkey` |
| 14 | **Cloud GPU** | 6 | Quản lý cụm máy chủ GPU AI, khởi động, dừng, xem tài nguyên | `htc_user.cloud_gpu.manage`, `htc_user.cloud_gpu.status` |
| 15 | **Cloud Service & Network** | 13 | Quản lý hạ tầng đám mây, mạng ảo VPC, trạng thái dịch vụ | `htc_user.cloud.network_services`, `htc_user.cloud.status` |
| 16 | **Bare Metal & Colocation** | 24 | Quản lý máy chủ vật lý riêng, thuê chỗ đặt server, cổng mạng | `htc_user.baremetal.power`, `htc_user.baremetal.reboot` |
| 17 | **vCloudStack Public Cloud** | 4 | Tương tác cụm Public Cloud vCloudStack | `htc_user.vcloudstack.instances`, `htc_user.vcloudstack.manage` |
| 18 | **Hosting Services** | 6 | Quản lý Web Hosting cPanel/DirectAdmin, dung lượng, băng thông | `htc_user.hosting.details`, `htc_user.hosting.change_pass` |
| 19 | **Cloud Instance & VM** | 34 | Quản lý toàn diện Cloud VM (Bật/Tắt/Khởi động lại/Snapshot/Resize) | `htc_user.cloud_vm.restart`, `htc_user.cloud_vm.snapshot` |
| 20 | **HiTechCloud AI Factory** | 36 | Điều phối tác vụ AI, huấn luyện mô hình, quản lý pipeline AI | `htc_user.ai_factory.pipeline`, `htc_user.ai_factory.train` |
| 21 | **Bảo mật (Passkey, MFA, VNeID)** | 28 | Passkey WebAuthn, Email MFA, định danh VNeID eKYC | `htc_user.security.passkey`, `htc_user.security.vneid_ekyc` |
| 22 | **Backup & Storage (Ceph S3, Proxmox)** | 20 | Backup VM Proxmox, tạo bucket Ceph S3, cấp quyền access key | `htc_user.backup.proxmox`, `htc_user.storage.ceph_s3` |
| 23 | **HiTechCloud PMG, Proxmox & IPAM** | 24 | Proxmox Mail Gateway (PMG), Proxmox Node, Quản lý dải IP (IPAM) | `htc_user.pmg.rules`, `htc_user.ipam.allocate_ip` |
| 24 | **Affiliates & Partner Program** | 33 | Chương trình đối tác đại lý, hoa hồng, chiến dịch affiliate | `htc_user.affiliate.summary`, `htc_user.partner.payouts` |
| 25 | **Tiện ích (WillExpired, URL Shortener)**| 16 | Quản lý dịch vụ sắp hết hạn, hệ thống rút gọn liên kết | `htc_user.tools.will_expired`, `htc_user.tools.shorten_url` |
| **Tổng cộng** | **37 Module** | **348** | **Toàn bộ hệ thống quản lý dịch vụ cloud khách hàng** | |

---

## 3. Chi tiết Bộ 2: API Tools — Trang Công cụ Kỹ thuật (`tools.hitechcloud.vn`)

Bộ API Tools bao gồm **455 endpoints** được phân chia thành **15 danh mục kỹ thuật**:

| STT | Danh mục Công cụ | Số EPs | Mô tả Chức năng Kỹ thuật | MCP Tools Tiêu biểu |
|---|---|---|---|---|
| 1 | **Doanh nghiệp & Thuế** | 64 | Tra cứu MST, Báo cáo điện tử, ĐKKD, Sổ sàn TMĐT, Chỉ số kinh tế quốc gia, Mã DUNS, ISO 20275 | `htc_tool.tax.dkkd_lookup`, `htc_tool.tax.ecommerce_check` |
| 2 | **Mã hoá & Kiểm tra** | 39 | Băm MD5/SHA256/SHA512, HMAC, Base64, UUID v4/v7, JWT decode & verify, CRC32, Bcrypt | `htc_tool.crypto.hash_sha256`, `htc_tool.crypto.jwt_verify` |
| 3 | **Thời gian & Lịch** | 18 | Đổi Unix timestamp, Lịch âm dương Việt Nam, Ngày lễ, Múi giờ quốc tế, Tính số ngày làm việc | `htc_tool.time.lunar_convert`, `htc_tool.time.working_days` |
| 4 | **Công cụ Lập trình** | 25 | Format & validate JSON/XML/YAML, Regex tester, Cron parser, Color hex/rgb, Diff so sánh văn bản | `htc_tool.dev.json_format`, `htc_tool.dev.cron_parse` |
| 5 | **Tên miền & SSL** | 53 | Tra cứu WHOIS, RDAP, Kiểm tra SSL cert, Certificate Transparency log, Quét bản ghi DNS (A/AAAA/CNAME/MX/TXT) | `htc_tool.dns_ssl.whois`, `htc_tool.dns_ssl.ssl_inspect` |
| 6 | **Email & DNS** | 17 | Kiểm tra tồn tại hòm thư (Email deliverability), Test kết nối SMTP, Kiểm tra SPF, DKIM, DMARC, DNSSEC | `htc_tool.email.verify_smtp`, `htc_tool.email.check_dmarc` |
| 7 | **Tài chính & Tỷ giá** | 16 | Tỷ giá Vietcombank, Tỷ giá Ngân hàng Nhà nước, Giá vàng SJC, Lãi suất liên ngân hàng, Chuyển đổi ngoại tệ | `htc_tool.finance.vcb_rate`, `htc_tool.finance.gold_sjc` |
| 8 | **Chuyển đổi & Định dạng** | 45 | Đổi đơn vị đo lường, Markdown to HTML, PDF to Text, Slugify tiếng Việt có dấu, Định dạng tiền tệ | `htc_tool.convert.markdown_html`, `htc_tool.convert.slugify` |
| 9 | **Dược phẩm & Y tế** | 5 | Tra cứu danh mục thuốc cấp phép, tra cứu số đăng ký lưu hành, cơ sở y tế | `htc_tool.health.drug_lookup`, `htc_tool.health.hospital_list` |
| 10 | **Pháp lý & Thủ tục** | 28 | Tra cứu văn bản quy phạm pháp luật, biểu mẫu hành chính, nghị định, thông tư chính phủ | `htc_tool.legal.law_search`, `htc_tool.legal.form_download` |
| 11 | **Xuất nhập khẩu & Logistics** | 15 | Tra cứu Mã HS (Harmonized System), Biểu thuế xuất nhập khẩu, Tra cứu cảng biển, Theo dõi vận đơn | `htc_tool.logistics.hs_code`, `htc_tool.logistics.tariff_calc` |
| 12 | **Mạng & Hạ tầng** | 46 | Ping, Traceroute, Port check, IP Geolocation, BGP ASN lookup, Quét IP Blacklist, Đo tốc độ mạng | `htc_tool.network.port_check`, `htc_tool.network.ip_blacklist` |
| 13 | **QR & Thanh toán** | 5 | Tạo mã VietQR chuẩn NAPAS, Phân tích mã QR EMVCo, Trích xuất thông tin chuyển khoản | `htc_tool.payment.vietqr_gen`, `htc_tool.payment.qr_parse` |
| 14 | **SEO & Web** | 13 | Phân tích Meta tags, Kiểm tra Robots.txt & Sitemap, Đo Core Web Vitals, Trích xuất thẻ OpenGraph | `htc_tool.seo.inspect_meta`, `htc_tool.seo.robots_parser` |
| 15 | **Dữ liệu Việt Nam** | 66 | 63 Tỉnh thành, Quận huyện, Phường xã, Biển số xe toàn quốc, Mã bưu chính Zipcode, Mã ngân hàng CITAD/BIN | `htc_tool.vn_data.provinces`, `htc_tool.vn_data.banks_napas` |
| **Tổng cộng** | **15 Danh mục**| **455** | **Kho công cụ tra cứu dữ liệu và tiện ích kỹ thuật toàn diện** | |

---

## 4. Đặc tả Quy trình Tự động Sinh Tool MCP & Bảo mật (Human-In-The-Loop)

Mọi endpoint được tự động bóc tách thành công cụ JSON-RPC 2.0 chuẩn MCP:

### 4.1. Ví dụ Tool Tra cứu An toàn (Read-Only Tool)
```json
{
  "name": "htc_tool_network_port_check",
  "description": "Scans if a specific TCP port is open on target host. Source: api-tools.hitechcloud.vn",
  "inputSchema": {
    "type": "object",
    "properties": {
      "host": { "type": "string", "description": "Domain or IP address" },
      "port": { "type": "integer", "description": "Port number (1-65535)" }
    },
    "required": ["host", "port"]
  },
  "metadata": {
    "auth": "apiKey",
    "requiresApproval": false,
    "sourceCategory": "Mạng & Hạ tầng"
  }
}
```

### 4.2. Ví dụ Tool Tác động Hạ tầng (High-Impact Tool với HITL Approval)
```json
{
  "name": "htc_user_cloud_vm_restart",
  "description": "Restarts a Cloud Virtual Machine instance. High-impact operational action.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "serviceId": { "type": "string", "description": "The unique ID of the cloud service" },
      "force": { "type": "boolean", "default": false, "description": "Force hard reboot" }
    },
    "required": ["serviceId"]
  },
  "metadata": {
    "auth": "bearerToken",
    "requiresApproval": true,
    "sourceModule": "Cloud Virtual Machine"
  }
}
```

Khi Agent gọi `htc_user_cloud_vm_restart`, Gateway tự động kích hoạt sự kiện `agent.hitl.request` qua giao thức **AG-UI**, yêu cầu lập trình viên xác nhận trên Dashboard trước khi gửi request tới `https://api.hitechcloud.vn`.


---

## 3. Quy trình Chuyển đổi Tự động (OpenAPI-to-MCP Pipeline)

Bộ chuyển đổi tự động trong crate `hitechcloud-registry` thực hiện:

### 3.1. Chuyển đổi OpenAPI Endpoint thành MCP Tool Definition
Mỗi `PathItem` và `Operation` trong tài liệu OpenAPI được phân rã thành một MCP Tool:

```json
{
  "name": "htc_tools_server_restart",
  "description": "Restarts a HiTechCloud compute instance or specific system service. High-impact action.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "instanceId": {
        "type": "string",
        "description": "The unique ID of the cloud instance (e.g. srv-master-01)"
      },
      "serviceName": {
        "type": "string",
        "description": "Optional service name (e.g. nginx, docker). Omit to restart full VM."
      },
      "force": {
        "type": "boolean",
        "default": false,
        "description": "Whether to force immediate kill if graceful shutdown times out."
      }
    },
    "required": ["instanceId"]
  },
  "metadata": {
    "sourceOpenAPI": "https://doc-api-tools.hitechcloud.vn/",
    "httpMethod": "POST",
    "path": "/v1/instances/{instanceId}/restart",
    "requiresApproval": true,
    "category": "Infrastructure"
  }
}
```

### 3.2. Đóng gói thành Universal Package (`hcp.json`)
```json
{
  "$schema": "https://specs.hitechcloud.vn/schemas/hcp-v1.json",
  "hcpVersion": "1.0.0",
  "kind": "McpServer",
  "metadata": {
    "name": "hitechcloud-cloud-tools",
    "namespace": "hitechcloud",
    "version": "1.0.0",
    "displayName": "HiTechCloud Infrastructure & Ops Tools",
    "description": "Official MCP server providing full cloud lifecycle management directly from OpenAPI.",
    "category": "devops",
    "tags": ["cloud", "k8s", "database", "dns", "ssl", "backup"]
  },
  "runtime": {
    "type": "remote_http",
    "entrypoint": "https://doc-api-tools.hitechcloud.vn"
  },
  "security": {
    "permissions": ["network:outbound", "auth:production_token"],
    "signature": {
      "algorithm": "ed25519",
      "publicKey": "HiTechCloudOfficialRootKeyBase64..."
    }
  }
}
```

---

## 4. Tự động Tạo Skills từ Kho Tools OpenAPI

Dựa trên các MCP tools sinh ra từ `doc-api-tools.hitechcloud.vn`, hệ thống tự động sinh các **Skills chuyên nghiệp**:

### Skill 1: `k8s-pod-auto-healing`
- **Mô tả**: Tự động phát hiện Pod bị CrashLoopBackOff, tra cứu log và scale/restart pod.
- **Tools sử dụng**: `htc_tools.k8s_list_pods`, `htc_tools.k8s_get_logs`, `htc_tools.k8s_restart_pod`.

### Skill 2: `database-performance-triage`
- **Mô tả**: Phát hiện slow query, chạy EXPLAIN ANALYZE và đưa ra khuyến nghị đánh index.
- **Tools sử dụng**: `htc_tools.database_list_slow_queries`, `htc_tools.database_explain_query`.

### Skill 3: `ssl-expiry-monitor-and-renew`
- **Mô tả**: Kiểm tra thời hạn chứng chỉ SSL dưới 14 ngày và tự động gửi yêu cầu gia hạn Let's Encrypt.
- **Tools sử dụng**: `htc_tools.ssl_check_expiry`, `htc_tools.ssl_renew`.

---

## 5. Đồng bộ Định kỳ & Quản lý Thay đổi (Spec Drift & Versioning)

- **Cron Ingestion Worker**: Rust Gateway định kỳ kiểm tra `ETag` hoặc băm SHA256 của tài liệu OpenAPI tại `https://doc-api-tools.hitechcloud.vn/` mỗi 6 giờ.
- **Zero-Downtime Hot Reload**: Khi phát hiện endpoint mới được bổ sung trong OpenAPI, Gateway tự động sinh tool schema mới và hot-reload vào MCP catalog mà không cần khởi động lại tiến trình server.
- **Deprecation Notification**: Nếu một API endpoint bị gắn tag `deprecated: true` trong OpenAPI, Gateway gắn cảnh báo vàng lên Dashboard và thông báo lập trình viên qua audit logs.
