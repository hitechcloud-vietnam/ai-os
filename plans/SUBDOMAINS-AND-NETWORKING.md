# Đặc tả Kỹ thuật: Hệ thống Tên miền Phụ (Subdomains) & Cấu hình Mạng (Networking & Ingress)

Tài liệu này đặc tả quy hoạch mạng, chứng chỉ SSL/TLS, định tuyến Nginx Ingress của **11 Subdomains MỚI của HiTechCloud Agent Platform** và danh mục **4 Hệ thống Hiện hữu Bên ngoài** mà Agent Platform tích hợp.

---

## 1. Danh mục 11 Subdomains MỚI Cần Triển khai Cho Agent Platform

Các subdomains này thuộc phạm vi quản lý và triển khai trực tiếp của dự án **HiTechCloud Agent Platform**:

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                 11 SUBDOMAINS MỚI CỦA HITECHCLOUD AGENT PLATFORM (CẦN TRIỂN KHAI)               │
├────┬─────────────────────────────┬──────────┬──────────────┬────────────────────────────────────┤
│ STT│ Subdomain                   │ Giao thức│ Port Mặc định│ Vai trò Kỹ thuật                   │
├────┼─────────────────────────────┼──────────┼──────────────┼────────────────────────────────────┤
│ 1  │ `api-mcp.hitechcloud.vn`    │ HTTPS    │ 443 ──▶ 8080 │ REST & JSON-RPC 2.0 Gateway Ingress│
│ 2  │ `gw.hitechcloud.vn`         │ WSS/HTTPS│ 443 ──▶ 8080 │ SSE & WebSocket Real-time Stream   │
│ 3  │ `mcp.hitechcloud.vn`        │ HTTPS    │ 443 ──▶ Nginx│ Developer Web Dashboard (Vite SPA) │
│ 4  │ `get-mcp.hitechcloud.vn`    │ HTTPS    │ 443 ──▶ S3   │ Phân phối CLI & VS Code Extension  │
│ 5  │ `registry-mcp.hitechcloud.vn`   │ HTTPS/OCI│ 443 ──▶ 8081 │ Universal Package & Artifact Repo  │
│ 6  │ `marketplace-mcp.hitechcloud.vn`│ HTTPS    │ 443 ──▶ Nginx│ AI Marketplace & MCP Apps Portal   │
│ 7  │ `a2a.hitechcloud.vn`        │ HTTPS/mTLS│ 443 ──▶ 8082 │ A2A Protocol & Agent Cards Registry│
│ 8  │ `copilot.hitechcloud.vn`    │ HTTPS    │ 443 ──▶ 8080 │ Microsoft 365 Federated MCP Ingress│
│ 9  │ `auth-mcp.hitechcloud.vn`       │ HTTPS    │ 443 ──▶ 8083 │ Identity Federation, OIDC & SSO    │
│ 10 │ `telemetry-mcp.hitechcloud.vn.`  │ gRPC/OTLP│ 443 ──▶ 4317 │ OpenTelemetry Collector & Traces   │
│ 11 │ `sandbox.hitechcloud.vn`    │ HTTPS/gRPC│ 443 ──▶ 8084 │ Zero-Trust WASM Sandbox Runners    │
└────┴─────────────────────────────┴──────────┴──────────────┴────────────────────────────────────┘
```

---

## 2. Danh mục 4 Hệ thống Hiện hữu Bên ngoài (External Existing Systems)

> **Lưu ý:** 4 tên miền sau là các **hệ thống có sẵn bên ngoài** của HiTechCloud, **không thuộc phạm vi triển khai mới** của Agent Platform. Agent Platform chỉ đóng vai trò là client kết nối hoặc nạp schema:

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                    4 HỆ THỐNG HIỆN HỮU BÊN NGOÀI (ĐÃ CÓ SẴN - AGENT PLATFORM CHỈ KẾT NỐI)       │
├────┬─────────────────────────────┬──────────────────────────┬───────────────────────────────────┤
│ STT│ Subdomain Hiện Hữu          │ Loại Dịch Vụ             │ Cách Agent Platform Sử Dụng       │
├────┼─────────────────────────────┼──────────────────────────┼───────────────────────────────────┤
│ 1  │ `docs.hitechcloud.vn`       │ Tài liệu OpenAPI Cổng My │ Ingest 348 Endpoints Schema JSON  │
│ 2  │ `doc-api-tools.hitechcloud.vn`│ Tài liệu OpenAPI Tools │ Ingest 455 Endpoints Schema JSON  │
│ 3  │ `api.hitechcloud.vn`        │ Backend API Cổng My      │ Tool Caller gọi tới khi thực thi  │
│ 4  │ `api-tools.hitechcloud.vn`  │ Backend API Trang Tools  │ Tool Caller gọi tới khi thực thi  │
└────┴─────────────────────────────┴──────────────────────────┴───────────────────────────────────┘
```

---

## 3. Chi tiết Vai trò 11 Subdomains của Agent Platform

### 3.1. Phân hệ Core Gateway & Trực tiếp Lập trình viên
1. **`api-mcp.hitechcloud.vn`**:
   - Cổng vào API chính cho các giao thức REST (OpenAPI 3.1) và JSON-RPC 2.0.
   - Nhận diện API Key `X-API-Key` hoặc `Authorization: Bearer`.
   - Điều hướng tới Rust AI Gateway (Crate `hitechcloud-ai-gateway`).
2. **`gw.hitechcloud.vn`**:
   - Cổng truyền phát sự kiện thời gian thực qua **Server-Sent Events (SSE)** và **WebSockets**.
   - Phục vụ luồng streaming **AG-UI**, truyền tải text delta, log tiến trình tool execution và cơ chế dừng/phê duyệt **Human-in-the-Loop (HITL)**.
3. **`mcp.hitechcloud.vn`**:
   - Giao diện Web Dashboard dành cho nhà phát triển (Thiết kế phong cách MCPHub, 100% trong dashboard, không AI gradient).
   - Tích hợp 11 module kỹ thuật: Server Manager, Marketplace, Skills/Prompts, Sandboxes, Tool Inspector, Activity Logs, RBAC.

### 3.2. Phân hệ Phân phối & Artifacts
4. **`get-mcp.hitechcloud.vn`**:
   - Cung cấp script cài đặt tự động 1 dòng:
     - Linux/macOS: `curl -sSL https://get-mcp.hitechcloud.vn/install.sh | bash`
     - Windows PowerShell: `iwr -useb https://get-mcp.hitechcloud.vn/install.ps1 | iex`
   - Phân phối file tải trực tiếp tiện ích mở rộng VS Code (`.vsix`) và nhị phân Rust CLI (`hitechcloud`).
5. **`registry-mcp.hitechcloud.vn`**:
   - Kho lưu trữ gói tài nguyên tập trung chuẩn OCI / MinIO S3 cho các bundle `.hcp`.
   - Cung cấp API kiểm tra chữ ký số Ed25519, SBOM (SPDX/CycloneDX), tra cứu phụ thuộc SemVer và tải artifact.
6. **`marketplace-mcp.hitechcloud.vn`**:
   - Cổng giao dịch và cài đặt 1-click các bộ công cụ, MCP Apps, Skills, Plugins và Agent Templates cho doanh nghiệp.

### 3.3. Phân hệ Liên tác Agent & Doanh nghiệp
7. **`a2a.hitechcloud.vn`**:
   - Gateway điều phối giao thức **A2A Protocol v1.0** (Linux Foundation).
   - Lưu trữ và phân phối **Agent Cards Catalog** (`/.well-known/agent-card.json`), hỗ trợ AI-to-AI Capability Negotiation và mTLS giữa các cụm Agent.
8. **`copilot.hitechcloud.vn`**:
   - Endpoint chuyên biệt phục vụ chuẩn **Federated MCP Connector** cho **Microsoft 365 Copilot**.
   - Microsoft Copilot gửi request trực tiếp qua HTTPS tới subdomain này để đọc live data thời gian thực từ hệ thống HiTechCloud.
9. **`auth-mcp.hitechcloud.vn`**:
   - Cổng Identity Federation, hỗ trợ Microsoft Entra ID (Azure AD), Google Workspace SSO, BetterAuth, OAuth 2.0 / OIDC Authorization Server, và SCIM Provisioning.
10. **`telemetry-mcp.hitechcloud.vn.`**:
    - Cổng thu thập phân tán **OpenTelemetry (OTLP)** qua giao thức gRPC (port 4317) và HTTP (port 4318).
    - Tập trung Distributed Tracing, Spans, Metrics Prometheus và Token Accounting.
11. **`sandbox.hitechcloud.vn`**:
    - Cổng giao tiếp với cụm máy chủ Worker chạy môi trường cô lập **Zero-Trust WASM (Wasmtime)** và Linux MicroVMs (gVisor/Firecracker) để thực thi an toàn các đoạn mã từ bên thứ ba.

---

## 4. Cấu hình Nginx Ingress & SSL/TLS Certificates

Toàn bộ 11 subdomains mới được quản lý tập trung qua Nginx Reverse Proxy với chứng chỉ SSL Wildcard `*.hitechcloud.vn` (Let's Encrypt / DigiCert) và cấu hình bảo mật TLS 1.3:

```nginx
# Cấu hình Mẫu: Nginx Upstream & Wildcard Ingress
ssl_certificate /etc/letsencrypt/live/hitechcloud.vn/fullchain.pem;
ssl_certificate_key /etc/letsencrypt/live/hitechcloud.vn/privkey.pem;
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers HIGH:!aNULL:!MD5;

# 1. api-mcp.hitechcloud.vn (REST & JSON-RPC Gateway)
server {
    listen 443 ssl http2;
    server_name api-mcp.hitechcloud.vn;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
    }
}

# 2. gw.hitechcloud.vn (SSE & WebSocket Transport)
server {
    listen 443 ssl http2;
    server_name gw.hitechcloud.vn;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 86400s;
    }
}

# 3. mcp.hitechcloud.vn (Developer Dashboard)
server {
    listen 443 ssl http2;
    server_name mcp.hitechcloud.vn;
    root /var/www/hitechcloud-dashboard/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}

# 4. copilot.hitechcloud.vn (M365 Federated MCP Ingress)
server {
    listen 443 ssl http2;
    server_name copilot.hitechcloud.vn;

    location / {
        proxy_pass http://127.0.0.1:8080/mcp/v1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

        proxy_set_header Host $host;
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 86400s;
    }
}

# 3. mcp.hitechcloud.vn (Developer Dashboard)
server {
    listen 443 ssl http2;
    server_name mcp.hitechcloud.vn;
    root /var/www/hitechcloud-dashboard/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}

# 4. copilot.hitechcloud.vn (M365 Federated MCP Ingress)
server {
    listen 443 ssl http2;
    server_name copilot.hitechcloud.vn;

    location / {
        proxy_pass http://127.0.0.1:8080/mcp/v1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```
