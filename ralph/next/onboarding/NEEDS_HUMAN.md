# Onboarding's operator decisions

## Store proof — option A chosen; official provider answer pending

`onboarding-store-proof`, read on **2026-11-04 UTC / 2026-11-03 local**.
The assigned row requires a stop here before an adapter. Local proof committed
in `0f72a101`; source excerpts, uncertainty and arithmetic:
`demo/runs/onboarding/2026-11-04-store/README.md`. Actual planted-red and
atomic-green runs: that directory's `ATOMIC.md`. No product lines changed.

### The operator's answers, 2026-11-04

The operator: "Let's go with A. The rest is reasonable too" and "We should
fail gracefully in the case we hit any of those limits."

1. **Option A.** Retain Upstash Redis Free and obtain the official written
   answers below before the adapter. No provisioning or payment is needed for
   the inquiry. The adapter stays held; native publication and host controls
   may proceed independently.
2. **The paired values are approved as proposed** under “Exact proposed
   choices”: EFF Short Wordlist 1, five words (51.69 bits), 30/min and
   300/day per IP, 20k/month global, 100 active secrets, 100k reservations or
   100 MB, 8 KiB bodies, one-day invitations.
3. **Every limit fails gracefully.** Commands/day/month, bandwidth, storage,
   request size, rate limits, timeouts and 5xx/429 show “joining is
   temporarily unavailable” with retry guidance; they never look like an ended
   invitation. Existing members are unaffected, no partial publication or
   close, no paid upgrade, no tight retry loop, and the host's pending receipt
   and underlying invitation stay recoverable. The policy handles any
   non-success, so it does not depend on undocumented provider codes.

The exact inquiry to send is `ralph/next/onboarding/PROVIDER_INQUIRY.md`.

### Director decision — operator fork, not approval

Reviewed on **2026-11-04 UTC**, against `fad8d8a8`, the assigned row and
`ctl/log/sessions/onboarding-store-proof-1.out:3278-3320`. **Keep the store
stop line.** The charter supplied to the director reserves phrase space and
request limits to the operator; the current `CHARTER.md:33-38,62-64` likewise
requires storage proof before an adapter, and STATE's (d) explicitly requires
the operator's answer. The earlier Free planning approval does not supply that
answer or establish hosted recovery. No false ordering premise was found;
no source-order change is needed. This unit remains incomplete.

**Recommendation:** choose option A below and approve the exact paired values
under “Exact proposed choices” only after the storage questions are answered.
The arithmetic is reproduced, but it is conditional on enforcement, not a
hosted-store or usability result. Do not waive never-reuse to close the row.

| Operator option | Cost and consequence |
|---|---|
| **A — retain Free, obtain official clarification first (recommended)** | No provisioning or payment is needed for a documentation/support inquiry. Costs operator coordination and an unknown wait; adapter work remains held. Ask the operator to obtain or authorize obtaining the written answers below. Approval of the paired values alone does not lift the store stop. |
| **B — choose a self-hosted authoritative store instead** | Requires a new provider/operating-cost decision, an operating owner, durable storage, recovery protection and handler transport work. The survey §2 lists those obligations; no monetary estimate or deployment is approved. Re-prove the chosen boundary before its adapter. |
| **C — hold onboarding pending evidence** | No new service spend or implementation; delays the welcoming-room demo. Existing storage and paired-choice questions remain open. |

For A, obtain a dated official response or documentation for the **planned
direct no-payment Free REST database configuration**, not paid/Prod Pack:

1. Does successful EVAL acknowledgment mean both reservation and close state
   survive supported crash, failover and partition recovery? Can concurrent
   REST EVALs ever acknowledge incompatible owners, or recovery/LWW roll back
   a close/reservation? Specify the authoritative endpoint/read operation and
   required configuration. Per-script locking alone does not answer this.
2. What happens at **10 GB/month Free bandwidth**: Redis error and REST HTTP
   status/body, affected operations (including close), reset period/timezone,
   and whether any charge or automatic plan change can occur without a payment
   method? The inspected billing page lists the allowance but not these details.
3. How are permanent reservations recovered after archival/restore? If any
   acknowledged state can be lost, serving the same URL namespace cannot resume
   until complete reconciliation; otherwise retire it permanently. A new blank
   database is not recovery. An answer inconsistent with never-reuse requires
   option B or C, not an adapter built on that assumption.

**Director reproduction:** foreground runs of `python3 -I
demo/runs/onboarding/2026-11-04-store/proof.py` with `plant-create`,
`plant-close`, then `atomic` returned **1, 1, 0**. Outputs matched the transcript
below, including the 136-byte sample and AOF reload assertions; Redis **7.2.4**,
create SHA-256 `c6856e3d9e0bded82774dda871ef81aba0392e1c0514bf9b1957f256b2f1d119`,
close SHA-256 `946d932e04057095828876e64756c4bb7393e30d6ab4192e10ae2cb99192e3a5`.
All owned Redis children exited before the commands returned.

Re-read with `curl -fsSL --max-time 45` (all exit 0):
`https://upstash.com/docs/redis/commands/scripting/eval.md` (single atomic step,
global lock), `/docs/redis/features/consistency.md` (asynchronous propagation,
eventual consistency, LWW and deprecated strong mode),
`/docs/redis/features/durability.md` (persistent storage; extra replication
excludes Free), `/docs/redis/features/replication.md` (paid replication versus
“all subscription plans” HA), and `/docs/redis/overall/billing.md` (250 MB,
500K commands, 10 GB; no card required; adding payment upgrades Free; EVAL plus
inner calls metered). Paths after the first URL use the same Upstash origin.
These reproduce the recovery ambiguity; they do not establish a hosted failure
or guarantee. Free bandwidth response remains **could-not-judge** in these
sources. Neither account inspection nor hosted testing was performed.

Downloaded the EFF source URL in the survey with Python `urllib.request` and
recomputed SHA-256/count/math (exit 0): **13,660 bytes**, hash
`8f5ca830b8bffb6fe39c9736c024a00a6a6411adb3f83a9be8bfeeb6e067ae69`,
**1,296 unique words**, excluding only `yo-yo` leaves **1,295**, lengths **3–5**;
`1295**5 = 3642074641559375`, entropy **51.69368191286958** bits,
`20000*100/1295**5 = 5.491375649411975e-10`, command envelope **368000**,
reserve **132000**. This reproduces the numerical recommendation only;
account settings and the implementation's command envelope still need proof.

### Questions, in order

1. **Approve the paired phrase/abuse recommendation below, or give revised
   values?** This is the choice intentionally left open by CHARTER:37-38 and
   the assigned row:99-103. Shorter phrases or higher guess/active limits need
   the probability recalculated together, not independently.
2. **How should the remaining provider proof be obtained before the adapter?**
   Official EVAL docs promise a global atomic lock; official consistency docs
   say eventual consistency/LWW and deprecate strong mode. Free redundancy
   statements conflict. The local crash proof cannot certify that a hosted
   acknowledged reservation/close is never lost at recovery. The exact Free
   bandwidth-exhaustion response is also undocumented in the pages read.
   Recommend obtaining official confirmation for the chosen Free endpoint
   and its no-reuse recovery semantics, plus bandwidth-limit behavior, before
   lifting the existing store-proof stop line. If those guarantees cannot
   be supplied, storage needs a new operator decision. This is not a request
   to weaken never-reuse or to provision a database for experimentation.

### Exact proposed choices — no answer yet

- **Words:** EFF Short Wordlist 1, CC BY 4.0, source and hash in the survey;
  exclude only `yo-yo`, leaving **1,295** words. **Five** independent uniform
  CSPRNG samples, rejection sampling, hyphen joined; **51.69 bits**, at most
  **29** path characters. No fuzzy lookup or human-picked phrase.
- **Lifetime:** existing **one day** for invitation secrets, absolute expiry,
  no refresh on retry. Reservations **permanent for the URL namespace**;
  no TTL/eviction/reuse, including after reset/archival/restore.
- **Lookup attempts:** per trusted ingress IP **30/minute, 300/day**; global
  **1,000/day and 20,000/calendar month**, including misses/status reads.
- **Host attempts:** authenticated; **5 new publications/owner/day**,
  **10 mutations/owner/minute, 60/hour**; global **100 new/day**, **3,000
  admitted mutations/month** including retries/closes. Prioritize closure
  over creation; a failed withdrawal stays visibly pending.
- **Capacity:** **100 active secrets**; stop new allocation at **100,000
  permanent reservations or 100 MB data**, whichever comes first. Keep old
  reservations intact. HTTP request/response bodies at most **8 KiB** each.
- **Command envelope:** prove **at most 16 metered commands/admitted request**
  including Lua internals and shared atomic limit bookkeeping, or revisit
  the paired limits. `(20000+3000)*16 = 368000` leaves **132000/month** for
  refusals/recovery/operations; those reserves can still exhaust under attack.
- **Proposed tradeoff:** direct Upstash **Free with no payment method**, no
  eviction/paid upgrade; new short-link arrivals become unavailable on quota
  exhaustion, existing native members continue. Global quotas and cheap edge
  rejection are required, not process-local counters. Free is a spend boundary,
  not an availability guarantee. Actual account settings, hosted recovery
  semantics and Vercel's separate no-paid-usage boundary remain predeployment
  proof obligations; no deployment is authorized.

At these limits, `20000*100/1295^5 = 5.49e-10` is the monthly union bound
for guessing any live phrase, conditional on uniform generation and enforced
global limits. Provider official billing counts EVAL **plus its inner
commands**. Storage and bandwidth estimates in the survey are planning
estimates, not provider measurements.

### What ran and what remains unknown

```text
python3 -I demo/runs/onboarding/2026-11-04-store/proof.py plant-create
create race: ['created', 'created']
failed: create-if-absent acknowledged two owners
exit: 1

python3 -I demo/runs/onboarding/2026-11-04-store/proof.py plant-close
close race: retry=replayed, state=open, secret_present=True
failed: delayed non-atomic retry reopened closed phrase
exit: 1

python3 -I demo/runs/onboarding/2026-11-04-store/proof.py atomic
create race: ['conflict', 'created']
close race: retry=closed, state=closed, secret_present=False
passed: idempotent create, immutable binding, no TTL refresh, compare-and-close
passed: actual TTL expiry erases secret; permanent reservation refuses replay/reuse
passed: close-before-create, duplicate close, stale close vs newer invitation, late expiry
passed: AOF reload after SIGKILL retains expired and closed reservations
sample reservation MEMORY USAGE: 136 bytes
passed: local atomic store proof
exit: 0
```

`ATOMIC.md` pastes complete version/hash-prefixed outputs and named plants.
Redis was already installed; one owned child with Unix socket only, AOF
`appendfsync always`, no eviction, stopped before each foreground run returned.
Official install and persistence URLs establish the self-hosting path; a
self-hosted handler still needs TLS, auth, private Redis, durable volume,
backups/restore protection and an operating owner. Ordinary Redis is not an
Upstash REST endpoint, so this is not a URL-only migration promise.

Official pricing/billing/monthly-error/eviction sources were read successfully
via web-reader or `curl` of official Markdown. They establish **500K
commands/month, 10 GB/month, conservatively 250 MB** (some pages say 256 MB),
no-card Free and command-limit refusal. No account settings, real quota event
or hosted failover was inspected. Specific guessed bandwidth troubleshooting
pages returned 404; Free bandwidth error/status/reset remains could-not-judge.
The survey names all URLs and access failures rather than treating a failed
read as confirmation. No Upstash or Vercel resources were created.

`python3 scripts/lean_check.py`: exit 0, unchanged **3,477 core / 19,441 `.` /
7,499 `js`**. `git diff -- scripts/check.sh`: no output. The assigned
row:104-106 explicitly does not require the full gate for this survey; it was
not run. The inherited V1.6 `publish.json` and `publish.log` were inspected,
are inconclusive about completion, and remain untouched/untracked by explicit
instruction. No checkbox, publication, push, product file or budget changed.

## Economizing pass — answered previously

Status: **answered by the operator on 2026-11-03.** The economizing pass is
complete; survey, revised estimate and passing full closing gate are recorded.
The unchanged retry passed in 1,141 seconds, exit 0
(`demo/runs/economize/2026-11-03-onboarding/data/check.txt`). The first
failure remains retained in `4555a32e`.

## The operator's answers

The operator: "Budget approved and we can plan on a free tier upstash deploy
(but again we don't deploy now)".

1. **Budget ceiling: approved at the upper estimate.** The caps are now
   3,494 core, 19,833 `.` and 8,090 `js`, landed in `scripts/lean_check.py`
   from the operator seat. Each future landing still raises only to where it
   lands.
2. **Resolver plan: Upstash Redis Free.** No paid overage is authorized, and
   nothing is deployed now. Before an adapter, `onboarding-store-proof`
   verifies that the free tier's limits and account controls prevent paid
   use, that reservation/closure are durable and atomic, and that a
   self-hosting path exists. The adapter is not built until that row passes.

Phrase-space size and request limits remain a paired decision after storage
behavior is understood (CHARTER:37-38); this answer does not choose them.

## The questions as asked

1. **What budget ceiling may onboarding spend?** The current-base survey
   (`194aa994`, `demo/runs/economize/2026-11-03-onboarding/SURVEY.md`)
   selected no product cut. `docs/ONBOARDING.md` §4 now estimates
   **+230–391 non-JS**, **+345–590 JS**, with **+0–16 core included in
   non-JS**. If the operator funds the upper estimate, the one-past ceilings
   would be **19,833 `.`**, **8,090 `js`**, **3,494 core** (increases of
   391, 590 and 16 over current caps). These are proposed ceilings, not edits
   or approval. Each future landing would raise only to where it lands.
2. **Which resolver storage/provider and operating-cost limit should we
   use?** ONBOARDING §5's provisional candidate is Vercel plus Upstash
   Redis; no database is provisioned or authorized by this pass. Choose it
   only after confirming the account's cost cap, durable atomic reservation/
   closure operations and self-hosting path. Alternatively name the chosen
   self-hosted transactional store and where the resolver will run. No
   provider abstraction or adapter has been built.

## What was tried and why it did not pay for the feature

- Surveyed current reused invitation, signature, HTTPS-client, secret-file,
  link/name/Copy, installer and folder/open surfaces. Exact sites and
  per-file counted lines are in the committed survey and `data/sites.txt`.
- The latest applicable compiler demotion, test-cfg, line-window and page
  name/structural instruments ran. Native and wasm scratch checks passed.
  Public test helpers are still used; moving inline tests would only alter
  accounting; similar room/Play and nonce/redirect code has distinct policy.
  No behavior-preserving product deletion was established. All existing
  tests and all caps remain unchanged.
- Semantic clone analysis **could-not-judge**: installed binaries say the
  developer-only command is not available. This is not a manual clone report
  or evidence of zero clones (`data/clone-availability.txt:2-20`).
- The shared HTTPS client extraction already landed (`http.rs:10-23`,
  `467d5d16`); no further extraction saving is available. Native status,
  persistent receipt, authenticated publication and replay-safe closure do
  not exist yet (`invite.rs:151-159`, `home.rs:597-628`). The current lookup
  is newest for a key (`svrngs-runtime/src/lib.rs:227-247`), so stale close
  protection cannot be assumed from it.
- Installer recovery is still actual work: stop at `web/install.sh:113-116`,
  join/refusal at :154-158, restart at :197. Changing that path is behavior
  work, not a saving claimed by this pass.

## Commands and actual output

`python3 scripts/lean_check.py` before survey (`75f9fd3b`) and again after
survey (`194aa994`), both exit 0:

```text
crates/svrngs-core     3477 / 3478 lines
.                     19441 / 19442 lines
js                     7499 / 7500 lines
swift                     0 / 1 lines
```

`cargo check --workspace --offline`, exit 0
(`demo/runs/economize/2026-11-03-onboarding/data/check-workspace.txt`):

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
```

The scratch's successful final commands were `cargo check --workspace
--offline --message-format=json` (native, round 24) and `cargo check
--target wasm32-unknown-unknown --offline --message-format=json` (web,
round 5). Both warning summaries end:

```text
build-finished success: True

exit: 0
```

The original count and all run files are under
`demo/runs/economize/2026-11-03-onboarding/data/`. The survey driver reuses
the earlier instruments unchanged; no new gate was added.

### Closing gate — first failure retained, unchanged full retry passed

The first `measure.py close` completed in 602 seconds, exit 1. Its complete
output is `demo/runs/economize/2026-11-03-onboarding/data/check-first.txt`
(committed in `4555a32e`), including:

```text
failed     tests (5 s)
failed     browser (161 s)
never-ran  scene give (chromium): Error: unable to connect to Podman socket: failed to connect: dial tcp 127.0.0.1:62636: connect: connection refused
never-ran  scene give_gone (chromium): Error: unable to connect to Podman socket: failed to connect: dial tcp 127.0.0.1:62636: connect: connection refused
never-ran  scene give_move (chromium): Error: unable to connect to Podman socket: failed to connect: dial tcp 127.0.0.1:62636: connect: connection refused
note       took 602 s in all
exit: 1
```

Seven Give tests refused to run without Podman (:51-100). Chromium failed
`the_home_page_lends_what_runs_here` at `tests/browser/main.rs:4574`, checking
the final chores lending redraw (:891-892). Its captured trace ended while
the final LENDING read was still in flight (:237-244); this suggests a
snapshot timing issue, but does not establish its cause.

Prerequisite recovery and focused retry, without product or test edits:

```text
$ podman machine start
Starting machine "podman-machine-default"
Machine "podman-machine-default" started successfully
$ podman info --format '{{.Host.Security.Rootless}}'
true
$ cargo test --quiet --workspace --test browser the_home_page_lends_what_runs_here -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 8.22s
```

All three commands exited 0. The isolated retry's trace completed with
`LENDING: answered (2 lent)` and the chores row present. One passing retry
does not resolve the initial failure or replace the full gate.

The full retry ran through the queue's await facility:

```sh
python3 -I demo/runs/economize/2026-11-03-onboarding/measure.py close
```

The driver ran `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh`,
preserving its complete output and exit code in `data/check.txt`, and
recorded the unchanged final recount in `data/lean-after.txt` (exit 0).
The loop reported the driver exited 0 after 19m01s; its session log is
`ctl/log/sessions/onboarding-economize-2.await.log`. Actual gate output
(`data/check.txt:4-40,71-74`, selected verdicts):

```text
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (1 s)
passed     trust floor (0 s)
passed     advisories (1 s)
passed     tests (493 s)
passed     ceiling canary (4 s)
passed     browser (156 s)
passed     types (2 s)
passed     typed lint (1 s)
passed     joining in a browser (43 s)
passed     pages (61 s)
passed     scenes (234 s)
passed     scene give (chromium)
passed     scene give_move (chromium)
passed     scene give_gone (chromium)
passed     conformance canary (91 s)
passed     app laws (41 s)
note       took 1141 s in all

exit: 0
```

Every gate passed, including all Give scenes. No product or test edits were
made between the failed gate and this retry. This closes the economizing
pass; it does not establish the cause of the first browser failure or judge
the future real-laptop onboarding scene. The two operator decisions above
are answered; storage proof remains the adapter's stop line.

## Storage evidence and remaining confirmation

The inherited price check in `194aa994:docs/ONBOARDING.md:179-185` cites
<https://upstash.com/pricing/redis>: one free database, 250 MB and 500,000
commands/month; usage commands $0.20/100,000, with Vercel execution and
bandwidth separate. Those are published-plan figures, not this account's
limits or a measured bill. In this session a web-reader recheck of that URL
and <https://upstash.com/docs/redis/features/restapi> both timed out; current
cost and atomic REST operations therefore remain **could-not-judge** here.

The official Redis install page was readable:
<https://redis.io/docs/latest/operate/oss_and_stack/install/install-stack/>
documents Docker, Linux and macOS installation. This establishes a possible
self-hosted database, not an already-compatible self-hosted resolver. A
resolver transport, persistence policy, operational owner and equivalent
atomic/replay behavior still need confirming before choosing the adapter.

Required store behavior is `docs/ONBOARDING.md:78-86`: recover lost replies,
create and close atomically, retain non-reusable reservations after erasing
expired invitation secrets, and never reopen on replay. A free allowance
does not prove these semantics; an in-memory code store cannot supply them.
