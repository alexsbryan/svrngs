# svrngs page — v2 of both pages, built beside v1 with Preact, htm and one machine each, then one switchover (operator, 2026-10-31)

svrn-docs `rings/current/PAGE_STATE.md` is this queue's exploration. Its §3
holds the rules, and its §5 the spike that ran under the live page's policy.
The decisions, the parts, who owns each file, the estimates and the parallel
proposal are in `CHARTER.md`.

**Why.** On the operator's iPhone, "Bring the ticked ones" brought Events,
and the page showed nothing:
- the answer replaced the paragraph above the boxes (`question.js:203-208`);
- the button never disabled, so a second tap posted twice;
- the page reads the group's list once (`web/app.js` `enter`, `:433-446`).

That is how both pages are built. Each section is drawn once, the last
writer to `#status` wins, and the home page redraws by reloading
(`home.js:163`, `:221`, `:283`, `:384`).

**The shape** (the operator, 2026-10-31: "Do the work in parallel, then we
do an atomic switchover when this v2 UI is done"):
- v2 is built in its own files beside v1, and v1 stays live and
  byte-frozen;
- one switch, `SVRNGS_UI=v2`, runs every browser test, scene and canary
  against v2;
- when the whole gate passes on v2, one commit makes v2 the pages and
  deletes v1.

**The demo.**
- On an iPhone, the bring's answer is beside the button, a second tap is
  refused, and Events shows in the group's list with no reload.
- On the computer, the same bring shows in the index with no reload.
- Then the whole gate passes on v2 before anything switches.

**Still the operator's**, asked at HUMAN-page-rung. Each has the session's
default, which holds until answered:
1. **The rung**, its paths and its cap of 1,100, and a temporary `.` raise
   for whatever page-economize cannot find (CHARTER, "Estimates").
2. **The switchover's bar** (`.` at or under 19,481 once v1 is deleted)
   is not reachable on these estimates (19,510 to 19,827). Default: the
   switchover runs its pass and then asks the raise for what is left.
3. **Pool or serial.** The proposal is `pool` with 3 lanes, which puts
   merge commits on `main` (CHARTER, "Running rows in parallel"). Default:
   serial `run` until the operator says pool.
4. **The home page moves whole**, every flow, not only its shelf
   (PAGE_STATE "Not decided"). Default: whole, since Phase 2b's "this v2 UI"
   reads as all of it.
5. **The two driver rows count as "tests unchanged".** page-home-drive and
   page-tabs-drive change `web/tests/home.mjs` and `tabs.mjs`, each green
   on v1 first, with every Rust assertion untouched. Default: yes.
6. **v1 is frozen, except a fix the gate needs.** Such a fix updates its
   pin in the same commit and is mirrored in v2 in that row. Default: yes.

The settled list in CHARTER (paths, `.js`, the derived `index.html`, one
root, `test-support`, the switch, `page_v2`, flash notices, the shared parts)
is the session's, and the operator can overrule any of it at the same row.

**The rows.**
- **The serial base:**
  - page-economize, then HUMAN-page-rung;
  - page-vendor beside page-machine;
  - page-skeleton;
  - page-parts beside page-switch, then page-frame;
  - page-copy beside page-home-drive and page-tabs-drive;
  - page-v2-enter beside page-home-root.
- **In parallel,** each depending only on that base and on the row that
  builds a shared part it names:
  - the phone's nine flows: shelf first, then index, door, join, visit,
    among, acts, uses, ask;
  - the computer's fifteen: shelf first, then door, links, leave, visits,
    wifi, play, acts, running, lending, join, make, open, ask, screen.
- **The close:**
  - page-components, then page-parity, with the operator's iPhone run;
  - HUMAN-page-switchover;
  - page-switchover, one atomic commit;
  - HUMAN-page-deploy.

That is 42 rows, and the loop adds one closing audit.

**Each flow row's header means:**
- ONE ROW: its move first, with v1's behavior kept, then each fix in its
  own commit;
- red first: the v1 tests it names fail under `SVRNGS_UI=v2` against the
  stub, and its v2-only tests in `crates/svrngs/tests/page_v2/` fail with
  `UI` flipped to `Ui::V1`;
- `scripts/check.sh` once, at the close, and those v1 tests run under
  `SVRNGS_UI=v2`, all under the gate lock, cited in the commit.

`a:` is `web/app.js`, `q:` is `crates/svrngs-node/src/home/question.js`,
`h:` is `home/home.js`, `i:` is `web/index.html`, and `b:` is
`crates/svrngs-node/tests/browser/`. Line ranges are the rung's, by
`lean_check.py`'s count, with v1's lines the switchover deletes in
brackets.

**Waits, each with its trigger:**
- **One `Index` for both pages, and one `Ask`.** After the switchover, when
  the two pages' wordings are agreed (`a:572` against `h:189`).
- **`Act`'s answer as `<output role="status">`** (components.md §1.1). After
  the switchover, since it needs a rule in look.css, which is frozen until
  then.
- **`lend.js`, `open.js`, `store.js` and `shim.js` drawn by Preact.** Never,
  unless one of them grows a view: they draw little or nothing
  (components.md §6).
- **A newer Preact.** A fix v2 needs, and the vendor row's commands run
  again.

- [ ] page-economize — depends [] — ONE ROW, each cut its own commit with its behavior kept, and `scripts/check.sh` once, at the close — room in `.` for the hooks the build adds, and the rung asked for (AGENTS.md "Done"):
  - **(a) A survey, nothing to product.**
    - Run the instruments of `demo/runs/economize/2026-10-25-give/scripts/` again, into `demo/runs/economize/2026-10-31-page/SURVEY.md`, each candidate with its lines.
    - It is scoped to what `.` gains during the build, +14 to +17 (CHARTER, "Estimates").
    - Look first where those hooks land: `crates/svrngs-node/src/home.rs`, `scripts/build-web.sh`, `scripts/check.sh`, `scripts/build-release.sh`, `scripts/lean_check.py` and `crates/svrngs/src/cli.rs`.
    - v1's page files are frozen and deleted whole later, so none is cut here. The full pass is page-switchover's.
  - **(b) The cuts, behavior kept.**
    - First, `home.rs:90-111` and `:141-144` become one `(path, type, body)` table, for −5 to −8. That is the `JS` header, `MODULE`, `QUESTION`, `LOOK` and `ICON`, and their four routes: 15 counted lines.
    - Then the survey's next, largest first, to −13 to −16 in all.
    - Each cut is its own commit, and the cap is lowered to where they land.
  - **(c) PLAN §1 names this queue,** in its own commit (AGENTS.md 3).
  - **(d) NEEDS_HUMAN.md, then stop.** It asks for:
    1. **the rung:** a `"v2"` key in `BUDGET` counting `web/v2/`, `crates/svrngs-node/src/home/v2/`, `crates/svrngs-node/src/home/ui/`, `web/vendor/` and `scripts/parity.sh`, with `.` skipping them. That is three lines in `code_lines`, as 2b1e6339 folded Swift's;
    2. its cap, 1,100;
    3. a temporary `.` raise of whatever (b) could not find, returned at the switchover;
    4. the switchover's bar, with the arithmetic that says it is not reached without a raise;
    5. STATE's six open choices.
  - **Plant:** the ratchet's own, a cap set one under the count, fails lean_check.
  - **Byte for byte:** v1's served bytes, with `tests/home.rs` unchanged by name, and every test name.
  - **Moves:** no demo. `.` from 19,480 to about 19,465.
- [ ] HUMAN-page-rung — depends [page-economize] — THE OPERATOR:
  - sets the rung, its paths and its cap;
  - answers STATE's six open choices, or lets the defaults stand;
  - agrees CHARTER's settled list, or overrules any of it.

  No builder row runs before it (design before code, 2026-10-25). The watching session lands `BUDGET` from the operator seat (cap raises land from the operator seat). Its plant: one counted line in a planted `web/v2/x.js` past the cap makes lean_check exit 1 naming `v2`, with `.` unchanged.
- [ ] page-vendor — depends [HUMAN-page-rung] — ONE ROW, red first, and `scripts/check.sh` once, at the close — Preact 11.1.1 and htm 3.1.1, with nothing loading them yet:
  - **Owns:** `web/vendor/`, `web/tests/vendor.test.mjs`, and check.sh's `page laws` step.
  - **(a) The bytes.**
    - `web/vendor/preact.js` is preact@11.1.1's `dist/preact.module.js`, `hooks.js` is its `hooks/dist/hooks.module.js`, and `htm.js` is htm@3.1.1's `dist/htm.module.js`.
    - `LICENSE.preact` (MIT) comes from the tarball. `LICENSE.htm` (Apache-2.0) comes from github.com/developit/htm at `3.1.1`, with its NOTICE if that tree has one, since the tarball carries none.
    - `README.md` follows `apps/grapevine/vendor/README.md`. It gives:
      - each tarball's `dist.integrity` and its sha256;
      - each file's upstream path and its sha256;
      - **the two edits:** `hooks.js`'s `from"preact"` becomes `from"./preact.js?v=__VERSION__"`, because the policy allows no importmap (`trust_floor.txt:60-67`), and every `//# sourceMappingURL=` trailer is stripped;
      - the commands that make these bytes again;
      - the `| \`file\` | \`sha256\` |` table `apps/pinned.mjs` reads.
  - **(b) `web/tests/vendor.test.mjs`** holds the pins through `apps/pinned.mjs` `unpinned`. It also checks that no file has a map trailer, and that `hooks.js` imports exactly `./preact.js?v=__VERSION__`.
  - **(c) The gate step `page laws`:** `node --test` over `find web/tests -name '*.test.mjs'`, never-ran when none is found, as `app laws` does (`check.sh:134-136`).
    - It is its own step, because those are apps' laws (AGENTS.md 12).
    - AGENTS.md's "Done" names it, in the same commit.
  - **Plant:** one byte changed in `hooks.js` turns it red, naming the file.
  - **Byte for byte:** the built v1 site. `find target/web/site -type f | sort | xargs shasum -a 256` is equal before and after.
  - **Moves:** `page laws`. +3 to +6 (rung), +3 (`.`, check.sh).
- [ ] page-machine — depends [HUMAN-page-rung] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the pure core, tested with no DOM, beside page-vendor:
  - **Owns:** `crates/svrngs-node/src/home/ui/machine.js` and `web/tests/machine.test.mjs`.
  - **(a) `ui/machine.js`.** No DOM, no Preact, no clock, no storage, and no module state.
    - `machine(name, { states, step, io }, state, first = [])` answers `{ get, dispatch, subscribe }`.
    - `state.at` holds one named state per region. `states` is each region's closed set, and a step that answers outside it is refused (AGENTS.md 9).
    - `step(s, m)` answers `{ state, effects, refused? }`, written with `go(s, at, patch, ...effects)` and `no(s, why)`.
    - An effect `{ run, failed, …args }` runs as `Promise.resolve().then(() => io[f.run](f))` and answers its done message. A rejection, a throw or a missing io answers `{ type: f.failed, n, why }`, so nothing idles quietly.
    - `ask(s, kind)` answers `[asked, n]`. `late(s, kind, n)` answers `''`, or "`<kind> <n> answered after <kind> <asked> was asked`".
    - Subscribers are told once per change, and never on a refusal.
  - **(b) The trace, this file its one home,** through `globalThis.__trace?.(line)` (as `web/store.js:88`). The lines are `<NAME>: <type>[ (<why>)]: <from> -> <to>[, running <run>, …]` and `<NAME>: <type> refused in <from>: <why>`. `from` and `to` are the regions' states joined by `/`.
  - **(c) `web/tests/machine.test.mjs`,** on a toy machine of two regions. It covers:
    - a step, traced;
    - a refusal: the same reference, nobody told, nothing run;
    - an effect rejected, thrown, or with no io;
    - a late answer;
    - a state outside its set;
    - the effects' order.
  - **Plant:** with `late`'s comparison removed, the late case turns red. With the closed-set check removed, the set case turns red.
  - **Byte for byte:** all of v1.
  - **Moves:** `page laws`. +18 to +25 (rung).
- [ ] page-skeleton — depends [page-vendor, page-machine] — ONE ROW, red first, and `scripts/check.sh` once, at the close — v2's empty pages, built and served beside v1, with v1 frozen:
  - **Owns:** CHARTER's page-skeleton line.
  - **(a) The pages.**
    - `web/v2/body.html` is `<!-- svrngs-ui-v2 --><div id="page"><h1 id="status">Opening…</h1></div>`. It never changes after this row: the frame draws into `#page`.
    - `web/v2/app.js` is a stub. It imports `ui/trace.js`, sets `window.__ui = 'v2'`, and draws nothing.
    - `home/v2/index.html` is v1's `home/index.html` byte for byte, except two things: `<script type="module" src="/v2/home.js?v=__VERSION__">` and the marker comment. `home/v2/home.js` is one empty `Home` root.
    - `ui/view.js` holds `html`, `render`, an 8-line `useStore` and `useMachine`. `useStore` is `useState` with `useLayoutEffect`, because `useSyncExternalStore` is `preact/compat`'s alone.
    - `ui/trace.js` is v2's sink, a copy of `a:55-64`. There are two sinks until the switchover deletes v1's, and this one is listed as a v2-only marker.
  - **(b) The v2 build beside v1.** `scripts/build-web.sh` takes `--ui v1|v2`, `v1` by default, a closed set. `v2` sources `web/v2/build.sh`, which:
    - derives `index.html` from `web/index.html`, replacing what lies between `<body>` and its `<script` line with `body.html`, and fails unless `<head>` is byte-equal (`cmp`);
    - copies `web/v2/app.js` to `$OUT/app.js` and `web/v2/*.js` to `$OUT/v2/`;
    - copies `vendor/` and `ui/` with their paths kept, and question.js and look.css as `SHARED`;
    - puts every one of them into the `VERSION` hash (`:123`) and the `__VERSION__` stamping, the shared `.js` included;
    - writes `$OUT/sw.js`'s `SHELL` from the files it copied, so `web/sw.js` is unchanged and no flow edits a list;
    - fails in words if any `__VERSION__` is left;
    - writes `target/web/site-v2` by default.

    The 404.html derivation runs on the derived page unchanged.
  - **(c) v1 frozen.**
    - `web/tests/v1.test.mjs` pins the sha256 of `web/index.html`, `app.js`, `sw.js`, `store.js`, `open.html`, `open.js`, `lend.js` and `shim.js`, and of `home/index.html`, `home.js`, `question.js`, `look.css` and `away.html`.
    - In the row, `build-web.sh` is run at the base and at the row's commit, and the two `shasum` lists of `target/web/site` are equal, `VERSION` included. The diff goes in the commit body.
  - **(d) The v2 home page, in a test build only.**
    - `home/v2/serve.rs` is under `#[cfg(feature = "test-support")]`. It serves `/`, `/ask`, `/v2/<name>`, `/ui/<name>`, `/vendor/<name>` and `/question.js` from the sources (`env!("CARGO_MANIFEST_DIR")`).
    - Names must match `^[a-z0-9-]+\.(js|css|html)$`, so no row adds a route. It sits behind `only_itself`, under `POLICY` (`home.rs:81`), and holds the string `svrngs-ui-v2`.
    - It is chosen per node by a builder method defined in `serve.rs`, and in the binary by `--ui v2` (`cli.rs`), both under `test-support`. home.rs merges its routes in one line.
    - v1's `/home.js` and `/question.js` stay served (`tests/home.rs:185-189`).
    - `web/tests/release.test.mjs` fails if:
      - any `[dependencies]` (not dev) or `default` names `test-support`;
      - `build-release.sh` (`:46`) or `deploy-page.sh` names `--features`, `--all-features` or `--ui`.

      `build-release.sh` refuses a built binary in which `grep -c svrngs-ui-v2` is not 0.
  - **(e) The policy, with no floor change.**
    - `web/tests/policy.test.mjs` asserts that the derived page's `<head>`, and so its policy line, equals `web/index.html:51` byte for byte.
    - `trust_check.py` exits 0 before and after, and `git diff <base> -- scripts/trust_floor.txt` is empty.
  - **(f) `web/tests/specifiers.test.mjs`:** every import into `vendor/`, `ui/`, `v2/` or `question.js` from v2 code carries `?v=__VERSION__`, and only `ui/view.js` imports `vendor/`.
  - **Plants:**
    - a byte in `web/app.js` turns `v1.test.mjs` red;
    - `'unsafe-inline'` in the policy's head line turns `policy` red, and the build's `cmp` fails;
    - `ui/` left out of the stamping fails the v2 build in words;
    - a planted `../x.js` name gets 404 from `serve.rs`, named in `tests/home.rs`;
    - `features = ["test-support"]` under svrngs's `[dependencies]` turns `release` red;
    - a bare `../vendor/preact.js` turns `specifiers` red.
  - **Moves:** `page laws`, and `tests/home.rs` gains v2's routes under `test-support`. +68 to +92 (rung), +6 to +8 (`.`).
- [ ] page-parts — depends [page-skeleton] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the seven primitives, beside page-switch:
  - **Owns:** `ui/parts.js` and `web/tests/parts.test.mjs`.
  - **(a) `ui/parts.js`** (CHARTER, "The parts"; components.md §1, items 1 to 7). Each is a pure view.
    - **`Act`:** `words`, `doing` (`{state: idle|busy|done|refused, said}`), `on`, `type`, `answer` (`beside` or `instead`) and `send`. It is disabled while busy, and never sends while busy. Its answer is a text node beside the button, as v1's is.
    - **`Said`:** `tag`, `id`, `class` and `words`. It draws nothing when the words are empty.
    - **`Fold`:** its `open` comes from props, and `onToggle` sends `{type: 'fold', fold, open}`. So a driver's `open =` (`web/tests/home.mjs:66`, `:134`) is taken.
    - **`Field`:** the `for` shape focuses on mount (`a:136`), the one hook below the roots. There is also the `wrap` shape, and `select`.
    - **`Row`:** `q:263`.
    - **`Keep`:** `q:140`, byte for byte.
    - **`Heading`:** `h1#status`, with `good`, `bad` or nothing.
  - **(b) `web/tests/parts.test.mjs`.** Each part is called as a function, and its vnodes (type, props, children, words) are checked against what the v1 builder it replaces draws. No DOM is needed, since Preact's `h` makes none.
  - **Plants:**
    - an `Act` that sends while busy turns it red;
    - `Keep` with its space before " Bookmark this." moved turns it red;
    - a `Fold` that does not send on toggle turns it red.
  - **Byte for byte:** v1; the vnodes match `keeping` and `row`.
  - **Moves:** `page laws`. +30 to +45 (rung).
- [ ] page-switch — depends [page-skeleton] — ONE ROW, its plants red first, and `scripts/check.sh` once, at the close — one switch for what the v1 tests drive, which fails loudly while v2 is short (beside page-parts and page-frame):
  - **Owns:** CHARTER's page-switch line.
  - **(a) The switch.**
    - `harness.rs` gains `enum Ui { V1, V2 }` and `ui()`, which reads `SVRNGS_UI` once (`OnceLock`): unset means V1, and any other value panics naming `v1` and `v2`.
    - `site()` becomes `site_of(ui())`. `site_of(Ui::V2)` builds with `--ui v2` into `target/web/site-v2`.
    - Every node start in a target that includes harness.rs takes `ui()`: in process through `serve.rs`'s builder method, as the binary with `--ui v2`. Those targets are browser, conformance, join_page, scenes, home_group, leave_page, install, relay, phone_dry and xenia (`git grep -l browser/harness.rs crates`). Each start is found by `git grep` and listed in the commit.
  - **(b) It never passes vacuously.**
    - After the build, the harness asserts that `index.html` carries `svrngs-ui-v2` if and only if the switch is V2.
    - Where the harness first hands a test a home page's address, it asserts the same of `/`.
    - A mismatch panics with its cause in words, such as a binary built without `test-support`. Every test then fails loudly, never quietly on v1.
    - **The baseline:** the first parity run, on the skeleton, is recorded in `demo/runs/page/2026-10-31-baseline/` (`verdicts.tsv`, the commands and the commit). A test that passes there is listed apart, as "passes with no v2". Parity counts only what v2 earned.
    - Its await budget is 3 h, since each test missing an element waits out its timeout.
  - **(c) `scripts/parity.sh`** (rung).
    - It `exec`s under `lockf -k /tmp/svrngs-gate.lock`, and runs each harness target as check.sh runs it, with `SVRNGS_UI=v2`.
    - It writes `target/parity/verdicts.tsv`: one row per test, `passed`, `failed`, `could-not-judge` or `never-ran`. could-not-judge is read from the files check.sh reads: `target/could-not-judge`, `target/conformance`, and `target/scenes/*/could-not-judge.txt`.
    - `never-ran` is a test `cargo test -- --list` names on v1 that has no verdict on v2, or a target that did not build.
    - It prints the counts and each not-passed test by name, so parallel rows see what is left, and exits 1 on any not-passed (AGENTS.md 6).
  - **(d) check.sh does not run parity yet.** It would double the slowest steps (browser 246 s, scenes 544 s, `check.sh:29`) to report failures that are expected. It joins at page-parity.
  - **(e) The v2-only target, `crates/svrngs/tests/page_v2/`.**
    - `main.rs` takes harness.rs by `#[path]` and holds `const UI: Ui = Ui::V2;`.
    - It has one empty `mod` file per flow:
      - `phone_{computer,enter,shelf,index,door,join,visit,among,acts,uses,ask}.rs`;
      - `home_{root,shelf,door,links,leave,visits,wifi,play,acts,running,lending,join,make,open,ask,screen}.rs`.
    - It is a `[[test]]` with `test = false` in `crates/svrngs/Cargo.toml`, as browser is, and gets its own check.sh step, `page v2` (+1 `.`).
  - **(f) DESIGN, in place,** where §5 says what the gate runs: the switch, the parity measure and `page_v2` (AGENTS.md 3).
  - **Plants:**
    - the skeleton with the switch on makes the browser, join_page, scenes and conformance targets exit non-zero;
    - a harness planted to ignore the switch, serving v1, fails every test on the marker;
    - `SVRNGS_UI=v3` panics in its words;
    - one test planted failing makes `parity.sh` exit 1, naming it.
  - **Byte for byte:** every v1 test and its default run.
  - **Moves:** the parity report and its baseline. +25 to +40 (rung), +1 (`.`).
- [ ] page-frame — depends [page-parts] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the phone's root, its composition, and a stub for every phone flow:
  - **Owns:** CHARTER's page-frame line.
  - **(a) What stays imperative,** in `web/v2/app.js`:
    - the wasm `init({ module_or_path: store.runtime() })` from an unawaited function, since WebKit wedged on a top-level await (u4-7, `a:659-662`);
    - the `hashchange` reload (`a:626`);
    - the worker's registration on `load` (`a:627`);
    - `matchMedia('(pointer: fine)')` and `navigator.standalone`, each read once into the first state.
  - **(b) One machine, `PAGE`, composed from flows.**
    - Each flow module exports `{ owns, states, takes, io, view, heading, response, acts }`.
    - The frame refuses three things: a message type taken by two flows, a region or field owned by two, and a take whose state changed a field outside its `owns`.
    - Flows talk only by messages: a take answers `{run: 'send', m}`.
    - The heading is the first one given, in a fixed order: failed, visit, ask, among, elsewhere, join, enter.
    - The responses are joined with `\n` in that order, and the acts parts are merged in `q:237`'s `PARTS` order.
    - This replaces `#status`'s eleven writers (`a:85`, `:88`, `:132-133`, `:152`, `:167`, `:177`, `:212`, `:214`, `:221`, `:223`, `:309`, `:322`, `:406`, `:424`, `:475`, `:583`, `:702`) and `#response`'s seven (`a:225`, `:236`, `:239`, `:304`, `:574`, `:584`, `:586`).
  - **(c) `Page` draws `i:62-98` in its order, inside `#page`.** Its static words are byte for byte: `#computer`'s three paragraphs and `#how`'s six. `useEffect` sets `document.title` from state (`a:405`, `:422`).
  - **(d) Stubs:** `web/v2/{enter,shelf,index,door,join,visit,among,acts,uses,ask}.js`, each take answering `no(s, 'not built yet: <flow>')`.
  - **(e) A failure.** A thrown or failed effect with no words is drawn as "This browser could not open the page." (`a:88`).
  - **Test:** `web/tests/v2/frame.test.mjs`: the three refusals, the heading's order, and the responses' join.
  - **Plant:** two stubs both taking `read` turn it red.
  - **Byte for byte:** "Opening…" comes first, and once: join.mjs's `__said` observer (`join.mjs:94-100`) sees it a single time.
  - **Moves:** no demo. +30 to +40 (rung) [`a:47-117`, about 40].
- [ ] page-copy — depends [page-switch, page-frame] — ONE ROW, red first, and `scripts/check.sh` once, at the close — v2's first component, proven by its own tests:
  - **Owns:** `ui/copy.js`, `web/tests/copy.test.mjs`, `web/v2/computer.js` (it passes to page-v2-join), and `page_v2/phone_computer.rs`.
  - **(a) `ui/copy.js`.**
    - The machine `COPY` is `{ copy: ['idle', 'copying', 'copied', 'unwritten'] }`. Its io `write` answers `written`, with `failed: 'unwritten'`.
    - The component is `Copy({ id, text, words: [label, copied, unwritten], beside })`.
      - Without `beside`, the answer replaces the button's word (`a:611`).
      - With it, every answer is said after the button, in order, as `h:137` and `:202` append. The second " Copied." is kept for parity, and saying it once is page-home-links's commit.
    - A Copy while copying is refused.
  - **(b) Its first use: v2's computer line.** `web/v2/computer.js` is `offer()` and `line()` (`a:595-614`) as `Line`: `section#computer`, `#line`, and `Copy` with `id="copy"`. It needs only the fragment, with no dial.
  - **(c) Tests.**
    - `copy.test.mjs` covers every move and refusal.
    - `phone_computer.rs`, `in_chromium!`, opens v2's page as a desktop with a token. It asserts the line, the button's "Copied", the text written, no `securitypolicyviolation`, no page error, and the lines `COPY: copy: idle -> copying, running write` and `COPY: written: copying -> copied`.
  - **(d) The v1 Copy tests with the switch on. This row cannot prove itself by them, and says so.**
    - `the_page_offers_a_computer_one_line_and_sends_the_token_nowhere` (`b:main.rs:2950-2985`) waits for `#status.good, #status.bad, #join:not([hidden])` (`computer.mjs:36`). It passes on v2 once page-v2-enter and page-v2-join land.
    - `the_home_page_makes_a_link_to_invite_someone` (`b:main.rs:4031`, `:4074`) passes once page-home-root and page-home-links land.
    - Until then, `parity.sh` shows them failed.
  - **Plants:**
    - `copy.js` calling `document.body.setAttribute('style', 'x')` turns the violation assertion red;
    - a `copied` that answers `'done'` turns `copy.test.mjs` red;
    - with `UI` flipped to `Ui::V1`, `phone_computer.rs` is red on its lines.
  - **Byte for byte:** `#copy` "Copied" (`computer.mjs:51-53`), and `#computer` and `#line` (`links.mjs:43-44`).
  - **Moves:** v2's computer line in Chromium. +18 to +26 (rung) [`a:595-614`, about 15].
- [ ] page-home-drive — depends [page-switch] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the home page's driver waits for a redraw as it waits for a reload, green on v1 first, its assertions unchanged (beside page-copy):
  - **Owns:** `web/tests/home.mjs`. Every flag a home row needs is added here, and no home row edits it afterward.
  - **(a) A redraw is waited for as a reload is.** `home.mjs:101`'s press without `stays` waits for `Promise.race([load, changed])`. `changed` is the `stays` rule (`:96`), caught across a navigation. Then the waits at `:102-103` run as now. This keeps unchanged every v1 test that waits on a reload: `b:main.rs:2376`, `b:play.rs:92`, `b:guests.rs:267` and `:349`, `leave_page.rs:113` and `:391`, and `home_group.rs:187` and `:301`.
  - **(b) New outputs:** `loads`, the `load` events the page fired, where 1 means it never reloaded; and `lines` (`window.__lines`).
  - **(c) New flags:**
    - `until`, a fourth element of a `do` step: words the `all` text must hold before it is read;
    - `visible`, which hides and then shows the page, as `preact-spike/run.mjs:58` does;
    - `after`, a second selector read once the `do` steps are done.
  - **Plant:** a `changed` with no catch turns `b:main.rs:2360` red on v1 ("Execution context was destroyed").
  - **Byte for byte:** every Rust assertion, and every v1 test's output.
  - **Moves:** none; the gate stays green on v1. 0 lines (tests).
- [ ] page-tabs-drive — depends [page-switch] — ONE ROW, red first, and `scripts/check.sh` once, at the close — `tabs.mjs` waits for a fresh answer about what apps use, green on v1 first, its assertions unchanged (beside page-copy):
  - **Owns:** `web/tests/tabs.mjs`.
  - **The gap.** `tabs.mjs:45-47` shuts `#uses`, then writes `#used.replaceChildren()`, to be sure what it reads next is a fresh answer. That is a DOM write under Preact's tree, which a later render cannot reconcile.
  - **(a) The change.**
    - The write goes. In its place is a wait that both UIs satisfy, and that guards the same thing: the list read after `#uses summary` is clicked is an answer given after the click.
    - The row reads `a:329-345` and `:564-567` first, and phone.md's v2 shape, whose `asking` draws `#used` empty.
    - The commit says what it guards, before and after.
  - **Plant:** a v1 page planted not to ask again on the toggle makes `the_group_s_page_in_two_tabs_reaches_the_group_from_one` red on v1, so the wait is shown to wait.
  - **Byte for byte:** every assertion and test name.
  - **Moves:** none. 0 lines (tests).
- [ ] page-v2-enter — depends [page-copy] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the spine every phone flow reads. It is the one serial phone flow: without the group's answer, no phone flow's browser test can run:
  - **Owns:** `web/v2/enter.js`, `page_v2/phone_enter.rs`, `web/tests/v2/enter.mjs` and `enter.test.mjs`.
  - **(a) The regions and the flow.**
    - `tab: [unheld, holding, elsewhere, taking]` and `list: [unasked, reading, read, unread]`. Together they own `group`, `lent`, `people`, `shelf`, `unanswered`, `unheard`, `refused`, `behind`, `removals`, `visiting` and `changed`.
    - The fragment's routing (`a:681-699`: `READ`, the unread parts, `linkOf`, `asked`).
    - `hold`; `elsewhere`, with Choose's "Open it here" Act; and `enter`'s persist, unseal and endpoint.
    - The read, with its next-to-answer fallback (`a:433-436`).
    - The refusals: removed, said no, and older (`a:438-451`).
    - The kept `computers` and `addresses` (`a:456-458`).
    - The key and the endpoint are held by the io, never in state (`a:427-428`).
  - **(b) Stays imperative, at the edge:** `navigator.locks.request` with its hold-forever promise (`a:391`), the AbortError reload (`a:393-397`), and the no-locks path (`a:386`).
  - **(c) The fix, its own commit.**
    - **Stale today:** the list is read once (`a:436`), so nothing redraws (PAGE_STATE §1).
    - `visibilitychange` sends `shown`, and `reread` reads again through the bound endpoint, counted with `ask` and `late`. A late answer is refused by name.
    - A read again that fails keeps the list drawn, and the heading says "What <group> lends cannot be read again just now: <why>".
  - **Red under the switch** (each also pinned through the move):
    - `tabs.mjs:35`, `:79-80` (`className === 'good'`, `#door button` "Open it here", "another tab");
    - `the_group_s_page_in_two_tabs_reaches_the_group_from_one`;
    - `the_group_s_page_tells_a_removed_person`;
    - `an_older_computer_is_said_as_that_on_both_pages`;
    - `the_phone_goes_on_to_a_computer_that_answers` (`carried.rs:709`);
    - `a_link_naming_no_relay_is_refused_and_n0_never_reached`.
  - **v2-only:** `phone_enter.rs`. Another person lends while the page is hidden, and coming back shows it. It is red with `UI` flipped to `Ui::V1`.
  - **Moves:** scene `life` (Cy's phone in lake-trip). +65 to +80 [`a:355-369`, `:385-458`, `:667-706`, about 105].
- [ ] page-home-root — depends [page-home-drive, page-copy] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the home page's root: one read of `/v1/groups`, the group machine, and a stub for every home flow (beside page-v2-enter):
  - **Owns:** `home/v2/home.js`, `home/v2/group.js`, the stubs, and `page_v2/home_root.rs`.
  - **(a) The `HOME` machine.**
    - Its regions are `groups: [reading, read, unread]` and `said: [none, made, joined, left]`.
    - It reads `location` once, at the edge. `/ask` goes to `ask.js` (`h:358`), and `?screen=` to `screen.js` (`h:359`).
    - It draws in v1's order:
      - each section (`h:100-104`: `h2 small`, `data-colour`);
      - the early returns (`h:113-114`);
      - `p.none` (`h:373`) and `section#left` (`h:386-388`);
      - the flow slots, in `h:378-427`'s order;
      - `p.address` last (`h:427`);
      - "This computer cannot read your groups: …" on a failure (`h:429`).
    - It routes one fixed message, `again`, to every region that reads from this computer. A flow sends it after its own write.
  - **(b) The `GROUP` machine, one per group.**
    - Its regions are `people` and `lends`, each `[reading, read, unread]`, each with a counter. The people are drawn before the lends (`h:121`, `:175`; `b:main.rs:3589`, which is timed).
    - Its messages are fixed: `read`, `again` and `visible`.
    - A read refused after the session lapses (`home.rs:201`; `session.rs:175`) is said in words in the region that failed.
  - **(c) Stubs:** `home/v2/{shelf,door,links,leave,visits,wifi,play,acts,running,lending,join,make,open,ask,screen}.js`, each drawing nothing and refusing every take with `not built yet: <flow>`. `play.js` and `wifi.js` export empty parts for `shelf.js` and `acts.js` to import.
  - **(d) The fix, its own commit (CHARTER 9).** The query notices (`h:363-367`: `made`, `also`, `unplaced`, `joined`, `person`, `left`) become `HOME`'s `said`, said once. A reload of `/?made=Trip` no longer says "You made Trip." again.
  - **Red under the switch:** `b:main.rs:2589` (the address) and `:3227` (no group), against the skeleton's empty root.
  - **v2-only:** `home_root.rs`: one `/v1/groups` read, with people drawn before lends; and `said` given once, not again on a reload.
  - **Byte for byte:** `h:100-104`'s shape, `p.none`, `section#left`, `p.address`, and `h:429`'s words.
  - **Moves:** the computer's home page on v2 shows its groups. +70 to +90 [`h:21-66`, `h:347-429`, about 75].

- [ ] page-v2-shelf — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the first feature, and the bring bug's phone half:
  - **Owns:** `web/v2/shelf.js`, `ui/shelf.js`, `page_v2/phone_shelf.rs`, and `web/tests/v2/shelf.mjs`.
  - **(a) The move.**
    - `ui/shelf.js` `Shelf`, on `Act` and `Fold`, is `q:199-210` `table()`, which both pages call (`h:203`).
    - It includes "Bring more to <name>", folded for someone who just joined (`q:209`).
    - Its region is `bring: [idle, bringing, brought, refused]`, owning `ticked`. It takes `tick`, `bring`, `brought` and `unbrought`.
    - In this commit the answer goes where v1 puts it: the `p` above the boxes (`q:203`, `:207`).
  - **(b) The fixes, each its own commit:**
    - the answer in the `Act`, beside the button;
    - the button and the boxes disabled while bringing, where a second tap posts twice today;
    - `brought` answers `{run: 'send', m: {type: 'reread'}}`, so Events shows in `#lent` with no reload.
  - **Red under the switch:** `the_phone_brings_things_to_the_table`.
  - **v2-only:** in `phone_shelf.rs`, each red with `UI` flipped to `Ui::V1`:
    - the answer is the button's next sibling;
    - the button is disabled while bringing;
    - Events is in `#lent` after the bring, with no reload.
  - **Byte for byte:**
    - `Bring the ticked ones to ${name}` (`carried.rs:433`);
    - "You brought … to house: everyone in it can open it in a few seconds." (`carried.rs:551-554`);
    - `#shelf input`, `#shelf button` and `/You brought|already|Nothing was brought/` (`shelf.mjs:15-22`);
    - "Tick something to bring first."
  - **Moves:** the house demo's bring, on an iPhone. +30 to +40 [`q:199-210` 12, `a:485-489`].
- [ ] page-v2-index — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the lend table, and opening a lend from the fresh list:
  - **Owns:** `web/v2/index.js` (page-local: CHARTER), `page_v2/phone_index.rs`, and `web/tests/v2/index.mjs`.
  - **(a) The move.** `Index`, narrow (`a:537-541`).
    - Each row is `span > button`, named, then `agreed`, " Full now.", and `Keep(kept(l))` on a fine pointer (`a:536`). The rows are keyed by label.
    - `open` takes `{label, at}` from state's `lent`, not from a closure. The closure's `lent` and `unanswered` (`a:539`) and the reassigned `group` (`a:495`) would go stale after a read again.
    - The pass and the hand-off stay at the edge (`a:234-284`).
    - `#open=` naming (`a:546-563`: by label, else alias, `missed`, `nor`, `anew`, its yes) and the unlisted lines (`a:572-574`) are `response(s)`.
  - **(b) The fix, its own commit.** "Full now." and the kept addresses are drawn again on each read. Today they are drawn once.
  - **Red under the switch:**
    - join.mjs `lent`, `kept` and `leads` (`:158-165`);
    - `reads.mjs:27`;
    - cold.mjs `keep` and `index` (`:83-86`);
    - `a_lends_alias_opens_it_from_the_groups_page`;
    - `a_lends_alias_is_one_in_every_time_zone`;
    - `a_lent_page_is_told_the_lends_it_reads_and_opens_one_at_a_path`;
    - `a_phone_opens_a_lend_that_answers_only_in_gzip`;
    - `the_group_s_page_draws_the_door_and_the_index`.
  - **v2-only:** a lend filled while the page is shown reads "Full now." on coming back. It is red with `UI` flipped to `Ui::V1`.
  - **Moves:** scene `films` and the night demo. +40 to +50 [about 68].
- [ ] page-v2-door — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — who is in, the removals, and who is visiting:
  - **Owns:** `ui/door.js`, `ui/visitors.js`, `web/v2/door.js`, `page_v2/phone_door.rs`, and `web/tests/v2/door.mjs`.
  - **(a) The move.**
    - **The shared parts:** `ui/door.js` `Door` is `q:161-198` (`people`, `door`), judged by `unanswered`, with "asking…" until answered and "not known now" on a failure. `ui/visitors.js` is `q:43` `visiting`.
    - **The phone's own, in `door.js`:**
      - `#door > p`, titled (`a:481`, join.mjs `whose`);
      - `ul#removals`, with the line saying it cannot be read when the answer is `null` (`a:461-462`);
      - `section#visitors`, with "Visiting now" and the "mine" note (`a:467-469`).
  - **(b) The fix, its own commit.** A visit that ended is listed today until the page reloads. An end timer at the edge, at the earliest `until`, sends `reread`, so the ended visit drops from the list.
  - **Red under the switch:**
    - drive.mjs `drawn`, `door`, `removals` and `visiting` (`:203-210`);
    - `the_group_s_page_shows_every_removal_and_ask`;
    - `a_phone_is_shown_a_lend_s_computer_that_did_not_answer_as_its_window_dark`;
    - `the_phones_page_shows_sam_visiting_and_kims_stop_ends_it` (`guests.rs:323`).
  - **v2-only:** a visit ends while the page is open, and its line goes. It is red with `UI` flipped to `Ui::V1`.
  - **Moves:** scene `life`'s removal. +22 to +30 [`a:459-469`, `:479-481`, `q:43`, `q:161-198`, about 30].
- [ ] page-v2-join — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — joining by a link, and the rest of the computer's line:
  - **Owns:** `web/v2/join.js`, `web/v2/computer.js` (from page-copy), `page_v2/phone_join.rs`, and `web/tests/v2/join.mjs`.
  - **(a) The region:** `join: [unasked, asking, naming, redeeming, taken, joined]`, owning `why`.
    - `a:148-196`: the ask, the group known already, a device link with no name, the `redemption` error as `why`, and the 409 loop with `taken`'s words.
    - **NameForm:** `form#join`, `p#keep`, a `Field` (`for`) with focus, `span#group` and `Said#why`. The Home Screen line shows only when `standalone === false` (`a:127-129`).
    - `crypto.getRandomValues` and `store.keep` stay at the edge.
  - **(b) The computer's line, finished, in `computer.js`:** which of `#joins`, `#founds` and `#opens` is shown, and `#how` shown on a fine pointer once in (`a:525`). Both derive from `fine`, the token and the heading's region, with no region of their own.
  - **Red under the switch:**
    - join.mjs `__said`: every heading in order, "Adding this phone to … as sam’s…" included (`join_page.rs:321`);
    - `asked`, `why` and `hint` (`join.mjs:114-132`, `:166-176`);
    - cold.mjs `how` (`:87`);
    - both `join_page.rs` tests;
    - `the_page_offers_a_computer_one_line_and_sends_the_token_nowhere`;
    - `the_page_opened_with_no_link_offers_a_founder_s_line`;
    - `a_lent_app_writes_a_link_into_itself_that_opens_on_a_phone`.
  - **No fix is owed.** +40 to +50 [about 76].
- [ ] page-v2-visit — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — a guest's visit (`a:206-226`):
  - **Owns:** `web/v2/visit.js`, `page_v2/phone_visit.rs`, and `web/tests/v2/visit.mjs`.
  - **(a) The region:** `visit: [none, naming, redeeming, on, ended]`. It uses NameForm as a guest, with the heading "You’re invited over as a guest", and the visit's key kept apart (`store.keepVisit`).
  - **(b) The fix, its own commit.** Today the open page goes on offering the visit's lends past its end. An edge timer at `v.until` sends `ended`: the heading becomes "Your visit as …’s guest ended …", and the guest's index empties.
  - **Red under the switch:** `visit.mjs:52-56`, `:93-99`, and `sam_opens_the_link_kim_sent_and_watches_kims_film_as_kims_guest`.
  - **v2-only:** a visit of 5 s, ended while shown. It is red with `UI` flipped to `Ui::V1`.
  - +20 to +26 [19].
- [ ] page-v2-among — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — `#open=` across groups, and the cold redirect:
  - **Owns:** `web/v2/among.js`, `page_v2/phone_among.rs`, and `web/tests/v2/among.mjs`.
  - **(a) The flow.**
    - `among` (`a:581-587`): its heading, "Open it in the one you mean.", and Choose's Acts sending `hold` for the group picked.
    - `cold` (`a:638-646`) stays imperative at the edge: the `sessionStorage` ONCE marker and `location.replace`. Its refusal is a `failed` message with its words.
    - The no-group refusals, and "This link opens in svrngs on your computer." (`a:702-705`).
    - The cold redirect is never named `hop` (vocab).
  - **Red under the switch:**
    - cold.mjs `landed` (`#status.bad`, and `#response` not ending in "…");
    - `a_lends_address_opens_cold_in_a_browser_that_joined`;
    - `a_person_in_two_groups_opens_each_ones_lend_where_it_is_lent`;
    - `a_links_token_with_what_it_opens_opens_it_and_a_garbled_fragment_says_why`;
    - `a_lends_bookmark_opens_cold_on_a_computer`.
  - **No fix is owed.** +16 to +22 [about 15].
- [ ] page-v2-acts — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "What acts as you", on the phone:
  - **Owns:** `ui/acts.js`, `ui/changes.js`, `web/v2/acts.js`, `page_v2/phone_acts.rs`, `web/tests/v2/acts.mjs` and `acts.test.mjs`.
  - **(a) The move.**
    - **The shared parts:** `ui/acts.js` is `q:237-245` (`section#acts > details > summary > h2`, the `PARTS` order, the notes). `ui/changes.js` is `q:220-236`: "N apps changed. See what’s new", and "Open the new version", whose done replaces its `p`.
    - **The phone's own, in `acts.js`:**
      - a `doing` region, keyed by act, `[idle, busy, done, refused]`;
      - Reading's Stop as an `Act` (`a:512-518`);
      - the yes: `/runs`, `sign_line`, `line` and `unsent`, kept before the send (`a:501-508`), with the signing at the edge, with the key.
  - **(b) The fix, its own commit.** Today `unsent` is tried again only at the next yes (`a:506`). v2 sends `carry` on each `read` while `group.unsent` is not empty, so a line lost to a dropped dial is carried at the next open.
  - **Red under the switch:**
    - runs.mjs (`See what’s new`, `Open the new version`, `new version for you now`);
    - acts.mjs: `#acts details` folded, and `#acts summary`;
    - `page.mjs:162` (`#acts > details`);
    - `a_new_version_runs_as_each_person_only_after_their_yes`;
    - `the_group_page_says_what_acts_as_you_and_its_stop_holds_at_the_next_pass`.
  - **v2-only:** each is red as its kind requires:
    - `acts.test.mjs`: a `read` holding `unsent` answers `carry`, red with the take removed;
    - `phone_acts.rs`: a carry planted to fail is carried on the next open, red with `UI` flipped to `Ui::V1`.
  - +38 to +48 [about 25, and `q:220-245`, 15].
- [ ] page-v2-uses — depends [page-v2-enter, page-tabs-drive] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "What apps use" (`a:329-345`, `:564-567`):
  - **Owns:** `ui/uses.js`, `web/v2/uses.js`, `page_v2/phone_uses.rs`, and `web/tests/v2/uses.mjs`.
  - **(a) The move.**
    - **The shared part:** `ui/uses.js` is `q:77-88`, drawing only the `uses` region. It takes a `tag`: the phone's `li`, the home's `p`. The fetch moves to the edge.
    - **The phone's:** `uses: [shut, asking, asked]`, each lend's answer or why. It is the `Fold` `details#uses > summary` "What apps use", over `ul#used`, given to acts' `opens` part through `acts(s)`, in place of moving the static element in (`a:566`).
    - Opening it sends `fold`, which answers one `choices` run per lend that asks. Its `asking` draws `#used` empty, which is what page-tabs-drive's wait reads.
    - A Stop is an `Act`, and a Stop that lands asks again.
    - It never reads `#used`'s DOM.
  - **Red under the switch:** `#used li` and `#used button` "Stop" (`ask.mjs:82-86`), `tabs.mjs:43-50`, and `the_group_s_page_in_two_tabs_reaches_the_group_from_one`'s asks.
  - **No fix is owed.** +18 to +24 [about 15, and `q:77-88`, 10].
- [ ] page-v2-ask — depends [page-v2-enter] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the person asked (`a:288-325`):
  - **Owns:** `web/v2/ask.js` (page-local: CHARTER), `page_v2/phone_ask.rs`, and `web/tests/v2/ask.mjs`.
  - **(a) The region:** `ask: [none, asking, sending, refused]`.
    - It is `form#ask`, with `blockquote#words`, Yes and `button#not`.
    - `question()` is the heading, and `unlent` is a response.
    - Yes or Not now sends `open {label: app, at}`, which index takes (a stub until page-v2-index lands).
    - `choose` dials as the person, at the edge.
  - **(b) The fix, its own commit.** Today the form hides on a refused yes (`a:316`), and only the heading says why. v2 keeps the form shown, with the why beside Yes.
  - **Red under the switch:** `ask.mjs:66-68`, `:99-100` and `:113` (`#ask:not([hidden])`, `#status`, `#words`, `#response`); `a_lent_page_asks_its_person_through_the_group_s_page_and_is_stopped`; and the framed case's heading.
  - **v2-only:** a refused yes leaves the form shown, with its why. It is red with `UI` flipped to `Ui::V1`.
  - +24 to +30 [about 32].
- [ ] page-home-shelf — depends [page-home-root, page-v2-shelf] — ONE ROW, red first, and `scripts/check.sh` once, at the close — the first of the computer's flows, and the bring bug's computer half (`h:203`):
  - **Owns:** `home/v2/shelf.js`, `page_v2/home_shelf.rs`.
  - **Stale today:**
    - `h:203` hands `table` a `bring` whose answer only replaces `said`, the `p` above the boxes.
    - The index `tbody` (`h:178-191`) is never read again, so Money is missing until a reload. `b:carried.rs:493` opens a second page to see it, and `house_rules.rs:124` polls by reopening every 2 s.
    - Brought apps stay offered as boxes.
    - "Reads and writes for you" (`h:204`) and the `ul.wifi` foot stay as they were.
  - **(a) The move.**
    - `shelf.js` is the wide index, page-local (`h:118-119`, `:179-191`): its `thead`, each lend by an `a` or the `reach` form, `play.js`'s Play cell, and the computers that did not say as spanning rows.
    - Beside it, `ui/shelf.js` `Shelf`, the region page-v2-shelf's.
  - **(b) The fix, its own commit.** The answer goes beside the button. `brought` sends `again`, so the index, the shelf, "Reads and writes" and the foot are drawn from one read.
  - **Red under the switch:** `b:carried.rs:439-469`, `money.rs:123`, and `house_rules.rs:134-140`.
  - **v2-only:** `the_home_page_s_bring_shows_in_its_index_without_a_reload`. It brings Money with `until: "Money Who owes"` over `table.index tr, section h3 ~ p`, and asserts `loads == 1` and the lines `SHELF: brought: …` and `GROUP: again: …`. It is red with `UI` flipped to `Ui::V1`, where `until` never arrives.
  - **Byte for byte:**
    - `#home label:has-text(…) input`;
    - "Bring the ticked ones to ${name}" and "Tick something to bring first.";
    - "You brought Money to your group: everyone in it can open it in a few seconds.";
    - `table.index tr` and `section h3 ~ p`;
    - the `details` for a newcomer, from `said: joined`.
  - **Moves:** the money and house-rules scenes on v2, and the bring seen on the computer. +15 to +25 [`h:118-119`, `:178-191`, `:203`, about 18].
- [ ] page-home-door — depends [page-home-root, page-v2-door] — ONE ROW, red first, and `scripts/check.sh` once, at the close — who is in, the folder, the removals, and who decides:
  - **Owns:** `home/v2/door.js`, `page_v2/home_door.rs`.
  - **Stale today:**
    - The door is judged once (`h:175-176`).
    - "Open this folder" appends " Opened, in a window of its own." on every press (`h:130`).
    - `deciders` is pushed into `#acts` (`h:123`).
  - **(a) The move.**
    - `ui/door.js` `Door`, from GROUP's `people` and `lends` (`h:122-124`).
    - The folder lines (`h:126-132`), and `ul.removals` only when there are any (`h:154`).
    - A `decides` part that `acts.js` reads from GROUP.
  - **(b) The fixes, each its own commit:** the folder's answer said once; the door judged again on `visible`.
  - **Red under the switch:** `b:main.rs:3697`, `:3058`, `:4097`, `:2876`, `:2523`, `:2566`, `:2616`, `:2690` and `:3834`.
  - **v2-only:** two presses say it once, and `visible` lights a computer that came back. Each is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** the `.door li` classes `on`, `off` and `phone`; "asking…"; "not known now"; and "Open this folder Opened, in a window of its own."
  - +20 to +30 [about 22].
- [ ] page-home-links — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — a link to invite someone, add a device, or have a guest:
  - **Owns:** `home/v2/links.js`, `page_v2/home_links.rs`.
  - **Stale today:**
    - `p.link` is replaced by `shown` (`h:136`).
    - Copy appends " Copied." on every press (`h:137`).
    - "Have a guest" swaps the button for the select by `replaceWith` (`h:143`).
  - **(a) The move.** The region is `link: [none, making, made, refused]`, with `ui/copy.js` `Copy` `beside`.
  - **(b) The fix, its own commit:** " Copied." is said once.
  - **Red under the switch:** `b:main.rs:4031` (which needs page-copy, CHARTER), `visit.mjs:66-74` (`b:guests.rs:19`), and `home_group.rs:188`.
  - **v2-only:** two copies say " Copied." once. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:**
    - `p.link code`;
    - `button:text-matches('^Invite someone to ')` and `text-is('Add your phone or another computer')`;
    - "Have a guest", "Make a link for your guest", and the 48, 24 and 6 select;
    - `ends_with("…everyone it let in stays. Copy the link Copied.")`.
  - +20 to +28 [about 12].
- [ ] page-home-leave — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — leaving, and handing on:
  - **Owns:** `home/v2/leave.js`, `page_v2/home_leave.rs`.
  - **Stale today:**
    - `asked` (`h:42`) reads `r.json()` without `r.ok`, so the node's refusal is swallowed as "cannot be changed on this computer now."
    - After `handed`, the hand-on select and button stay, though the person no longer keeps the order (`h:151`).
    - A leave reloads to `/?left=` (`h:42`, `:367`).
  - **(a) The move.** The region is `leave: [idle, asking, handed, decides, refused]`. `left` sets HOME's `said: left`, then the groups are read again.
  - **(b) The fixes, each its own commit:** the refusal read through `answer` (`h:28-31`); `handed` sends `again`, so the hand-on paragraph goes.
  - **Red under the switch:** `leave_page.rs:102-122` and `:391`.
  - **v2-only:** `the_home_page_hands_on_then_leaves_without_a_reload`. It is red with `UI` flipped to `Ui::V1`. No browser test hands on today; `removal.rs` covers the route.
  - **Byte for byte:** "Leave ${name}", "You left Trip.", "Groups you left", "Trip (Sam’s, since", and `p.leave`.
  - +18 to +24 [about 10].
- [ ] page-home-visits — depends [page-home-root, page-v2-door] — ONE ROW, red first, and `scripts/check.sh` once, at the close — who you have over, and its Stop:
  - **Owns:** `home/v2/visits.js` (not `guests.js`: `guests` is a vocab noun), `page_v2/home_visits.rs`.
  - **Stale today:** Stop reloads at the record's `ts_unix` (`h:163`). Sooner, Sam would still read as here.
  - **(a) The move.** `ui/visitors.js` under `h3` "Who you have over", with `ul.yours` and a Stop `Act` for each of your guests (`h:162-166`). The Stop's effect is `{run: 'wait', until: ts}`, with the clock read at the edge, then a read again.
  - **(b) The fix, its own commit:** no reload.
  - **Red under the switch:** `b:guests.rs:268`, `:349`.
  - **v2-only:** `loads == 1`, with `VISITS: waited`. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** `#home .yours button`, "Who you have over", "Visiting now", and "Stop ${guest}’s visit".
  - +12 to +18 [about 8].
- [ ] page-home-wifi — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — one `room` region feeding both of its lists:
  - **Owns:** `home/v2/wifi.js`, `page_v2/home_wifi.rs`.
  - **Stale today:**
    - The foot (`h:202`, `ul.wifi`) and "Open to this WiFi" (`h:382`, `ul#room`) each draw `wifi()` from one read of `/v1/room` (`h:377`), and each redraws only itself (`h:77`).
    - So a press in the foot leaves `#room` stale, and `#room` is absent if nothing was open at load.
    - "Copy its address" appends on every press.
  - **(a) The move.** `ROOM: [reading, read, unread, turning]`. It exports the foot and `WifiPart`, for `acts.js`.
  - **(b) The fixes, each its own commit:** both lists drawn from `ROOM`; the copy said once.
  - **Red under the switch:** `b:main.rs:3392` (two runs), `:3467` and `:3325`.
  - **v2-only:** opening it in the foot shows it in `#acts #room li` in the same page, with `loads == 1`. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** "Open it to this WiFi", "Close it to this WiFi", the `on.said` words, "Closed to this WiFi: …", `table.index ~ ul.wifi li`, `#room`, and "Copied: <origin>. Paste it…"
  - +18 to +26 [about 10].
- [ ] page-home-play — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — Play, in the index and under "What acts as you":
  - **Owns:** `home/v2/play.js`, `page_v2/home_play.rs`.
  - **Stale today:** the index cell redraws itself (`h:91`), and the acts Stop reloads (`h:384`). Neither tells the other.
  - **(a) The move.** `PLAY: [reading, read, unread, turning]`, with the clipboard an edge effect. It exports the `Play` cell, for `shelf.js`, and `PlayPart`.
  - **(b) The fix, its own commit:** one region, no reload.
  - **Red under the switch:** `b:play.rs:57-111`, `ports.rs:84`, `:183`.
  - **v2-only:** Play and its Stop in one page, with `loads == 1`. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** "Play", "Stop", "Connect to …", " Copied.", "Open on this computer: played (cy’s), while Play is on. Stop", and the clipboard's text.
  - +14 to +20 [about 9].
- [ ] page-home-acts — depends [page-home-root, page-v2-acts, page-v2-uses] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "What acts as you", on the computer:
  - **Owns:** `home/v2/acts.js`, `page_v2/home_acts.rs`.
  - **Stale today:**
    - The parts arrive by imperative `add` pushes (`h:123`, `:204-206`, `:381-385`).
    - The uses Stop reloads (`h:221`).
    - A yes given in another tab shows only after a reload (`acts.mjs:60`).
  - **(a) The move.** `ui/acts.js`, its parts derived:
    - reads, from GROUP (`carrying`, `ui/changes.js`);
    - opens, from REACH (`/reaching`) and `ui/uses.js` with `tag` `p` (`/choices` per lend, `[asking, asked, failed]`);
    - knows;
    - `WifiPart` and `PlayPart`, imported, as stubs until their rows land;
    - decides, from `door.js` through GROUP.

    "Open the new version" sends `again`. The reach Stop stays a form, since a yes opens a tab (`acts.mjs:64` waits for its load).
  - **(b) The fixes, each its own commit:** a uses Stop asks `/choices` again; `visible` reads `/reaching` again.
  - **Red under the switch:** `b:acts.rs:68`, `b:carried.rs:640`, and `b:opens.rs:41`.
  - **v2-only:** after a yes in another tab, `visible` lists it with no reload. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** `#acts > details`, `summary > h2`, "Each Stop holds from the app’s next request.", "Open the new version", and "Money opens at its new version for you now."
  - +20 to +30 [about 14].
- [ ] page-home-running — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "Already running here":
  - **Owns:** `home/v2/running.js`, `page_v2/home_running.rs`.
  - **Stale today:** a lend redraws "What you lend" (`h:252`), but not the groups' index or foot, so jellyfin is missing there until a reload.
    - The look stays on the button by design: it is seconds of work (`h:229-236`). It is never automatic, and `visible` never looks.
  - **(a) The move.** `RUNNING: [unlooked, looking, looked, unread]`, with a region per row, `lend`.
  - **(b) The fix, its own commit:** `lent` sends `again`, which root routes to `lending.js` and GROUP.
  - **Red under the switch:** `b:main.rs:4394` (the timed `#lent h2` and `#running li`).
  - **v2-only:** the index lists "ada’s jellyfin" with `loads == 1`. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** "Look for programs running on this computer", "You looked just now", `input[name=name|kind]`, "Lend … to everyone in your groups", and "You lent …".
  - +18 to +24 [about 14].
- [ ] page-home-lending — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "What you lend":
  - **Owns:** `home/v2/lending.js`, `page_v2/home_lending.rs`.
  - **Stale today:** every change reloads (`h:283`), and "Say how many: 1 or more. " is appended on every press (`h:285`).
  - **(a) The move.** `LENDING: [reading, read, unread]`, with each lend's `change: [idle, changing, refused]`. The secret from "Sign people in" is kept in state, `shown`, which a read again never clears ("it is not shown again", `h:261`).
  - **(b) The fixes, each its own commit:** a read again, not a reload; the why said once.
  - **Red under the switch:** `b:main.rs:2360`, `:2414`.
  - **v2-only:** three changes with `loads == 1`, and `visible` keeping the secret on screen. Each is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** "Let in this many", "Let everyone in", "Stop lending", "You lend nothing yet.", a `sent` holding no secret, and `section#lent`.
  - +16 to +22 [about 14].
- [ ] page-home-join — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "Join a group from a link":
  - **Owns:** `home/v2/join.js`, `page_v2/home_join.rs`, `web/tests/v2/home-join.test.mjs`.
  - **Stale today:**
    - The guard at `h:411` compares the typed value, so an older answer for the same string typed again is taken.
    - Joining reloads to `/?joined=` (`h:409`).
  - **(a) The move.** `JOIN: [typing, checking, checked, refused, joining]`, guarded by `ask` and `late` on `link`. `joined` sets HOME's `said: joined` (person, unplaced) and reads the groups again.
  - **(b) The fix, its own commit:** no reload.
  - **Red under the switch:** `home_group.rs:258-320` (both engines) and `leave_page.rs:167`.
  - **v2-only:**
    - `home-join.test.mjs`, whose plant is the `late` check removed;
    - `home_join.rs`, `loads == 1`, red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:**
    - `#join input[name=link|person]`;
    - "Join Trip as Dee";
    - "Asking the computer that sent it what it is for…";
    - "You’re in Trip, as Dee.";
    - xenia's `includes('You’re in House')` (`xenia/main.rs:84`).
  - +16 to +22 [about 9].
- [ ] page-home-make — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "Make a group":
  - **Owns:** `home/v2/make.js`, `page_v2/home_make.rs`, `web/tests/v2/home-make.test.mjs`.
  - **Stale today:**
    - The guard at `h:420` compares the typed value.
    - A refusal is appended on every press (`h:421`).
    - Making a group reloads to `/?made=…&also=` (`h:364`).
  - **(a) The move.** `MAKE: [typing, checking, checked, making, refused]`, with a counter on `name`.
  - **(b) The fixes, each its own commit:** the refusal said once; no reload.
  - **Red under the switch:** `home_group.rs:123-250`.
  - **v2-only:**
    - `home-make.test.mjs`, whose plant is the counter removed;
    - `home_make.rs`, red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** `#make input[name=name|person]`, "Make a group", "You made Trip.", and the namesakes words (`q:149`).
  - +14 to +20 [about 8].
- [ ] page-home-open — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — "Open a link from your group":
  - **Owns:** `home/v2/open.js`, `page_v2/home_open.rs`.
  - **Stale today:** the refusal is appended on every press (`h:397`).
  - **(a) The move.** `within` (`h:335`) is a pure function in `step`. "Open it" still opens a tab with `window.open` (`h:398`), at the edge.
  - **(b) The fix, its own commit:** the refusal said once.
  - **Red under the switch:** `b:main.rs:3616`, `links.mjs:46`.
  - **v2-only:** two refusals say it once. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** `#open input`, `#open button`, and "That is not a link to something lent in your groups."
  - +8 to +12 [about 7].
- [ ] page-home-ask — depends [page-home-root] — ONE ROW, red first, and `scripts/check.sh` once, at the close — `/ask`:
  - **Owns:** `home/v2/ask.js` (page-local: CHARTER), `page_v2/home_ask.rs`.
  - **(a) The move.** From `lentIn` (`h:66`) and `h:338-345`: a section, `h2` question, a `blockquote`, and Yes and Not now. Yes still leaves by `location.assign` (`h:341`), at the edge.
  - **(b) The fix, its own commit:** the refusal said once, where `h:343` appends it.
  - **Red under the switch:** `b:main.rs:2315`.
  - **v2-only:** two refused yeses say it once. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** `#home h2` is `question()`, and `UNLENT`.
  - +8 to +12 [about 8].
- [ ] page-home-screen — depends [page-home-root, page-v2-door] — ONE ROW, red first, and `scripts/check.sh` once, at the close — a group on a screen:
  - **Owns:** `home/v2/screen.js`, `page_v2/home_screen.rs`.
  - **(a) The move.** `SCREEN: [reading, read, unread]`, with `seen` and `newest` kept in state, pure.
    - The poll at `h:328`, a read begun a second after the last began, becomes an edge timer that answers `tick`. The time's words are stamped at the edge.
    - The door is judged before it takes the last one's place, so a window never reads "asking…" again.
  - **Red under the switch:** `b:main.rs:3753` (`screen.mjs`), and xenia's `/?screen=` (`xenia/main.rs:265`).
  - **No visible fix.** `home_screen.rs` holds one v2-only check: through ten ticks, no window reads "asking…" after the first answer. It is red with `UI` flipped to `Ui::V1`.
  - **Byte for byte:** `body.screen`, the title, "Since this screen opened", `mark`, and "3 people · 2 on now · 2 things lent".
  - +18 to +24 [about 22].
- [ ] page-components — depends [page-v2-shelf, page-v2-index, page-v2-door, page-v2-join, page-v2-visit, page-v2-among, page-v2-acts, page-v2-uses, page-v2-ask, page-home-shelf, page-home-door, page-home-links, page-home-leave, page-home-visits, page-home-wifi, page-home-play, page-home-acts, page-home-running, page-home-lending, page-home-join, page-home-make, page-home-open, page-home-ask, page-home-screen] — ONE ROW, each fold its own commit with its behavior kept, and `scripts/check.sh` once, at the close — the primitives the flows wrote apart, made one:
  - **Owns:** `ui/parts.js` and `parts.test.mjs`, and, for each fold, the flow files it touches. No other row is open by then.
  - **(a) Find.**
    - Every primitive a flow wrote in its own file (fence 8), found by `git grep` over `web/v2/` and `crates/svrngs-node/src/home/v2/`.
    - Every near-duplicate across the two pages: `~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report --scope <dir>` for `web/v2` and `crates/svrngs-node/src/home`, pasting each clone with a side this queue added.
  - **(b) Fold** each into `ui/parts.js`, behavior kept, each its own commit, with its case in `parts.test.mjs`. If none is found, the row says so with the commands' output.
  - **Plant:** a fold that changes a drawn word turns `parts.test.mjs` red.
  - **Byte for byte:** every word, id and element kind; `parity.sh`'s verdicts before and after.
  - 0 to −20 (rung).
- [ ] page-parity — depends [page-components] — ONE ROW, the full gate on v2, and the operator's iPhone run prepared:
  - **Owns:** check.sh's `parity` step and `demo/runs/page/`.
  - **(a) Every v1 test on v2.**
    - `scripts/parity.sh` reports every v1 test passed on v2, or could-not-judge where check.sh allows it. Nothing is failed or never-ran.
    - Each test in the baseline's "passes with no v2" is named against the row that since earned it.
  - **(b) The whole gate on v2.**
    - `SVRNGS_UI=v2 lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh` exits 0. Every v1 test is unchanged since `46ba8288` but `web/tests/home.mjs` and `tabs.mjs`: `git diff 46ba8288 -- crates/*/tests web/tests` lists only those two, `page_v2/` and `web/tests/*.test.mjs` and `v2/`.
    - `page v2` is green.
  - **(c) The record,** `demo/runs/page/<date>-parity/`: `verdicts.tsv`, each step's verdict on v1 and on v2 side by side, the commands and the commit. One run is not a rate (AGENTS.md 7), so a second is recorded, on another day or on the Fedora box's image (`scripts/gate/README.md`).
  - **(d) check.sh gains `parity`** (`scripts/parity.sh`), held until the switchover so that v2 cannot slip back (+1 `.`).
  - **(e) Leftovers:** `grep -rn "not built yet" web/v2 crates/svrngs-node/src/home/v2` finds none.
  - **(f) The count beside the bar.** v2's lines that stay (the rung less `parity.sh`, `serve.rs` and what of `build.sh` does not fold) are set against v1's 703 deleted, with the arithmetic of CHARTER's "Estimates". If they are over, the switchover's pass is sized here.
  - **(g) The iPhone run, prepared, never run by the loop.**
    - `demo/runs/page/<date>-iphone/PREDICTIONS.md` is committed before any run.
    - Its demo: on a v2 build served from this Mac to the operator's iPhone, bring Events, see the answer beside the button, see a second tap refused, and see Events in the list with no reload. On the computer, the same bring is shown in the index with no reload.
    - Each bar has a chance written beside it.
    - `NEEDS_HUMAN.md` gives the operator's steps, then stop. The results are told to the watching session, which writes `README.md` there.
  - **Plant:** a v1 test left failing on v2 makes `parity.sh` exit 1, naming it, and `check.sh`'s `parity` step fail.
- [ ] HUMAN-page-switchover — depends [page-parity] — THE OPERATOR:
  - reads the parity record and the iPhone run;
  - if they choose, uses v2 on their own computer (a node from a test build with `--ui v2`) and on their iPhone (`node web/tests/serve.mjs` over `target/web/site-v2`);
  - says go, or names what is owed.

  The switchover's raise, if (f) found one, is set here or after its pass.
- [ ] page-switchover — depends [HUMAN-page-switchover] — ONE ROW, the swap in ONE commit so no tree holds half, any economizing commits before it, and `scripts/check.sh` once, at the close — v2 becomes the pages, and v1 goes:
  - **(a) The pass first, if page-parity's (f) is over.**
    - Write `demo/runs/economize/<date>-page-switchover/SURVEY.md`. Cuts are in v2 only, since v1 goes whole, each its own commit with its behavior kept, with `parity.sh` green after each batch (steward resources: batch the cuts, one run per batch).
    - Past the bar after the pass, write `NEEDS_HUMAN.md` with the raise asked, naming what the pass cut and why the rest could not be found, and stop until it lands (AGENTS.md "Done").
  - **(b) The swap, one commit.**
    - **The phone page:**
      - `web/v2/app.js` becomes `web/app.js`, and `web/v2/<flow>.js` move to `web/page/`;
      - `body.html`'s contents become `web/index.html`'s body. `git diff -U0 web/index.html` touches no line in `<head>`.
    - **The home page:** `home/v2/index.html` becomes `home/index.html`, `home/v2/home.js` becomes `home/home.js`, and `home/v2/<flow>.js` move to `home/page/`, served at `/page/`.
    - **Deleted:**
      - v1's `app.js` and `home.js`;
      - both pages' v1 bodies;
      - question.js's DOM builders (`visiting`, `uses`, `el`, `keeping`, `people`, `door`, `table`, `changes`, `acts`, `row`). Its word functions stay.
    - **The build:** `build-web.sh` takes the one UI. `--ui` and `web/v2/build.sh` are folded in, `SHELL` is still written by the build, `sw.js`'s hand list goes, and the 404.html derivation is kept.
    - **home.rs** serves the new files by `include_str!`, through one `macro_rules!` over their names on page-economize's table. `serve.rs` is deleted.
  - **(c) The switch removed, in the same commit:**
    - `SVRNGS_UI`, `Ui`, `ui()` and `site_of`;
    - `--ui` in cli.rs and build-web.sh, and the builder method;
    - the `svrngs-ui-v2` markers, and build-release.sh's refusal of them;
    - `window.__ui`, and v1's trace sink (with `app.js`), so `ui/trace.js` is the one;
    - `v1.test.mjs` and `policy.test.mjs`;
    - `release.test.mjs`'s `--ui` case. Its `test-support` case stays: it holds of every release;
    - `parity.sh` and check.sh's `parity` step;
    - `page_v2`'s `const UI`. The target becomes `tests/pages/`, and its step `pages`;
    - the rung, from `BUDGET`.
  - **(d) The tests default to the only UI**, every one unchanged by name.
  - **(e) The documents, rewritten in place** (AGENTS.md 3):
    - DESIGN §5's `svrngs-web` row (`docs/DESIGN.md:1203`) is said again;
    - PAGE_STATE §3's rules go where §5 says how the pages are built, each with the gate that holds it;
    - §3's home page no longer redraws by reloading;
    - page-switch's paragraph on the switch goes;
    - `ARCHITECTURE.md:482` and `:734`, and PLAN §1, are updated with it;
    - svrn-docs `PAGE_STATE.md` gets a closing line pointing at what landed.
  - **(f) Lines re-measured.** `.` is at or under its cap, 19,481 or the operator's raise, and the cap is lowered to one past where it lands.
  - **Plant:** `web/tests/one-ui.test.mjs`, in `page laws`, fails on any of these left in the tree: `SVRNGS_UI`, `svrngs-ui-v2`, `web/v2/`, `home/v2/`, `--ui` or `page_v2`. It is watched red on the swap with harness.rs's `ui()` left in, then green.
  - **Moves:** the live page and the home page, at the operator's deploy and release.
- [ ] HUMAN-page-deploy — depends [page-switchover] — THE OPERATOR:
  - deploys the page and cuts a release from the switchover's commit (memory "release from a commit": a detached worktree, `npm ci --prefix web`, `target/debug` dropped, `CARGO_INCREMENTAL=0 scripts/deploy-page.sh`);
  - reads it back: the page's version, the home page's `/page/` routes from a released binary, and `svrngs-ui-v2` in no tarball;
  - plays the house demo once on their iPhone against the live page.
