# What the product carries that it does not need — canon-economize's survey, 2026-10-16

Read-only, at main `a0129152`. Every count is `scripts/lean_check.py`'s
rule, per file by `scripts/count.py` here (groups-economize's, unchanged),
agreeing with its total: **16,419 / 16,420**, and svrngs-core **2,991 /
2,992** (`python3 scripts/lean_check.py` at a0129152). The canon queue's
budget (the operator, 2026-10-16, "+150 repo, +40 core") is over the caps
this pass leaves.

The instruments are groups-economize's (`../2026-10-16/SURVEY.md`): the
compiler with every `pub` demoted, the clone report, and the open findings
that name a thing said twice. Grep only checks what they name. The scripts
here are that survey's, copied unchanged but for loop.sh's scratch paths
(/tmp/canon-econ-*).

## What changed since groups-economize's survey

`data/delta-02e7e75.txt` (`scripts/delta.py` over `count.py` at
`02e7e75c` and at `a0129152`): 16,144 → 16,419, **+275**. groups-economize
cut 9 after its count; northstar-economize cut upstream's local-only
namespaces (befedc5b, #200: core lib.rs −11, runtime lib.rs −6). The rest
is the groups, gv and northstar queues' raises: ring_sync.rs +36,
core register.rs +34, cli.rs +32, removal.rs +25, web/app.js +23,
ring_routes.rs +22, iroh.rs +20, origin.rs +18, vocab_check.py +17,
surface.rs +16, and smaller; folder.rs −24 (#177's one connection cache).

## 1. The compiler: one item reached only by tests

In a scratch clone of `a0129152` (/tmp/canon-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the four
crates and `web/src` `pub(crate)` (69 files). Then `scripts/loop.sh` built
the product (`cargo build --workspace --offline`) and `scripts/repub.py`
made `pub` again each item a downstream crate was refused, round after
round. The two refusals repub.py does not parse were made `pub` by hand,
as before: `NotASyncPath` (synced.rs:37; E0446) and `Refusal::exit_code`
(cli.rs:633). The product ended `build-finished success: True` (round 15:
`errors 0 fixed 0`), and `web/` for `wasm32-unknown-unknown` (`DIR=web
ARGS="--target wasm32-unknown-unknown"`) on its first round. The scratch
diff is `data/demoted.diff`; what still warns is `data/warnings-prod.txt`
and `data/warnings-web.txt`.

Against groups-economize's, line for line: its five cuts are gone from
the list (`Node::sync_with`, `IrohTransport::endpoint`, `cli::kept_relay`,
`svrngs::issuer`, share.rs's import), `<Leave as Command>::READS` is the
`Reads` warning every command has (register.rs:213, gv-leave's new
command), and one is new:

| Warning at a0129152 (scratch lines) | Who uses it | Counted lines | Read as |
|---|---|---:|---|
| crates/svrngs-node/src/iroh.rs:48 `pub use crate::alpn::ALPN` (iroh.rs:47 at HEAD, beside `Alpn`) | tests: node tests/accept.rs:17; the product reads `crate::alpn` | 0 | **not a cut**: the name shares its line with `Alpn`, which the product reads; dropping it saves no counted line |
| crates/svrngs-node/src/group.rs:116 `read_origin` (group.rs:113-120 at HEAD) | tests: svrngs tests/new.rs:316, :322, tests/join_by_link.rs:310. The product reads an origin by `invite::kept_page` (invite.rs:178, group.rs:262, door.rs:1302), each caller saying its own refusal | 6 | **cut**: groups-economize left it a follow-up because new.rs asserts its refusal names the file "origin". No queue took it up (`grep -rn read_origin ralph/next/*/STATE.md`: none). What that test guards is #20's rule, only no file is an absence (REVIEW_FINDINGS #20: "each reader, planted to read every failure as `None`"): the test asks `kept_page`, the reader the product runs, that an unreadable origin is refused and never `None`. The file's name in the refusal was a wrapper's that nothing in the product said |

The rest is what the earlier surveys read and kept (`At::Here`,
`MemoryLookup`, `LINK_WITHIN`, which the page compiles, the core's 9
re-exports only tests read, the node lib.rs:41 glob shadow, which is the
demotion's own effect), at their lines now.

`scripts/cfgtest.py` finds node.rs:199-200 and :385-386 (`Node`'s
`routes`) and :946-947 (`pub use tests_support::splice`, #7) as before, 3
lines, and nothing new.

`scripts/jsnames.py` over the page's JavaScript (12 files,
`data/jsnames.txt`): every top-level name is said again. Its plant, a
function appended to web/app.js in the scratch clone and never called,
went red: `web/app.js: plantedNeverCalled said 1`; with the file restored,
green (`data/jsnames-plant.txt`).

## 2. What no check, test or person runs: nothing

`git diff --name-status 02e7e75c a0129152 -- scripts` modifies
lean_check.py and vocab_check.py and adds no file.

## 3. The clone report: nothing to cut

A fresh corpus, since the latest audit's (gv-95b24ac8) predates 29 changed
files (`git diff --stat 95b24ac8 a0129152 -- crates web scripts Cargo.toml
Cargo.lock`): `git archive a0129152 crates web scripts Cargo.toml
Cargo.lock` into /tmp/canon-a0129152 (230 files; apps/ left out, as it is
not counted), `code index . --corpus-id canon-a0129152` (3148 chunks in
273 s, `data/index.out`) and `refresh` there (rust 17459 symbols, 101448
references; typescript and python "skipped (export failed)", I4;
`data/refresh.out`). Then `~/dev/commonwealth-ai/target/debug/sovereign-cli
code dry-report --corpus-id canon-a0129152 --scope <dir> --threshold <t>`
for each crate, `web` and `scripts`, at 0.95 and 0.90
(`data/dry-<scope>-<t>.md`). A first pass before the refresh said
could-not-judge for every exact tier; these files are the second.

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 4 | 0 | 11 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 1 | 1 | 10 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (I4) | 0 | 2 |
| scripts | could-not-judge (I4) | 0 | 0 |

Read cluster by cluster: the exact ones are three-line accessors on
different types (`lines`, `is_empty`, `as_str`, `signed_by_link` on the ask
and the request, api.rs's two `addr`) and test code (runtime's `open`); the
near ones are test code (core's 0.90 list, ten of eleven) or overlapping
spans, one function inside its neighbour (I32: home.js `running`/`look`,
sign_in.rs `of`/`forget`, synced.rs `kept_files`/`kept_at`, question.js
`door`/`judge`, redeem.rs `router`/`ask_route`, choices.rs `load`/`read`,
rail.rs `live_push`/`live_drain`, share.rs `helper_of`/`signs_in`, lend.rs
`from_mbit`/`bps`, op.rs `from_hash`/`as_bytes`, core lib.rs
`name`/`verify`, web/app.js `uses`/`choicesOf`, lend.js
`cookieFor`/`keepJar`). None is a cut.

## 4. Said twice: the open findings

Open rows under principle 8 opened since groups-economize's survey (#178
on), each read at a0129152; the earlier ones are as the earlier surveys
read them.

| # | Site | Lines a fix saves | Why |
|---|---|---:|---|
| 180, 188, 189 | | — | fixed (groups-leave-others (c), northstar) |
| 185 | origin.rs:268-270 `" on %-d %b"`; cli.rs:1354-1357 `", since %-d %b"` | −2 (costs 2) | the finding's fix, one `day_said(unix) -> Option<String>` in svrngs-node, is 4 counted lines and saves 1 at each caller: one decider, not a cut |
| 186 | apps/ | — | not counted (with #150) |
| 202 | build.rs reading apps/ into the binary | — | a choice of direction, the operator's |
| 204 | the seed kept twice, `seed.json` and `roster.json` | — | with #107; a change to what a computer holds is the operator's (CHARTER, "existing groups") |

## 5. What this queue's rows will replace: left as they are

So no cut is written twice:

- **`carried.rs`'s serving of a folder's files.** canon-pin (b) serves a
  carried app at the hashes its bringing recorded, where `Folders::file`
  (carried.rs:217-229) serves the hash the group's layout places now, and
  `brought` (carried.rs:145-163) gains the pin. `Folders::carried`
  (:176-192) and `app_of` (:165-168) read the layout too. Each is
  rewritten by that row; nothing in them is cut here.
- **The page policies the floor reads.** canon-floor (b) pins each served
  policy token by token: carried.rs:63-73 (`FLOOR`, `POLICY`), home.rs:68,
  web/index.html, web/open.html, and the listeners at api.rs:235 and
  room.rs:130. Each is left as it is served, so the floor pins what runs.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, `group::read_origin` | 6 | 0 |
| §1, `iroh::ALPN`'s re-export | 0, not a cut | 0 |
| §2, §3 | 0 | 0 |
| §4, #185 | costs 2, not a cut | 0 |
| §5 | left for the rows that rewrite it | 0 |
| **Total cut here** | **6** | **0** |

The cut would leave the repository at **16,413**, its cap one past at
**16,414**, and this queue's ceiling **16,564**. svrngs-core stays
**2,991 / 2,992**, its ceiling **3,032**.
