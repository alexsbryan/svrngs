# Onboarding store survey — 4 November 2026 UTC

Assigned unit: `onboarding-store-proof`, `ralph/next/onboarding/STATE.md:79-106`.
Operator-local date is 3 November; `date -u '+%Y-%m-%dT%H:%M:%SZ'` returned
`2026-11-04T06:16:33Z`. All URLs below were read on **2026-11-04 UTC**.
Base: `c7714367`; local primitive proof: `0f72a101`, [ATOMIC.md](ATOMIC.md).

**Result:** local atomic proof passed; documented Free-only cost boundary
established; self-hosting path established. Hosted failover/consistency and
exact Free bandwidth-exhaustion response are **could-not-judge**, not proved
by the local run. Phrase/abuse choices below are a recommendation awaiting
the operator, not authorization to implement or deploy the adapter.

## 1. Official evidence and its limits

These are excerpts/observations, not full-page archival copies. URLs are the
claim sources. `web-reader` read the pricing, REST, eviction, consistency,
installation and EFF pages. For pages where it timed out, a foreground
`curl -fsSL --max-time 45 <URL>.md` read the official Markdown successfully
as named below. No provisioning endpoint was called.

### Free plan and the cost boundary

| Source | Evidence read | Consequence |
|---|---|---|
| [Pricing & Billing](https://upstash.com/docs/redis/overall/billing) (`curl` `.md`, exit 0) | “Every account starts with a free database. No credit card required.” Free table: **$0/month, 250 MB, 500K commands/month, 10 GB/month bandwidth, 10,000 commands/s, 10 MB request, 100 MB record, 1 database**. | These are published limits, not measured usage or inspected account configuration. |
| Same billing page | “Adding your first payment method moves every free resource on the account to its paid plan”; “An account with a payment method cannot have free databases.” | Plan for a direct Upstash account with **no payment method**, not an already-paid account or a marketplace integration whose billing boundary has not been verified. Entering a card is a plan change, not a harmless prerequisite. |
| [Redis pricing](https://upstash.com/pricing/redis), [older pricing docs](https://upstash.com/docs/redis/overall/pricing) | Pricing FAQ says adding a card upgrades the database to PAYG. PAYG is $0.20/100K commands; paid usage is not covered by the Free allowance. Budget controls are **PAYG only**, with email at 70%/90% and rate limiting at cap. | A PAYG budget is not the chosen free-only control and no paid plan is approved. Billing docs also say budgets are per DB, not account-wide. |
| [Monthly command exhaustion](https://upstash.com/docs/redis/troubleshooting/max_requests_limit) (`curl` `.md`, exit 0) | Symptom: `ReplyError: ERR max requests limit exceeded.` Diagnosis: exceeds max **monthly** request count. Solutions: reduce commands or enter payment method to upgrade. | Exceeding the Free command allowance produces a refusal, not automatic paid overage without a payment method. A future resolver must report unavailable, not an empty/nonexistent invitation, on this error. |
| [Eviction](https://upstash.com/docs/redis/features/eviction) | “By default eviction is disabled” and writes are rejected at max data size. Enabling eviction can remove **nonvolatile keys too**, including durable storage. | **Eviction must stay disabled.** Exhaustion must refuse new storage, never delete reservations to make space. |
| [FAQ](https://upstash.com/docs/redis/help/faq) (`curl` `.md`, exit 0) | Free databases may be archived after **at least 30 days inactive**, with warning emails and a backup restorable through console. Also says excess free commands return exception. | Account owner must monitor mail and restore reservations before serving. Do not silently create a blank replacement under the old resolver address. The FAQ's “10,000/day” heading is stale relative to current billing/monthly-error pages. |

**Conflicting size figures:** older `/overall/pricing` says **256 MB** and
the public pricing comparison has 256 MB, while its FAQ/plan overview and
the newer `/overall/billing` say **250 MB**. Use **250 MB conservatively**;
this survey does not resolve the inconsistency by pretending all pages agree.

**What has not been observed:** no account, invoice, dashboard notification,
real quota crossing or Free bandwidth-limit response was inspected. The
pricing pages describe bandwidth blocking for *Fixed* plans, not explicitly
Free. Do not transfer that behavior to Free. Guessed official troubleshooting
paths `max_bandwidth_limit.md`, `max_monthly_bandwidth_limit.md`, and
`max_data_size_limit.md` returned curl exit 56 / HTTP 404; the specific Free
bandwidth error/status/reset and dashboard wording remain **could-not-judge**.
The monthly-error page documents the Redis exception, not its exact HTTP
status through REST or the command-counter reset timezone.

No-card Free is the published Upstash cost boundary, **not a guarantee of
availability or of the entire service's zero bill**. Vercel execution,
bandwidth and account billing are separate (`docs/ONBOARDING.md` §5); neither
account has been configured or verified here. Before any future deployment,
record the actual Free/no-payment/eviction-off settings and the hosting
account's own no-paid-usage boundary. No trial, integration, auto-upgrade,
Prod Pack or payment method is authorized by this survey.

### Atomic execution and durability

- [EVAL](https://upstash.com/docs/redis/commands/scripting/eval) (`curl` `.md`,
  exit 0) explicitly calls a script a **single atomic step** using a global
  lock by default, and documents `redis.call('TIME')`. Its REST section gives
  `POST <REST_URL>` with JSON `['EVAL', script, numkeys, ...keys, ...args]`
  (double-quoted JSON on the wire). This is sufficient documented transport
  for the exact local scripts' primitives; no provider SDK is necessary.
- [Compatibility](https://upstash.com/docs/redis/overall/compatibility)
  (`curl` `.md`, exit 0) supports both TCP and HTTPS REST and links EVAL's REST
  contract. [REST API](https://upstash.com/docs/redis/features/restapi) says
  `/pipeline` is **not atomic**; `/multi-exec` is atomic but command errors
  do not stop subsequent commands. A static transaction is not a substitute
  for conditional compare-and-close. Standard REST tokens have full DB
  privileges: keep credentials server-side, not in the group's page.
- [Durable Storage](https://upstash.com/docs/redis/features/durability)
  (`curl` `.md`, exit 0 after web-reader timeouts) says persistence is always
  enabled and writes go to memory and block storage; a restart reloads data.
  It distinguishes harmless memory eviction to disk from the destructive
  max-size eviction option above. Its final paragraph excludes Free from
  paid-tier extra replicated redundancy.
- [Consistency](https://upstash.com/docs/redis/features/consistency)
  (`curl` `.md`, exit 0) says **eventual consistency**, asynchronous replica
  propagation, LWW partition recovery, and causal guarantees on a single TCP
  connection. Its Markdown note says strong-consistency mode was deprecated.
  [Replication](https://upstash.com/docs/redis/features/replication) (`curl`
  `.md`, exit 0) starts with replication for paid databases, then says primary
  replicas are HA in all subscription plans. These statements do not establish
  a Free guarantee against an acknowledged reservation/close being lost at
  failover. A REST request is not a persistent TCP session.

**Required adapter consequence:** use EVAL for every reservation mutation and
an authoritative state-and-secret read; do not mix a stale read-region GET
with close. The official EVAL lock documents per-operation isolation. It
does **not**, alongside the conflicting replication statements, prove the
permanent no-reuse rule across hosted failure/recovery. Obtain provider
confirmation of the selected Free endpoint's acknowledged-write recovery
semantics before lifting the store stop line. The operator cannot convert
this unknown to a pass by choosing shorter tombstone retention.

`ATOMIC.md` contains the exact primitives, red and green commands/output,
Redis version, script hashes, absolute secret expiry and permanent
reservations after a forced local crash. No hosted compatibility, failover
or HTTP quota test was substituted for it.

### Command accounting matters

[Pricing & Billing](https://upstash.com/docs/redis/overall/billing) says EVAL
itself **plus each `redis.call`/`redis.pcall`** counts. A script containing GET
and SET counts as **3**, not 1. MULTI/EXEC adds 2 commands; pipelining saves
round trips, not allowance. The local scripts use at most **5** metered
commands for first create (EVAL + HGET + TIME + HSET + SET), **4** for replay,
and **4** for close. These counts follow the scripts, not measured provider
metrics. Quota bookkeeping and reads must also be counted in the adapter.

## 2. A real self-hosting path, not an implementation

[Official installation](https://redis.io/docs/latest/operate/oss_and_stack/install/install-stack/)
documents `brew install redis` on macOS, signed Redis APT packages on
Ubuntu/Debian, RPM, and a Docker image. The already installed local
`/opt/homebrew/bin/redis-server` and `redis-cli` report **7.2.4**; no installer
was run. [Redis Lua](https://redis.io/docs/latest/develop/programmability/eval-intro/)
(`curl` `index.html.md`, exit 0) guarantees script atomicity and conditional
multi-key operations. [Persistence](https://redis.io/docs/latest/operate/oss_and_stack/management/persistence/)
(`curl` `index.html.md`, exit 0) documents AOF, `appendfsync always`, backups,
and the possible loss from every-second fsync or snapshot-only recovery.

A self-hosted resolver can run its handler beside a single authoritative
Redis and execute these EVAL primitives over a private Unix/TCP connection.
Ordinary Redis does **not** implement Upstash's HTTPS API. Moving off Upstash
therefore needs a concrete handler/store transport change, not just an
environment URL swap; no provider framework is proposed or built here.

What the operator would still supply:

- DNS and public HTTPS/TLS termination and certificate renewal for the
  **resolver**, with Redis private; database auth/ACLs and protected secrets;
  authenticated domain-separated host requests and public arrival limits.
- Durable volume, AOF/fsync policy and no-eviction configuration; refuse on
  storage failure, monitor capacity, and retain reservations without TTL.
- Encrypted backups and restore drills. An old snapshot can forget a closed
  phrase: reconcile all reservations/closes before reopening the same URL
  namespace, or retire that namespace permanently. Backups containing expired
  secrets need retention/erasure policy too; live-key TTL is not backup erasure.
- An operating owner for patching, alerting, capacity, abuse handling, incident
  recovery and bills. Self-hosting is an exit path, not zero operating cost.

These are requirements derived from ONBOARDING §3's never-reuse/close rules,
not claims that TLS, backups or a self-hosted deployment have been tested.

## 3. Paired recommendation for the operator

### Phrase and reservation

Recommend **five independent words** joined with `-`, sampled uniformly with
the OS/browser CSPRNG using rejection sampling (no modulo bias), from EFF's
**Short Wordlist 1**, excluding only `yo-yo` so every word is `[a-z]{3,5}`.
No adjective/noun grammar, truncation, fuzzy matching or human-chosen phrase.
At most **29** path characters, including four separators.

Source: [EFF's explanation](https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases),
[actual list](https://www.eff.org/files/2016/09/08/eff_short_wordlist_1.txt),
[EFF copyright](https://www.eff.org/copyright). EFF's original material is
**CC BY 4.0 unless otherwise noted**; preserve attribution, source, licence
link and the removal notice when vendoring. This is a reviewed-for-generation
list, not a claim that every word is culturally neutral or usability-tested
for this welcome. No list is vendored into product by this row.

Actual downloaded-source inspection (`curl -fsSL --max-time 45` of the list,
piped to Python SHA-256, split/regex/count/math), exit 0:

```text
source bytes: 13660
source sha256: 8f5ca830b8bffb6fe39c9736c024a00a6a6411adb3f83a9be8bfeeb6e067ae69
words: 1296 unique: 1296
excluded: ['yo-yo']
kept: 1295 length: 3 5
space: 3642074641559375 bits: 51.69368191286958
```

Entropy = `5 log2(1295)` = **51.69 bits**; space `N = 1295^5` above. Sampling
must retry on a reserved collision; at 100,000 reservations the chance of at
least one candidate collision is at most `100000*99999/(2N) = 1.373e-6`.
An atomic reservation conflict must never replace the old identity.

Invitation secret lifetime: the **existing one day**, with its absolute
expiry; retries never extend it. Reservation/tombstone lifetime: **forever
within the resolver URL namespace**, no TTL, no eviction, no reuse after a
database reset or archival. Retain owner/publication/digest/expiry binding,
not the invitation capability or group name. Close deletes the live secret;
expiry removes it via TTL. Native closure must still invalidate the original
long invitation before claiming admission closed (ONBOARDING §3).

### Initial limits — exact proposed values, not yet code

| Limit | Proposed choice |
|---|---|
| Public lookup | Per trusted ingress IP: **30/minute, 300/day**; global **1,000/day, 20,000/calendar month**. Include misses, expired/closed addresses and status lookups; no public list/scan. |
| Host mutations | Verify signature before mutation; at most **5 new publications/owner/day**, **10 mutation attempts/owner/minute**, **60/hour**. Global **100 new publications/day**, **3,000 admitted mutation requests/month**, including retries/closes. Prioritize closure/status over new creation as the limit approaches; a failed withdrawal remains visibly pending. |
| Active secrets | **100 global**, bounded one-day expiry. This bound must count all active publications, not a client report. |
| Permanent reservations | Stop new allocation at **100,000**; also stop earlier on a conservative **100 MB** data watermark. Keep reads/closure available where possible. Never evict old reservations or silently reset. |
| Payload | **8 KiB** public/host HTTP request body and **8 KiB** response; bounded signature fields, exact phrase syntax, no invitation bodies in logs. |
| Metered command envelope | Target **at most 16 commands per admitted request**, including EVAL bodies and distributed quota bookkeeping. Prove actual worst-case count before adapter landing; if it exceeds 16, reduce request allowance or return for a paired decision. |

Use trusted ingress addresses only, not caller-supplied forwarded headers.
Counters must be shared/atomic across handler instances; process-local limits
or an IP-only limit do not establish the global guessing bound. The limits
are for an initial small service, not a claim of production capacity.

With **100** active phrases and **20,000** lookup guesses/month, the union
bound on any guessed live phrase is `Q*A/N = 20000*100/3642074641559375`
= **5.49e-10/month**, conditional on uniform sampling and globally enforced
limits. One targeted phrase is at most **5.49e-12/month**. This ignores leaks
through recipients/history and does not make a public bearer link private.
Rate limits protect guessing; signatures do not stop a person with many keys
from exhausting publication capacity. Rejecting collisions must not reveal
another invitation's metadata.

Allowance arithmetic: `(20000 + 3000) * 16 = 368,000` commands/month, leaving
**132,000** of the published 500,000 for rejected requests, operations and
recovery. At an estimated **32 KiB total store traffic/admitted request**
(scripts/metadata included), `23000 * 32768 = 753,664,000` bytes, below 10 GB.
At **512 bytes/reservation** planning allowance, 100,000 reservations use
**51.2 MB**, before other data/overhead. The local tiny fixture measured
**136 bytes** for one reservation only; neither 512 bytes nor 32 KiB is a
measured Upstash bound. Provider metrics must confirm them before deployment.

**Limits do not guarantee staying inside Free under attack.** A rejected
request that reaches Redis still consumes limiter commands, and a rejected
request reaching Vercel can consume execution/bandwidth. Enforce cheap
syntax/body/method/edge rejection before Redis; meter the remaining refusals;
fail unavailable when Free exhausts. The no-payment account is the cost
boundary; quota exhaustion/denial of new arrivals is an explicit tradeoff.
Do not claim an unlimited-traffic zero-bill guarantee. Native members and
their record/folders continue without the resolver (ONBOARDING §3).

## 4. Handoff and unchanged inherited work

`ralph/next/onboarding/NEEDS_HUMAN.md` carries the decisions requested by
the row. No state checkbox was edited and no other queue row was opened.
`scripts/check.sh` is unchanged and was not required/run for this survey
(assigned row:104-106); the lean count is unchanged in `ATOMIC.md`.

Inspected inherited `demo/runs/v1.6/2026-11-03/publish.json:1-6`: source
`75f9fd3b`, scratch `/private/tmp/svrngs-release-v1.6`, state `running`.
`publish.log:1326-1352` ends during compilation, not with a publication
verdict. No inference that it finished; both files remain untouched and
untracked, per explicit operator instruction. Nothing was deployed, published
or pushed in this row.
