-- Billing & Usage tracking tables
-- Phase 2: Usage records for MCP calls, AI tokens, bandwidth

CREATE TABLE IF NOT EXISTS usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id),
    user_id UUID REFERENCES users(id),
    session_id UUID,
    provider VARCHAR(64) NOT NULL DEFAULT 'hitechcloud',
    model_name VARCHAR(128),
    resource_type VARCHAR(64) NOT NULL,     -- 'mcp_call', 'ai_tokens', 'bandwidth', 'skill_exec'
    input_tokens INT DEFAULT 0,
    output_tokens INT DEFAULT 0,
    cached_tokens INT DEFAULT 0,
    effective_unit_price NUMERIC(12, 6),
    total_cost_usd NUMERIC(12, 6) NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    occurred_at TIMESTAMPTZ DEFAULT NOW(),
    billing_period VARCHAR(7) NOT NULL      -- '2026-09'
);

CREATE INDEX IF NOT EXISTS idx_usage_org_period ON usage_records(org_id, billing_period);
CREATE INDEX IF NOT EXISTS idx_usage_user ON usage_records(user_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_usage_resource ON usage_records(resource_type, occurred_at);

-- Budget quotas per org
CREATE TABLE IF NOT EXISTS budget_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) UNIQUE,
    monthly_limit_usd NUMERIC(12, 2) NOT NULL DEFAULT 100.00,
    soft_limit_pct INT NOT NULL DEFAULT 80,
    hard_limit_action VARCHAR(32) NOT NULL DEFAULT 'fallback',  -- 'fallback', 'block', 'notify'
    current_usage_usd NUMERIC(12, 6) NOT NULL DEFAULT 0,
    billing_period VARCHAR(7) NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Dynamic pricing cache (synced from provider APIs)
CREATE TABLE IF NOT EXISTS model_pricing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(64) NOT NULL,
    model_name VARCHAR(128) NOT NULL,
    input_price_per_million NUMERIC(12, 6),
    output_price_per_million NUMERIC(12, 6),
    cache_read_price_per_million NUMERIC(12, 6),
    synced_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(provider, model_name)
);
