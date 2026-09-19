# Đặc tả Kỹ thuật: OpenAPI & Kho Tools/Skills/Plugins Tự động

Tài liệu này đặc tả:
1. **OpenAPI 3.1 của HiTechCloud Agent Platform**: API tiêu chuẩn cho Gateway, Registry, A2A, AG-UI và MCP Hub.
2. **Cơ chế Nạp Tự động từ 2 tệp OpenAPI JSON Bên Ngoài** để tự động sinh kho **Skills, MCP Servers và Plugins**:
   - `https://docs.hitechcloud.vn/endpoint/login?env=production` (API của Cổng Dịch vụ Khách hàng `my.hitechcloud.vn`)
   - `https://doc-api-tools.hitechcloud.vn/` (API của Trang Công cụ Kỹ thuật `tools.hitechcloud.vn`)

---

## 1. OpenAPI 3.1 Spec của Nền tảng (Platform Ingress)

```yaml
openapi: 3.1.0
info:
  title: HiTechCloud Agent Platform & AI Connect API
  version: "1.0.0"
  description: >
    Enterprise AI Operating System API: Unified Gateway for MCP, A2A, AG-UI, Skills,
    Plugins, and Model Providers (Anthropic, OpenAI, Nube.sh).
servers:
  - url: https://api-mcp.hitechcloud.vn/v1
    description: Production REST & JSON-RPC Gateway
security:
  - bearerAuth: []
  - apiKeyAuth: []
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
    apiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key
  schemas:
    HcpPackage:
      type: object
      properties:
        id: { type: string }
        kind: { type: string, enum: [Skill, Plugin, Agent, McpServer] }
        name: { type: string }
        version: { type: string }
        displayName: { type: string }
        description: { type: string }
        isVerified: { type: boolean }
        runtime:
          type: object
          properties:
            type: { type: string, enum: [binary, node, python, docker, wasm, remote_http] }
            entrypoint: { type: string }
    A2ATask:
      type: object
      properties:
        taskId: { type: string }
        sourceAgentId: { type: string }
        targetAgentId: { type: string }
        action: { type: string }
        input: { type: object }
        status: { type: string, enum: [Created, Assigned, In_Progress, Completed, Failed, Requires_Input] }
paths:
  /registry/packages:
    get:
      summary: Liệt kê hoặc tìm kiếm Packages trong Registry
      parameters:
        - in: query
          name: q
          schema: { type: string }
        - in: query
          name: kind
          schema: { type: string, enum: [Skill, Plugin, Agent, McpServer] }
      responses:
        '200':
          description: Danh sách packages phù hợp
    post:
      summary: Phát hành một package mới (.hcp bundle hoặc manifest)
      responses:
        '201': { description: Package Published }
  /a2a/tasks:
    post:
      summary: Điều phối task giữa Agent với Agent (A2A Protocol v1.0)
      requestBody:
        content:
          application/json:
            schema: { $ref: '#/components/schemas/A2ATask' }
      responses:
        '202': { description: Task Accepted }
  /models/pricing:
    get:
      summary: Lấy bảng giá tổng hợp thời gian thực từ các Provider (Bao gồm Nube.sh dynamic pricing)
      responses:
        '200': { description: Current Effective Pricing }
```

---

## 2. Quy trình Tích hợp OpenAPI Kho Tools làm MCP/Skill

Hệ thống cung cấp pipeline tự động nạp (Ingestion Worker) biến tài liệu OpenAPI thành các công cụ MCP:
- Xem chi tiết tại [HITECHCLOUD-OPENAPI-TOOL-CATALOG.md](HITECHCLOUD-OPENAPI-TOOL-CATALOG.md).
- SDK Client (TypeScript, Python, Rust) được sinh tự động bằng `openapi-generator` để đảm bảo đồng bộ hoàn toàn giữa Gateway, CLI (`hitechcloud`) và Dashboard.

