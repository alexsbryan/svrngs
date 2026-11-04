# Generation guard — 4 November 2026

Completes `onboarding-store-proof` (f), against base `a13f027e`. These are
recorded experiments, not resolver product code or a public deployment.
The earlier atomic red/green transcripts remain in [ATOMIC.md](ATOMIC.md);
the unchanged `atomic` experiment passed again in this session.

## Contract and scope

The live namespace now has permanent key `svrngs:{onboarding}:generation`,
value `onboarding-2026-11-04-v1`. The value is a public deployment pin, not a
credential. The future resolver must pin that expected value outside Redis.
It must prefix **every** publication, lookup, close and status operation with
`generation.lua` in the **same EVAL**, before accessing invitation state.
A separate GET preflight would permit reset between the check and mutation.
All keys use the same `{onboarding}` hash tag, including the generation key;
the earlier per-phrase hash tags must not be carried into that adapter.

Missing or mismatched generation returns `unavailable`, with no phrase
allocation, secret read, close or generation initialization. A confirmed
ended invitation is distinguishable only after a matching generation and a
successful state-and-secret read (`lookup.lua`). Provider failures follow
[FAILURES.md](FAILURES.md); an unavailable read is never an ended invitation.

Only the operator's initial commissioning creates this key. The service must
have **no** seed-if-missing or automatic recovery path. After any suspected
reset, keep service unavailable until the operator reconciles reservations
and closes, or retires the URL namespace. Restoring a generation key alone is
not reconciliation. Never rerun the initialization command below as recovery.

**Limit of this instrument:** a generation sentinel detects a blank store,
a lost sentinel, or the wrong deployment pin. It cannot detect selective loss
of a reservation or restoration of an old snapshot that still contains the
sentinel. It does not settle the hosted failover unknowns in README §1. Any
known/suspected rollback requires the same operator reconciliation. No hosted
crash, rollback, quota exhaustion or partition was induced here.

## Planted red, then green

`proof.py` uses its existing owned local Redis child and cleanup, no TCP
listener. The plant omits only the Lua guard. Both runs erase a fixture
reservation **and** generation, then try the old phrase with a different
owner. Both assert that this request must be unavailable. The green also
checks lookup/close, mismatched generation, zero writes on refusal, and the
ordinary lifecycle with the matching generation. Local fixture restoration
is test setup, not an automatic service behavior.

Actual command:
`python3 -I demo/runs/onboarding/2026-11-04-store/proof.py plant-reset`
— exit **1**:

```text
Redis server v=7.2.4 sha=00000000:0 malloc=libc bits=64 build=2d86b7859915655e
create.lua sha256=c6856e3d9e0bded82774dda871ef81aba0392e1c0514bf9b1957f256b2f1d119
close.lua sha256=946d932e04057095828876e64756c4bb7393e30d6ab4192e10ae2cb99192e3a5
missing generation: publication=created
failed: missing generation reopened an old phrase
```

Actual command:
`python3 -I demo/runs/onboarding/2026-11-04-store/proof.py reset`
— exit **0**:

```text
Redis server v=7.2.4 sha=00000000:0 malloc=libc bits=64 build=2d86b7859915655e
create.lua sha256=c6856e3d9e0bded82774dda871ef81aba0392e1c0514bf9b1957f256b2f1d119
close.lua sha256=946d932e04057095828876e64756c4bb7393e30d6ab4192e10ae2cb99192e3a5
missing generation: publication=unavailable
missing generation: lookup=unavailable
missing generation: close=unavailable
passed: missing/mismatched generation refuses publication, lookup and close without writes
passed: matching generation permits lifecycle; generation has no TTL
passed: local atomic store proof
```

## Live commissioning and guard proof

The operator-provided endpoint is the one in `live.txt`. The driver reads
Keychain internally; no key is printed, passed as a command-line argument or
stored in this repository. This authorized store proof initializes a key in
the existing database; it provisions/deploys no database or public handler.

Actual **one-time commissioning** command:
`python3 -I demo/runs/onboarding/2026-11-04-store/live.py --generation-proof --initialize-generation`
— exit **0**:

```text
generation initialization: created
live namespace generation: matched; TTL=-1
live missing generation: create=unavailable
live missing generation: lookup=unavailable
live missing generation: close=unavailable
live mismatched generation: all operations unavailable
live guarded lifecycle: created, resolved, closed, replay refused; reservation TTL=-1
cleaned fixture keys; permanent namespace generation retained
live generation proof passed
```

Actual **repeat verification** command (does not initialize):
`python3 -I demo/runs/onboarding/2026-11-04-store/live.py --generation-proof`
— exit **0**:

```text
live namespace generation: matched; TTL=-1
live missing generation: create=unavailable
live missing generation: lookup=unavailable
live missing generation: close=unavailable
live mismatched generation: all operations unavailable
live guarded lifecycle: created, resolved, closed, replay refused; reservation TTL=-1
cleaned fixture keys; permanent namespace generation retained
live generation proof passed
```

The live missing-generation test substitutes an absent **fixture** generation
key; it never deletes or changes the real sentinel. The mismatch test changes
only the request's expected value. Only synthetic phrase keys are deleted.
The second command confirms the real generation survives fixture cleanup.

The original live atomic smoke test is still `live.py` without flags, with
its already recorded output in `live.txt`. No provider exhaustion test is
claimed by either live run.

## Closing verification

`python3 -I demo/runs/onboarding/2026-11-04-store/proof.py atomic`
— exit **0**, unchanged assertions and Lua sources:

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

`python3 scripts/lean_check.py` — exit **0**:

```text
crates/svrngs-core     3486 / 3494 lines
.                     19739 / 19833 lines
js                     7499 / 8090 lines
swift                     0 / 1 lines
```

`git diff --check` and `git diff -- scripts/check.sh` — both exit **0**,
no output. No product file or size cap changed. The assigned row's check
(`STATE.md:122-127`) explicitly exempts this survey from the full gate;
`scripts/check.sh` was unchanged and not run. No browser test was needed.
The inherited V1.6 publish artifacts remain untouched/untracked. No push,
publication, public deployment or paid upgrade was performed.
