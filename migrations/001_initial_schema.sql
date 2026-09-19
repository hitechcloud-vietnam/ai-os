-- HiTechCloud AI OS - Initial Database Schema
-- Version: 001

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Organizations
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    plan TEXT NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID REFERENCES organizations(id),
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- API Keys (hashed)
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    org_id UUID REFERENCES organizations(id),
    key_hash TEXT NOT NULL,
    scope TEXT NOT NULL,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- RBAC
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID REFERENCES organizations(id),
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID REFERENCES roles(id),
    permission TEXT NOT NULL,
    PRIMARY KEY (role_id, permission)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID REFERENCES users(id),
    role_id UUID REFERENCES roles(id),
    PRIMARY KEY (user_id, role_id)
);

-- MCP Servers
CREATE TABLE IF NOT EXISTS mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    transport TEXT NOT NULL,
    namespace TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'public',
    owner_org_id UUID REFERENCES organizations(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS mcp_server_versions (
    mcp_server_id TEXT REFERENCES mcp_servers(id),
    version TEXT NOT NULL,
    manifest JSONB NOT NULL,
    package_hash TEXT NOT NULL,
    signature TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    PRIMARY KEY (mcp_server_id, version)
);

-- Skills
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    owner_id UUID,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS skill_versions (
    skill_id TEXT REFERENCES skills(id),
    version TEXT NOT NULL,
    content_ref TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    PRIMARY KEY (skill_id, version)
);

-- Plugins
CREATE TABLE IF NOT EXISTS plugins (
    id TEXT PRIMARY KEY,
    owner_org_id UUID REFERENCES organizations(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS plugin_versions (
    plugin_id TEXT REFERENCES plugins(id),
    version TEXT NOT NULL,
    manifest JSONB NOT NULL,
    package_hash TEXT NOT NULL,
    signature TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    PRIMARY KEY (plugin_id, version)
);

-- Installations
CREATE TABLE IF NOT EXISTS installations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    version TEXT NOT NULL,
    scope TEXT NOT NULL,
    scope_ref TEXT,
    installed_by UUID REFERENCES users(id),
    org_id UUID REFERENCES organizations(id),
    installed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Audit Logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    org_id UUID REFERENCES organizations(id),
    actor_id UUID,
    action TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    result_status TEXT NOT NULL,
    request_meta JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Usage Records
CREATE TABLE IF NOT EXISTS usage_records (
    id BIGSERIAL PRIMARY KEY,
    org_id UUID REFERENCES organizations(id),
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    quantity NUMERIC NOT NULL,
    unit TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_audit_logs_org_created ON audit_logs (org_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_usage_records_org_period ON usage_records (org_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_installations_scope ON installations (scope, scope_ref);
CREATE INDEX IF NOT EXISTS idx_skills_search ON skills USING GIN (to_tsvector('simple', name || ' ' || description));
