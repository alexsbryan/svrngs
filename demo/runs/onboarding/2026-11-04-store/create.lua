-- Local primitive proof, not a deployed resolver. ARGV[1] is an immutable
-- binding of owner, publication id, invitation digest and absolute expiry.
local identity = redis.call('HGET', KEYS[1], 'identity')
if identity then
  if identity ~= ARGV[1] then return 'conflict' end
  if redis.call('HGET', KEYS[1], 'state') == 'closed' then return 'closed' end
  if redis.call('EXISTS', KEYS[2]) == 0 then return 'expired' end
  return 'replayed'
end
local now = redis.call('TIME')
local millis = tonumber(now[1]) * 1000 + math.floor(tonumber(now[2]) / 1000)
local expires = tonumber(ARGV[3])
if not expires or expires <= millis then return 'expired' end
redis.call('HSET', KEYS[1], 'identity', ARGV[1], 'state', 'open')
redis.call('SET', KEYS[2], ARGV[2], 'PXAT', ARGV[3])
return 'created'
