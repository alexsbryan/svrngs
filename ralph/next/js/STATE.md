# svrngs js — the pages' JavaScript formatted, typed in JSDoc, linted for structure, and read (operator, 2026-11-02)

`CHARTER.md` holds why, what was measured, what the operator decided, the
shape the code takes, and the open choices with their defaults. Read it
whole before a row.

**Launched 2026-11-02**, a pool of one lane, once the security work was on
main (9338948f) and the `js` rung landed.

**The demo.** A reader who knows JS/TS opens `web/page/shelf.js`,
`ui/machine.js` and the home page's flows:
- each machine names its states and messages as types;
- `step` is a `switch` the compiler checks is exhaustive;
- each view is a named component with typed props;
- no line is packed.

`scripts/check.sh` holds all of it: `format`, `types` and the lint's
structural rules each fail on a planted file. Every browser test passes
unchanged, so no person sees a change.

The rows run in order, since all of them touch the same files.

- [x] HUMAN-js-agreed — depends [] — DONE 2026-11-02: the security work is on main, its seventh part at 9338948f (the operator: "as recommended"); the `js` rung landed from the operator seat, `js` 2,233 uncapped and `.` lowered to 18,506; plant: a planted `web/page/x.js` line moved `js` 2,233 → 2,234 and left `.` at 18,505. — THE OPERATOR (CHARTER's choices answered, 2026-11-02: oxfmt; a `js` rung with no cap until a jubilee at the queue's end; the rest as proposed):
  - confirms the security work is on main.

  The watching session then:
  - lands the `js` rung from the operator seat (CHARTER, "Answered" 2), with its plant: one counted line in a planted `web/page/x.js` moves `js` and leaves `.` unchanged;
  - ticks this row and starts the loop.
- [x] js-format 46ecdadd — depends [HUMAN-js-agreed] — ONE ROW, each part its own commit, and `scripts/check.sh` once, at the close — oxfmt in the gate, then one commit that only formats:
  - **(a) The tool.**
    - oxfmt is pinned exactly in `web/package.json`, and `web/package-lock.json` is updated by `npm install --prefix web`.
    - Its config sits at the root, at width 100 with single quotes, and its ignores match `.oxlintrc.json`'s.
    - Shown on a scratch copy: the htm templates' text comes out byte for byte (CHARTER, "Answered" 1). Otherwise, stop.
    - No step is wired yet, so every commit stays green.
    - oxfmt's check on today's tree is the plant run: it fails, and the commit cites its count of unformatted files.
  - **(b) One commit that only formats, and the step.**
    - oxfmt over every file (a) covers.
    - `check.sh` gains a `format` step, right after `lint`, checking every file it covers and naming each unformatted one.
    - Nothing else goes in the commit.
    - **Its proof of no behavior change:** the pages' laws (`node --test web/tests/*.test.mjs`), `parts.test.mjs`, and the browser tests run under the gate lock. Every one passes unchanged.
    - **Lines:** `js` and `.` from `lean_check` before and after, by directory, as in CHARTER's table. No cap stops it (CHARTER, "Answered" 2).
  - **(c) `.git-blame-ignore-revs`** names (b)'s commit, so blame skips it, and CONTRIBUTING or the README says how to use it.
  - **Plant:** a planted unformatted line in `web/page/shelf.js` fails `format`, naming the file.
  - **Moves:** nothing a person sees. It moves the line count to the code's real size.
- [x] js-types 2b9660ff — depends [js-format] — DONE 2026-11-03 by its director, on an A/B and not a green gate: every `check.sh` step passes in js-types-9 but `browser`, whose four failing WebKit tests fail the same with main's page code. At load 5.4-6.4 (12 cores) on 2026-11-03, `lockf -k /tmp/svrngs-gate.lock cargo test --quiet --workspace --test browser` at 2b9660ff gave 90 passed, 4 failed in 231.31 s. The same command after `git checkout main -- web/ crates/svrngs-node/src/home/` (restored afterwards) gave 90 passed, 4 failed in 231.64 s. Both failed the same four: `tabs::the_group_s_page_in_two_tabs_reaches_the_group_from_one`, `the_page_offers_a_computer_one_line_and_sends_the_token_nowhere`, `the_phone_goes_through_the_link_s_computer_while_it_answers` and `the_phone_reaches_whoever_s_awake`, each `::webkit` and each `page.goto: Timeout 30000ms` (drive.rs:114) apart from tabs (tabs.rs:117). So the row changes no browser test's outcome (fence 3's guard), and main's own gate is red on this Mac now. js-format-3 passed `browser` in 161 s at 8e085041. The fix is js-webkit's, below, before js-lint closes. The row's own checks: `node scripts/types.mjs` 0 errors in 55 files, no `@ts-nocheck` (a73f9b65 fails `types` on one). V1.1 set the precedent: published with a load-timed WebKit tab test red (canon STATE.md:59) — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — the pages' code typed in JSDoc, checked by `tsc --checkJs --strict`:
  - **(a) The checker.**
    - `typescript` is pinned exactly in `web/package.json`.
    - `jsconfig.json` sits at the root and holds:
      - `checkJs`, `strict` and `noEmit`;
      - `noImplicitReturns`, `noFallthroughCasesInSwitch` and `noUncheckedIndexedAccess`;
      - `include` set to the pages' code (CHARTER, "Scope").
    - `check.sh` gains a `types` step.
    - Preact 11.1.1's and htm 3.1.1's `.d.ts` are vendored from the same tarballs (`web/vendor/README.md` gives the commands), with `lean_check`'s skip, per CHARTER "Answered" 3.
    - Red first: the step's first run lists the errors at HEAD, and the commit cites the count.
    - Files that aren't typed yet carry `// @ts-nocheck`, which (b) to (d) remove. A file still carrying one when the row closes fails the row.
  - **(b) The core.** `ui/machine.js` takes a generic `Machine<State, Message>`:
    - `go`, `no`, `ask` and `late` are typed;
    - `assertNever` takes the `never` an exhaustive `switch` leaves;
    - `ui/view.js`'s `useStore` and `useMachine` are typed;
    - `ui/trace.js` is typed.
  - **Corrected 2026-11-02, (b)'s session: no step at HEAD is a `switch`** (`git grep "switch ("` over both pages finds none). Each takes a message by a table, two ways:
    - **The phone's flows** each export `takes` (message type → step), which `compose` reads at run time to refuse a type two flows claim (`web/page/page.js:61-76`, `:94`). A `switch` would hide the types from it. So each flow's `takes` is typed `Takes<State, Message>` (`ui/machine.js`): a mapped type over the flow's Message union, which fails `types` on a type with no step or a step for no type, and gives each step its own message's type. That is the same check as an exhaustive `switch`, with no runtime change.
    - **The home page's steps** are object-literal tables of thunks, `({ ... })[m.type]?.() ?? no(s, '... takes no ${m.type}')` (21 of them, such as `crates/svrngs-node/src/home/page/ask.js:70`), whose thunks read `m` unnarrowed. Each becomes a `switch (m.type)` ending in `assertNever`, which refuses in the same words. The `switch` is the guard narrowing needs, so "Kept" below allows it.
  - **(c) The phone page,** one commit per machine:
    - its states and messages as discriminated-union `@typedef`s;
    - its `takes` a `Takes<State, Message>`, exhaustive;
    - `io` typed;
    - each component's props a `@typedef`.
  - **(d) The home page,** the same way, one commit per machine, its `step` a `switch` exhaustive at `assertNever`.
  - **Plant:** a planted message `{ type: 'nope' }` that no case takes fails `types`: at the `assertNever` in a home step, and at the `takes` in a phone flow. A planted wrong prop type fails it too.
  - **Kept:** every runtime byte but `assertNever` and the guards narrowing needs. `git diff` on a typed file shows comments only, apart from those.
- [x] js-webkit 9c9baa22 — depends [js-types] — ONE ROW, `scripts/check.sh` once, at the close — the gate's `browser` step green again on this Mac, at main's page code:
  - **The failure** (js-types' DONE note, 2026-11-03). Four WebKit tests time out in the parallel `browser` step at 2b9660ff, and at main's `web/` and home page too. They passed with `--test-threads=1` (211.76 s) and alone in js-types-10's runs. `browser` took 161 s in js-format-3, and 231-264 s since.
  - **(a) Instrument and reproduce first** (AGENTS.md 2). Find what the four wait on when they time out, and why the step got about 40% slower since js-format-3: the harness's shared browsers and contexts, the relay, name resolution for `*.svrnme.test`, or the Mac's own load (System Settings' StorageManagement processes ran at about 160% CPU through 2026-11-02 and 03). Record the run under `demo/runs/`.
  - **(b) Fix the cause** in the harness or the code it names, as a general fix, never tuning one scene (no raised timeout, no test marked serial, unless (a) shows the wait is that test's real cost). Every test still pins what it pins.
  - **Plant:** the condition (a) finds, restored, turns the four red again.
  - **(c) The `carried` fixture — decided 2026-11-03 by its director, C** (js-webkit-2's question; `demo/runs/webkit-wall/2026-11-03/README.md`, "The `carried` failure, read again"). `carried::a_new_version_runs_as_each_person_only_after_their_yes::webkit` failed 2 of the 4 `browser` steps since (b). Its doc comment says "bo's phone", but `node(…, &bo)` admits bo through `computer_on`'s bare `Admit` (`tests/browser/main.rs:157`), which is `Device::Computer`, so every round waits out bo's 4 s dial (`node.rs` `dialled`; `svrngs-core/src/lib.rs` `Device::dialled`; `carried-trace.txt`, rounds 4003-4054 ms) and bo's yes leaves up to 4 s late.
    - **The fixture matches the scenario:** apply `option-a.diff` (`seeded` plus `from_a_phone(&ada, "bo", 7)`), in its own commit. Fence 3 guards behavior kept while the pages' JavaScript changes. This changes no assertion and no page byte. It makes the fixture what the doc comment already says, so it is in the fence's spirit, and the fence's letter yields here only, for this one fixture.
    - **Plant:** the diff reverted, a solo traced run (`rails`, `folder`, `svrngs_node::ring_sync` in `harness.rs` `traced`) shows rounds of 4 s each waiting on bo's key. Applied, the rounds are under 100 ms. The commit cites both. Red under load alone is 2 in 4, so the round's length is the plant, not the test's outcome.
    - **The node's part (B) is not this queue's.** A write nudged mid-round leaves only after that round's slowest dial (`ring_sync.rs` `spawn_ring_sync_loop`, the comment above its `tokio::select!`). It is in `NEEDS_HUMAN.md` for the operator to place in a node queue. This row does not wait on it.
  - **(d) The home harness reads at rest — decided 2026-11-03 by its director, the harness fix, in this row** (js-webkit-6's question; `NEEDS_HUMAN.md`, "The home page is read before a joined group is"). Both gates since (c) failed only on this: a `home_group` WebKit test read after a press before the new group was read (`home-group-gate.txt`, `home-group-gate-2.txt`).
    - **Why here.** (b) already makes the gate's WebKit failures this row's, fixed "in the harness or the code it names". The race predates this queue (page-parity-4 and -5's gates), so it is no behavior this queue changed. Showing a group as being read until it is would change what a person sees, which fence 3 leaves to the operator. Closing the row with `check.sh` red would weaken its pass bar.
    - **What is wrong.** `web/tests/home.mjs` `settled` resolves at the first frame whose words changed and hold no "…", and `after` is read later, at another frame. In `home-group-gate-2.txt`, the read had "Reading your groups…" in it, so `settled`'s frame and the read's frame differ. The page's own trace, `window.__lines` (fence 5: every step is traced), shows when the page is still working: `HOME: again … rereading`, `GROUP: … reading/reading, running get`, `ACTS: … running read`.
    - **First, reproduce** (AGENTS.md 2). Hold the groups' read in a scratch copy (a delay in the node's groups answer or the page's `get`, never committed) so the race fails on every run, and record the frame `settled` took and the frame read, with `lines`, under `demo/runs/webkit-wall/2026-11-03/`.
    - **The fix, in `home.mjs` only.** A press is read once the page is at rest, judged in the same frame as the read: no "…", and no machine whose last traced line is still reading or running. That rule is general to every `do` step, not tuned to these two tests. No timeout raised, no test marked serial, no assertion or page byte changed. Every test still pins what it pins. Fence 3's letter yields here, as it did for (c): no assertion moves, and the read waits for what the page already says.
    - **Plant:** with the held read, the old `settled` fails `home_group`'s make and join tests, and the fix passes them. The commit cites both runs.
    - **Then** the close's gate, as Pass below.
  - **Pass:** `scripts/check.sh` exits 0. `browser` passes in three gate runs, under the gate lock, at the load these lanes normally run at (AGENTS.md 7: one run is not a rate).
- [x] js-lint da5192c1 — depends [js-webkit] — ONE ROW, each part its own commit, `scripts/check.sh` once, at the close — the structural rules, each on with its hits fixed in the same commit:
  - **The scope.** `.oxlintrc.json` gains an `overrides` entry for the pages' code (CHARTER, "Scope"). The rest keeps today's rules.
  - **(a) Names:**
    - `id-length` (min 2; `i` and `_` excepted);
    - `no-shadow`.

    The 813 one-letter names become names a reader follows. A rename only: the words a person reads are untouched.
  - **(b) Control flow:**
    - `no-sequences`;
    - `no-nested-ternary`;
    - `no-unused-expressions` (no `a && b()` as a statement);
    - `one-var`;
    - `complexity` at 10;
    - `max-params` at 4;
    - `max-depth` at 3.
  - **(c) Modules:**
    - `import/no-cycle`;
    - `oxc/no-barrel-file`: `ui/kit.js` goes, and each flow imports what it uses, from its module;
    - `no-restricted-imports`, a `regex` pattern per override, for CHARTER's layers (corrected: `import/no-restricted-paths` is not in oxlint 1.89.0, `grep -c no-restricted-paths web/node_modules/oxlint/configuration_schema.json` is 0; `no-restricted-imports` is, at :5227);
    - `web/tests/specifiers.test.mjs` still passes.
  - **(d) Type-aware,** per CHARTER "Answered" 4, or each rule named never-ran with why.
  - **Plant:** each rule watched failing on a planted file before its fixes. The commit names the rule and the plant.
- [ ] js-read — depends [js-lint] — ONE ROW, each part its own commit, `scripts/check.sh` once, at the close — the pages read as code meant for people, behavior kept:
  - **(a) Survey.** `demo/runs/economize/<date>-js-read/SURVEY.md` lists:
    - each view packed into an expression;
    - each machine not yet in its own module;
    - each comment with no why;
    - each duplicate across the two pages (`sovereign-cli code dry-report`).

    With the cut or move each one takes.
  - **(b) The phone page, then (c) the home page,** one commit per flow:
    - views become named components, one per file;
    - templates are broken up by hand where `embeddedLanguageFormatting: "off"` left them long;
    - the machine goes to its own module;
    - `io` sits at the edge;
    - comments are rewritten to say why.
  - **(d) The rules written down** (AGENTS.md 3): CHARTER's "The shape the code takes" goes into svrn-docs `rings/current/PAGE_STATE.md` §3 and DESIGN §5, rewritten in place.
  - **Kept:** every browser test, the pages' laws and `parts.test.mjs`, unchanged and passing. A change a person would see is NEEDS_HUMAN, not this queue's.
  - **Lines:** `js` and `.` before and after each part, with no cap to stop it.
  - **(e) The jubilee** (CHARTER, "Answered" 2). After the gate passes at the close:
    - a ready diff in `NEEDS_HUMAN.md` setting `js`'s cap to one past where it lands, and `lean_check`'s "no cap" mode removed;
    - the queue's total named: `js` at HUMAN-js-agreed against `js` now, by row;
    - then stop. The watching session lands it.
  - **Moves:** the operator's reading of the pages, the demo above.
