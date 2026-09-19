use redis::{AsyncCommands, RedisResult, Script, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};
use tracing;

// ═══════════════════════════════════════════════════════════════
//  HiTechCloud Redis — Cache, Rate Limiting, Pub/Sub
// ═══════════════════════════════════════════════════════════════

/// Redis client wrapper with caching, rate limiting, and pub/sub
#[derive(Clone)]
pub struct RedisCache {
    conn: ConnectionManager,
}

impl RedisCache {
    /// Connect to Redis
    pub async fn connect(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        tracing::info!("Connected to Redis");
        Ok(Self { conn })
    }

    // ───────────────────────────────────────────────────────────
    //  Config Cache (cfg:* keys)
    // ───────────────────────────────────────────────────────────

    /// Cache MCP server config (TTL 60s, invalidate via pub/sub)
    pub async fn cache_mcp_server(&self, id: &str, data: &serde_json::Value) -> RedisResult<()> {
        let key = format!("cfg:mcp_server:{}", id);
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(data).unwrap_or_default();
        conn.set_ex(&key, &json, 60).await
    }

    /// Get cached MCP server config
    pub async fn get_mcp_server(&self, id: &str) -> RedisResult<Option<serde_json::Value>> {
        let key = format!("cfg:mcp_server:{}", id);
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.get(&key).await?;
        match result {
            Some(json) => Ok(Some(
                serde_json::from_str(&json).unwrap_or(serde_json::Value::Null),
            )),
            None => Ok(None),
        }
    }

    /// Cache plugin manifest (TTL 120s)
    pub async fn cache_plugin(
        &self,
        id: &str,
        version: &str,
        data: &serde_json::Value,
    ) -> RedisResult<()> {
        let key = format!("cfg:plugin:{}:{}", id, version);
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(data).unwrap_or_default();
        conn.set_ex(&key, &json, 120).await
    }

    /// Get cached plugin manifest
    pub async fn get_plugin(
        &self,
        id: &str,
        version: &str,
    ) -> RedisResult<Option<serde_json::Value>> {
        let key = format!("cfg:plugin:{}:{}", id, version);
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.get(&key).await?;
        match result {
            Some(json) => Ok(Some(
                serde_json::from_str(&json).unwrap_or(serde_json::Value::Null),
            )),
            None => Ok(None),
        }
    }

    /// Cache generic key-value with TTL
    pub async fn cache_set<T: Serialize>(
        &self,
        key: &str,
        data: &T,
        ttl_seconds: u64,
    ) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(data).unwrap_or_default();
        conn.set_ex(key, &json, ttl_seconds).await
    }

    /// Get cached value
    pub async fn cache_get<T: DeserializeOwned>(&self, key: &str) -> RedisResult<Option<T>> {
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.get(key).await?;
        match result {
            Some(json) => Ok(serde_json::from_str(&json).ok()),
            None => Ok(None),
        }
    }

    /// Invalidate a specific key
    pub async fn cache_invalidate(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        conn.del(key).await
    }

    /// Invalidate all MCP server caches (on registry change)
    pub async fn invalidate_all_mcp_servers(&self) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let keys: Vec<String> = conn.keys("cfg:mcp_server:*").await?;
        if !keys.is_empty() {
            let _: () = redis::cmd("DEL").arg(&keys).query_async(&mut conn).await?;
            tracing::info!(count = keys.len(), "Invalidated MCP server caches");
        }
        Ok(())
    }

    // ───────────────────────────────────────────────────────────
    //  Rate Limiting (Token Bucket via Lua)
    // ───────────────────────────────────────────────────────────

    /// Check rate limit using Lua token bucket script (atomic)
    /// Returns true if request is allowed, false if rate limited
    pub async fn check_rate_limit(
        &self,
        api_key_id: &str,
        limit: i64,
        window_seconds: i64,
    ) -> RedisResult<bool> {
        let key = format!(
            "ratelimit:{}:{}",
            api_key_id,
            chrono::Utc::now().timestamp() / window_seconds
        );

        let script = Script::new(
            r#"
            local current = tonumber(redis.call('GET', KEYS[1]) or "0")
            if current >= tonumber(ARGV[1]) then
                return 0
            end
            redis.call('INCR', KEYS[1])
            redis.call('EXPIRE', KEYS[1], ARGV[2])
            return 1
            "#,
        );

        let mut conn = self.conn.clone();
        let result: i32 = script
            .key(&key)
            .arg(limit)
            .arg(window_seconds)
            .invoke_async(&mut conn)
            .await?;

        Ok(result == 1)
    }

    /// Check rate limit for a specific tool
    pub async fn check_tool_rate_limit(
        &self,
        tool_id: &str,
        limit: i64,
        window_seconds: i64,
    ) -> RedisResult<bool> {
        let key = format!(
            "ratelimit:tool:{}:{}",
            tool_id,
            chrono::Utc::now().timestamp() / window_seconds
        );

        let script = Script::new(
            r#"
            local current = tonumber(redis.call('GET', KEYS[1]) or "0")
            if current >= tonumber(ARGV[1]) then
                return 0
            end
            redis.call('INCR', KEYS[1])
            redis.call('EXPIRE', KEYS[1], ARGV[2])
            return 1
            "#,
        );

        let mut conn = self.conn.clone();
        let result: i32 = script
            .key(&key)
            .arg(limit)
            .arg(window_seconds)
            .invoke_async(&mut conn)
            .await?;

        Ok(result == 1)
    }

    /// Get current rate limit count for an API key
    pub async fn get_rate_limit_count(
        &self,
        api_key_id: &str,
        window_seconds: i64,
    ) -> RedisResult<i64> {
        let key = format!(
            "ratelimit:{}:{}",
            api_key_id,
            chrono::Utc::now().timestamp() / window_seconds
        );
        let mut conn = self.conn.clone();
        let count: Option<i64> = conn.get(&key).await?;
        Ok(count.unwrap_or(0))
    }

    // ───────────────────────────────────────────────────────────
    //  Session Cache
    // ───────────────────────────────────────────────────────────

    /// Store session (token → user_id, org_id)
    pub async fn set_session(
        &self,
        token: &str,
        user_id: &str,
        org_id: &str,
        ttl_seconds: u64,
    ) -> RedisResult<()> {
        let key = format!("session:{}", token);
        let mut conn = self.conn.clone();
        let data = serde_json::json!({
            "user_id": user_id,
            "org_id": org_id,
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        conn.set_ex(&key, data.to_string(), ttl_seconds).await
    }

    /// Get session data
    pub async fn get_session(&self, token: &str) -> RedisResult<Option<serde_json::Value>> {
        let key = format!("session:{}", token);
        let mut conn = self.conn.clone();
        let result: Option<String> = conn.get(&key).await?;
        match result {
            Some(json) => Ok(Some(
                serde_json::from_str(&json).unwrap_or(serde_json::Value::Null),
            )),
            None => Ok(None),
        }
    }

    /// Invalidate session
    pub async fn invalidate_session(&self, token: &str) -> RedisResult<()> {
        let key = format!("session:{}", token);
        let mut conn = self.conn.clone();
        conn.del(&key).await
    }

    // ───────────────────────────────────────────────────────────
    //  Pub/Sub — Cache Invalidation
    // ───────────────────────────────────────────────────────────

    /// Publish cache invalidation event
    pub async fn publish_invalidation(
        &self,
        entity_type: &str,
        entity_id: &str,
        version: Option<&str>,
    ) -> RedisResult<()> {
        let mut conn = self.conn.clone();
        let msg = serde_json::json!({
            "type": entity_type,
            "id": entity_id,
            "version": version,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        let _: () = redis::cmd("PUBLISH")
            .arg("hitechcloud:invalidate")
            .arg(msg.to_string())
            .query_async(&mut conn)
            .await?;
        tracing::info!(
            entity_type = entity_type,
            entity_id = entity_id,
            "Published cache invalidation"
        );
        Ok(())
    }

    // ───────────────────────────────────────────────────────────
    //  Health Check
    // ───────────────────────────────────────────────────────────

    /// Check if Redis is connected
    pub async fn ping(&self) -> RedisResult<bool> {
        let mut conn = self.conn.clone();
        let result: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(result == "PONG")
    }

    /// Get Redis info (memory, clients, etc.)
    pub async fn info(&self) -> RedisResult<String> {
        let mut conn = self.conn.clone();
        redis::cmd("INFO").query_async(&mut conn).await
    }

    /// Get memory usage info
    pub async fn memory_usage(&self) -> RedisResult<serde_json::Value> {
        let mut conn = self.conn.clone();
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await?;

        let mut result = serde_json::Map::new();
        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                if !key.starts_with('#') {
                    result.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
        }
        Ok(serde_json::Value::Object(result))
    }
}

/// Rate limit result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: i64,
    pub reset_at: i64,
}

/// Extended rate limit check with detailed info
pub async fn check_rate_limit_detailed(
    cache: &RedisCache,
    api_key_id: &str,
    limit: i64,
    window_seconds: i64,
) -> anyhow::Result<RateLimitResult> {
    let current_timestamp = chrono::Utc::now().timestamp();
    let window_start = current_timestamp / window_seconds;
    let reset_at = (window_start + 1) * window_seconds;

    let key = format!("ratelimit:{}:{}", api_key_id, window_start);
    let mut conn = cache.conn.clone();

    let script = Script::new(
        r#"
        local current = tonumber(redis.call('GET', KEYS[1]) or "0")
        if current >= tonumber(ARGV[1]) then
            return current
        end
        redis.call('INCR', KEYS[1])
        redis.call('EXPIRE', KEYS[1], ARGV[2])
        return current + 1
        "#,
    );

    let current: i64 = script
        .key(&key)
        .arg(limit)
        .arg(window_seconds)
        .invoke_async(&mut conn)
        .await?;

    Ok(RateLimitResult {
        allowed: current <= limit,
        remaining: (limit - current).max(0),
        reset_at,
    })
}
