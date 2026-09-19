use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use uuid::Uuid;

/// RBAC roles with hierarchical permissions
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Owner,
    Admin,
    Developer,
    Viewer,
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "owner" => Role::Owner,
            "admin" => Role::Admin,
            "developer" => Role::Developer,
            "viewer" => Role::Viewer,
            _ => Role::Viewer,
        }
    }

    /// Check if this role has the given permission
    pub fn has_permission(&self, permission: &str) -> bool {
        match self {
            Role::Owner => true, // Owner has all permissions
            Role::Admin => {
                matches!(
                    permission,
                    "server.read"
                        | "server.write"
                        | "server.delete"
                        | "skill.read"
                        | "skill.write"
                        | "skill.delete"
                        | "skill.publish"
                        | "plugin.read"
                        | "plugin.write"
                        | "plugin.delete"
                        | "plugin.approve"
                        | "member.read"
                        | "member.write"
                        | "member.remove"
                        | "apikey.read"
                        | "apikey.create"
                        | "apikey.revoke"
                        | "audit.read"
                        | "marketplace.read"
                        | "marketplace.approve"
                        | "installation.read"
                        | "installation.create"
                        | "installation.remove"
                )
            }
            Role::Developer => {
                matches!(
                    permission,
                    "server.read"
                        | "skill.read"
                        | "skill.write"
                        | "skill.publish"
                        | "plugin.read"
                        | "member.read"
                        | "apikey.read"
                        | "apikey.create"
                        | "marketplace.read"
                        | "installation.read"
                        | "installation.create"
                        | "installation.remove"
                )
            }
            Role::Viewer => {
                matches!(
                    permission,
                    "server.read"
                        | "skill.read"
                        | "plugin.read"
                        | "member.read"
                        | "marketplace.read"
                        | "installation.read"
                        | "audit.read"
                )
            }
        }
    }
}

/// Required permission for each API endpoint
pub fn required_permission(method: &str, path: &str) -> Option<&'static str> {
    match (method, path) {
        // MCP Servers
        ("GET", p) if p.starts_with("/v1/registry/mcp-servers") && !p.contains("/mcp-servers/") => Some("server.read"),
        ("GET", p) if p.starts_with("/v1/registry/mcp-servers/") => Some("server.read"),
        ("POST", "/v1/registry/mcp-servers") => Some("server.write"),
        ("PUT", p) if p.starts_with("/v1/registry/mcp-servers/") => Some("server.write"),
        ("DELETE", p) if p.starts_with("/v1/registry/mcp-servers/") => Some("server.delete"),

        // Skills
        ("GET", p) if p.starts_with("/v1/registry/skills") && !p.contains("/skills/") => Some("skill.read"),
        ("GET", p) if p.starts_with("/v1/registry/skills/") => Some("skill.read"),
        ("POST", "/v1/registry/skills") => Some("skill.write"),
        ("PUT", p) if p.starts_with("/v1/registry/skills/") => Some("skill.write"),
        ("DELETE", p) if p.starts_with("/v1/registry/skills/") => Some("skill.delete"),

        // Plugins
        ("GET", p) if p.starts_with("/v1/registry/plugins") && !p.contains("/plugins/") => Some("plugin.read"),
        ("GET", p) if p.starts_with("/v1/registry/plugins/") => Some("plugin.read"),
        ("POST", "/v1/registry/plugins") => Some("plugin.write"),
        ("PUT", p) if p.starts_with("/v1/registry/plugins/") => Some("plugin.write"),
        ("DELETE", p) if p.starts_with("/v1/registry/plugins/") => Some("plugin.delete"),

        // Installations
        ("GET", "/v1/registry/installations") => Some("installation.read"),
        ("POST", "/v1/registry/installations") => Some("installation.create"),
        ("DELETE", p) if p.starts_with("/v1/registry/installations/") => Some("installation.remove"),

        // Search
        ("GET", "/v1/registry/search") => Some("marketplace.read"),

        // Admin
        ("GET", "/v1/admin/organizations") => Some("member.read"),
        ("GET", p) if p.starts_with("/v1/admin/organizations/") => Some("member.read"),
        ("GET", "/v1/admin/audit-logs") => Some("audit.read"),

        // Health — no permission needed
        ("GET", "/health") => None,

        _ => None,
    }
}

/// RBAC middleware that checks permissions
pub async fn rbac_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Skip RBAC for health and unauthenticated endpoints
    if path == "/health" || path.starts_with("/mcp/") || path == "/tools" {
        return Ok(next.run(request).await);
    }

    // Get required permission for this endpoint
    let perm = match required_permission(&method, &path) {
        Some(p) => p,
        None => return Ok(next.run(request).await), // No permission required
    };

    // Extract user info from auth header (set by auth middleware)
    // For now, check if user has API key and resolve their role
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Try to resolve user from API key
    let role = if api_key.starts_with("hitechcloud-sk_") {
        resolve_role_from_api_key(&pool, api_key).await
    } else if auth_header.starts_with("Bearer ") {
        let token = &auth_header[7..];
        resolve_role_from_token(&pool, token).await
    } else {
        None
    };

    let role = match role {
        Some(r) => r,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    if !role.has_permission(perm) {
        // Log the denied access
        tracing::warn!(
            permission = perm,
            method = %method,
            path = %path,
            "RBAC: Permission denied"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

async fn resolve_role_from_api_key(pool: &PgPool, key: &str) -> Option<Role> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT u.role FROM api_keys ak JOIN users u ON ak.user_id = u.id WHERE ak.key_hash = encode(digest($1, 'sha256'), 'hex') AND (ak.expires_at IS NULL OR ak.expires_at > now())"
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .ok()??;

    Some(Role::from_str(&row))
}

async fn resolve_role_from_token(pool: &PgPool, token: &str) -> Option<Role> {
    // For JWT tokens, decode and resolve role
    // For now, check if it's a valid API key used as bearer token
    let row = sqlx::query_scalar::<_, String>(
        "SELECT u.role FROM api_keys ak JOIN users u ON ak.user_id = u.id WHERE ak.key_hash = encode(digest($1, 'sha256'), 'hex') AND (ak.expires_at IS NULL OR ak.expires_at > now())"
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .ok()??;

    Some(Role::from_str(&row))
}
