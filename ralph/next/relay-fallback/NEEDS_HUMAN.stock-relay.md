# stock-relay: the landing is +90 test lines against the row's 80

One line: the stock-relay scene is green with its plant red first and the
focused checks pass; the landing is +90 nonblank noncomment test lines in
`crates/svrngs/tests/join_page.rs` against the row's 80 (its own, apart
from the queue's audited 450), the +10 over being the plain→TLS splice
the corrected premise needs; accept the landing as it is, or direct a cut.

CHARTER.md: "If measured necessary growth does not fit the current caps,
stop with the post-pass counts and the smallest ready cap diff for the
operator." This is that stop. No cap was edited by this session; no
product line landed.

## What landed

Commits `cee748a3..64a468bf` (each body carries its own evidence):

- `cee748a3` — the row's premise corrected: the computer cannot home at
  the stock binary's own HTTPS URL at HEAD. `build_relayed_endpoint`
  builds from `presets::Minimal` with no `ca_tls_config`
  (`crates/svrngs-node/src/guest.rs:142-148`) and iroh 1.0.2 takes the
  embedded webpki roots (`iroh-1.0.2/src/endpoint.rs:261`), so a per-run
  certificate is `UnknownCA`. Probe: a plain-HTTP relay homed in 3 s; the
  same endpoint at `https://127.0.0.1:<port>/` never homed (6 s,
  `home: []`), the binary logging
  `TLS[manual] accept: received fatal alert: UnknownCA`.
- `efd784b6` — the joining scene's browser half shared, behaviour kept
  (`sam_meeting_at` takes the build as a URL and asks the group's own at
  startup, as `svrngs new --relay` does; `phone_joins_through` is the
  browser half); the six existing join_page tests pass.
- `ba93683a` — the scene: the stock binary built from the lock
  (`cargo build -p iroh-relay --bin iroh-relay --target-dir
  target/stock-relay`, its `server` feature the workspace's), run on
  127.0.0.1 with a manual per-run certificate, its own config file and
  TLS termination; named as the group's own relay with the build's
  fallback unanswering, so the computer homes there through a test-made
  plain→TLS splice; a fresh browser under the published policy joins
  through the binary's TLS and opens the shelf lend.
- `64a468bf` — one sentence in `docs/DESIGN.md` §3 "A relay anyone runs".

## The plant, red first

The same test with `access.allowlist = []` (the binary started with an
access config that denies every key), watched red before the green:

```
thread 'a_group_meets_at_a_stock_iroh_relay::webkit' panicked at
crates/svrngs/tests/join_page.rs:852:5:
  "debug: LAYER relay-connect: REFUSED — no relay answered within 10s,
   and the endpoint is closed"
  "The computer that sent this link did not answer. Try again later."
test result: FAILED. 0 passed; 1 failed; finished in 27.47s
```

the computer did not home (`home: None`) and the browser did not join.
Green, `access = "everyone"`: `19.96 s` (the first run `269.87 s`, the
binary's own target dir built). All seven join_page tests `38.81 s`;
relay_tiers' two `24.14 s`; `cargo fmt --all --check` clean;
`cargo clippy -p svrngs --tests` clean.

## The measured counts

`python3 scripts/lean_check.py`, exit 0 at HEAD:

```
crates/svrngs-core     3491 / 3494 lines
.                     19911 / 19913 lines
js                     8187 / 8188 lines
swift                     0 / 1 lines
```

Product growth: 0 lines (test-only, plus 3 doc lines). Test growth,
counted as lean_check counts (nonblank, noncomment):
`crates/svrngs/tests/join_page.rs` 577 → 667, **+90** against the row's
80.

## The economizing pass

The landing was cut over the pass (measured 116 → 90): the plant is a red
run, not a committed second test; the browser half is shared with the
existing fallback scene (net −12); `free` is three lines; the port wait
is bounded; the build's stderr is kept; `cargo` builds into the binary's
own target dir (the row's correction — no env games). What remains is
each line the row's proof names: `free` (4), the splice (22), the binary's
build/config/spawn (39), the shared browser half (27), the scene test
(10), and the `sam_meeting_at`/existing-test delta (−12). Cutting to 80
would drop the homed assert (the row's plant names "the computer does not
home"), the join assert, or the build's stderr — each a weaker check,
which this row's instruction says to package instead.

## How to verify

```
env CARGO_INCREMENTAL=0 lockf -k /tmp/svrngs-gate.lock \
  cargo test -p svrngs --test join_page
env CARGO_INCREMENTAL=0 lockf -k /tmp/svrngs-gate.lock \
  cargo test -p svrngs-node --test relay_tiers
CARGO_INCREMENTAL=0 cargo fmt --all --check
env CARGO_INCREMENTAL=0 lockf -k /tmp/svrngs-gate.lock \
  cargo clippy -p svrngs --tests
python3 scripts/lean_check.py
```

The first join_page run builds `target/stock-relay` (~4 min, cached
after); later runs ~20 s. No full gate: the row says the audit already
ran it and this row adds no product code.
