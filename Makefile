# HiTechCloud AI OS — Makefile
# ═══════════════════════════════════════════════════════════════
# Pure Linux binaries — NO Docker, NO containers.
# Port allocation avoids conflicts with existing services.

.PHONY: all build-rust build-go build start stop restart test health seed clean

RUST_BIN := target/release
GO_BIN   := bin

# ═══════════════════════════════════════════════════════════════
# Build
# ═══════════════════════════════════════════════════════════════

all: build

build-rust:
	@echo "🦀 Building Rust workspace (release)..."
	cargo build --release

build-go:
	@echo "🐹 Building Go services..."
	@mkdir -p $(GO_BIN)
	cd skills-svc && go build -o ../$(GO_BIN)/skills-svc .
	cd plugin-svc && go build -o ../$(GO_BIN)/plugin-svc .
	cd admin-svc  && go build -o ../$(GO_BIN)/admin-svc .

build: build-rust build-go
	@echo "✅ All binaries built"

# ═══════════════════════════════════════════════════════════════
# Run — requires .env or environment variables
# ═══════════════════════════════════════════════════════════════

start: build
	@echo "🚀 Starting HiTechCloud AI OS..."
	@echo "  Ports: 8080(AI) 8081(MCP) 8082(Skills) 8083(Plugin) 8084(A2A) 8085(Admin)"
	@mkdir -p logs pids
	@if [ -f .env ]; then set -a && source .env && set +a; fi; \
	nohup $(RUST_BIN)/hitechcloud-ai-gateway   > logs/ai-gateway.log   2>&1 & echo $$! > pids/ai-gateway.pid; \
	nohup $(RUST_BIN)/hitechcloud-mcp-gateway  > logs/mcp-gateway.log  2>&1 & echo $$! > pids/mcp-gateway.pid; \
	nohup $(RUST_BIN)/hitechcloud-a2a-gateway  > logs/a2a-gateway.log  2>&1 & echo $$! > pids/a2a-gateway.pid; \
	nohup $(GO_BIN)/skills-svc  > logs/skills-svc.log  2>&1 & echo $$! > pids/skills-svc.pid; \
	nohup $(GO_BIN)/plugin-svc  > logs/plugin-svc.log  2>&1 & echo $$! > pids/plugin-svc.pid; \
	nohup $(GO_BIN)/admin-svc   > logs/admin-svc.log   2>&1 & echo $$! > pids/admin-svc.pid; \
	sleep 2 && echo "✅ All services started"

stop:
	@echo "⏹ Stopping services..."
	@for svc in ai-gateway mcp-gateway a2a-gateway skills-svc plugin-svc admin-svc; do \
		if [ -f pids/$$svc.pid ]; then \
			kill $$(cat pids/$$svc.pid) 2>/dev/null && echo "  Stopped $$svc" || true; \
			rm -f pids/$$svc.pid; \
		fi; \
	done

restart: stop start

# ═══════════════════════════════════════════════════════════════
# Test & Health
# ═══════════════════════════════════════════════════════════════

test:
	cargo test --workspace
	cd skills-svc && go test ./...
	cd plugin-svc && go test ./...
	cd admin-svc  && go test ./...

health:
	@echo "🏥 Health check..."
	@curl -sf http://127.0.0.1:8080/health | python3 -m json.tool && echo "  ✅ AI Gateway"    || echo "  ❌ AI Gateway DOWN"
	@curl -sf http://127.0.0.1:8081/health | python3 -m json.tool && echo "  ✅ MCP Gateway"   || echo "  ❌ MCP Gateway DOWN"
	@curl -sf http://127.0.0.1:8082/health | python3 -m json.tool && echo "  ✅ Skills Service" || echo "  ❌ Skills DOWN"
	@curl -sf http://127.0.0.1:8083/health | python3 -m json.tool && echo "  ✅ Plugin Service" || echo "  ❌ Plugin DOWN"
	@curl -sf http://127.0.0.1:8084/health | python3 -m json.tool && echo "  ✅ A2A Gateway"    || echo "  ❌ A2A DOWN"
	@curl -sf http://127.0.0.1:8085/health | python3 -m json.tool && echo "  ✅ Admin Service"  || echo "  ❌ Admin DOWN"

seed:
	PGPASSWORD='$(DB_PASSWORD)' psql -h $(DB_HOST) -U $(DB_USER) -d $(DB_NAME) -f seed.sql

clean:
	cargo clean
	rm -rf $(GO_BIN) logs/*.log pids/*.pid
