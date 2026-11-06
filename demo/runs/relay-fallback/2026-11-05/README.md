# Relay fallback — the production failure reproduced, the transport proven

The unit `fallback-proof`'s evidence (ralph/next/relay-fallback/STATE.md):
(b) the production-HTTPS/CSP failure with a cold synthetic keeper and a
fresh browser; (c) the smallest existing-transport way for an advertised
public fallback to reach that issuer while a local primary exists, with
the actual authenticated ask; and the runnable plan and readiness
evidence for `fallback-connect`. No product code changed in this unit.

## How it ran

```
cargo build --locked -p svrngs           # the keeper and relay binaries
scripts/build-web.sh                     # the page, target/web/site
python3 demo/runs/relay-fallback/2026-11-05/run.py
cargo test --locked -p svrngs-node --test relay_fallback   # the transport proof
```

`run.py` starts a plain-HTTP `svrngs relay --at 127.0.0.1:<port> --open`
(the Pi's stand-in), a cold `svrngs new house --as Sam --relay
http://127.0.0.1:<port>/` keeper, and `open.mjs` drives fresh Chromium
twice: once with the page served as production serves it (its own meta
CSP, `connect-src 'self' wss: https:`; `web/index.html:51`), once with
that policy given the tests' loopback allowance only. Each keeper is a
scratch directory removed before the run; the token is redacted from
`keeper.out` and never in `repro.json`.

## (b) The failure, reproduced cold

`data/repro.json` `failure`, the production policy: the page's status is
**"The computer that sent this link did not answer. Try again later."**,
`nameForm: false`, and its log holds 18 CSP refusals, the first:

```
error: Connecting to 'http://127.0.0.1:51801/ping' violates the following
Content Security Policy directive: "connect-src 'self' wss: https:".
The action has been blocked.
```

— the incident's shape exactly: no name form, no issuer connection, the
CSP refusing the HTTP relay before the dial leaves.

The control, same page, same plain relay, only the policy's `connect-src`
given the tests' loopback allowance: the relay handshake completes
(`LAYER relay-connect: ok — endpoint reports online`), `POST /ask`
answers 200, and the page reaches its name form (`status: "Join house"`,
`nameForm: true`). So the failure is the policy, not the keeper. A test
that relaxes CSP cannot prove the correction (CHARTER.md); this control
only names the cause.

## (c) The transport, proven

`crates/svrngs-node/tests/relay_fallback.rs`
`a_fallback_home_reaches_the_issuer_and_the_local_primary_takes_over`:
Sam (cold keeper) is homed at the public fallback R_pub alone, keeps an
unowned group, and gets the local primary R_own added beside it with the
pinned `Endpoint::insert_relay` (`iroh-1.0.2/src/endpoint.rs:982`) — the
call `crates/svrngs-node/src/tiers.rs:102` already makes. A real
`Host::link` token carries R_pub alone; Alex, holding only R_pub, reaches
Sam with the authenticated ask and redeem (`Host::join` over
`Alpn::Redeem`) and is let in. Control: Sam relieved of R_pub, the relay
stopped, leaves Alex2 `IssuerAway` and moves Sam's home to R_own; the
keeper then names R_own with `Host::meet` and Bo, holding only R_own, is
let in. Green: `ok`, 20.5 s.

Watched red first: with R_pub taken from Sam before Alex's join (the
plant the test names), the run failed `Err(IssuerAway)`; restored, the
same test passed.

**The inventory, pinned iroh 1.0.2.** An `Endpoint`'s `RelayMap` takes a
set and `insert_relay` adds one; a relay client connection is kept to
every relay in the map while it is used, but only the **home** relay's
connection is kept for good (`iroh-1.0.2/src/socket/transports/relay/
actor.rs:5`, "Unless it is for the home relay, this one never exits"); a
non-home connection closes after 60 s idle (`:64-65`), and the relay
server drops a packet to any endpoint with no connected client
(`iroh-relay-1.0.2/src/server/clients.rs:200-210`). `EndpointAddr`
reports one home relay in practice
(`iroh-base-1.0.2/src/endpoint_addr.rs:145-150`). So iroh's existing API
**cannot keep both relays warm at once**; the charter's fallback holds:
the public fallback is the browser-compatible home online, and the local
primary is held in the map and takes over when the fallback stops
answering (iroh's net report re-homes — the move
`crates/svrngs-node/tests/relay_tiers.rs` measures). No new transport
subsystem or dependency is needed.

## Readiness plan for `fallback-connect`

1. **One policy.** `svrngs_core::relay::Meeting::decide`
   (`crates/svrngs-core/src/relay.rs:139-149`) holds the build's tier
   only when none of the group's own answers today; invert it for a group
   that names its own: while the build's fallback answers, meet there and
   hold it as the home; the group's own relays when it does not.
   `Meeting::carried` (`:154-163`) already carries two relays, home
   first; `Host::link` (`crates/svrngs-node/src/invite.rs:207-212`) needs
   no second policy.
2. **The one ask.** `Host::check` (`crates/svrngs-node/src/tiers.rs:122-137`)
   asks only the own tier today; it must ask the tier it prefers and
   decide on that answer. `answers` (`:226-249`) is the existing
   instrument, kept.
3. **The browser takes every route it is given.**
   `web/src/lib.rs:97-107` `bind_for` takes only the token's **first**
   relay; `relay_of` (`:347-350`) likewise for a pass's holder. The
   correction binds the endpoint with every relay the token carries (one
   endpoint per key; the RelayMap takes the set), and the join's ask
   (`web/page/join.js:246`, `dial.js` `reach`) is tried against each
   eligible route; a loser's underlying dial is closed, not only its JS
   wait (`Dialled::close`, `dial.js:155-163` already closes late
   computers).
4. **The same candidates through reopening and lends.**
   `web/page/hand.js:80` and `web/page/visit.js:149` hand a lend's
   worker one relay from `relay_of`; both must carry the candidates the
   group's link names, so an app does not revert to the HTTP-only route.
5. **Regressions, compact and parameterized, using existing harnesses:**
   the production-CSP browser scene above (it must turn green: the
   fallback tried without a blocked request first), a stalled primary
   that does not delay the fallback, a relay that answers `/ping` but
   cannot reach the issuer losing, reopen and holder retry, all-routes-
   down cleanup with bounded failure, and the refusal preserved.

## Economizing survey

Baseline at this tree, `python3 scripts/lean_check.py`, exit 0:

```
crates/svrngs-core     3486 / 3494 lines
.                     19832 / 19833 lines
js                     8187 / 8188 lines
swift                     0 / 1 lines
```

So the repository has **1 line** of room, core 8, JS 1; the queue's
allowance is +150 product lines, not yet usable without an operator cap
decision (CHARTER.md: "If measured necessary growth does not fit the
current caps, stop with the post-pass counts and the smallest ready cap
diff for the operator").

The named owners, read for this survey: `svrngs_core::relay::Meeting`
(core relay.rs:74-164), `tiers::Held`/`Host::{refresh,decide,meet_at,
check,home,at_fallback,moved}` and `everywhere`/`answers`
(crates/svrngs-node/src/tiers.rs), `Host::{mint,link,link_held,
keep_linked,printed_page}` and `draw`/`kept_page`
(crates/svrngs-node/src/invite.rs), `bind_for`/`connect_to_join`/
`endpoint`/`holder_endpoint`/`relay_of` (web/src/lib.rs:97,308,322,333,
347), and `dial.js`'s `reach`/`patiently`/`exchange`
(web/page/dial.js:40-165). Each is small and single-purpose; no
duplicate decider or dead helper was found in them, and `cargo clippy`
reports nothing on the touched tests. **No behavior-preserving cut was
found in the fallback path this unit; none was landed, so no cap was
ratcheted.** Candidates examined and their disposition:

- `Tier`'s `serde::Serialize` (core relay.rs:84-94) — the home page
  reads it; kept.
- `tiers::made_at` / `home` / `at_fallback` / `moved` — each has a
  caller (`group.rs`, the home page, `tiers.rs`); kept.
- `invite::kept_page` / `linked` — used by the page and the command;
  kept.
- `dial.js`'s `PATIENCE`/`patiently`/`exchange` — the deadline and
  cancellation the fallback work must reuse; kept.

**Remaining estimate.** The proof's test file is 126 added code lines
(nonblank, noncomment) of the queue's 150 test-line allowance, leaving
about 24 for `fallback-connect`'s regressions — likely short of the
"compact parameterized regressions" that row asks for. Product growth
is estimated at roughly +60–110 Rust (core ≤10: `Meeting::decide` and
its tests) and +20–30 JS (`bind_for`/`relay_of` and the page's route
choice), within the +150 allowance but above the current repository cap
by about 60–110 lines. The operator's post-pass decision (CHARTER.md)
will need the measured counts and the smallest ready cap diff.

## What it leaves out, each said

- **The real page and the Pi**: not run here; the incident's private
  artifact is the operator's, and only the synthetic run above is
  committed.
- **A relay with QUIC address discovery**, a real phone, and a real
  modem cut: never ran.
- **The browser's second route**: not implemented in this unit; the
  control only shows the policy is the cause.
