// SPDX-License-Identifier: AGPL-3.0-or-later
// All keys share {onboarding}. Pending reservations survive any script error;
// only the final marker exposes a secret. Redis isolates EVAL, not rollback.
// Reservations are permanent, so a lookup with no record is a phrase never
// issued ('missing'), told apart from a closed or expired one ('ended').
//
// Guessing is bounded by misses, lookups of a phrase never issued; finding an
// open, ended or pending invitation spends none. A miss spends its network's
// budget (30 an hour, 100 a day) and the global one (1,000 a day, MISSES a
// month). A spent budget refuses every lookup, hits too, checked from the
// counters alone, so a refusal says nothing about which phrases exist. Every
// lookup and status also counts per network (30 a minute, 300 a day) and
// toward a month's 30,000, which keeps accepted commands inside Upstash
// Free's 500,000 (ONBOARDING.md §5). With handler.mjs's SPACE these bound a
// guesser's chance a month at MISSES x OPEN / SPACE (resolver-contracts.test.mjs).
export const MISSES = 20000;
export const OPEN = 100;
export const GENERATION = 'onboarding-2026-11-04-v1';
export const PREFIX = 'svrngs:{onboarding}';
export const SCRIPT = `
local function reply(state, r, secret)
  return cjson.encode({state=state, phrase=r and r.phrase or '', secret=secret or false})
end
local function refuse(reason) return cjson.encode({state='unavailable', reason=reason}) end
if redis.call('GET', KEYS[1]) ~= ARGV[1] then return refuse('generation') end
local q = cjson.decode(ARGV[2])
local now = tonumber(redis.call('TIME')[1])
if now < q.monthStart or now >= q.monthEnd then return refuse('clock') end
local id = q.idkey
local lookup = q.action == 'lookup'
if lookup then id = redis.call('HGET', KEYS[2], 'p:'..q.phrase) end
local raw = id and redis.call('HGET', KEYS[2], 'i:'..id)
local r = raw and cjson.decode(raw)
local fresh = q.action == 'publish' and not r
local miss = lookup and not r and 1 or 0
local quotas = redis.call('MGET', KEYS[4], KEYS[5])
local global = quotas[1] and cjson.decode(quotas[1]) or {}
local subject = quotas[2] and cjson.decode(quotas[2]) or {}
local function count(at, name, bucket, cap, add)
  local old = at[name]
  local n = old and old[1] == bucket and old[2] or 0
  if type(n) ~= 'number' or n < 0 or n >= cap then return false end
  at[name] = {bucket, n + (add or 1)}
  return true
end
local minute, hour, day = math.floor(now/60), math.floor(now/3600), math.floor(now/86400)
local activeCap, mutationCap = ${OPEN}, 3000
local public = lookup or q.action == 'status'
if public then
  if not count(subject, 'minute', minute, 30) or not count(subject, 'day', day, 300)
    or not count(global, 'lookupMonth', q.monthStart, 30000) or lookup and not (count(global, 'missDay', day, 1000, miss)
    and count(global, 'missMonth', q.monthStart, ${MISSES}, miss)) then return refuse('lookup_limit') end
  if lookup and not count(subject, 'missHour', hour, 30, miss) then return refuse('miss_hour') end
  if lookup and not count(subject, 'missDay', day, 100, miss) then return refuse('miss_day') end
else
  if not count(subject, 'minute', minute, 10) or not count(subject, 'hour', hour, 60)
    or not count(global, 'mutationMonth', q.monthStart, mutationCap) then return refuse('mutation_limit') end
  if fresh and global.mutationMonth[2] > mutationCap - 2 * activeCap then return refuse('close_headroom') end
  if fresh and (not count(subject, 'newDay', day, 5)
    or not count(global, 'newDay', day, 100)) then return refuse('publication_limit') end
end
redis.call('SET', KEYS[4], cjson.encode(global), 'EX', 2764800)
redis.call('SET', KEYS[5], cjson.encode(subject), 'EX', 2764800)
if r and q.action ~= 'lookup' and r.binding ~= q.binding then return refuse('binding') end
if r and q.action == 'publish' and r.digest ~= q.digest then return refuse('digest') end
if q.action == 'publish' and (not r or r.state == 'pending') then
  if q['until'] <= now or q['until'] > now + 86400 then return refuse('lifetime') end
  local active = redis.call('ZCOUNT', KEYS[3], now + 1, '+inf')
  if active >= activeCap and not redis.call('ZSCORE', KEYS[3], id) then return refuse('active_limit') end
end
if fresh then
  if redis.call('HGET', KEYS[2], 'p:'..q.phrase) then return refuse('collision') end
end
if fresh or (q.action == 'close' and not r) then
  -- Counting fields conservatively bounds both ordinary and close-only records.
  if redis.call('HLEN', KEYS[2]) >= 100000 then return refuse('reservation_limit') end
  local memory = redis.call('INFO', 'memory')
  local used = tonumber(string.match(memory, 'used_memory:(%d+)'))
  if not used or used >= 100000000 then return refuse('memory_limit') end
end
if public then
  if miss == 1 then return reply('missing') end
  if not r or r.state == 'closed' or r['until'] <= now then return reply('ended', r) end
  if r.state ~= 'open' then return refuse('pending') end
  local secret = redis.call('GET', KEYS[2]..':secret:'..id)
  if not secret then return refuse('missing_secret') end
  return reply('open', r, q.action == 'lookup' and secret or false)
end
if q.action == 'close' then
  r = r or {binding=q.binding, digest='', ['until']=q['until'], phrase=''}
  redis.call('DEL', KEYS[2]..':secret:'..id)
  redis.call('ZREM', KEYS[3], id)
  r.state = 'closed'
  redis.call('HSET', KEYS[2], 'i:'..id, cjson.encode(r))
  return reply('closed', r)
end
if r and (r.state == 'closed' or r['until'] <= now) then return reply('ended', r) end
if not r then
  r = {binding=q.binding, digest=q.digest, ['until']=q['until'], phrase=q.phrase, state='pending'}
  redis.call('HSET', KEYS[2], 'i:'..id, cjson.encode(r), 'p:'..q.phrase, id)
end
if r.state ~= 'open' then
  redis.call('ZADD', KEYS[3], r['until'], id)
  redis.call('SET', KEYS[2]..':secret:'..id, q.secret, 'EXAT', r['until'])
  r.state = 'open'
  redis.call('HSET', KEYS[2], 'i:'..id, cjson.encode(r))
end
if not redis.call('GET', KEYS[2]..':secret:'..id) then return refuse('missing_secret') end
return reply('open', r)
`;
