-- Prefix each operation in the same EVAL; a separate preflight can race reset.
-- KEYS[3] and ARGV[4] are the namespace's permanent key and pinned generation.
if redis.call('GET', KEYS[3]) ~= ARGV[4] then return 'unavailable' end
