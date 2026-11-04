# Resolver adapter — 4 November 2026

Unit: `onboarding-resolver`, base `8dcccc14`. Product is in `resolver/`;
the existing page-laws gate includes `web/tests/resolver.test.mjs` without
changing `scripts/check.sh`. No deployment, publication, push or paid upgrade.
The inherited V1.6 publish artifacts were inspected and remain untouched.

## Reuse and ownership

Rechecked `publication.rs:243-305` and the native run's HTTP contract,
`svrngs-core/src/app.rs:38-53` and its frozen stranger vector, the store
survey's RESET.md/FAILURES.md, and upstream
`clients/desktop/src-tauri/src/friendly_names.rs` (display-name vocabulary,
not bearer entropy). The upstream onboarding filenames found were a developer
shell script and desktop E2E spec, not a durable resolver. No resolver existed
to fork. The new handler uses native Node Ed25519 and fetch, the existing
signature domain, the commissioned generation pin and proven EVAL transport;
no provider SDK/framework or alternative native invitation implementation.

The handler accepts the existing signed native POST and JSON lookup for the
future arrival UI. The hosting edge must pass a trusted IP. Config/route/body/
signature failures reject before Redis; generation failure writes nothing.
Counters are shared across handler instances. Only generation-checked,
successful authoritative reads can say ended. No record/folder/member
operation imports this service; existing members need no resolver availability.

Permanent owner/id and phrase reservations contain hashes and expiry, not
invitation capability or group name. The hash has no TTL. Pending reservation,
capacity slot, secret with absolute TTL, then final open marker: no pending
capability is readable. Close deletes the secret, releases the slot, then
commits closed. A failed/uncertain operation is retried with the same binding.
An open marker with a missing unexpired secret means unavailable, not ended.
No runtime-error rollback or hosted failover durability is claimed.

## Red controls (actual runs, TAP boilerplate omitted)

Before the handler existed:

```text
$ node --test web/tests/resolver.test.mjs
Error [ERR_MODULE_NOT_FOUND]: Cannot find module '.../resolver/handler.mjs'
# pass 0
# fail 1
exit 1
```

The test harness transforms only its copy of the Lua sent to its owned local
Redis. Product has no plant switches. All following commands exited **1**:

```text
$ env RESOLVER_PLANT=reopen node --test --test-name-pattern='authenticated lifecycle' web/tests/resolver.test.mjs
not ok - authenticated lifecycle, immutable binding, closure and no reopen
200 !== 503

$ env RESOLVER_PLANT=reuse node --test --test-name-pattern='old phrase' web/tests/resolver.test.mjs
not ok - old phrase cannot name a later owner and close-before-publish is final
200 !== 503

$ env RESOLVER_PLANT=reset node --test --test-name-pattern='missing and mismatched' web/tests/resolver.test.mjs
not ok - missing and mismatched generation refuse every operation without writes
200 !== 503

$ env RESOLVER_PLANT=limits node --test --test-name-pattern='distributed guessing|active, reservation' web/tests/resolver.test.mjs
not ok - distributed guessing and owner limits include misses, retries and status
200 !== 503
not ok - active, reservation and memory caps fail before publication writes
200 !== 503

$ env RESOLVER_PLANT=partial node --test --test-name-pattern='every write boundary' web/tests/resolver.test.mjs
not ok - every write boundary is uncertain-safe and reconciles the same operation
publish/before/5: pending must be invisible
200 !== 503

$ env RESOLVER_PLANT=expiry node --test --test-name-pattern='expiry erases' web/tests/resolver.test.mjs
not ok - expiry erases actual secret while reservation permanently refuses reuse
1 !== 0

$ env RESOLVER_PLANT=reuse node --test --test-name-pattern='concurrent allocation' web/tests/resolver.test.mjs
not ok - concurrent allocation and publish-close races keep one immutable owner
actual: [200, 200]
expected: [200, 503]
```

Plants respectively reset a closed state to pending, remove the phrase
collision refusal, remove the generation guard, lift request/capacity ceilings,
let lookup read pending state, and remove the secret's EXAT. The concurrent
test runs two real Redis EVALs, not an in-memory mock transaction.

Two temporary handler plants were applied and then removed:

```text
# Plant: catch returns ended/200 instead of unavailable/503.
$ node --test --test-name-pattern='provider failures are' web/tests/resolver.test.mjs
not ok - provider failures are unavailable with one attempt and no false ended
200 !== 503
exit 1

# Plant: false && !verify(...) bypasses domain-signature verification.
$ node --test --test-name-pattern='authenticated lifecycle' web/tests/resolver.test.mjs
not ok - authenticated lifecycle, immutable binding, closure and no reopen
200 !== 503
exit 1
```

Two implementation omissions were caught before their fixes:

```text
# Before reserving monthly headroom for closing:
$ node --test --test-name-pattern='two close attempts' web/tests/resolver.test.mjs
not ok - new creation leaves two close attempts per active secret near monthly ceiling
200 !== 503
exit 1

# Before moving capacity-slot reservation ahead of the secret write:
$ node --test --test-name-pattern='pending secret reserves' web/tests/resolver.test.mjs
not ok - pending secret reserves capacity before the capability write
tentative capability must already occupy a slot
99 !== 100
exit 1
```

The second matters: invisible staged capabilities still consume storage and
must count toward the 100-secret ceiling. A retry can finish its own reserved
slot at capacity, but cannot allocate an extra one. The rate test's clock is
held at a real sampled Redis-compatible timestamp to avoid a minute boundary
changing its fixture halfway through the test.

## Green and command envelope

Redis reports `7.2.4`, AOF with `appendfsync always`, no TCP listener and
no eviction. The harness kills/restarts its Redis child and waits for AOF
recovery; closed reservations survive and the deleted capability stays absent.

```text
$ web/node_modules/.bin/oxlint resolver/handler.mjs resolver/store.mjs web/tests/resolver.test.mjs
exit 0 (no output)
$ node --test web/tests/resolver.test.mjs
# lifecycle command maximum: 16
# before/after all 11 lifecycle writes and lost commit replies: passed
# all 9 approved request ceilings plus status counting: passed
# tests 16
# pass 16
# fail 0
# skipped 0
# duration_ms 15906.977541
exit 0
```

The failure test injects before **and** after each of six publication writes
and five close writes, plus a response lost after each final commit. It reads
between failure and retry, checks pending invisibility/uncertain close, and
reconciles the same operation. Counts begin at one for EVAL itself and wrap
every inner `redis.call`, as the provider bills them. Worst successful fresh
publication is **16**:

```text
EVAL, GET generation, TIME, HGET id, MGET quotas,
SET global quota, SET subject quota, ZCOUNT active, HGET phrase,
HLEN reservations, INFO memory, HSET pending, ZADD slot,
SET secret EXAT, HSET open, GET committed secret
```

Lookup hit is 9, open replay/status 8, existing close 10, close-before-publish
12; pending recovery at a full active cap is at most 13, including ZSCORE.
No admitted path exceeds 16. Quota counters may conservatively consume an
attempt on uncertain failure; they never expose a partial invitation.

Creation stops at 100 active slots (tentative ones included), 100,000 hash
fields or 100 MB used memory. Counting fields is deliberately conservative:
a published reservation uses two, an unseen-close tombstone one. Existing
close/read operations do not require new-reservation capacity. New creation
stops at 2,800 monthly mutation requests, preserving 200 attempts (two for
each possible active secret) for close/recovery before the approved 3,000
ceiling. Status uses the lookup budget. Every boundary remains unavailable
with manual retry guidance; none triggers plan changes or automatic retries.

Provider fixtures include 429, 500, arbitrary non-success (401/509), HTTP-200
Redis command/day/month/bandwidth/storage errors, malformed/oversized JSON,
missing result, connection/DNS/TLS/timeout errors and a stalled response stream
that must be canceled by the actual eight-second whole-response deadline.
Ingress checks include oversized valid signed JSON with trailing whitespace,
wrong domain, changed signed bytes, missing trusted IP and a directory request.

## Wordlist provenance

Fetched the EFF source again with `curl -fsSL --max-time 45` and compared the
normalized words to `resolver/words.txt` using Node SHA-256 and deep equality:

```text
EFF source SHA-256: 8f5ca830b8bffb6fe39c9736c024a00a6a6411adb3f83a9be8bfeeb6e067ae69
1295 normalized words SHA-256: d2ee5adec14a5e9a63ab145e169bbb052091c4d8b5fae9d36a55099f76cf3072
vendored list matches source; only yo-yo excluded
exit 0
```

Attribution/license/removal notice are in `resolver/README.md`. The laws pin
the normalized hash, word count and a Rust `App` Ed25519 frozen vector
(`app_tests.rs:90-102`), and check generation from this vocabulary. Node
`crypto.randomInt` supplies unbiased cryptographic sampling. Collision refuses
one attempt without disclosing another invitation; explicit retry resamples.

## Live adapter smoke test

`live.mjs` reads the Keychain key internally. It uses the actual handler,
script and Free endpoint, remapping only fixture data keys under the existing
`{onboarding}` slot. The real generation is read, never initialized or changed.
Missing/mismatched generation is tested with an absent fixture key/a different
expected value. The synthetic token is not a usable invitation. Only fixture
keys are deleted; no namespace reservation is retired or reused.

Actual command, run successfully again after the final store changes:

```text
$ node demo/runs/onboarding/2026-11-04-resolver/live.mjs
live adapter: publish, lookup, idempotent retry, permanent reservation and expiring secret passed
live adapter: missing/mismatched generation refused all four operations
live adapter: close, ended guidance, replay refusal and secret deletion passed
live adapter requests: 14; no deployment or quota exhaustion
isolated fixture keys cleaned; real generation retained
exit 0
```

This proves REST/Lua adapter compatibility, including INFO memory and absolute
expiry. It is not a hosted crash/rollback/Free-exhaustion or people-scene run.
The earlier store survey's durability and hosting cost-boundary limits remain.

## Closing gate

Targeted resolver laws, lint and the live adapter proof have passed. The full
closing `scripts/check.sh` is still owed, to run once under
`lockf -k /tmp/svrngs-gate.lock`. It will be handed to the loop as a foreground
awaited command after this implementation/evidence commit; no gate success is
claimed here. The loop must resume this same unit, record its actual exit/log,
rerun either named known flake alone if needed, commit the closing evidence,
then report done. No other queue row was opened or checkbox edited.
