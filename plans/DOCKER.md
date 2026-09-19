# Docker

## Dockerfile mẫu (Rust service, đa giai đoạn)

```dockerfile
# --- Build stage ---
FROM rust:1.82-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --bin hitechcloud-mcp-gateway

# --- Runtime stage ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/hitechcloud-mcp-gateway /usr/local/bin/app
USER 1000:1000
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/app"]
```

## docker-compose cho môi trường dev

```yaml
version: "3.9"
services:
  gateway:
    build: ./crates/mcp-gateway
    ports: ["8080:8080"]
    environment:
      - DATABASE_URL=postgres://hitechcloud:hitechcloud@postgres:5432/hitechcloud
      - REDIS_URL=redis://redis:6379
    depends_on: [postgres, redis]

  registry:
    build: ./crates/registry
    ports: ["8081:8081"]
    environment:
      - DATABASE_URL=postgres://hitechcloud:hitechcloud@postgres:5432/hitechcloud
    depends_on: [postgres]

  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: hitechcloud
      POSTGRES_PASSWORD: hitechcloud
      POSTGRES_DB: hitechcloud
    volumes: ["pgdata:/var/lib/postgresql/data"]

  redis:
    image: redis:7-alpine

volumes:
  pgdata:
```

## Nguyên tắc image

- Base image tối thiểu (distroless hoặc `debian-slim`), không cài compiler vào runtime image.
- Chạy container với **non-root user**.
- Không bake secret vào image — luôn inject qua biến môi trường/secret manager lúc runtime.
- Scan image bằng `trivy` hoặc tương đương trong CI trước khi push lên registry container.
