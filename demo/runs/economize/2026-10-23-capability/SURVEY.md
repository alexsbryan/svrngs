# What the product carries that it does not need — capability-economize's survey, 2026-10-23

Read at main `5dd53719`, before any cut. Every count is
`scripts/lean_check.py`'s rule, per file by `scripts/count.py` here
(oidc-economize's, unchanged), agreeing with its total: **17,634 /
17,635**, and svrngs-core **3,149 / 3,150** (`data/lean-before.txt`). One
counted line was left under each cap. The bundle's one ceiling is the
operator's to give after this pass (CHARTER.md, "Leave for the operator";
AGENTS.md "Done"); §6 gives the number to ask with.

The instruments are oidc-economize's (`../2026-10-21-oidc/SURVEY.md`):
the compiler with every `pub` demoted, the clone report, and the open
findings that name a thing said twice. Its scripts are copied here
unchanged but for the scratch paths (`/tmp/cap-econ-*`, loop.sh and
refix.py). One instrument is new, `scripts/windows.py`, a line-window
pass: each run of three counted lines said in two places of the product,
so a fold smaller than the clone report's chunks is seen. It found every
cut here.

## What changed since oidc-economize's survey

`data/delta-a167c203.txt` (`scripts/delta.py` over oidc-economize's
`count-HEAD.txt` at `a167c203` and this one): 16,927 → 17,634, **+707**;
svrngs-core 3,072 → 3,149, **+77**. It is the oidc queue's provider
(oidc.rs, oidc/, token.rs: +680), the retired sign-in hook (door/sign_in.rs
−420, share.rs −25), the room and the network as domains of their own
(room/, network.rs: +300 less room.rs's −30), and the door's front channel
(door.rs +86). Each of those rows ran its own pass or audit:
REVIEW-audit-oidc-auto-1 and REVIEW-audit-models-auto-1
(`ralph/REVIEW_FINDINGS.md`) and the room's economizing pass of
2026-10-22 (`scripts/lean_check.py`'s budget comment).

## 1. The compiler: one new item, cut

In a scratch clone of `5dd53719` (/tmp/cap-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the four
crates and `web/src` `pub(crate)` (76 files). `scripts/loop.sh` built the
product and `scripts/repub.py` made `pub` again what a downstream crate
was refused, round by round, with `scripts/refix.py` for the field and
method errors it does not parse. The hand edits were the ones the earlier
surveys name, and one more:

- `standing_reads` (membership.rs:789; E0603 through `pub use
  membership::*`);
- `NotASyncPath` (synced.rs:37; E0446);
- `CliError::exit_code` (cli.rs:640), which the binary's `main.rs:66`
  calls across the crate boundary (E0624).

The product ended `build-finished success: True`, and `web/` for
`wasm32-unknown-unknown` with 0 errors on its first round. What still
warns is `data/warnings-prod.txt` and `data/warnings-web.txt`.

The lines that say an item is unreachable, against oidc-economize's:

- `LINK_WITHIN` (dial.rs:46), which the page compiles; `iroh::ALPN`'s
  re-export (iroh.rs:48) and `MemoryLookup` (iroh.rs:64); `At::Here`
  (relay.rs:63), how a test runs `svrngs relay`. The same four, each read
  and kept by earlier surveys.
- **New: `Key::kid` (oidc.rs:126), "method `kid` is never used".** Its
  three callers are `oidc/tests.rs:48`, `:74` and `:99`, a child module
  that can read the private field. **Cut, 3 lines** (d2596b56).
- `lib.rs:41`, "private item shadows public glob re-export": this pipe's
  own, the demoted `pub(crate) use svrngs_core::alpn` beside a glob. Not
  a product finding.

The svrngs-core re-exports that warn unused (`Reads`, `PayloadError`,
`act_id`, `redemption_message`, `ring_op_message`, `sign_ring_op`,
`NotASyncPath`, `DIGEST_V`) are the list start-economize read and kept
(`../2026-10-13/SURVEY.md`:59): only other crates' tests read them,
through modules that are private. `project::Invites`, `project::Wire`
and `synced::Entry` no longer warn, the pipe's over-republications oidc's
survey named.

`scripts/cfgtest.py` finds the same three lines under a test cfg in
counted files as oidc-economize did, now at `node.rs:204-205`, `:397-398`
and `:959-960` (`data/cfgtest.txt`): 3 lines, nothing new.

`scripts/jsnames.py` over the page's JavaScript (13 files,
`data/jsnames.txt`): every top-level name is said again. Its plant, a
function appended to web/app.js and never called, went red,
`web/app.js: plantedNeverCalled said 1`; with the file restored, green
(`data/jsnames-plant.txt`).

## 2. What no check, test or person runs: nothing

`git diff --name-status a167c203 5dd53719 -- scripts` changes three
files and adds none: `check.sh` (each step's time, clippy over the
`test = false` targets, the ceiling canary's never-ran), `lean_check.py`
(the budget comment and caps) and `trust_floor.txt` (the room's two lines
moved to `room/owner.rs` and `network.rs`). Each is run by `check.sh`.

## 3. The clone report

A fresh corpus, since REVIEW-audit-models-auto-1's (at 076f057a) differs
from 5dd53719 only by two renames of a type alias (carried.rs `Served` →
`Given`, room/owner.rs `Asked` → `TurnAsk`; `git diff 076f057a 5dd53719 --
crates web`), read again so the report is this commit's own: `git archive
5dd53719 crates web scripts Cargo.toml Cargo.lock` into
/tmp/cap-econ-corpus (250 files; apps/ left out, as it is not counted),
`sovereign-cli code index . --corpus-id cap-econ-corpus` (3499 chunks in
522 s, `data/index.out`), `sovereign-cli refresh` (rust 19181 symbols,
111317 edges, `data/refresh.out`), then `code dry-report --scope <dir>
--threshold <t>` per crate, `web` and `scripts`, at 0.95 and 0.90
(`data/dry-<scope>-<t>.md`).

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 4 | 0 | 11 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 1 | 0 | 8 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (I4, I34) | 0 | 2 |
| scripts | could-not-judge (I4) | 0 | 0 |

Against the models audit's table, the same counts. The exact clones are
the older three-line accessors on different types (`signed_by_link`,
`lines`, `as_str`, `is_empty`, api.rs `addr`) and a test helper (runtime
`open`). The near clusters are test pairs and spans that overlap, one
function inside its neighbour (I32): door.rs/surface.rs `lends`, home.js
`running`/`look`, home.rs `lent`/`change`, node synced.rs
`kept_files`/`kept_at`, choices.rs `load`/`read`, rail.rs
`live_push`/`live_drain`, lend.rs `from_mbit`/`bps`, app.js
`uses`/`choicesOf`, lend.js `cookieFor`/`keepJar`. Each was read by an
earlier survey or audit and kept. None is a cut: what this survey cuts
is below the report's eight-line floor, which is why §4's pass exists.

## 4. Said twice: the line-window pass and the open findings

`scripts/windows.py 3` over every non-test product source of the four
crates, `web/src` and the page's JavaScript (`data/windows-3.txt`, 126
runs). Most are a struct's field list beside the one that builds it, a
handler's extractor arguments, `use` lines, a log's `target:` and its
fields: the same words because they say the same shape, with nothing to
fold. Three are one thing written twice, each in a file this queue
touches or beside one, and each is cut:

| Cut | Where | Lines | Commit |
|---|---|---:|---|
| a message's `Connection` names, read once | door.rs:612 and :645, `strip_hop_by_hop` and `upgrade_of` | 6 | caf15672 |
| the page's IndexedDB opened one way | web/store.js:43 and :175, `open` and `lendDb` | 3 | cda6851f, its helper renamed `openDb` in b6faecbf |
| a computer's key parsed once, with its one refusal | web/src/lib.rs:91 and :181, `bind_for` and `dial` | 4 | 93a90fd7 |

Read and not cut, each for its net:

- `cli.rs:1520` and `:1554`, `match origin { Some(page) =>
  Some(checked_page(page).await?), None => None }` in `new` and `link`: a
  helper taking the `Option` saves at most 2, and `checked_page`'s own
  signature is the tests' (tests/new.rs:544-579, tests/join_page.rs:135,
  :275), so it stays.
- `removal.rs:142` and `:293`, a name asked that is no name, said as
  `NotInGroup`: the handlers answer an `Outcome`, not a `Result`, so a
  helper saves no line. Parsing the name in `RemovalAsk`'s serde would
  answer a bad name as a 422 where it says `not_in` today: a behaviour,
  not a pass's.
- `oidc/provider.rs:293` and `:396`, the declaration looked up with its
  `missing` refusal in two handlers; `:334-363`, five `refuse(BAD_REQUEST,
  ..)`: each is one refusal per field, said where it is decided.

Open since oidc-economize's survey, principle 8: **#219** (the provider a
second home of the clock, `oidc/provider.rs:72-77` beside door.rs's
`unix_now`) and **#221** (the authorize branch before `choices::check`).
Both change behaviour (#219 says a clock before 1970 where it now
substitutes 0; #221 is the design's and the operator's), so neither is a
pass's cut. #217 names test code only. The older ones stand as oidc's
survey read them.

## 5. What the queue's rows will touch

Each was read at 5dd53719, so no cut here is written again by a row.

- **The door's reach: `origin.rs` `app_of` (:31-50) and `fetch`
  (:52-57).** capability-navigation changes the one `own` condition:
  `mode == "navigate"` is the person's own page today whatever `Origin`
  it carries. Nothing to cut: `fetch` is read by `app_of` and by
  `door::paged`, `app_of` is 20 lines, and every branch traces.
- **The yes: `door/choices.rs` (253 counted).** capability-open keeps a
  yes to an app that reads, "where the door already keeps a person's
  yes". `own_choose` and `own_choices` (:369-381) share
  `found(..).and_then(|lend| Ok((own(door)?, lend)))`; a helper saves no
  line. `judge` and `check` are the one decider for `--ask`.
- **`surface.rs` `reaches` (:329-343) and `reads` (:353-).** The open
  row's yes for `--reads` is judged here; 15 lines, every branch traced.
- **The carried apps: `carried.rs`, `crates/svrngs-core/src/synced.rs`
  `Runs` (:146-166) and `written` (:172-).** Authorship changes
  `on_behalf_of.unwrap_or(person)` (:182-184) to the signer, and keys
  changes `said`'s `(kind, person name)` to a key. Nothing dead: `Runs::of`
  is the one place what runs as a person is said.
- **The record: `rail.rs` `stamp` (:304-327), `record.rs` `Append`
  (32 counted), `lib.rs`'s `RailAct` and `membership.rs`.**
  capability-signed is where `on_behalf_of` for new acts is decided; its
  design row writes it first. A cut now would decide it for them.
- **The passes: `crates/svrngs-core/src/pass.rs` (141 counted).** The pass
  key row moves where the holder key lives in `web/`, not how a pass is
  signed (`Pass::sign`, `Pass::check` unchanged).
- **The pages: `home/home.js` `uses` (:168-174), `home/question.js`
  `uses` (:59-), `web/app.js` `uses` (:327-) and `opening` (:557-),
  `web/lend.js`, `web/open.js`, `web/store.js` `lendKept`/`lendKeep`.**
  The pass-key, open and what-acts rows. `question.js`'s `uses` is already
  the one both pages call; the names said in two files (`uses`, `post`,
  `said`, `opening`, `asking`, `trace`) are each a different thing in its
  own page, read above.

## 6. Each later row's lines, so the ceiling has a number

An estimate, not a measurement. Each range is grounded on the counted
code at the sites the row names, at 5dd53719, and set beside the note's
sizings (svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §10: consent
+80 to +150 and 0 to +3; authorship, with keys everywhere, +135 to +255
and +60 to +120). The note sized consent and authorship; the three fixes
it left to this pass.

| Row | Its sites | Repository | svrngs-core | Grounded on |
|---|---|---:|---:|---|
| capability-navigation | `origin.rs` `app_of`/`fetch`: a navigation whose `Origin` names another lend, and a frame's, judged as that lend's | +5 to +15 | 0 | `app_of` 20 lines, `fetch` 6; one more condition and its trace |
| capability-money | `apps/expenses/` | 0 (not counted) | 0 | fence 7: `apps/` is not counted |
| capability-pass-key | `web/app.js:264-273`, `open.js:44`, `store.js:165-181`, `lend.js:135`, `web/src` if the key is held behind the worker | +20 to +80 | 0 | `lendDb` and its keeps ~10; `dialFor`'s holder ~10; a key kept apart from the page and handed only to the worker, with the restart path, 20 to 60. Its upper end is could-not-judge until the row measures the trade-off it names |
| capability-open | `choices.rs` (a yes per person and app), `surface.rs` `reaches`, `home.rs`'s route, both pages' open buttons, `shelf.txt`'s words | +55 to +100 | 0 | note: +3 to +8 and +50 to +90; `choices.rs`'s `keep`/`kept`/`own_choose` ~60 is the shape a yes for `--reads` repeats in part |
| capability-what-acts | `home.js`, `question.js`, `app.js`'s group page; moves first | +30 to +70 | 0 to +3 | note: +25 to +55 / 0 to +3; both pages, so the top rises by the group page's half |
| HUMAN-effort-high, capability-authorship | a design and golden tests | 0 | 0 | tests and `demo/` only |
| capability-signed | `svrngs-core` `Grant`, folds by signer (`synced.rs:182-184`, `record.rs`, `lib.rs`'s line versions), `rail.rs` `stamp`, `blob.rs:179`, the phone's worker signing | +110 to +200 | +55 to +110 | note's authorship, less keys; `record.rs` 32 and `stamp` 24 are the sites it replaces in part |
| capability-keys | consent and authorship by key where by name (`synced.rs:152-155`, `Runs.said`, `changed`), "Ana (joined 3 Oct)" on the pages | +25 to +55 | +5 to +10 | note's authorship, its keys part; `svrngs remove`'s joined-date already in cli.rs |
| capability-release | docs and notes | 0 | 0 | not counted |
| **Total** | | **+245 to +520** | **+60 to +123** | |

The note's own sum for consent and authorship is +215 to +405 / +60 to
+123; the three fixes add +25 to +95 / 0, and what-acts on both pages up
to +15 more.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, the compiler: `Key::kid` | 3 | 0 |
| §4, the line-window pass: three folds | 13 | 0 |
| §2, §3, §5 | 0 | 0 |
| **Total cut here** | **16** | **0** |

`python3 scripts/lean_check.py`, before (`data/lean-before.txt`, at
5dd53719): 17,634 / 17,635 and 3,149 / 3,150. After the cuts: **17,618**
and 3,149. The repository's cap is lowered to one past where they land,
**17,619**; svrngs-core's stays at 3,150, one past already (038f0ff1; its
body names the store cut "7d…", which is cda6851f). The plant at the
lowered caps, one counted line in scripts/ and one in svrngs-core,
refused both, exit 1, and the tree passed again with them removed
(`data/ratchet-plant.txt`).

The row's close: the first `scripts/check.sh` failed the browser,
joining and conformance steps, because cda6851f named its helper
`opened`, which store.js:135 exports already, so the module did not parse
(`data/check-close-1.txt`:211, :227). b6faecbf named it `openDb`; the
second run passed every step, exit 0, in 530 s (`data/check-close.txt`).
No test was renamed or removed by any cut: the four cut commits and the
fix touch only `oidc/tests.rs`'s three assertions among test files.

The number to ask with: **+520 repository and +123 svrngs-core over the
caps this pass leaves**, at most 18,139 and 3,273, each row raising only
what it lands, the range's top so the pass-key row's open end and the
signed row's are inside it (NEEDS_HUMAN.md).
