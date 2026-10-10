# Where the lines go — the seat's survey, 2026-10-10

Read-only, at main `8b508d0` and `folders` `51a624c`; their merge-base
`b482e08`. Every count is `scripts/lean_check.py`'s rule (a line not blank
and not starting `//`, `#`, `/*` or `*`; tests, `apps/`, `node_modules/` and
`target/` apart), replicated per file and agreeing with its totals: main
15,284, `folders` 15,704, the merge-base 14,529 — so main and `folders`
merged count about 16,459, past either cap (15,350; 15,750 on `folders`).
The cuts below were found by the compiler, not by grep, unless said: in a
scratch clone every `pub` item of the four crates demoted to `pub(crate)`, the
product (`cargo build --workspace`, no tests) and `web/` for wasm32 rebuilt,
each item a downstream crate needed made `pub` again, and what still drew
`dead_code` deleted and rebuilt. Two branches keep the scratch diffs, built and
checked there: `survey/lean-unreached` (`ecaa616`, A and B) and
`survey/lean-tests-moved` (`4799321`, C). They are a start, not evidence: a
row redoes each step and shows its own.

## Where main's 15,284 are

| Part | Lines |
|---|---:|
| crates/svrngs-node | 6,310 |
| crates/svrngs-core | 2,635 |
| demo/runs | 1,858 |
| web (`*.js` 946, `src/*.rs` 571) | 1,517 |
| crates/svrngs | 1,379 |
| crates/svrngs-runtime | 1,162 |
| scripts | 423 |

## demo/runs — 1,858, none run by a check

`git grep "demo/"` over `scripts/check.sh`, `crates/**/tests` and
`web/tests` finds only comments; every row of the eleven
`ralph/next/*/STATE.md` was `[x]`. No commit decided that `demo/runs` counts:
every commit that changed `BUDGET` or the counting rule (25a2656, a2c4390,
aa013c0, 4c3bae3, 910e373, 319dac0, 357d07c, b764b6a, d697b18, 508356d,
aaecbdb, 30d975a) is silent on it, and three notes record it as found
(9bdcd1f; 28c430a; `ralph/REVIEW_FINDINGS.md`:4559, sup-4's 11).

- *Live*, 126: `u4/2026-10-05/drive.mjs`, whose `serve` (18 lines) the app
  checks run (`apps/jellyfin/check/check.sh:168`,
  `apps/movie-night/check/check.sh:187`, and so `apps/taste/check/check.sh:119`).
- *Superseded copies*, 528: u5's `drive.mjs` (278; 76% in
  `apps/jellyfin/check/person.mjs`), `measure.sh` (135; forked into
  `apps/jellyfin/check/check.sh`), `scan.py` (63; 83%), `summary.py` (37;
  identical), mw-2's `run.sh` (15; `check.sh` `mw2()`). #61 kept them on
  purpose as the record of what ran.
- *A run's own driver*, 1,204: everything else.

The operator, 2026-10-10, chose "Budget counts product only" — `demo/runs/`
apart, as `apps/` is — over retiring the copies or leaving it (lean-1).

## Main: test code in counted files — 548

`lean_check` skips a test file by its name, so an inline `mod tests` counts.

| Block | Lines |
|---|---:|
| `crates/svrngs-node/src/iroh.rs:513-916`, inline `mod tests` | 297 |
| `crates/svrngs-runtime/src/oplog.rs:486-770`, inline `mod tests` | 200 |
| `crates/svrngs-runtime/src/journal.rs:261-290`, `#[cfg(test)] RingJournal::append` | 26 |
| `crates/svrngs-runtime/src/op.rs:126-156`, `Op::new` (`cfg(any(test, test-support))`) | 14 |
| oplog.rs's `cfg(test)` items: `use serde` :97, `path` :150, `append` :155, `read_all` :290 | 10 |

Every other `#[cfg(test)]` in a `src/` is one line, `mod tests;`. On the
branch, all targets built, `clippy --workspace --all-targets -D warnings`
clean, the iroh (10) and oplog (14) tests ran.

## Main: what nothing reaches — 147, and 27 unreachable

**A, nothing reaches it, tests included** (147). On the branch, product, web
and both clippys clean; `--all-targets` fails only on
`guest/tests.rs:156` `a_non_member_dials_a_lenders_alpn_and_a_request_answers`,
the test of the tunnel cut.

| Item | Lines |
|---|---:|
| `crates/svrngs-node/src/ring_checkpoint.rs` whole, and `pub mod` at node lib.rs:35-37 | 22 |
| `crates/svrngs-node/src/guest/tunnel.rs` whole (upstream's "pb-reach-guest" dial, 18 of it inline tests), guest.rs:23-24 | 67 |
| guest.rs:524-560 `parse_dial_string` (its one caller `GuestTunnel::open`) | 33 |
| iroh.rs:337-340 `Dialer::key` | 3 |
| core lib.rs:563-575 `RailAct::payloads` | 7 |
| core act_set.rs:36-41 `ActSet::merge` | 5 |
| core hash.rs:60-65 `ContentHash::from_bytes`, :90-95 its `Display` | 8 |
| core command.rs:17-18 `Reads::All`, its arm :184 | 2 |

**D, upstream's local-only namespaces** (27): core lib.rs:152-174
`LOCAL_ONLY_NAMESPACES` (`peer_preferences`, `wikipedia-newsworthy-*`, …) and
`is_local_only`, checked at runtime lib.rs:98-105, journal.rs:353-359 and
register.rs:108. A group's namespace is its seed's 64-hex BLAKE3
(`crates/svrngs-core/src/seed.rs:47-58`), and a directory that does not hash
to its seed is dropped (`crates/svrngs/src/group.rs:158`); behaviour differs
only for a directory of one of those names placed by hand under `rings/`.
`tests_local_only.rs` goes with it; the doc at runtime lib.rs:95 changes.
*Not taken* (lean-3): group.rs:158 drops such a directory from the
command's list only. The node reads `rings/` with no group.rs between —
the round lists `RingRail::namespaces` (`ring_sync.rs:286`), the sync route
asks `RingRail::standing` (`ring_routes.rs:99`) — and with `is_local_only`
planted false a hand-placed `rings/notes-private/` naming dee is listed and
stands dee, where it is `([], NotHeld)` now. So D changes behaviour, and
stays.

**B, used only by tests** (~120): `Digest::{new, insert, contains_key, keys,
iter, is_empty, len, entries}` (core view.rs:115-151; `get` is the product's,
24); `Discovery`, its `FromStr`, `RelayConfig::from_parts` (guest.rs:78-129,
32); `PumpSide::Acceptor` and its arm (guest.rs:600, :638-650, 13);
`ops_missing_from` and its re-export (core sync.rs:282-302, 10);
`Roster::person_for`, `knows` (core lib.rs:317-328, 9); `Node::api_routes`
with its `routes` field, `transport()`, `round()` (11); oplog.rs:459-464
`Tail::skipped`, whose own `expect(dead_code, "the cursor tests read it")`
says so (~6); `OpId::from_hex`, `Payload::as_value`, `Upload::new` (9); the
re-exports only tests read, iroh.rs:61-62, :70, core lib.rs:150 (4). Keep
`ActSet::is_empty` (clippy's `len_without_is_empty`) and `on_loopback_only`
(guest.rs:131-138), the seam DESIGN.md:1395 and ARCHITECTURE.md:119 name.

Grep only, not taken: web/src/lib.rs:384-387 `Endpoint::close`, a wasm export
nothing calls (3); eighteen core root re-exports no product crate reads (1-2).

## Main: written twice — about 167, estimated

A copy-paste detector of the seat's (5- and 6-line windows, exact and with
names normalised; every pair read by hand), the clone tool's near tier at
0.92 and 0.88 confirming three:

| Pair | Est. |
|---|---:|
| 21 `Err(x) => return x` (rail.rs 7, door.rs 5, surface.rs 5, node.rs 2, choices.rs 1, origin.rs 1) and rail.rs:418, :430, as `?` on `Result<Response, Response>` | ~65 |
| lend.rs:38-71 `LendName` and :76-113 `Kind`: one parse, `TryFrom`, `From`, `as_str`, `Display` | ~25 |
| `as_str` and `Display` for names: core lib.rs:229-238, :259, :283; pass.rs:115, :133; door.rs:731-740; link.rs:186 | ~15 |
| a task aborted on drop: api.rs:194-198, iroh.rs:404-408, ring_sync.rs:130-134 — n0-future's `AbortOnDropHandle`, already a dependency (svrngs-node Cargo.toml:24) | 15 |
| door.rs:495-507 and :578-590, headers removed and counted | ~7 |
| core sync.rs:407-418 and :442-453, the budgeted push loop (forked) | ~6 |
| door.rs:758-772 `through_pass` and :901-915 `through` (tool 0.892) | ~6 |
| declared.rs:126-154, the owner-only writes' cfg pairs (tool 0.97) | ~6 |
| oplog.rs:176-184 and :239-247, ops to lines (forked) | ~7 |
| origin.rs:74-94 `app_of` and home.rs:113-128 `only_itself`, Sec-Fetch-* and Origin read twice | ~6 |
| origin.rs:217-222 and :293-298, the `Uri` refusal | ~5 |
| door.rs:1241-1246 and :1294-1299, the share header inserted | ~4 |

Open findings in counted code whose fix is a move, never closed since
`ralph/REVIEW_FINDINGS.md`:2491: #42 (issuer.rs:41-54, web/src/lib.rs
`bind`), #48 and #58 (the 32-byte key read five times in web/src; node.rs:544
and web/src/lib.rs:337), #49 (a lend's address built and read in five
places), #63 (`PAGE_COOKIES`, lend.js:271, shim.js:53), #64 (share.rs:107,
:120), #89 (sign_in.rs:307-333, lend.js:338-339), #107 (the owner-only
atomic write thrice: api.rs:51-53, lend.rs:461-463, choices.rs:176-181; the
branch `folders` adds a fourth, not atomic, at synced.rs:134), #111
(lend.js:409-440, the pass-and-lend guard three times).

**The phone and the computer, the same thing twice** (~49): the ask and the
yes post (web/app.js:217-254; `crates/svrngs-node/src/home/home.js:25-29`,
:88-96) and "What apps use" (app.js:258-280; home.js:68-85) — #110; one
question, two wordings (app.js:175 `what`; home.js:48 `named`, which drops the
kind) — sup-4's no-prediction 2. `/svrngs-reads` shaped twice (lend.js:374-389
`told`, surface.rs:206-243 `reads`, #104) saves nothing without a change of
shape: the worker answers offline from what the page handed it.

## The branch `folders` — +1,175 over the merge-base

By file: node folder.rs 508, `demo/runs/sy/2026-10-09/run.py` 180, node
synced.rs 176, core synced.rs 111, core folder.rs 81, cli.rs 46, core lib.rs
21, core admit.rs 20, node.rs 14, iroh.rs 7, gf's run.sh 4, membership.rs 3,
alpn.rs 2, node lib.rs 1, check.sh 1 (+1,187, −12). Each removal below was
applied to a scratch copy and `cargo check -p svrngs-node -p svrngs --offline`
passed with no warning; the tests were not adapted.

| Cut, sites at `51a624c` | Lines |
|---|---:|
| The plants and gf's digest: folder.rs:50-67 `Plant`, :87, :109-117, :132, :199-203, :273/:277, :309-313 `within`, the digest :488, :514, :561; synced.rs:70-73, :84 and its close; node.rs:756-759 | 42 |
| The loopback endpoint: `bytes` folder.rs:422-538, `range` :540-553, `Served` :555-569; `start`'s wiring (`SVRNGS_FOLDER_PORT` :118-124, `at` :89-90, :134, `serve` :138-142, the `Body` import :19); core `Admission::holders` folder.rs:102-111, read only by its test | 158 |
| gf's one-way share, which nothing reads once the endpoint goes: core folder.rs 81, lib.rs 9, admit.rs 6, membership.rs 1; node folder.rs 47 (`all`, `share`, `unshare`, half of `shares`, two `FolderAsk` variants); cli.rs 7 | 151 |

The operator, 2026-10-10, chose "Drop streaming + share" (syp-1). The sy and
gf READMEs' sites are stale against `51a624c` (sy cites synced.rs:74-76 and
:88, now :70-72 and :84). Written twice on the branch: synced.rs:153-158
rebuilds node.rs:422-440 `Host::standings()` without its traces;
folder.rs:160-165 `admissions` walks the namespaces a fourth time and drops a
failure unsaid (the branch's #109, #113 — #113, #117 since
syp-3's merge); folder.rs:280-339 `hold` remembers
an unanswered holder for 60 s where node.rs:488-535 `lent` remembers none
(node.rs:98) — two deciders of who is asleep; a hash's holders listed at
folder.rs:458-462, core :102-111 and synced.rs:186-187 (the branch's #110 — #114 since syp-3's merge);
cli.rs:555-570 adds reqwest 0.12 beside the command's reqwest 0.13.
`ralph/REVIEW_FINDINGS.md` numbers #108-#111 were given on both sides apart:
main's #110 and #111 (:4764-4765) are not the branch's (:4445-4446).
syp-3's merge renumbered the branch's #108-#116 as #112-#120.

## What the cuts make of the merge

| After | Lines |
|---|---:|
| main and `folders`, merged | ~16,459 |
| `demo/runs/` apart (main 1,858, the branch 184) | ~14,417 |
| the tests moved (548) | ~13,869 |
| A and D (174), B (~120) | ~13,575 |
| the branch's plants, endpoint and share (~342) | ~13,233 |
| main's pairs (~167, estimated) | ~13,066 |

Measured: `demo/runs`, the tests moved, A, D and the branch's cuts; B and the
pairs are estimates. The synced folder's production rows, about 180 by sy §9
(13-52% over, by the spikes' misses), land inside 15,350.
