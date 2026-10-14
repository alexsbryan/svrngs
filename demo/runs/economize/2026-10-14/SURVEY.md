# What the product carries that it does not need — events-economize's survey, 2026-10-14

Read-only, at main `460898b`. Every count is `scripts/lean_check.py`'s
rule, per file by `scripts/count.py` here (start-economize's, unchanged),
agreeing with its total: **16,236 / 16,237**, and svrngs-core **3,149 /
3,150** (`python3 scripts/lean_check.py` at 460898b). The follow-up's
budget (the operator, 2026-10-14, "+80 ceiling, fixes included") is +80
over the cap this pass leaves.

The instruments are start-economize's (`../2026-10-13/SURVEY.md`): the
compiler with every `pub` demoted, the clone report, and the open findings
that name a thing said twice. Grep only checks what they name.

## What grew since start-economize's survey

`data/delta-9267bc1.txt` (`scripts/delta.py` over `count.py` at
`9267bc1` and at `460898b`): 16,036 → 16,236, **+200**. That is
start-economize's cuts (−50), onair-economize's (−36) and onair's raises
(+286, 15,950 → 16,236). The growth is in five files:

| File | At 9267bc1 | At 460898b | |
|---|---:|---:|---:|
| crates/svrngs-node/src/blob.rs (onair-blob) | 0 | 202 | +202 |
| crates/svrngs-node/src/surface.rs (onair-economize (a), onair-sign) | 183 | 253 | +70 |
| crates/svrngs-core/src/app.rs (onair-sign) | 0 | 26 | +26 |
| crates/svrngs-node/src/folder.rs (onair-blob) | 525 | 544 | +19 |
| crates/svrngs-node/src/home/question.js (onair-economize (d)) | 15 | 25 | +10 |

The rest is under ten lines a file. events-app, events-run and events-doc
added nothing counted: `apps/` is not counted, and `git diff --stat
43c0552 460898b -- crates web scripts Cargo.toml Cargo.lock` is empty.

## 1. The compiler: nothing new reached only by tests

In a scratch clone of `460898b` (/tmp/ev-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the four
crates and `web/src` `pub(crate)`. It split each `pub use a::{b, c}` to one
name a line, and kept the two glob re-exports and the page's
`#[wasm_bindgen]` items `pub`. This is the 2026-10-13 survey's demotion
(done by hand there), written as a script. Then `scripts/loop.sh` built
the product (`cargo build --workspace --offline`) and `scripts/repub.py`
made `pub` again each item a downstream crate was refused, round after
round. Three refusals repub.py does not parse were made `pub` by hand:

- the `ProcessManager` trait, whose associated type runtime's
  journal.rs:588 reaches through `react`;
- `NotASyncPath` (E0446, a private type in a public interface);
- `Refusal::exit_code`, which main.rs:57 calls across the lib and the bin.

`ProcessManager`'s re-export was then demoted again: it builds, and it
warns as unused, as it did at 9267bc1. Both builds ended
`build-finished success: True`: the product, and `web/` for
`wasm32-unknown-unknown`.

What still warns (`data/warnings-prod.txt`, `data/warnings-web.txt`; the
whole scratch diff, `data/demoted.diff`) is what the 2026-10-13 survey
read and kept, and nothing else:

| Warning at 460898b | Read at 9267bc1 as |
|---|---|
| crates/svrngs/src/relay.rs:57 `At::Here` never constructed | not a cut: how a test runs `svrngs relay` |
| crates/svrngs-node/src/iroh.rs:68 `MemoryLookup` unused import | not a cut: the version pin in one place |
| crates/svrngs-core/src/lib.rs, 13 re-exports only tests read | not a cut: each test would need a private module made public |
| crates/svrngs-node/src/lib.rs:41 `alpn` shadows the glob re-export | the demotion's own effect, not code |

None of blob.rs, app.rs, the live lane's `Live`, `SIGN_ROUTE` or
`standing_asker` warns: each is reached by product code.

A scan of every counted source for a test cfg ahead of an item other than
`mod x;` (`scripts/cfgtest.py`, start-economize's, unchanged) finds one:

- node.rs:187-188 and :368-369, `Node`'s `routes` field under
  `#[cfg(any(test, feature = "test-support"))]`, read by
  node/tests_support.rs:12 `api_routes`, which tests/surface.rs:95 and
  :332 call. That is 2 lines.
- **Not a cut.** A test asks the API as a caller no socket can be, and
  nothing else gives it the router.

The page's JavaScript has no compiler. `scripts/jsnames.py` counts every
top-level `function`, `const`, `let` and `class` of `web/*.js` and
`crates/svrngs-node/src/home/*`, and how often each name is said in those
files and their HTML: every name is said again. Its plant was a function
appended to web/app.js and never called, and it went red:
`web/app.js: plantedNeverCalled said 1`. With the file restored it went
green.

Suppressions: `#[expect(dead_code)]` at door/choices.rs:76 and
`#[allow(dead_code)]` at web/src/lib.rs:31, as on 2026-10-13. Neither
hides anything.

## 2. What no check, test or person runs: nothing

Every file in `scripts/` is named by what runs it (`git grep -l <name>`
outside `demo/runs/` and `ralph/`): check.sh, lean_check.py,
vocab_check.py and principles_check.py by check.sh; build-web.sh by
check.sh, the browser tests' harness and build-release.sh; build-release.sh
by tests/install.rs; relay-answers.sh by tests/relay.rs; ports.py by DESIGN
§4's routine; deploy-*.sh by the operator.

## 3. The clone report: nothing

`~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report
--corpus-id prod-43c0552 --scope <dir> --threshold <t>` was run for each
crate, `web` and `scripts`, at 0.95 and 0.90 (`data/dry-<scope>-<t>.md`).
The corpus is REVIEW-audit-events-auto-1's: 43c0552's crates/, apps/,
web/, scripts/, Cargo.toml and Cargo.lock, indexed 2026-10-14
(`ralph/REVIEW_FINDINGS.md`, that audit's "The instrument"). No product
file changed between 43c0552 and 460898b (the empty diff above), so it is
HEAD's product.

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 4 | 0 | 12 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 1 | 0 | 7 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (I4: no SCIP for JavaScript) | 0 | 2 |
| scripts | could-not-judge (I4) | 0 | 0 |

Each is one the 2026-10-13 survey read, or fewer:

- **The exact groups** are the same three-line accessors on different types:
  `as_str`, `lines`, `signed_by_link`, `is_empty`, api.rs `addr`. The
  runtime's `open` is in test files.
- **The near clusters in core and the node** are the same neighbours and
  the same test files: lib.rs `name`/`verify` and `actor`/`sign`, op.rs,
  record.rs `decide`, sign_in.rs `of`/`forget`, synced.rs
  `kept_files`/`kept_at`, redeem.rs, choices.rs `load`/`read`, rail.rs
  `live_push`/`live_drain`, lend.rs. The origin.rs/surface.rs pair is gone
  (onair-economize (a) made it one decider, `read_by`).
- **web**: lend.js `cookieFor`/`keepJar` as before, and app.js `uses` (:285-307)
  with `choicesOf` (:288-303). `choicesOf` is a closure inside `uses`, so
  the two spans are one span counted twice (I32's window), not a clone.

No cluster has blob.rs, app.rs or the sign route on either side.

## 4. Said twice: the open findings

Every open row of `ralph/REVIEW_FINDINGS.md` under principle 8 that names
counted code, each read at 460898b. Rows the 2026-10-13 survey already
read are marked "as then".

| # | Site | Lines a fix saves | Why |
|---|---|---:|---|
| 107 | crates/svrngs-node/src/synced.rs:127-133 `write_whole`, beside declared.rs:128-138 `write_whole`, whose doc says it is "the one way this node writes a file whole" | **2** | **A cut.** dfa0b31 (lean-4 (a)) made declared.rs's the one home of #107's three. synced.rs's landed the same day from syp-4 (e) (db60756) and was not among them. It writes the same way: the partial beside the file, then renamed into place, with an error naming whichever of the two failed. Asking declared.rs keeps every error (`FolderError::Read`, the same path). Only the partial's name changes, `synced-<g>-<f>.part` to `.synced-<g>-<f>.json.partial`. Neither is a kept state to `kept_files` (synced.rs:345, `synced-…` ending `.json`). No test and no person reads either name (`git grep -n '\.part\b'` finds synced.rs:130 alone). |
| 146 | ring_routes.rs `standing_asker`, register.rs `Standing::names`, blob.rs:89-97 `standing_in` | −4 (it costs) | Its fix is an accessor, about +4. It is events-principles (c). |
| 147 | cli.rs:306-307 `VALUED`, a closed set kept as text | − (it costs) | events-principles (d). |
| 136 | cli.rs `ask_running` beside api.rs `kept_address` | 0 | as then: a change in what a person is told |
| 135 | cli.rs `join_group`'s posture beside `SHIPPED` | 0 | as then: a change of signature |
| 138 | group.rs `Asking::found` against cli.rs `new_group` | 0 | a change in what `svrngs new` does, which tests/new.rs:74-79 pins the other way |
| 42, 48, 63, 89 | issuer.rs / web/src; the page's key reads; lend.js / shim.js; sign_in.rs / lend.js | — | as then: lean-4 (a) said why not (f22f7cb) |
| 49 | the lend's address built and parsed by hand in app.js, open.js, lend.js and door.rs | − (it costs) | One home is a module every one of them imports, a shell file in sw.js and build-web.sh: more lines, not fewer. |
| 58 | a lender's 64-hex key read into a label's bytes | 0 | `Label::of_hex` (svrngs-core pass.rs) is now the one reading; node.rs:538, :545 and web/src/lib.rs:335 ask it |
| 111 | lend.js's pass-and-lend guard | 0 | the refusals are in one set of words now (lend.js:280-282, "#111") |
| 113 | home.js and app.js walking question.js's `uses` | 0 | onair-economize (d) made it one walk, question.js `uses` |
| 2 | chain.rs's act header written five times | — | chain.rs no longer exists |
| 31, 137, 150, 7, 8, 62, 68, 70, 71 | test code and `apps/` | — | no budget line |

Found while reading #107, and **not a cut**: crates/svrngs/src/identity.rs:58-71
`save_key` writes this computer's key through a `.tmp` renamed into
place. It is forked (`ported from commonwealth-ai
…/commonwealth-transport/src/identity.rs@e1a43bcd2`) and behaves
differently from declared.rs's two writers:

- it calls `sync_all` before the rename;
- it narrows the key to 0600 only after the rename, ignoring a failure
  (`let _ =`, "Best-effort").

Asking declared.rs's writers would change what is written and when. Its
ignored hardening is principle 6's, the same thing declared.rs:105-109
fixed in its own fork. That is a follow-up with a test, red first.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, the compiler | 0 | 0 |
| §2, what nothing runs | 0 | 0 |
| §3, the clone report | 0 | 0 |
| §4, the open findings: #107's fourth writer, synced.rs `write_whole` | ~2 | 0 |
| **Total** | **~2** | **0** |

Onair's +286 went to blob.rs, the sign route, the app's signature and the
live lane's sender. The compiler finds all of it reached, and the clone
report finds none of it written twice. The two economizing passes before
this one took what these instruments reach. Measured again in the cut's
commit, the repository would land near **16,234**. svrngs-core stays
**3,149**: nothing in it is unreached or written twice.
