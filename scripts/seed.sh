#!/bin/bash
# ═══════════════════════════════════════════════════════════════
#  HiTechCloud AI OS — Database Seed Script
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Load .env if exists
if [ -f "$PROJECT_ROOT/.env" ]; then
    set -a && source "$PROJECT_ROOT/.env" && set +a
fi

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-hitechcloud}"
DB_PASSWORD="${DB_PASSWORD:-hitechcloud_dev_2026}"
DB_NAME="${DB_NAME:-hitechcloud}"

echo "Seeding database: $DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"

# Apply migrations
echo "Applying migrations..."
for migration in "$PROJECT_ROOT"/migrations/*.sql; do
    echo "  Applying: $(basename "$migration")"
    PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$migration" 2>&1 | grep -v "NOTICE" || true
done

# Apply seed data
echo "Applying seed data..."
PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$PROJECT_ROOT/seed.sql" 2>&1 | grep -v "NOTICE" || true

echo "Seed complete!"
