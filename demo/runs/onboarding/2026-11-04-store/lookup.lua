-- Read state and capability together, only after the generation guard.
local state = redis.call('HGET', KEYS[1], 'state')
if not state or state == 'closed' then return 'ended' end
local secret = redis.call('GET', KEYS[2])
if not secret then return 'ended' end
return secret
