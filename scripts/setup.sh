#!/bin/bash
# ═══════════════════════════════════════════════════════════════
#  HiTechCloud AI OS — Development Setup Script
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
err() { echo -e "${RED}[✗]${NC} $1"; }

echo "═══════════════════════════════════════════════════════════"
echo "  HiTechCloud AI OS — Development Setup"
echo "═══════════════════════════════════════════════════════════"
echo

# Check prerequisites
command -v cargo >/dev/null 2>&1 || { err "Rust not installed. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; exit 1; }
command -v go >/dev/null 2>&1 || { err "Go not installed. Install: https://go.dev/dl/"; exit 1; }
command -v psql >/dev/null 2>&1 || { err "PostgreSQL client not installed"; exit 1; }
command -v redis-cli >/dev/null 2>&1 || { err "Redis client not installed"; exit 1; }

log "Prerequisites check passed"

# Setup .env
if [ ! -f .env ]; then
    cp .env.example .env
    log "Created .env from template — please edit with your values"
else
    log ".env already exists"
fi

# Build Rust workspace
log "Building Rust workspace..."
cargo build --release 2>&1 | tail -3

# Build Go services
log "Building Go services..."
mkdir -p bin
cd services/skills-svc && go build -o ../../bin/skills-svc . && cd ../..
cd services/plugin-svc && go build -o ../../bin/plugin-svc . && cd ../..
cd services/admin-svc  && go build -o ../../bin/admin-svc .  && cd ../..

# Install CLI
log "Installing CLI..."
cp target/release/hitechcloud bin/hitechcloud

log "Setup complete!"
echo
echo "Next steps:"
echo "  1. Edit .env with your database/redis credentials"
echo "  2. Run: make start"
echo "  3. Run: make health"
echo "  4. Run: hitechcloud login --key <API_KEY>"
