# Deployment

## Môi trường

```
dev → staging → production
```

Mỗi môi trường có Registry/Gateway riêng, KHÔNG chia sẻ database, để tránh dữ liệu test lẫn vào production (đặc biệt quan trọng với audit log/billing).

## Thành phần cần deploy

```
hitechcloud-ai-gateway     (stateless, scale ngang)
hitechcloud-mcp-gateway    (stateless, scale ngang; kết nối MCP server qua sandbox runner)
hitechcloud-skills-svc     (stateless)
hitechcloud-plugin-svc     (stateless)
hitechcloud-registry       (stateful — DB + object storage cho package)
hitechcloud-admin-console  (frontend, SPA tĩnh + API riêng)
hitechcloud-developer-portal (frontend, SPA tĩnh)
postgres                   (chính, control plane)
redis                       (cache config + rate limit counter)
object-storage (S3-compatible) (lưu package plugin/skill đã đóng gói)
vault/kms                   (lưu secret của MCP server)
```

## Chiến lược release

- Gateway/Registry release theo **blue-green** hoặc **canary** (route 5% traffic sang bản mới, theo dõi error rate trước khi full rollout).
- Migration DB chạy theo kiểu **backward-compatible trước, dọn dẹp sau** (expand-contract pattern): thêm cột mới không xóa cột cũ ngay, deploy code mới, xác nhận ổn định rồi mới dọn schema cũ.

## Cấu hình theo môi trường

```
.env.production
├── DATABASE_URL
├── REDIS_URL
├── VAULT_ADDR
├── OBJECT_STORAGE_BUCKET
├── SIGNING_KEY_ID (của chính platform, khác key publisher)
└── RATE_LIMIT_DEFAULT
```

## Rollback

- Mọi migration DB phải có script `down` tương ứng hoặc chiến lược expand-contract để không cần rollback destructive.
- Gateway/Registry đóng gói dạng container, rollback = deploy lại image tag trước đó (giữ tối thiểu 5 image gần nhất trong registry container).

Chi tiết container hoá: [DOCKER.md](DOCKER.md). Chi tiết orchestration: [KUBERNETES.md](KUBERNETES.md).
