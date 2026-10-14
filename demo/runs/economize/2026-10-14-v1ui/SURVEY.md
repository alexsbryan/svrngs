# What the product carries that it does not need — v1ui-economize's survey, 2026-10-14

Read-only, at main `4a2e8a7`. Every count is `scripts/lean_check.py`'s
rule, per file by `scripts/count.py` here (start-economize's, unchanged),
agreeing with its total: **16,314 / 16,315**, and svrngs-core **3,149 /
3,150** (`python3 scripts/lean_check.py` at 4a2e8a7). The v1 UI's budget
(the operator, 2026-10-14, "+650 repo, core unchanged") is +650 over the
cap this pass leaves.

The instruments are the 2026-10-13 and morning 2026-10-14 surveys'
(`../2026-10-13/SURVEY.md`, `../2026-10-14/SURVEY.md`): the compiler with
every `pub` demoted, the clone report, and the open findings that name a
thing said twice. Grep only checks what they name. The scripts here are
that survey's, copied unchanged but for loop.sh's scratch paths
(/tmp/v1ui-econ-*).

## What grew since events-economize's survey

`data/delta-460898b.txt` (`scripts/delta.py` over `count.py` at
`460898b` and at `4a2e8a7`): 16,236 → 16,314, **+78**. That is the events
queue's raises after its pass: events-asleep, events-wait, events-links and
events-principles, less #107's cut (synced.rs −3). The growth by file:

| File | At 460898b | At 4a2e8a7 | | What added it |
|---|---:|---:|---:|---|
| crates/svrngs-node/src/door.rs | 991 | 1013 | +22 | events-links: `LABEL_HEADER`, `PAGES_HEADER`, `pages` |
| web/src/lib.rs | 234 | 245 | +11 | events-asleep, events-wait: the capped dial, `held` |
| crates/svrngs-node/src/guest.rs | 326 | 337 | +11 | events-asleep: the bridge's capped dial |
| crates/svrngs-node/src/blob.rs | 202 | 213 | +11 | events-principles (a) (#148): `heard`, `given` |
| crates/svrngs-node/src/home/home.js | 117 | 124 | +7 | events-links (c): "Open a link from your group" |
| crates/svrngs-node/src/dial.rs | 0 | 6 | +6 | events-asleep, events-wait: `DIAL_WITHIN`, `held` |
| web/app.js | 309 | 313 | +4 | events-links |
| crates/svrngs-runtime/src/register.rs | 214 | 218 | +4 | events-principles (c) (#146): `Standing::member` |
| crates/svrngs-node/src/ring_routes.rs | 175 | 178 | +3 | events-principles (b) (#149): `err` traced |
| lib.rs, iroh.rs, home.rs | | | +3 | one line each |
| invite.rs, synced.rs | | | −4 | synced.rs: #107's cut |

`look.css` (efadec2) is not code by lean_check's `CODE`, and the merge's
counted change (build-web.sh, sw.js, home.rs) is net 0 but for home.rs's
one line.

## 1. The compiler: nothing reached only by tests

In a scratch clone of `4a2e8a7` (/tmp/v1ui-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the four
crates and `web/src` `pub(crate)` (67 files). Then `scripts/loop.sh` built
the product (`cargo build --workspace --offline`) and `scripts/repub.py`
made `pub` again each item a downstream crate was refused, round after
round. The three refusals repub.py does not parse were made `pub` by hand,
the same three as before:

- the `ProcessManager` trait (process.rs:43), whose associated type
  `Effect` the runtime reaches;
- `NotASyncPath` (synced.rs:37; E0446, a private type in a public
  interface);
- `Refusal::exit_code` (cli.rs:618), which main.rs calls across the lib
  and the bin.

The product ended `"build-finished","success":true` (round 2 of the last
loop: `errors 0 fixed 0`), and `web/` for `wasm32-unknown-unknown`
(`DIR=web ARGS="--target wasm32-unknown-unknown"`) on its first round.
The whole scratch diff is `data/demoted.diff`; what still warns is
`data/warnings-prod.txt` and `data/warnings-web.txt`.

Against the morning's warnings, line for line, two differ:

| Warning at 4a2e8a7 | Read as |
|---|---|
| crates/svrngs-node/src/dial.rs:39 `held` is never used (product only) | **not a cut.** dial.rs is compiled into the page too (web/src/lib.rs:36-37, `#[path]`), and the page asks `dial::held()` at web/src/lib.rs:133; the web build does not warn on it. Its doc says so ("The page's alone"). |
| crates/svrngs-core/src/lib.rs:145 `process::ProcessManager` unused import | the morning demoted that re-export again by hand and this pass did not; the morning read it as not a cut. |

The rest is what the 2026-10-13 survey read and kept (`At::Here`,
`MemoryLookup`, the 13 core re-exports only tests read, the node lib.rs:41
glob shadow, which is the demotion's own effect), at the same lines but
for cli.rs's two, two lines on.

A scan of every counted source for a test cfg ahead of an item other than
`mod x;` (`scripts/cfgtest.py`) finds node.rs:187-188 and :368-369 again,
`Node`'s `routes` field: 2 lines, as then **not a cut**.

The page's JavaScript has no compiler. `scripts/jsnames.py` finds every
top-level name of `web/*.js` and `crates/svrngs-node/src/home/*` said
again (12 files). Its plant, a function appended to web/app.js and never
called, went red: `web/app.js: plantedNeverCalled said 1`; with the file
restored, green: `every top-level name is said again` (`data/jsnames.txt`).

Suppressions: `#[expect(dead_code)]` at door/choices.rs:76 and
`#[allow(dead_code)]` at web/src/lib.rs:31, as before. Neither hides
anything.

## 2. What no check, test or person runs: nothing

`git diff --stat 460898b 4a2e8a7 -- scripts` touches lean_check.py and
build-web.sh alone, and adds no file. Every script is named by what runs
it, as the morning's survey found.

## 3. The clone report: nothing new in the product

`~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report
--corpus-id prod-8e97b27 --scope <dir> --threshold <t>` for each crate,
`web` and `scripts`, at 0.95 and 0.90 (`data/dry-<scope>-<t>.md`). The
corpus is REVIEW-audit-events-auto-3's: 8e97b27's crates/, apps/, web/,
scripts/, Cargo.toml and Cargo.lock (`ralph/REVIEW_FINDINGS.md`, that
audit's "The instrument"). Between 8e97b27 and 4a2e8a7 the product changed
only by the look's merge (`git diff --stat 8e97b27 4a2e8a7 -- crates apps
web scripts Cargo.toml Cargo.lock`: home.rs, home.css gone, index.html,
look.css, build-web.sh, web/index.html, sw.js), none of it a Rust function
the report reads but home.rs's stylesheet route.

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 4 | 0 | 12 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 2 | 0 | 7 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (I4) | 0 | 2 |
| scripts | could-not-judge (I4) | 0 | 0 |

Against the morning's reports, name for name: core's, the runtime's and
web's are the same groups (core's in another order). The node's has one
more exact group, `flush` in blob_tests.rs == relay_tests.rs: test code,
#152, no budget line. Nothing the events queue added after the morning's
pass is on either side of a cluster.

## 4. Said twice: the open findings

Open rows under principle 8 opened since the morning's survey, each read
at 4a2e8a7; the earlier ones are as that survey read them (#107 is cut:
synced.rs:129-131 now asks `declared::write_whole`).

| # | Site | Lines a fix saves | Why |
|---|---|---:|---|
| 146 (updated) | blob.rs:89-97 `standing_in`, surface.rs:229-234 | 0 | each reads `standing_of`'s map, not a `Standing`: `.contains_key` and `.get` on the map are the accessor already; `Standing::member` would add a conversion |
| 152 | blob_tests.rs, tests/common, tests/home.rs, relay tests | — | test code, no budget line |
| 153 | home.js:172 beside web/open.js:55-56 | − (+1) | a change of behaviour (`/\evil.example` refused), red first: not this row's |
| 154 | tests/browser/main.rs | — | test code |
| 155 | web/src/lib.rs:203, :249; web/lend.js:345 | − (it costs) | a change across the wasm boundary |
| 156 (principle 9) | web/src/lib.rs:83, :111, :133 `held: bool` | 0 at best | passing the config itself drops the bool and the branch but also the trace's `held: {held}` at :136, which says what a person's dial waited; an enum costs its declaration |

Found while reading, and **not a cut**: the dial's cap is applied twice
in the same shape, guest.rs:403-406 (`tokio::time::timeout`) and
web/src/lib.rs:197-200 (`n0_future::time::timeout`), four lines each. One
helper in dial.rs, which both compile, saves three lines at each site and
costs about six of its own: **net 0**. Not done: no line freed.

## 5. What 3.1's restructure will replace

The row asks what 3.1 (the door and the index, V1_UI_BUILD §3.1) will
write over, so nothing is cut now only to be written again. 3.1 redraws
the people and the lends of each group, on the home page first, then the
group's page:

| Span | Counted lines | |
|---|---:|---|
| crates/svrngs-node/src/home/home.js:52-57 `people` (one line of names) | 6 | the grid of names, lit or drawn off |
| home.js:85-99, `show`'s lends list and its unanswered/unheard/refused lines | 11 | the index |
| web/app.js:334-343, `enter`'s list of buttons | 10 | the index |
| web/app.js:355-365, `enter`'s unanswered/unheard/refused lines | 11 | the door: a computer that did not answer drawn off |

**Said twice there, for 3.1 to write once:** the three sentences for a
computer that did not say what its person lends are written in both pages
(home.js:96-98, app.js:356-358), differing only in "this computer" and
"your group's computer". question.js is already the one wording both
pages share (#110, build-web.sh's `SHARED`); 3.1, drawing those computers
off in both pages, can say them once there, about 3 lines fewer. Cut now,
3.1 would move them a second time.

3.2 moves `svrngs share`'s probe and table (crates/svrngs/src/share.rs,
129 lines) into the node, behaviour kept: moved, not cut.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, the compiler | 0 | 0 |
| §2, what nothing runs | 0 | 0 |
| §3, the clone report | 0 | 0 |
| §4, the open findings | 0 | 0 |
| §5, 3.1's spans | 0 now; ~3 when 3.1 writes the three sentences once | 0 |
| **Total** | **0** | **0** |

The events queue's +78 after its pass went to the dial's cap on both
sides, the lend's label and pages headers, #148's two silences and #146's
accessor. The compiler finds all of it reached, the clone report finds
none of it written twice, and the open findings that would touch it cost
lines or change behaviour. Three passes before this one took what these
instruments reach. **Nothing to cut**: the repository stays at **16,314**,
its cap at **16,315**, already one past the count, so the cap the pass
leaves is 16,315 and the ceiling for the v1 UI rows is **16,965**.
svrngs-core stays **3,149 / 3,150**.
