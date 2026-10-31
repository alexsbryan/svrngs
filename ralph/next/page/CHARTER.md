# svrngs page — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so. The exploration is
svrn-docs `rings/current/PAGE_STATE.md`. Read it whole before the first row:
its §3 states the rules every v2 line keeps.

## What the operator decided, 2026-10-31

- **Preact with htm, vendored, no build step, and a hand-written machine.**
  The operator: "Let's explore Preact with htm, vendored with no build step,
  plus a small hand-written state machine; but make sure that we enumerate
  what the desktop app satisfies with its frameworks and make sure we're
  able to do all that state management, updating, unidirectional data flow,
  etc." PAGE_STATE §2 is that enumeration, and §5 the spike that ran under
  the live page's policy.
- **The versions.** The operator: "I'm aligned with the 11.1.1 version."
  Preact 11.1.1 and htm 3.1.1. PAGE_STATE §4's softer pin is overruled.
- **Scaffold first.** The operator: "Let's do scaffold first, then extrude
  features in the house style."
- **Components planned.** The operator: "Let's leverage reusable components
  too (so we should plan the component we will need)." The catalog is
  "The parts" below.
- **v2 beside v1, then one switch.** The operator: "Do the work in parallel,
  then we do an atomic switchover when this v2 UI is done; we make it so we
  can run the browser tests against this new UI and when we have functional
  parity we switch over and remove the old code". This overrides PAGE_STATE
  §6's order, which moved v1 in place.

## Settled by the session, 2026-10-31, the operator able to overrule

Each is a reconciliation of four plans (scaffold, components, phone, home)
read at `46ba8288`. STATE.md's opening lists which of these are still the
operator's to answer.

1. **Paths.**
   - `web/v2/` is the phone page. Its entry is served as `/app.js` in the
     v2 build, because `drive.mjs:40`, `join.mjs:39` and `visit.mjs` find
     `./wasm/` and `./store.js` from `script[type=module]`'s src. Its
     flows are served under `/v2/`.
   - `crates/svrngs-node/src/home/v2/` is the home page.
   - `crates/svrngs-node/src/home/ui/` is shared by both and served at
     `/ui/` on each. It holds the core (`machine.js`, `view.js`,
     `trace.js`), the primitives (`parts.js`), `copy.js`, and one file for
     each shared part (`ui/<part>.js`).
   - Only `ui/view.js` imports `web/vendor/`.
   - v1 is untouched: `web/{index.html,app.js,sw.js,…}`, `home/home.js`,
     `home/index.html`, and `question.js`. v2 imports `question.js`'s word
     functions only, never its DOM builders.
2. **Vendored as `.js`, not `.mjs`.** `web/tests/page.mjs:20-24` answers
   `.mjs` as `application/octet-stream`, and under `nosniff`
   (`web/vercel.json:8`) a module is refused.
   - The files: `preact.js` (11.1.1), `hooks.js`, `htm.js` (3.1.1).
   - Two edits, both said in the README: `hooks.js`'s bare `preact`
     becomes `./preact.js?v=__VERSION__`, since the policy allows no
     importmap; and each `sourceMappingURL` trailer is stripped.
   - htm's LICENSE comes from htm's repository at 3.1.1, because its npm
     tarball carries none. The README carries the sums that
     `apps/pinned.mjs` reads.
   - **One URL per module.** Every import into `vendor/`, `ui/`, `v2/` or
     `question.js` carries `?v=__VERSION__`. Two URLs load Preact twice,
     and the spike's hooks crashed on it (`reading '__H'`).
   - The phone build stamps the shared files too. The home page leaves the
     literal, which matches itself.
3. **v1 frozen.**
   - `web/tests/v1.test.mjs`, in the gate's `page laws`, pins the sha256 of
     each v1 page file.
   - `scripts/build-web.sh` with no `--ui` is byte-identical to the base,
     checked in the row that touches it.
   - `--ui v1|v2` is a closed set, `v1` when absent.
4. **The phone's v2 `index.html` is derived, not written.**
   - At build, `web/index.html` is taken with what lies between `<body>`
     and its `<script` line replaced by `web/v2/body.html`, as `404.html`
     is derived today (`build-web.sh:133-136`). The build fails unless
     `<head>` is byte-equal.
   - So there is one page policy, and `scripts/trust_floor.txt:60-67` is
     unchanged.
   - Each page has one root, `Page` and `Home`. components.md's islands
     are superseded (phone.md): `#status`'s class and `#join`'s and
     `#ask`'s `hidden` are attributes of the island elements themselves,
     which a `render` into them cannot set.
5. **The v2 home page is served only by a test build.**
   - It is served under the existing `test-support` feature
     (`crates/svrngs-node/Cargo.toml:73-76`, "What only a test asks of a
     node"), together with a runtime choice: a builder method in process,
     `--ui v2` for the binary.
   - home.md's `ui-v2` feature is superseded, so one test build drives
     both UIs.
   - `web/tests/release.test.mjs` proves that no release enables it. The
     release build refuses a binary that carries the marker
     `svrngs-ui-v2`.
6. **The switch.**
   - `SVRNGS_UI=v1|v2` is read once, in
     `crates/svrngs-node/tests/browser/harness.rs` `ui()`, into
     `enum Ui { V1, V2 }`. Unset means V1, and any other value panics,
     naming the two.
   - It feeds `site()` and every node start in a target that includes
     harness.rs (`git grep -l browser/harness.rs crates`).
   - The marker `svrngs-ui-v2` (an HTML comment in `web/v2/body.html` and
     `home/v2/index.html`) is asserted present under V2 and absent under
     V1, so v2 can never silently be v1. An empty v2 fails loudly.
   - The tests that pass on the empty skeleton are listed apart, in the
     baseline: they are not parity v2 earned.
   - `scripts/parity.sh` writes passed, failed, could-not-judge or never-ran
     for each test to `target/parity/verdicts.tsv`, under the gate lock.
     `check.sh` runs it only from page-parity to the switchover.
   - **v2-only tests: one mechanism, a test target of their own,
     `crates/svrngs/tests/page_v2/`.**
     - Its `main.rs` holds `const UI: Ui = Ui::V2;` and one `mod` per
       flow: `phone_<flow>.rs`, `home_<flow>.rs`.
     - It is `test = false` with its own gate step, `page v2`, so it runs
       on every gate whatever the switch says.
     - It lives in `crates/svrngs` because the home page's leave and join
       run the binary (`leave_page.rs`, `home_group.rs`), and in-process
       nodes link there too.
     - Red first: the row flips `UI` to `Ui::V1`, watches its test fail,
       names that run in the commit, and restores it.
     - Superseded: scaffold.md's `v2_only!` macro and home.md's
       `#![cfg(feature = "ui-v2")]` files under `tests/browser/v2/`.
7. **The order** is STATE.md's dependency graph:
   - the serial base: economize, the rung, vendor beside machine, skeleton,
     parts, frame beside switch, copy, then the phone's enter and the home's
     drive and root;
   - every flow row in parallel, each depending only on the base and on the
     row that builds a shared part it names;
   - then page-components, parity, the operator's go, one atomic
     switchover, and the operator's deploy.
8. **Two test-driver edits, each its own row, green on v1 first, with
   assertions unchanged.**
   - `web/tests/home.mjs:101` waits for a reload or a redraw (page-home-drive).
   - `web/tests/tabs.mjs:47` waits for a fresh answer, in place of
     writing `#used`'s DOM under Preact's tree (page-tabs-drive).

   These are the only v1 test files v2 changes.
9. **The home page's query-string notices may go in v2.** That covers
   `/?made=`, `?joined=` and `?left=` (`home.js:42`, `:364`, `:409`): nothing
   outside home.js links to them (grep at `46ba8288`). Each visible change
   is its own commit, after the move, with a v2-only test.
10. **One owner per file.** See "Who owns each file" below. Parallel rows
    never touch the same file.
11. **The budget.** A `v2` rung of lean_check, sized below. The switchover's
    bar is `.` at or under 19,481 once v1 is deleted. On these estimates
    that bar is not reached without a pass and a raise.
12. **Parallel runs, a proposal.** See "Running rows in parallel" below.
    Until the operator's go, the queue runs serially.

The session also settled these, from what the four plans left open:
- **A shared part is exactly one of `question.js`'s DOM builders**, which
  both pages already call: `table` becomes `ui/shelf.js`; `door` and `people`
  become `ui/door.js`; `visiting` becomes `ui/visitors.js`; `changes` becomes
  `ui/changes.js`; `acts` becomes `ui/acts.js`; `uses` becomes `ui/uses.js`.
  `keeping` and `row` become `Keep` and `Row` in `parts.js`, and `el` goes.
  - What v1 draws per page stays in that page's flow file: the index
    (`app.js:537-541` against `home.js:179-191`), the ask, the removals
    list, Reading's Stop.
  - Unifying those is after the switchover. So home rows need no layout
    prop from a phone row.
- **`web/v2/computer.js`** (the computer's line) is page-copy's, then
  page-v2-join's. Ownership passes along a dependency, never between
  rows running at once.
- **After the switchover** the flows live in `web/page/` and
  `home/page/`, served at `/page/`. `ui/` keeps its name.

## The parts

Each primitive and shared part is a pure view: props in, vnodes out, and
`send` for what it asks. It replaces the v1 code named.
- **Primitives** (`ui/parts.js`, page-parts):
  - `Act`: a button with its answer beside it, or in its place;
  - `Said`;
  - `Fold`: a `<details>` whose open is in the store. A driver's `open =`
    fires `toggle`, so `home.mjs:66` and `:134` hold;
  - `Field`: the `for` and `wrap` shapes, and `select`;
  - `Row`;
  - `Keep`;
  - `Heading`.
- **`ui/copy.js`** (page-copy).
- **Shared parts**, each built by its phone row:
  - `ui/shelf.js` (page-v2-shelf);
  - `ui/door.js` and `ui/visitors.js` (page-v2-door);
  - `ui/acts.js` and `ui/changes.js` (page-v2-acts);
  - `ui/uses.js` (page-v2-uses).
- The page-only compositions (NameForm, Line, Choose, Group, Links, Leave,
  Wifi, Play, Running, Lending, JoinLink, MakeGroup, OpenLink, Notices,
  Screen) are in each flow's file (components.md §1, the last list).
- An htm trap: whitespace-only text with a newline is dropped, so `el()`'s
  space between a button and its words is written `${' '}`.

## Who owns each file

| File | Owner | Others |
|---|---|---|
| `web/vendor/*`, `web/tests/vendor.test.mjs`, check.sh's `page laws` step | page-vendor | none |
| `ui/machine.js`, `web/tests/machine.test.mjs` | page-machine | none |
| `ui/view.js`, `ui/trace.js`, `web/v2/body.html`, `web/v2/build.sh`, `home/v2/serve.rs`, `home/v2/index.html`, build-web.sh's `--ui`, build-release.sh's marker refusal, home.rs's v2 merge line, cli.rs's `--ui`, `web/tests/{v1,release,policy,specifiers}.test.mjs`, tests/home.rs's v2 cases | page-skeleton | none |
| `lean_check.py` `BUDGET` and the rung's paths | HUMAN-page-rung (the operator's seat) | none |
| `ui/parts.js`, `web/tests/parts.test.mjs` | page-parts, then page-components | a flow that needs a primitive writes it in its own file; page-components folds it |
| `web/v2/app.js` (`Page`, the composition, heading and response order), a stub `web/v2/<flow>.js` for each phone flow, `web/tests/v2/frame.test.mjs` | page-frame | none |
| `harness.rs` (`Ui`, `ui()`, `site_of`, the marker checks, each node start), `scripts/parity.sh`, `crates/svrngs/tests/page_v2/main.rs` with an empty `mod` file for every flow, its `[[test]]` entry, check.sh's `page v2` step | page-switch | each flow row fills only its own `mod` file |
| `ui/copy.js`, `web/tests/copy.test.mjs`, `web/v2/computer.js` | page-copy | `computer.js` passes to page-v2-join |
| `web/tests/home.mjs` | page-home-drive | none after it; every flag a home row needs is added there |
| `web/tests/tabs.mjs` | page-tabs-drive | none |
| `home/v2/home.js`, `home/v2/group.js`, a stub `home/v2/<flow>.js` for each home flow | page-home-root | flows send GROUP `again` and never edit these |
| `web/v2/<flow>.js`, `page_v2/phone_<flow>.rs`, `web/tests/v2/<flow>.mjs` and `.test.mjs` | that phone row | none |
| `home/v2/<flow>.js`, `page_v2/home_<flow>.rs`, `web/tests/v2/home-<flow>.test.mjs` | that home row | none |
| `ui/shelf.js`, `ui/door.js`, `ui/visitors.js`, `ui/acts.js`, `ui/changes.js`, `ui/uses.js` | the phone row named in "The parts" | home rows import them and never edit them |
| check.sh's `parity` step, `demo/runs/page/<date>-parity/` | page-parity | none |
| every v1 file, `docs/DESIGN.md`, `ARCHITECTURE.md` | page-switchover | DESIGN's gate paragraph is page-switch's; PLAN §1's line is page-economize's |

`SHELL` is written by v2's build from what it copies, and `serve.rs` serves
`home/v2/`, `ui/` and `vendor/` by name (`^[a-z0-9-]+\.(js|css|html)$`).
So no row edits a list or adds a route.

## Estimates: lines, by `scripts/lean_check.py`'s count

The count is a line that is not blank and does not start with `//`, `#`,
`/*` or `*`, in a `.rs`, `.js`, `.mjs`, `.ts`, `.py` or `.sh` file outside
the tests (`lean_check.py` `code_lines`). So a `#[cfg]` line, HTML and every
test are free. At `46ba8288`: `.` is 19,480 of 19,481, and svrngs-core is
3,241 of 3,242.

**v1 by that count:**
- `web/app.js` 412;
- `home/home.js` 231;
- `question.js` 87, of which its DOM builders are 59: `visiting` 1, `uses`
  10, `el` 1, `keeping` 1, `people` 7, `door` 11, `table` 12, `changes` 10,
  `acts` 5, `row` 1;
- `web/sw.js` 19, which stays, since it serves v2.

home.md's "−430 home.js, −110 question.js" counted physical lines (`wc -l`:
429 and 263). scaffold.md's 231 is the count that binds.

**The `v2` rung while both live** (it counts `web/v2/`, `home/v2/`,
`home/ui/`, `web/vendor/` and `scripts/parity.sh`, and `.` skips them):

| Rows | Lines |
|---|---|
| page-vendor (three minified modules) | +3 to +6 |
| page-machine | +18 to +25 |
| page-skeleton (view, trace, `build.sh`, `serve.rs`, the two stubs) | +68 to +92 |
| page-parts (seven primitives; the session's estimate, components.md gives none) | +30 to +45 |
| page-frame | +30 to +40 |
| page-switch (`parity.sh`) | +25 to +40 |
| page-copy | +18 to +26 |
| page-v2-enter | +65 to +80 |
| the nine phone flows | +248 to +320 |
| page-home-root | +70 to +90 |
| the fifteen home flows | +235 to +317 |
| page-components | 0 to −20 |
| **The rung** | **+810 to +1,081** |

That is scaffold's +130 to +190 (vendor, machine, skeleton, switch, copy:
+132 to +189), phone's +350 to +440 (frame, enter and flows: +343 to
+440), home's +300 to +410 (root and flows: +305 to +407), and the
primitives, which no plan sized. **Proposed cap: 1,100**, the sum's top
and 19 lines more. Each row reports where it lands against its range. A row
past its range stops.

**`.` while both live: +14 to +17**, lines the switchover deletes:
- `lean_check.py`'s rung, +3;
- build-web.sh's `--ui`, +2 to +3;
- build-release.sh's marker refusal, +1;
- check.sh's `page laws` (+3), `page v2` (+1) and `parity` (+1);
- home.rs's v2 merge, +1;
- cli.rs's `--ui`, +2 to +3.

`.` has one line of room, so page-economize must cut 13 to 16. home.rs's
four served-file consts and their routes (`home.rs:90-111`, `:141-144`, 15
counted lines) as one table give −5 to −8. The survey finds the rest, or
asks a raise of what is left, returned at the switchover.

**The switchover's bar: `.` at or under 19,481 once v1 is deleted.**
- `.` after = 19,480 − E + S + T − D, where:
  - E, economize's cuts, kept: about 15;
  - S, v2's lines that stay: the rung less `parity.sh`, `serve.rs` and the
    part of `build.sh` that does not fold into build-web.sh, about −45 to
    −70, so +740 to +1,035;
  - T, home.rs's table of v2's files once `serve.rs` is gone: about 31
    modules, through one `macro_rules!` over their names, +8 to +30;
  - D, v1 deleted: 412 + 231 + 59 + 1 (sw.js's hand `SHELL`) = 703.
- So `.` lands at **19,510 to 19,827**, which is 29 to 346 over.
- It breaks even only if S + T is at most about 719: v2 at most about 700
  lines, 5% under the low end.
- **Plainly: the bar is not reachable on these estimates.**
  - The switchover runs an economizing pass first.
  - Past passes cut 3 to 93 lines (ralph/next/ios/CHARTER.md, "Estimates").
  - So at the middle of the range, expect a raise of about +100 to +250
    after the pass. That raise is the operator's.
- Where the excess comes from:
  - the fixed cost of the core and the vendored files, about +80 to +130;
  - the home page, +305 to +407 against v1's 231. Every redraw v1 did by
    reloading becomes a region.

  So each home row aims below its range, and every move aims shorter than
  what it replaces (BRIEF).

## Running rows in parallel: a proposal, until the operator's go

**What exists.** commonwealth-ai `scripts/ralph.py` has two loops:
- `run`, one row at a time in the main tree;
- `pool`, rows in lanes (`--lanes N`, default 2; `ralph.py:4251-4258`).

How the pool works:
- Each lane is a worktree beside the tree, `~/dev/svrngs-lanes/<row>`, on
  the branch `ralph/<row>` (`ralph.py:1320-1329`, `:3037-3051`).
- Its `target/` is an APFS `clonefile` copy of the main tree's
  (`cp -a -c`, `:3107-3113`), which shares blocks until rebuilt.
- The base is merged into a lane at each session (`:3056-3067`).
- A finished lane is merged into `main` in the main tree with
  `git merge --no-ff -m "merge <row>"` (`:2447`). Before that, the queue's
  `ralph/merge-check` runs if there is one (`:3180-3214`).
- `REVIEW-` rows run in the main tree, alone (`:2575`). `HUMAN-` rows wait
  for the operator.
- `conflicts.txt` and `heavy.txt` beside `queue.toml` keep named pairs
  apart, and `<row> *` runs a row alone (`:890`, `:2622-2652`).
- No svrngs queue has run `pool` before.

**The proposal: one `pool` with 3 lanes.** `ralph.py pool --queue page
--lanes 3`, in `~/dev/svrngs`, launched from the queue.toml header after
the operator's go.
- **One loop per tree holds.** The main tree runs one loop, and each lane is
  that loop's own worktree.
- **Work lands on `main` in the real checkout**, by the loop's merge. The
  branches are the loop's and go once they are merged.
- **The cost in history:** about 40 merge commits on `main`. svrngs has
  merged before (`efadec28`, `4475a898`). If the operator wants a linear
  `main`, run serially.
- **Disk.** `target/` is 20 GB, and 134 GB is free (`df`, 2026-10-31).
  - A lane shares blocks until it rebuilds. Each gate builds debug, and
    `release` for the ceiling canary (`check.sh:76`), so expect 5 to 10 GB
    a lane, 15 to 30 GB in all.
  - The pool's own floor is 5 GB (`ralph.py:1259`), under the 15/10 GB this
    host agreed, so the watching session holds to 15/10.
  - Each lane's plist needs `CARGO_INCREMENTAL=0`.
- **The gate lock serializes every gate.** Every `check.sh`, and every run
  that starts a browser, takes `lockf -k /tmp/svrngs-gate.lock` (PROMPT
  fence 12).
  - A gate takes 15 to 30 min (`check.sh:29`'s slowest: tests 307 s,
    browser 246, scenes 544, canary 178, before the build), so at most 2 to
    4 gates an hour.
  - The 24 flow rows need at least 8 to 12 hours of the lock alone. That
    is about 12 to 16 hours with 3 lanes, against 30 to 40 serially.
  - A fourth lane only waits at the lock.
  - Lanes compiling beside a gate load the host, and the grapevine scene
    failed at a load of 6.96 (give STATE, give-docs NOTE). Fewer lanes, or
    a lighter cargo share, are the levers.
- **Merges.** The ownership table makes the flow rows disjoint by file, so
  `conflicts.txt` names none at the start.
  - What a file table cannot see is two rows that each pass alone and break
    together.
  - For that, a `ralph/merge-check` (`page laws` and both builds, a few
    minutes) is proposed. It is not written: it lies outside this queue's
    directory.
  - The parity row is the backstop.
- **The tree.** The main tree must be clean at launch and between merges
  (memory: a dirty tree parks a row). Today it holds the operator's
  `README.md`, `demo/*.md` and `docs/guide/`.
- **The alternative, not recommended:** two serial queues, `page-phone`
  and `page-home`, in two clones, as `~/dev/svrngs-folders` ran syp.
  - It gives two at once, not three.
  - It costs a second `target/`.
  - Its main reaches `main` only by hand merges (`05eee01e`, `fb98c92b`).

## Decide

- the region, state and message names, held to `vocab_check.py:62`'s nouns;
- what a row splits into when it cannot execute; the order of ready flow
  rows;
- whether a primitive a flow needs goes page-local now, and is folded by
  page-components;
- the smaller reversible step;
- a v1 test that cannot pass on v2 with its assertions unchanged. The
  director names it and the reason, and it is the operator's
  (NEEDS_HUMAN), never edited quietly.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that. Where its letter and spirit part, rewrite the letter, say both
in the commit, and keep going.

## Leave for the operator

For each of these, write `NEEDS_HUMAN.md` and stop:
- **the budget:** the rung and its cap, any `.` raise, any svrngs-core
  line, a row past its range, and the switchover's raise after its pass;
- **the trust floor:** any change to `scripts/trust_floor.txt` or a page's
  policy. None is planned;
- **v1:** any edit to a v1 file before the switchover, beyond a gate fix
  whose pin updates in the same commit (PROMPT fence 2);
- **a v1 test changed** beyond the two driver rows;
- **a dependency** beyond Preact 11.1.1 and htm 3.1.1, vendored;
- **the people runs:** the iPhone run in page-parity, and the switchover's
  go;
- **outward-facing steps:** deploying the page, a release, pushing,
  history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
