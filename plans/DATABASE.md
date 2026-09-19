# Database Schema (Postgres)

## Bảng chính

```sql
CREATE TABLE organizations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  plan TEXT NOT NULL DEFAULT 'free',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID REFERENCES organizations(id),
  email TEXT UNIQUE NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE api_keys (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id),
  org_id UUID REFERENCES organizations(id),
  key_hash TEXT NOT NULL,          -- lưu hash, không lưu key gốc
  scope TEXT NOT NULL,             -- user | project | org
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE roles (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID REFERENCES organizations(id),
  name TEXT NOT NULL
);

CREATE TABLE role_permissions (
  role_id UUID REFERENCES roles(id),
  permission TEXT NOT NULL,
  PRIMARY KEY (role_id, permission)
);

CREATE TABLE user_roles (
  user_id UUID REFERENCES users(id),
  role_id UUID REFERENCES roles(id),
  PRIMARY KEY (user_id, role_id)
);

CREATE TABLE mcp_servers (
  id TEXT PRIMARY KEY,             -- kebab-case id
  name TEXT NOT NULL,
  transport TEXT NOT NULL,
  namespace TEXT NOT NULL,
  visibility TEXT NOT NULL DEFAULT 'public',
  owner_org_id UUID REFERENCES organizations(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE mcp_server_versions (
  mcp_server_id TEXT REFERENCES mcp_servers(id),
  version TEXT NOT NULL,
  manifest JSONB NOT NULL,
  package_hash TEXT NOT NULL,
  signature TEXT,
  status TEXT NOT NULL DEFAULT 'draft',
  published_at TIMESTAMPTZ,
  PRIMARY KEY (mcp_server_id, version)
);

CREATE TABLE skills (
  id TEXT PRIMARY KEY,
  scope TEXT NOT NULL,             -- user | org | public
  owner_id UUID,                   -- user_id hoặc org_id tuỳ scope
  name TEXT NOT NULL,
  description TEXT NOT NULL
);

CREATE TABLE skill_versions (
  skill_id TEXT REFERENCES skills(id),
  version TEXT NOT NULL,
  content_ref TEXT NOT NULL,       -- pointer tới object storage
  status TEXT NOT NULL DEFAULT 'draft',
  PRIMARY KEY (skill_id, version)
);

CREATE TABLE plugins (
  id TEXT PRIMARY KEY,
  owner_org_id UUID REFERENCES organizations(id),
  name TEXT NOT NULL,
  description TEXT NOT NULL
);

CREATE TABLE plugin_versions (
  plugin_id TEXT REFERENCES plugins(id),
  version TEXT NOT NULL,
  manifest JSONB NOT NULL,         -- nội dung plugin.json
  package_hash TEXT NOT NULL,
  signature TEXT,
  status TEXT NOT NULL DEFAULT 'draft',
  PRIMARY KEY (plugin_id, version)
);

CREATE TABLE installations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_type TEXT NOT NULL,       -- mcp_server | skill | plugin
  entity_id TEXT NOT NULL,
  version TEXT NOT NULL,
  scope TEXT NOT NULL,             -- user | project | org
  scope_ref TEXT,                  -- project id nếu scope=project
  installed_by UUID REFERENCES users(id),
  org_id UUID REFERENCES organizations(id),
  installed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE audit_logs (
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

CREATE TABLE usage_records (
  id BIGSERIAL PRIMARY KEY,
  org_id UUID REFERENCES organizations(id),
  resource_type TEXT NOT NULL,
  resource_id TEXT,
  quantity NUMERIC NOT NULL,
  unit TEXT NOT NULL,
  occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## Index quan trọng

```sql
CREATE INDEX idx_audit_logs_org_created ON audit_logs (org_id, created_at DESC);
CREATE INDEX idx_usage_records_org_period ON usage_records (org_id, occurred_at);
CREATE INDEX idx_installations_scope ON installations (scope, scope_ref);
CREATE INDEX idx_skills_search ON skills USING GIN (to_tsvector('simple', name || ' ' || description));
```

## Ghi chú

- `audit_logs` và `usage_records` nên **partition theo tháng** khi dữ liệu lớn (Postgres native partitioning) để query/aggregate nhanh và dễ archive.
- Không lưu API key gốc, chỉ lưu `key_hash` (vd SHA-256 + salt), verify bằng so khớp hash.
