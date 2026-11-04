-- Closing before a delayed create reserves the phrase closed as well.
local identity = redis.call('HGET', KEYS[1], 'identity')
if identity and identity ~= ARGV[1] then return 'conflict' end
redis.call('HSET', KEYS[1], 'identity', ARGV[1], 'state', 'closed')
redis.call('DEL', KEYS[2])
return 'closed'
