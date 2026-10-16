# What the product carries that it does not need — groups-economize's survey, 2026-10-16

Read-only, at main `02e7e75c`. Every count is `scripts/lean_check.py`'s
rule, per file by `scripts/count.py` here (start-economize's, unchanged),
agreeing with its total: **16,144 / 16,145**, and svrngs-core **2,952 /
2,953** (`python3 scripts/lean_check.py` at 02e7e75c). The groups queue's
budget (the operator, 2026-10-14, "+200 repo, +40 core") is over the caps
this pass leaves.

The instruments are v1ui-economize's (`../2026-10-14-v1ui/SURVEY.md`):
the compiler with every `pub` demoted, the clone report, and the open
findings that name a thing said twice. Grep only checks what they name.
The scripts here are that survey's, copied unchanged but for loop.sh's
scratch paths (/tmp/groups-econ-*), and one added: `cargo_warnings.py`,
which writes a build's cargo JSON as the `data/warnings-*.txt` lines the
earlier surveys wrote.

## What changed since v1ui-economize's survey

`data/delta-4a2e8a7.txt` (`scripts/delta.py` over `count.py` at
`4a2e8a7` and at `02e7e75c`): 16,314 → 16,144, **−170**. The v1ui
queue's raises (the door, the lend, the shelf, the polish: carried.rs
+195, share.rs moved into the node +165, home.rs +162, join.rs moved
+101, home.js +41, question.js +39, door.rs +39), less the arch queue's
cuts (guest.rs −238, journal.rs −123, cli.rs −102, process.rs −98,
ring_sync.rs −83, live.rs −73, and smaller).

## 1. The compiler: five items reached only by tests

In a scratch clone of `02e7e75c` (/tmp/groups-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the
four crates and `web/src` `pub(crate)` (69 files). Then
`scripts/loop.sh` built the product (`cargo build --workspace
--offline`) and `scripts/repub.py` made `pub` again each item a
downstream crate was refused, round after round. Two refusals repub.py
does not parse were made `pub` by hand, as before: `NotASyncPath`
(synced.rs:37; E0446) and `Refusal::exit_code` (cli.rs:619, called by
main.rs across the lib and the bin). The third of the earlier surveys,
`ProcessManager`, is gone with process.rs (arch-core). The product
ended `build-finished success: True` (round 15: `errors 0 fixed 0`), and
`web/` for `wasm32-unknown-unknown` on its first round. The scratch diff
is `data/demoted.diff`; what still warns is `data/warnings-prod.txt` and
`data/warnings-web.txt`.

The page's warnings are the 2026-10-14 survey's, less what the arch
queue cut (`Seal`, `InviteFault`, `ProcessManager`, `Uncaused`,
`membership`, `MAX_PAYLOAD_BYTES`): nothing of the page's own. The
product's, against v1ui-economize's, line for line, differ by six:

| Warning at 02e7e75c (scratch lines) | Who uses it | Counted lines | Read as |
|---|---|---:|---|
| crates/svrngs-node/src/node.rs:424 `Node::sync_with`, a wrapper of `Host::sync_with` (node.rs:423-426 at HEAD) | tests: node conformance.rs:162, browser/main.rs:154, door/newcomer.rs:40, :156, door/main.rs:794; svrngs default_relay.rs:159. The product asks `Host::sync_with` (join.rs:114) | 3 | **cut**: the tests ask `node.host()` (node.rs:411), as start-economize cut `Node::say_relay` |
| crates/svrngs-node/src/iroh.rs:162 `IrohTransport::endpoint` (iroh.rs:155-158 at HEAD) | nothing: no test or product asks a transport its endpoint (`git grep -n "transport.endpoint()\|transport().endpoint"`, none) | 3 | **cut** |
| crates/svrngs/src/cli.rs:106 `pub use crate::group::kept_relay` | tests/new.rs:286-291, as `cli::kept_relay` | 1 | **cut**: the test asks `group::kept_relay`, where it is decided (svrngs-node group.rs:324) |
| crates/svrngs/src/lib.rs:11 `pub use svrngs_node::join as issuer` | tests/new.rs:9, tests/join_by_link.rs:21 | 1 | **cut**: the tests import `svrngs_node::join as issuer` (a dev-dependency, crates/svrngs/Cargo.toml:43) |
| crates/svrngs-node/src/group.rs:116 `read_origin` (group.rs:113-120 at HEAD) | tests/new.rs:313, :319, tests/join_by_link.rs:306; the product reads an origin by `invite::kept_page` | 6 | **a follow-up, not cut here**: new.rs's `an_unreadable_seed_or_origin_refuses_naming_its_file` asserts `read_origin`'s refusal names the file "origin"; `kept_page`'s `io::Error` does not. Cutting it changes what a test asserts |
| crates/svrngs-node/src/dial.rs:46 `LINK_WITHIN` | the page: web/src/lib.rs:415 (dial.rs is compiled in, lib.rs:36) | — | **not a cut**, as `held` was |

The rest is what the earlier surveys read and kept (`At::Here`,
`MemoryLookup`, the core's re-exports only tests read, now 9, the node
lib.rs:41 glob shadow, which is the demotion's own effect), at their
lines now.

`scripts/cfgtest.py` finds node.rs:199-200 and :382-383 (`Node`'s
`routes`) and :948-949 (`pub use tests_support::splice`, the
integration tests' reach to a test-support item, #7) as before, and one
new:

| Item | Counted lines | Read as |
|---|---:|---|
| crates/svrngs/src/share.rs:13-15 `#[cfg(test)] use svrngs_node::lend::{Kind, LendName, RefusedPath};`, read only by share/tests.rs | 1 | **cut**: the import moves into share/tests.rs, a test file |

The page's JavaScript has no compiler. `scripts/jsnames.py` finds every
top-level name of `web/*.js` and `crates/svrngs-node/src/home/*` said
again (12 files, `data/jsnames.txt`). Its plant, a function appended to
web/app.js and never called, went red: `web/app.js: plantedNeverCalled
said 1`; with the file restored, green: `every top-level name is said
again` (`data/jsnames-plant.txt`).

## 2. What no check, test or person runs: nothing

`git diff --name-status 4a2e8a7 02e7e75c -- scripts` modifies
lean_check.py and vocab_check.py and adds no file. Every script is run
by what ran it before. crates/svrngs-node/build.rs (33 lines, new) is
cargo's.

## 3. The clone report: nothing new in the product

`~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report
--corpus-id prod-0a2e6b3a --scope <dir> --threshold <t>` for each crate,
`web` and `scripts`, at 0.95 and 0.90 (`data/dry-<scope>-<t>.md`). The
corpus is REVIEW-audit-arch-auto-1's: 0a2e6b3a's crates/, web/,
scripts/, apps/, Cargo.toml and Cargo.lock (`ralph/REVIEW_FINDINGS.md`,
that audit's "The instrument"). Between 0a2e6b3a and 02e7e75c the
counted product changed only by iroh.rs's rename of `Answered` to
`Streamed` (`git diff --stat 0a2e6b3a 02e7e75c -- crates apps web
scripts Cargo.toml Cargo.lock`: the rest is test code).

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 4 | 0 | 10 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 3 | 2 | 11 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (I4) | 0 | 2 |
| scripts | could-not-judge (I4) | 0 | 0 |

These are the audit's counts, cluster for cluster, and the audit read
each: test code (`flush`, `write`; made one in 684dade8, after the
corpus), three-line accessors on different types, overlapping spans
(I1, I32). None is a cut.

## 4. Said twice: the open findings

Open rows under principle 8 opened since v1ui-economize's survey (#157
on), each read at 02e7e75c; the earlier ones are as the earlier surveys
read them.

| # | Site | Lines a fix saves | Why |
|---|---|---:|---|
| 157, 158, 166, 168, 170, 171, 172, 174 | | — | fixed (v1ui-findings, v1ui-polish, arch-helper) |
| 169 | share.rs, build.rs, home.rs `"helper"`, `"-"` | — | fixed in 8a3c044 (arch-helper (c)) |
| 177 | folder.rs:565-597 `fetch`'s kept connection beside iroh.rs:228-244 `send`'s | not this row's | a blob fetch's dial follows the one rule after it, `ANSWER_WITHIN` for `DIAL_WITHIN`: a change of behaviour, red first, and groups-tabs (c)'s by the queue |

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, `Node::sync_with` | 3 | 0 |
| §1, `IrohTransport::endpoint` | 3 | 0 |
| §1, `cli::kept_relay`'s re-export | 1 | 0 |
| §1, `svrngs::issuer`'s re-export | 1 | 0 |
| §1, share.rs's test-only import | 1 | 0 |
| §1, `group::read_origin` | 6, a follow-up (a test's assertion changes) | 0 |
| §2, §3, §4 | 0 | 0 |
| **Total cut here** | **9** | **0** |

The cuts, largest first, would leave the repository at **16,135**, its
cap one past at **16,136**, and the groups rows' ceiling **16,336**.
svrngs-core stays **2,952 / 2,953**, its ceiling **2,993**.
