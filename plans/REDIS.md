# Redis

## Vai trò trong hệ thống

1. **Cache cấu hình routing** đồng bộ từ Registry (giảm tải Postgres, giảm latency route request).
2. **Rate limiting** (token bucket / sliding window) theo API key và theo tool.
3. **Session/token cache** ngắn hạn cho Admin Console/Developer Portal.
4. **Pub/Sub** để Registry broadcast sự kiện `invalidate_cache` / `revoke_version` xuống mọi instance Gateway gần như tức thời.

## Cấu trúc key gợi ý

```
cfg:mcp_server:{id}              -> JSON manifest (TTL 60s, invalidate qua pub/sub)
cfg:plugin:{id}:{version}        -> JSON manifest
ratelimit:{api_key_id}:{minute}  -> counter (INCR + EXPIRE 60)
ratelimit:tool:{tool_id}:{minute}-> counter riêng theo tool nhạy cảm
session:{token}                  -> user_id, org_id (TTL theo thời hạn session)
```

## Rate limit — token bucket đơn giản (Lua script, atomic)

```lua
-- KEYS[1] = bucket key, ARGV[1] = limit, ARGV[2] = window_seconds
local current = tonumber(redis.call('GET', KEYS[1]) or "0")
if current >= tonumber(ARGV[1]) then
  return 0
end
redis.call('INCR', KEYS[1])
redis.call('EXPIRE', KEYS[1], ARGV[2])
return 1
```

## Pub/Sub invalidate cache

```
Registry: PUBLISH hitechcloud:invalidate '{"type":"mcp_server","id":"github","version":"2.1.0"}'
Gateway (subscriber): nhận message → xoá key cfg:mcp_server:github khỏi cache cục bộ → lần request sau load lại từ Registry
```

## Lưu ý vận hành

- Dùng Redis Cluster hoặc managed Redis (đảm bảo HA) cho production — mất cache không được làm sập hệ thống, chỉ được làm chậm (fallback query thẳng Postgres khi cache miss).
- Không lưu secret thật (token MCP server) trong Redis — secret thuộc về Vault/KMS, Redis chỉ cache metadata không nhạy cảm.
- Giám sát hit/miss ratio của cache routing để tinh chỉnh TTL hợp lý (quá ngắn tốn tải Postgres, quá dài chậm phát hiện revoke).
