# relay-survey: counts, the chosen versions, and the economizing pass

The queue's first unit (ralph/next/relay-commons/STATE.md, `relay-survey`),
ONE UNIT with no feature code: (a) the economizing pass over the owners
CHARTER names, with the per-row estimates; (b) the versions; (c) the port
map (`PORT_MAP.md`, beside this); (d) a stop only if a cap or a version
blocks the queue — neither does. Baseline: `d59c8246`, the branch point.

## The counts

`python3 scripts/lean_check.py` at this unit's commit, exit 0:

```
the core (svrngs-core)     3,499   most   3,539
all code but JavaScript   21,474   most  23,101
JavaScript                 8,544   most   8,651
Swift                          0   most       0
```

Headroom: the core 40, all code but JavaScript 1,627 (the core's 40 inside
it), JavaScript 107.

## The owners, counted

The files CHARTER's "Read before building" names, nonblank and noncomment
lines as `lean_check.py` counts them (tests and `apps/` apart):

| Owner | Counted |
|---|---|
| `crates/svrngs-core/src/relay.rs` (`Meeting`, `Tier`) | 89 |
| `crates/svrngs-node/src/tiers.rs` | 207 |
| `crates/svrngs-node/src/door.rs` (`LendRefusal`) | 1,369 |
| `crates/svrngs-node/src/invite.rs` | 178 |
| `crates/svrngs-node/src/iroh.rs` | 486 |
| `crates/svrngs-node/src/relay.rs` (the dotless lookup) | 65 |
| `crates/svrngs/src/relay.rs` | 146 |
| `crates/svrngs/src/cli.rs` (`SHIPPED`, `DEFAULT_RELAY`) | 1,576 |
| `web/src/lib.rs` | 457 |
| `web/lend.js` | 952 |
| `web/shim.js` | 162 |
| `web/app.js` | 77 |
| `web/sw.js` | 40 |

The harnesses the rows name — `crates/svrngs-node/tests/relay_tiers.rs`,
`crates/svrngs/tests/join_page.rs` and `web/tests/browsers.mjs` — are test
files, which the budget does not count; the queue's test allowance is 900
lines in all.

## The per-row estimates against each row's allowance

Each row's allowance is its ceiling, not a target (STATE.md, CHARTER
"Budget"). Worst case, every row spends its ceiling:

| Row | Product ceiling | Where it lands |
|---|---|---|
| iroh-1-4 | +30 Rust, 0 test | the pins in `Cargo.toml`s (uncounted) and the call sites any 1.4 API change reaches: `tiers.rs`'s endpoint calls, `web/src/lib.rs`'s `t!` string |
| relay-share | +20 Rust, 60 test | `crates/svrngs/src/relay.rs`: the share's constants, `max_burst_bytes`, `metrics_addr`, the warn filter; `OPEN_RATE` and its use go |
| n0-emergency | +50 (core ≤15), 120 test | core `Meeting`: the third set and `decide`/`carried`; `tiers.rs`; `cli.rs`; the home page's words |
| share-refusal | +70 Rust, +40 JS, 150 test | `door.rs`'s `Share` and 429; `tiers.rs` watching `relay_conns_ratelimited`; `web/lend.js`'s words and reconnect |
| webrtc-node | +250 Rust, 200 test | a new node module (the carrier and the str0m loop) and the router in `iroh.rs` |
| webrtc-page | +120 browser Rust, +60 JS, 250 test | `web/src/lib.rs`, `web/lend.js` and the window |
| self-host | +50 (core ≤15), 120 test | core `decide`'s order; `crates/svrngs/src/relay.rs` `Admits`; the guide |
| deploy-relay | +140 script | `scripts/deploy-relay.sh` |
| **all** | **590 Rust (core ≤30) + 140 script + 100 JS** | |

590 + 140 = 730 counted lines against 1,627 of non-JS headroom (the
core's 40 inside it); 100 against 107 of JavaScript. It fits, as CHARTER says; the JavaScript rung is
the tight one (7 to spare), and `webrtc-page`'s +60 JS is the largest
single slice of it. The core's 30 of 40 is the next tightest: `n0-emergency`
and `self-host` each hold themselves to ≤15.

## The economizing pass

Every item of the small owners was read with its callers (a grep of each
name outside its own file, tests apart):

- `crates/svrngs-core/src/relay.rs`: `parse` (link.rs:152, membership.rs:287,
  group.rs:294, guest.rs:189), `dotless` (node `relay.rs:24-33`), `Meeting::of`
  (tiers.rs:297, 301), `relays` (iroh.rs:91, `carried`), `own`/`build`
  (tiers.rs, `decide`), `join` (tiers.rs:300), `decide` (tiers.rs:81, 114),
  `carried` (invite.rs:221). All live.
- `crates/svrngs-node/src/tiers.rs`: `Held::built` (node.rs:341, tests),
  `refresh` (tiers.rs, group.rs:204, 259, join.rs:150, ring_sync.rs:295),
  `decide` (the node's one place), `meet_at` (join.rs:79, cli.rs:1787,
  1929), `check`/`check_now` (node.rs:434, home.rs:253), `made_at`
  (group.rs:368, cli.rs:1933), `home` (invite.rs:221, `made_at`,
  `at_fallback`), `at_fallback` (surface.rs:294, `moved`), `moved`
  (node.rs:433), `everywhere` (node.rs:340), `any_answers`/`answers`
  (inside `check` and `check_now`). All live.
- `crates/svrngs/src/relay.rs`: `name` (cli.rs:494), `spawn` (tests,
  `run`), `run` (cli.rs), `verdict` (`on_connect`, its test). All live.
- `crates/svrngs-node/src/invite.rs`: `claim_visit` (redeem.rs:203),
  `link_held` (publication.rs:180, 223), `printed_page` (carried.rs:543),
  `kept_page` (door.rs:1814, group.rs:367, update.rs:238), `own_person`
  (publication.rs:138, 156, redeem.rs:155, 205), `keep_linked`
  (cli.rs:1935). All live.

Candidates weighed, none landed:

- `web/shim.js`'s two `Object.assign` calls (the socket's state and its
  handler slots) folded into one: **measured +3 lines**, not −1 — the
  second call holds four properties on one line, and the formatter splits
  them in the object. Reverted; the tree here is the original.
- `crates/svrngs-node/src/tiers.rs:147-148`'s two tuple bindings folded
  into one: −1 line, in a rung with 1,627 to spare, in the fallback loop
  review F1/F2 stand behind. Declined.
- `web/lend.js`'s single-use named steps (`actsPosted`, `partAt`,
  `unheldAt`, `letGo`, …): inlining moves the body to the call site and
  saves at most the two signature lines each, at the cost of the file's
  named-step style — the path `share-refusal` extends. Declined.
- Repeated header-name literals in `web/lend.js` (`content-type` ×8,
  `content-length` ×5, `content-security-policy` ×4): each is at its use,
  and hoisting them costs lines elsewhere. Declined.
- `web/lend.js`'s media predicate written twice (`bounded`'s
  `destination === 'video' || destination === 'audio'`, `ranged`'s
  `['video', 'audio'].includes(...)`): one helper is 0 to +1 lines. Not a
  cut.
- No item in the owners is reached by tests alone, and no file has a
  duplicated three-line window (checked by a window hash over `web/lend.js`,
  `web/shim.js` and `web/src/lib.rs`; the only hits are `web/src/lib.rs`'s
  two line-signing parameter lists, which are two functions' signatures).

So no cut is committed and no cap is ratcheted: the owners are at their
floor for supported behavior-preserving cuts, as relay-fallback's pass
found for its own touched files
(`demo/runs/economize/2026-11-05-relay-fallback/SURVEY.md`). The queue
fits without one.

## The chosen versions (b)

Confirmed in a scratch copy under `target/relay-survey/`, which is
git-ignored and nowhere near the locks: **no edit to `Cargo.lock` or
`web/Cargo.lock` here** (`git status --short` clean after the probes).
`cargo tree` resolves the set; nothing was built (the row's check is
resolution).

The workspace probe, whose manifest is reproduced here since `target/`
is not kept:

```toml
[dependencies]
iroh = { version = "=1.4.2", features = ["unstable-custom-transports"] }
iroh-relay = { version = "=1.4.2", features = ["server"] }
iroh-base = "=1.4.2"
iroh-blobs = { version = "=0.104.0", default-features = false, features = ["fs-store"] }
irpc = { version = "=0.17.0", default-features = false }
str0m = { version = "=0.25.0", default-features = false, features = ["rust-crypto"] }
```

`cargo tree -e normal --depth 1`:

```
relay-survey-probe v0.0.0
├── iroh v1.4.2
├── iroh-base v1.4.2
├── iroh-blobs v0.104.0
├── iroh-relay v1.4.2
├── irpc v0.17.0
└── str0m v0.25.0
```

with `iroh`'s `unstable-custom-transports`, `iroh-relay`'s `server`,
`iroh-blobs`'s `fs-store` and `str0m`'s `rust-crypto`; one iroh line (no
1.0.2 beside it), `curve25519-dalek` 5.0.0 and `dimpl` 0.9.0 agreeing, as
the transport spike found for 1.4.1.

**Why `iroh-blobs` 0.104.0 and not 0.105.0**: 0.105.0 requires `irpc`
1.0.0 (`iroh-blobs-0.105.0/Cargo.toml`, `[dependencies.irpc]`), a new
major outside CHARTER's set; 0.104.0 requires `irpc` 0.17.0, the version
`crates/svrngs-node/Cargo.toml:78` already pins, and `iroh` 1.0.0, which
1.4.2 satisfies. 0.103.1 would also resolve; 0.104.0 is the newest that
keeps the authorized set.

The web probe, the same pins `web/Cargo.toml` holds with `iroh` bumped and
`unstable-custom-transports` added, `cargo tree --target
wasm32-unknown-unknown -e normal --depth 1`:

```
svrngs-web-probe v0.1.0
├── iroh v1.4.2
├── js-sys v0.3.105
├── n0-future v0.3.2
├── tokio v1.54.1
├── wasm-bindgen v0.2.128
├── wasm-bindgen-futures v0.4.78
└── web-sys v0.3.105
```

So `web/`'s existing pins (`web/Cargo.toml:25-27`) survive the bump:
`wasm-bindgen` 0.2.128, `wasm-bindgen-futures` 0.4.78, `js-sys`/`web-sys`
0.3.105, `n0-future` 0.3.2. The web probe carries
`unstable-custom-transports` too, so the page's carrier resolves for wasm.

The API each dependent row needs, checked in the resolved crates
(`~/.cargo/registry/src/index.crates.io-*/`; paths below are
crate-relative):

- `relay-share`: `ClientRateLimit { bytes_per_second, max_burst_bytes }`
  (`iroh-relay-1.4.2/src/server.rs:506-523`) — the rate and the 8 MiB burst
  are both expressible; `metrics_addr` on the server config
  (`iroh-relay-1.4.2/src/server.rs:119`).
- `share-refusal`: the relay tells the client `Status::RateLimited`
  (`iroh-relay-1.4.2/src/protos/relay.rs:139`), iroh counts it once per
  connection in `relay_conns_ratelimited`
  (`iroh-1.4.2/src/socket/transports/relay/actor.rs:816`), reachable as
  `Endpoint::metrics().socket.relay_conns_ratelimited`
  (`iroh-1.4.2/src/metrics.rs:20-27`, `src/endpoint.rs:1638`).
- `webrtc-node`: `CustomTransport`/`CustomEndpoint`/`CustomSender`,
  `RecvInfo`, `Transmit`, `CustomAddr`,
  `Endpoint::builder(..).add_custom_transport`
  (`iroh-1.4.2/src/socket/transports/custom.rs:24-90`, `src/endpoint.rs:822`);
  the spike's carrier needs no change for the traits (the spike README,
  "3. str0m against iroh 1.4").
- `iroh-1-4`: `insert_relay`/`remove_relay` (`iroh-1.4.2/src/endpoint.rs:998,
  1012`), `home_relay_status` (`:1406`), `default_relay_mode` (`:2055`) and
  `NET_REPORT_TIMEOUT` (`src/lib.rs:294`) all survive; `PATH_MAX_IDLE_TIMEOUT`
  is still 15 s and the relay path's 30 s (`src/socket.rs:117, 129`).

## What is not proven here

- The bumped set is resolved, not compiled or tested: the iroh-1-4 row
  runs `cargo check --workspace --all-targets`, the wasm build and the
  focused suites, and owns the behavior-kept landing.
- The economizing pass cut nothing, so it has no focused check of its own;
  the counts above are the whole of its evidence.
- The queue's rows, the real phone and the real box were not run: the
  `HUMAN-` rows stay never-ran until the operator runs them.
