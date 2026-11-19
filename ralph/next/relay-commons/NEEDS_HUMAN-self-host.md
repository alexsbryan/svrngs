# self-host — no commit can pass: the trust floor refuses every commit while webrtc-network's floor change sits uncommitted

> **Landed 2026-11-19 (self-host-2):** (a) b3ae73f2, (b) 744a41e2, (c) 340064a3 — the row's three checks are green (relay_tiers 13, crates/svrngs/tests/relay.rs 8, fallback.test.mjs 2, rebind.test.mjs 6), DESIGN §3's "Where a computer meets its groups" and "A relay anyone runs" and risk 5 are rewritten in place, and the guide is docs/guide/relays.md. The row's product is +102 lines of its +50 estimate (the operator's decision of 2026-11-19: the caps decide — core 3,524/3,539, all code but JavaScript 22,298/23,101, JavaScript 8,618/8,674 — and each commit carries its `Size:` paragraph).

> **Answered 2026-11-19 (director):** commits pass again — heapless's nine are excepted as unreachable (675b0cc1), webrtc-network landed with its floor line (6807cb53), deploy-relay landed (c8399de5). §2's four files are still uncommitted and green. Still open: the queue's test envelope (1,400) stands at 1,396, so (a)'s +55 does not fit; the operator is asked to raise it. §2's four files are set aside in `ralph/next/relay-commons/ctl/self-host-a.patch` while webrtc-page shares the worktree: `git apply` it first. Once raised, continue at §4: commit (a) with its DESIGN §3 paragraph, then (b) and (c).

Unit: `self-host` (ralph/next/relay-commons/STATE.md:32). Written
2026-11-19 by the unit's session. One operator decision is open, and it is
the parked one — but the blocker is not the one the parked package names.

## 1. The blocker, and a correction to NEEDS_HUMAN-webrtc-network.md

Nothing can be committed while webrtc-network's work is uncommitted — but
the refusing step is the **trust floor**, not the advisories. This
session's commit attempt (toolbox, the hook's own `scripts/check.sh
--fast`; full output in `/tmp/opencode/self-host-commit.txt`):

```
note       open files: 524288, raised to 10240
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (1 s)
failed     trust floor (0 s)
           trust floor: 13 policies, 12 listeners, 10 gates
           BROKEN  scripts/trust_floor.txt is not committed here, so whether its change gives a reason could not be judged
passed     advisories (3 s)
passed     app nouns gate (0 s) … passed script laws (7 s)
COMMIT EXIT 1
```

- **The advisories pass** with the parked package's nine-entry ready diff
  in the worktree (`deny.toml`). The parked package's "`scripts/advisories.py`
  fails on all of them, so the pre-commit hook refuses every commit"
  describes the tree *before* its own ready diff is applied; with the diff
  applied, `passed advisories (3 s)`.
- What refuses every commit is `scripts/trust_check.py:94-98`: during a
  commit, a floor change *staged whole* is said could-not-judge
  (`.githooks/commit-msg` then judges the `Trust floor:` line); any other
  state — an unstaged change included — is `BROKEN`. webrtc-network's
  change to `scripts/trust_floor.txt` (its (e); the operator's UDP
  decision of 2026-11-19) is in the worktree, unstaged, so every commit
  that does not include it fails here (`git status --porcelain -- 
  scripts/trust_floor.txt` → ` M`).
- The only states that pass: the floor change goes into a commit (with its
  `Trust floor:` reason), or it is reverted. This session did neither: the
  change is the parked row's work (STATE.md:29 (e)), the queue's own
  instruction is "Parked rows wait on the operator; do not open them:
  webrtc-network", and `--no-verify` is refused by CHARTER. `ralph-result
  done` is likewise refused (the tree lists every changed file, the parked
  row's and this row's).

So the queue's commits are blocked on the same answers as webrtc-network
(`NEEDS_HUMAN-webrtc-network.md`): §1 (advisories: record the nine, or
hold) and §2 (its allowance). **If §1 is "record them"**: the next session
commits `deny.toml` first, then the unit (its §4 message, whose `Trust
floor:` paragraph carries the floor change and its reason); the tree
becomes clean, and this row's work below can be committed, then (b) and
(c). **If §1 is "hold"**: the nine entries come out, the advisories fail
as that package says, and the queue waits for a postcard/iroh bump.

## 2. What is done (uncommitted, green): (a) the group's own HTTPS relay comes before ours

Files: `crates/svrngs-core/src/relay.rs`, `crates/svrngs-core/src/relay_tests.rs`,
`crates/svrngs-node/src/tiers.rs`, `crates/svrngs-node/tests/relay_tiers.rs`
(none of them the parked row's).

- `Meeting::own_https` is the own tier's relays a page may take (HTTPS
  alone); `Meeting::decide` holds them alone while they answer, the
  build's alone while they do not, and the own tier whole when the build's
  does not answer; a plain-HTTP own relay stays behind the build's, as
  before.
- `Host::check` and `Host::check_now` ask the tier that comes first — the
  own HTTPS relays when a group names one, else the build's — so the
  registration, `LEFT_AFTER` and `HELD_AT_MOST` rules apply to it.

Green (toolbox, CARGO_INCREMENTAL=0, TOKIO_WORKER_THREADS=4): core relay
8 passed; node lib 189 passed; relay_tiers 12 passed; relay_fallback 3
passed; `cargo fmt --all --check` clean; `python3 scripts/lean_check.py`
exit 0 — core 3,524/3,539, all code but JavaScript 22,143/23,101,
JavaScript 8,611/8,651. Product +29 of the row's +50 (core +14 of ≤15);
tests +55 of 120.

Planted, each watched red then reverted (the plant: `(true, _, true)` in
`Meeting::decide` forced to `Tier::Fallback`):
- `a_groups_own_https_relay_comes_before_the_builds` (core):
  "left: (Fallback, {relay.svrnme.sh}) right: (Own, {relay.example.org})".
- `a_group_meets_at_its_own_https_relay_while_the_build_s_answers`
  (relay_tiers): "left: [\"http://127.0.0.1:2/\"] right:
  [\"https://relay.example.org/\"]".

The ready commit message for (a) is `/tmp/opencode/self-host-a.txt` (and
§4 below).

## 3. What remains of the row

- **(b) a group-only `svrngs relay` admits a pass's holder for the
  group's lends.** Design, as far as this session read it (the mechanism
  is iroh-relay's one channel beside the endpoint id):
  - `ClientRequest::auth_token` (`iroh-relay-1.4.2/src/server.rs:253-274`)
    reads a `Bearer` token (native) or a `?token=` (wasm) off the client's
    upgrade request; `RelayConfig::with_auth_token`
    (`iroh-relay-1.4.2/src/relay_map.rs:266`) sets it, and
    `iroh::RelayMode::Custom(RelayMap)` carries it
    (`iroh-1.4.2/src/lib.rs:292` re-exports `RelayMap`).
  - So `Admits::on_connect` (`crates/svrngs/src/relay.rs:105-119`) should
    decode the token as a `Pass` (`svrngs_core::pass::Pass::decode`),
    admit it when the dialer is the pass's holder, the pass is signed and
    live (`until`), and the pass's person stands in a group held here
    (`RingRail::standing_anywhere`); a key of no group stays refused. The
    door already admits any dialer on `Alpn::Lend`
    (`crates/svrngs-node/src/node.rs:1252-1254`), so no node change.
  - The phone's side: `holder_endpoint` (`web/src/lib.rs:460`) gains the
    pass and `bind` (`web/src/lib.rs:202`) builds its `RelayMap` with it;
    `web/lend.js:353` hands the pass it already holds; `apps/taste/…/other.js`
    and `web/tests/rebind.test.mjs` call `holder_endpoint`.
  - Tests belong in `crates/svrngs/tests/relay.rs` (its `relay_over`,
    `client_at` — `ClientBuilder::auth_token` — and `admits` helpers):
    "a phone opens a lend through a group-only relay" and "a key of no
    group is still refused there", each planted. PLAN §2's newcomer row
    closes on the same admission: the joiner's dial reaches the issuer
    through the build's fallback the link also carries.
- **(c) a one-page guide in `docs/guide`** (a new `docs/guide/relays.md`,
  and its line in `docs/guide/README.md`).
- DESIGN §3 ("Where a computer meets its groups" and "A relay anyone
  runs") and risk 5 rewritten in place, in (a)'s and (b)'s commits.

## 4. What the next session does

1. After NEEDS_HUMAN-webrtc-network.md §1 is "record them": commit
   `deny.toml`, then webrtc-network's unit (its §4 message), then this
   row's (a) (message: `/tmp/opencode/self-host-a.txt`), then (b) and (c),
   each with its plants.
2. If §1 is "hold": remove the nine entries; nothing commits.
3. This row's (a) is uncommitted in the worktree (as webrtc-network's work
   was before it); do not discard it — it is the four files in §2, green
   and planted. `ralph-result done` for self-host will list them until
   they are committed.
