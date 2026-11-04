# Local atomic storage proof

Run on 2026-11-04 UTC (2026-11-03 operator local date), base `c7714367`.
This is onboarding-store-proof (b): an experiment, not resolver product code.
Uses the already installed Homebrew Redis 7.2.4 and Python's standard library;
no installation, dependency, hosted database or external connection is needed.

## What is proved

`proof.py` starts and owns one Redis child with TCP disabled, a private Unix
socket in a temporary directory, `appendonly yes`, `appendfsync always` and
`maxmemory-policy noeviction`. It waits for shutdown and removes its temporary
data on every normal exit. Each client call is a separate redis-cli process.

The two Lua scripts access only their two declared `KEYS`: permanent
`proof:{phrase}:reservation` (a hash) and expiring `proof:{phrase}:secret`
(a string). The shared hash tag also keeps both keys in one Redis Cluster
slot. Both operations run in a single EVAL, not a client read followed by
a write and not a pipeline.

- **Create if absent:** write the reservation and secret together. An existing
  reservation accepts only the identical binding and never rewrites the
  secret. A retry reports replayed, expired or closed. It never extends expiry.
- **Compare and close:** only the same immutable binding closes the reservation
  and deletes the invitation. A different owner/publication returns conflict.
  Close arriving before a delayed create reserves the phrase closed.
- **Expiry:** only the secret has `PXAT`, an absolute deadline. The reservation
  has no TTL. Expired or closed phrases can never be allocated to another
  identity, even after SIGKILL and AOF reload in this run.

The identity strings here are fixtures. The future authenticated boundary
must derive the binding from owner key, publication id, invitation digest and
absolute expiry, checking all fields and signatures before EVAL. In particular,
it must not allow one identity to be paired with changed bytes/deadlines. These
scripts are not a signature verifier, native invitation invalidator, HTTP
handler, quota implementation or deployable adapter. They are valid-input
primitive proofs, not a proof against malformed Lua arguments or storage errors.
Redis scripts isolate operations but do not roll back writes after a runtime
error: any future reader must check reservation state as well as the secret;
any error is an uncertain result requiring an idempotent status/retry.

## Instrument validation: planted red, then green

Actual commands and complete output follow. The two plants exit **1** at the
assertion of the required invariant, not because Redis is absent. Missing
tools/startup/transport errors are **could-not-judge**, exit 2. No zero-test
success path is offered. These are deterministic schedules, not race rates.

`python3 -I demo/runs/onboarding/2026-11-04-store/proof.py plant-create`
— exit **1**:

```text
Redis server v=7.2.4 sha=00000000:0 malloc=libc bits=64 build=2d86b7859915655e
create.lua sha256=c6856e3d9e0bded82774dda871ef81aba0392e1c0514bf9b1957f256b2f1d119
close.lua sha256=946d932e04057095828876e64756c4bb7393e30d6ab4192e10ae2cb99192e3a5
create race: ['created', 'created']
failed: create-if-absent acknowledged two owners
```

The plant puts a barrier between two clients' absence reads and unconditional
writes. Both acknowledge creation; the assertion demands exactly one winner.

`python3 -I demo/runs/onboarding/2026-11-04-store/proof.py plant-close`
— exit **1**:

```text
Redis server v=7.2.4 sha=00000000:0 malloc=libc bits=64 build=2d86b7859915655e
create.lua sha256=c6856e3d9e0bded82774dda871ef81aba0392e1c0514bf9b1957f256b2f1d119
close.lua sha256=946d932e04057095828876e64756c4bb7393e30d6ab4192e10ae2cb99192e3a5
close race: retry=replayed, state=open, secret_present=True
failed: delayed non-atomic retry reopened closed phrase
```

The plant pauses a publication retry after it reads open, completes close,
then lets that stale retry unconditionally rewrite the secret and open state.

`python3 -I demo/runs/onboarding/2026-11-04-store/proof.py atomic`
— exit **0**:

```text
Redis server v=7.2.4 sha=00000000:0 malloc=libc bits=64 build=2d86b7859915655e
create.lua sha256=c6856e3d9e0bded82774dda871ef81aba0392e1c0514bf9b1957f256b2f1d119
close.lua sha256=946d932e04057095828876e64756c4bb7393e30d6ab4192e10ae2cb99192e3a5
create race: ['conflict', 'created']
close race: retry=closed, state=closed, secret_present=False
passed: idempotent create, immutable binding, no TTL refresh, compare-and-close
passed: actual TTL expiry erases secret; permanent reservation refuses replay/reuse
passed: close-before-create, duplicate close, stale close vs newer invitation, late expiry
passed: AOF reload after SIGKILL retains expired and closed reservations
sample reservation MEMORY USAGE: 136 bytes
passed: local atomic store proof
```

The green uses the same invariant assertions with EVAL replacing the planted
client read/write sequence. Lifecycle assertions additionally check a foreign
close cannot erase the current secret, real TTL removal, close-before-create,
duplicate close, expired-first-create refusal, and closed/expired reservations
surviving a forced crash. This one local crash says nothing about Upstash HA,
provider quotas, power loss, old-backup recovery or a people acceptance rate.

## Reuse and scope

The existing feature inventory is `docs/ONBOARDING.md` §4 and
`ralph/next/onboarding/CHARTER.md:46-54`, including the surveyed commonwealth
ring surfaces at `e1a43bcd2`. Current-tree search
`grep` tool pattern `Redis|redis|reservation|tombstone` in `*.py` found only
unrelated charter-reservation text in the preshift driver. The existing
invitation/signature/secret-file surfaces implement neither this hosted store
nor persistent phrase reservations; no product module or type is introduced.

`python3 scripts/lean_check.py` — exit **0**, after adding this experiment:

```text
crates/svrngs-core     3477 / 3494 lines
.                     19441 / 19833 lines
js                     7499 / 8090 lines
swift                     0 / 1 lines
```

`git diff -- scripts/check.sh` produced no output, exit 0. The assigned row's
check explicitly exempts this survey from the full gate; `scripts/check.sh`
was not run here. Its last full run is the prior economizing pass, not a
new result for this work. Product files, size caps and checks are unchanged.
