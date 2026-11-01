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

There is one row per feature across both pages, not one per flow per page
(the operator, 2026-10-31: "42 rows. That's absurd. Cut it in half.").

**The demo.**
- On an iPhone, the bring's answer is beside the button, a second tap is
  refused, and Events shows in the group's list with no reload.
- On the computer, the same bring shows in the index with no reload.
- Then the whole gate passes on v2 before anything switches.

**THE OPERATOR DECIDED, 2026-10-31,** to the six below: "1. Accept 2.
Three parallel when we get there 3. Approved -- the other main work
doesn't block us, it's just docs".
- **1 and 2:** the rung at 1,150, and the switchover's pass then a raise.
  Landed in 510e4a0e. `.`'s cap is now 19,115, at 19,114, since the
  diagrams' generators went uncounted (963dcdac). The room is still one
  line, and every estimate here is relative to it.
- **3:** run as a pool of 3 lanes. The base runs one row at a time by its
  dependencies, and the six features take the 3 lanes when page-open
  lands.
- **4 to 6, and CHARTER's settled list:** approved as written.
- **Lanes, not the main tree,** because the main tree holds another
  session's uncommitted work. Each lane is a clean worktree of `main`, and
  the loop merges into `main`.

**Decided before launch, no row.** The queue does not launch until the
operator has answered these. The watching session writes the answers here
and lands `BUDGET` from the operator seat (cap raises land from the
operator seat), with its plant: one counted line in a planted
`web/v2/x.js` past the cap makes `lean_check` exit 1 naming `v2`, with `.`
unchanged. Each item has the session's default.
1. **The rung**, its paths and its cap of 1,150 (CHARTER, "Estimates"):
   - **paths:** `web/v2/`, `crates/svrngs-node/src/home/v2/`, `crates/svrngs-node/src/home/ui/`, `web/vendor/` and `scripts/parity.sh`;
   - **`.`** skips those paths;
   - **the room:** `.` has one line of room, so if page-scaffold (a) cannot find the 13 to 16 lines the build's hooks take, it stops for a raise.
2. **The switchover's bar** (`.` at or under 19,481 once v1 is deleted)
   is not reachable on these estimates (19,540 to 19,834). Default: the
   switchover runs its pass, then asks the raise for what is left.
3. **Pool or serial.** The proposal is `pool` with 3 lanes, which puts
   merge commits on `main` (CHARTER, "Running rows in parallel"). Default:
   serial `run`.
4. **The home page moves whole** (PAGE_STATE "Not decided"). Default:
   whole.
5. **The two driver edits** in page-switch count as "tests unchanged".
   They are `web/tests/home.mjs` and `tabs.mjs`, each green on v1 first,
   with every Rust assertion untouched. Default: yes.
6. **v1 is frozen, except a fix the gate needs.** Such a fix updates its
   pin in the same commit and is mirrored in v2 in that row. Default: yes.

The settled list in CHARTER is the session's, and the operator can overrule
any of it at the same time. It covers the paths, `.js`, the derived
`index.html`, one root, `test-support`, the switch, `page_v2`, the flash
notices and the shared parts.

**The rows,** 13 of them, and the loop adds one closing audit.
- **The serial base:** page-scaffold, then page-switch, then page-open.
  After page-open, the browser tests can drive both v2 pages into a group.
- **In parallel,** each depending only on page-open, each covering both
  pages where the feature exists on both: page-shelf (first), page-index,
  page-members, page-join, page-acts and page-home-rooms.
- **The close:** page-parity, with the operator's iPhone run;
  HUMAN-page-switchover; page-switchover, one atomic commit; and
  HUMAN-page-deploy.

**How a merged row runs.**
- Each lettered part is its own commit: its move first, with v1's behavior
  kept, then each fix.
- A row runs over several sessions. Each session commits whole parts and
  ends with `ralph-result continue` naming the next one.
- `scripts/check.sh` runs once, at the row's close.
- **Red first:**
  - the v1 tests a part names fail under `SVRNGS_UI=v2` against the stub;
  - its v2-only tests, in `crates/svrngs/tests/page_v2/<row>.rs`, fail with
    `UI` flipped to `Ui::V1`;
  - both are cited in the part's commit, run under the gate lock.

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
  unless one grows a view (components.md §6).
- **A newer Preact.** A fix v2 needs, and page-scaffold (b)'s commands run
  again.

- [x] page-scaffold 8f0c512c — depends [] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — room in `.`, the vendored bytes, the pure core, the primitives, and both v2 pages empty, built and served beside v1, with v1 frozen:
  - **Owns:** CHARTER's page-scaffold line.
  - **(a) Economize, for `.`'s hooks** (AGENTS.md "Done").
    - **The survey:** run the instruments of `demo/runs/economize/2026-10-25-give/scripts/` again, into `demo/runs/economize/2026-10-31-page/SURVEY.md`. It is scoped to what `.` gains, +14 to +17 (CHARTER, "Estimates"), and looks first in `home.rs`, `build-web.sh`, `check.sh`, `build-release.sh`, `lean_check.py` and `cli.rs`. v1's page files are frozen and go whole later, so none is cut.
    - **The cuts:** first `home.rs:90-111` and `:141-144` (`JS`, `MODULE`, `QUESTION`, `LOOK`, `ICON` and their four routes, 15 counted lines) as one `(path, type, body)` table, −5 to −8; then the survey's next, largest first, to −13 to −16. Each cut is its own commit, behavior kept, and the cap is lowered to where they land.
    - **PLAN §1** names this queue.
    - If the room is not found, `NEEDS_HUMAN.md` asks the raise of what is left, and the row stops.
    - **Plant:** the ratchet's own, a cap one under the count. **Byte for byte:** v1's served bytes, with `tests/home.rs` unchanged by name.
  - **(b) Vendor Preact 11.1.1 and htm 3.1.1, with nothing loading them.**
    - **The files:**
      - `web/vendor/preact.js` (`dist/preact.mjs`), `hooks.js` (`hooks/dist/hooks.mjs`) and `htm.js` (htm@3.1.1 `dist/htm.module.js`): Preact 11.1.1's tarball has no `.module.js`, only `.mjs` (corrected, page-scaffold (b));
      - `LICENSE.preact` (MIT), from the tarball;
      - `LICENSE.htm` (Apache-2.0), from htm's tarball, which does carry one, byte-equal to github.com/developit/htm's at `3.1.1`, whose tree has no NOTICE (corrected, page-scaffold (b)).
    - **The README** follows `apps/grapevine/vendor/README.md`. It gives:
      - each tarball's `dist.integrity` and its sha256;
      - each file's upstream path and its sha256;
      - the two edits: `hooks.js`'s `from"preact"` becomes `from"./preact.js?v=__VERSION__"`, since the policy allows no importmap (`trust_floor.txt:60-67`), and every `//# sourceMappingURL=` trailer is stripped;
      - the commands that make these bytes again;
      - the table `apps/pinned.mjs` reads.
    - **`web/tests/vendor.test.mjs`** holds the pins through `apps/pinned.mjs` `unpinned`. It also checks that no file has a map trailer, and that `hooks.js` imports exactly `./preact.js?v=__VERSION__`.
    - **The gate step `page laws`** is `node --test` over `find web/tests -name '*.test.mjs'`, never-ran when none is found (`check.sh:134-136`). It is its own step, since apps' laws are apps' (AGENTS.md 12), and AGENTS.md's "Done" names it.
    - **Plant:** a byte changed in `hooks.js` turns it red, naming the file.
  - **(c) The pure core, `ui/machine.js`.** No DOM, no Preact, no clock, no storage, and no module state.
    - `machine(name, { states, step, io }, state, first = [])` answers `{ get, dispatch, subscribe }`.
    - `state.at` holds one named state per region, each region a closed set. A step that answers outside its set is refused (AGENTS.md 9).
    - `step(s, m)` answers `{ state, effects, refused? }`, written with `go(s, at, patch, ...effects)` and `no(s, why)`.
    - An effect `{ run, failed, …args }` runs as `Promise.resolve().then(() => io[f.run](f))`. A rejection, a throw or a missing io answers `{ type: f.failed, n, why }`.
    - `ask(s, kind)` answers `[asked, n]`. `late(s, kind, n)` answers `''`, or "`<kind> <n> answered after <kind> <asked> was asked`".
    - Subscribers are told once per change, never on a refusal.
    - **The trace,** this file its one home, through `globalThis.__trace?.(line)` (`web/store.js:88`): `<NAME>: <type>[ (<why>)]: <from> -> <to>[, running <run>, …]` and `<NAME>: <type> refused in <from>: <why>`.
    - **`web/tests/machine.test.mjs`,** on a toy machine of two regions, covers:
      - a step, traced;
      - a refusal: the same reference, nobody told, nothing run;
      - an effect rejected, thrown, or with no io;
      - a late answer;
      - a state outside its set;
      - the effects' order.
    - **Plants:** `late`'s comparison removed, and the closed-set check removed, each turn its case red.
  - **(d) The view and the seven primitives.**
    - `ui/view.js` holds `html`, `render`, an 8-line `useStore` and `useMachine`. `useStore` is `useState` with `useLayoutEffect`, since `useSyncExternalStore` is `preact/compat`'s alone.
    - `ui/trace.js` is v2's sink, a copy of `a:55-64`, and a v2-only marker until the switchover.
    - **`ui/parts.js`** (CHARTER, "The parts"; components.md §1, 1 to 7):
      - `Act`: `words`, `doing` (`{state: idle|busy|done|refused, said}`), `on`, `type`, `answer` (`beside` or `instead`) and `send`. It is disabled and silent while busy, and its answer is a text node beside, as v1's is;
      - `Said`;
      - `Fold`: `onToggle` sends `{type: 'fold', fold, open}`, so a driver's `open =` (`web/tests/home.mjs:66`, `:134`) is taken;
      - `Field`: the `for` shape focuses on mount (`a:136`). There is also the `wrap` shape, and `select`;
      - `Row` (`q:263`);
      - `Keep` (`q:140`, byte for byte);
      - `Heading`.
    - **`web/tests/parts.test.mjs`** calls each as a function and checks its vnodes against what the v1 builder it replaces draws, with no DOM.
    - **Plants:**
      - an `Act` that sends while busy turns it red;
      - `Keep`'s space moved turns it red;
      - a `Fold` silent on toggle turns it red.
  - **(e) The phone's v2 page and its build.**
    - `web/v2/body.html` is `<!-- svrngs-ui-v2 --><div id="page"><h1 id="status">Opening…</h1></div>`, and never changes after.
    - **`web/v2/app.js`,** the frame, keeps at the edge:
      - the wasm init from an unawaited function (u4-7, `a:659-662`);
      - the `hashchange` reload (`a:626`);
      - the worker's registration on `load` (`a:627`);
      - `matchMedia('(pointer: fine)')` and `navigator.standalone`, each read once into the first state;
      - `window.__ui = 'v2'`.
    - **One machine, `PAGE`, composed from flows.**
      - Each flow module exports `{ owns, states, takes, io, view, heading, response, acts }`.
      - The frame refuses a message type taken by two flows, a region or field owned by two, and a take that changed a field outside its `owns`.
      - Flows talk only by `{run: 'send', m}`.
      - The heading is the first given, in the order failed, visit, ask, among, elsewhere, join, enter. The responses are joined with `\n` in that order, and the acts parts are merged in `q:237`'s order.
      - This replaces `#status`'s eleven writers (`a:85`, `:88`, `:132-133`, `:152`, `:167`, `:177`, `:212`, `:214`, `:221`, `:223`, `:309`, `:322`, `:406`, `:424`, `:475`, `:583`, `:702`) and `#response`'s seven (`a:225`, `:236`, `:239`, `:304`, `:574`, `:584`, `:586`).
      - `Page` draws `i:62-98` in its order, inside `#page`, with `#computer`'s three paragraphs and `#how`'s six byte for byte. `useEffect` sets `document.title` (`a:405`, `:422`).
      - A failed effect with no words is drawn as "This browser could not open the page." (`a:88`).
      - Stubs: `web/v2/{enter,computer,shelf,index,door,visit,join,among,acts,uses,ask}.js`, each take answering `no(s, 'not built yet: <flow>')`.
      - **Where it landed** (corrected, page-scaffold (e), 0d9e1c95): `compose` and `Page` are `web/v2/page.js`, page-scaffold's like app.js, apart from app.js's edge so node tests them. A flow leaves out what it has none of. A stub is `export const unbuilt = [types]`, refused by `compose`, and `view` draws its places as `i:` draws them, so a row starts from v1's markup. A take may be keyed by a part's name, `fold <fold>` or `typed <field>`, so two flows can each draw a Fold. A slot is shared: a flow with no slot of its own, such as index's unlisted lines, answers in `enter`.
    - **The build.** `scripts/build-web.sh` takes `--ui v1|v2`, `v1` by default, a closed set. `v2` sources `web/v2/build.sh`, which:
      - derives `index.html` from `web/index.html`, with `body.html` between `<body>` and the `<script` line, and fails unless `<head>` is byte-equal (`cmp`);
      - copies `web/v2/app.js` to `$OUT/app.js`, `web/v2/*.js` to `$OUT/v2/`, and `vendor/` and `ui/` with their paths kept;
      - hashes and stamps them all, the shared `.js` included (`:123`);
      - writes `$OUT/sw.js`'s `SHELL` from what it copied;
      - fails in words on any leftover `__VERSION__`;
      - writes `target/web/site-v2` by default.
    - **`web/tests/v2/frame.test.mjs`:** the three refusals, the heading's order and the responses' join. **Plant:** two stubs both taking `read` turn it red.
    - **Byte for byte:** "Opening…" comes first, and once (`join.mjs:94-100`).
  - **(f) The home's v2 page, in a test build only.**
    - `home/v2/index.html` is v1's byte for byte, except two things: `<script type="module" src="/v2/home.js?v=__VERSION__">` and the marker comment. `home/v2/home.js` is an empty `Home` root.
    - `home/v2/serve.rs`, under `#[cfg(feature = "test-support")]`, serves `/`, `/ask`, `/v2/<name>`, `/ui/<name>`, `/vendor/<name>` and `/question.js` from the sources (`env!("CARGO_MANIFEST_DIR")`).
      - Names must match `^[a-z0-9-]+\.(js|css|html)$`, so no row adds a route.
      - It sits behind `only_itself`, under `POLICY` (`home.rs:81`), and holds the string `svrngs-ui-v2`.
    - It is chosen per node by a builder method in `serve.rs`, and in the binary by `--ui v2` (`cli.rs`), both under `test-support`. home.rs merges its routes in one line. v1's `/home.js` and `/question.js` stay served (`tests/home.rs:185-189`).
    - **Where it landed** (corrected, page-scaffold (f), 7156bbd2):
      - The builder method is `Node::ui(self, Ui) -> Self`, a write-once choice. The binary's choice is `--ui v1|v2`, anywhere in its arguments (`Ui::asked`).
      - home.rs's one line is a `.layer`, not a merge, inside `only_itself`: axum panics on a second `/` route. The layer answers v2's paths at a node chosen v2 and passes every other request to v1's routes.
      - `cargo test` never gives a bin target the features of its dev-dependencies. So a test that runs the binary on v2 builds it with `--features svrngs/test-support` (svrngs's own feature, `crates/svrngs/Cargo.toml`), which page-switch (b)'s mismatch words name.
  - **(g) The guards.**
    - **`web/tests/v1.test.mjs`** pins the sha256 of `web/index.html`, `app.js`, `sw.js`, `store.js`, `open.html`, `open.js`, `lend.js` and `shim.js`, and of `home/index.html`, `home.js`, `question.js`, `look.css` and `away.html`.
    - **In the row,** the two `shasum` lists of `target/web/site`, built at the base and at the close, are equal, `VERSION` included, and the diff goes in the commit body.
    - **`web/tests/policy.test.mjs`:** the derived `<head>` equals `web/index.html`'s, policy line `:51` included. `trust_check.py` exits 0, and `git diff -- scripts/trust_floor.txt` is empty.
    - **`web/tests/specifiers.test.mjs`:** every import into `vendor/`, `ui/`, `v2/` or `question.js` carries `?v=__VERSION__`, and only `ui/view.js` imports `vendor/`.
    - **`web/tests/release.test.mjs`** fails if any `[dependencies]` (not dev) or `default` names `test-support`, or if `build-release.sh` or `deploy-page.sh` names `--features`, `--all-features` or `--ui`. `build-release.sh` refuses a built binary in which `grep -c svrngs-ui-v2` is not 0.
    - `tests/home.rs` gains v2's routes under `test-support`.
    - **Plants:**
      - a byte in `web/app.js` turns `v1` red;
      - `'unsafe-inline'` in the head turns `policy` red, and the build's `cmp` fails;
      - `ui/` left out of the stamping fails the build in words;
      - a planted `../x.js` gets 404, named in `tests/home.rs`;
      - `features = ["test-support"]` under svrngs's `[dependencies]` turns `release` red;
      - a bare `../vendor/preact.js` turns `specifiers` red.
    - **Where it landed** (corrected, page-scaffold (g), 6f998f80..34297d41):
      - `policy.test.mjs` runs `web/v2/build.sh` itself, sourced as build-web.sh sources it over a stand-in for the wasm, so the derived `<head>` it reads is the build's, in about 0.3 s, with no wasm compiled. A `<head>` in `body.html` and `ui/` left out of `STAMPED` are kept as its own cases.
      - The refusal decides on `grep -qa`'s exit status, not on `grep -c`'s count: this Mac's interactive grep is ugrep, which prints no count for a binary (`grep -c svrngs-ui-v2 target/debug/svrngs`: nothing, exit 1).
      - The `../` plant asks for files that are there (`/v2/../home.js`, `/ui/../question.js`), since a 404 for a missing `x.js` proves nothing. The case carries no `#[cfg]`, so it can never quietly not compile.
      - The two `shasum` lists sit in this row, not in a commit body: a gate's result has no commit of its own (fence 14). `target/page-scaffold/base.sha` (built at b50592ca, whose diff from 3df32f1f touches nothing under `web/` or svrngs-web) and `close.sha` (the closing gate's `target/web/site` at 5809ca63, page-scaffold-6.await.log, exit 0) are 19 lines each, every page at `v=8c6090c43f4f`, and `diff` prints nothing.
      - The trust floor is unchanged against the base: `git diff 3df32f1f HEAD -- scripts/trust_floor.txt` is empty.
      - `.`'s hooks are +14 (build-web.sh +3, build-release.sh +1, check.sh +3, home.rs +2, node.rs +2, cli.rs +3), net −2 against (a)'s 16 cut. That is past +9 to +11 by node.rs's choice and home.rs's layer, which no estimate sized. The operator's widening (47552a83) approved the package that showed `.` at 19,118 and named (g)'s one line. `.` is 19,119 of 19,122, so its room of 3 covers page-switch's `page v2` and page-parity's `parity`, +1 each.
  - **Byte for byte:** all of v1, and every test name.
  - **Moves:** `page laws`, and both v2 pages served empty. +149 to +250 (rung; the operator widened it from +208, 2026-10-31: the phone frame 92 against +30 to +40, serve 62; demo/runs/page/2026-10-31-scaffold-range/NEEDS_HUMAN.md), +9 to +11 (`.`), against (a)'s cuts [`a:47-117`, about 40].
- [x] page-switch 3d459972 — depends [page-scaffold] — ONE ROW, each part its own commit, its plants red first, and `scripts/check.sh` once, at the close — one switch for what the v1 tests drive, which fails loudly while v2 is short, and the two driver edits, green on v1 first:
  - **Owns:** CHARTER's page-switch line.
  - **(a) The switch.**
    - `harness.rs` gains `enum Ui { V1, V2 }` and `ui()`, which reads `SVRNGS_UI` once (`OnceLock`): unset means V1, and any other value panics naming `v1` and `v2`.
    - `site()` becomes `site_of(ui())`, and `site_of(Ui::V2)` builds with `--ui v2`.
    - Every node start in a target that includes harness.rs takes `ui()`. Those targets are browser, conformance, join_page, scenes, home_group, leave_page, install, relay, phone_dry and xenia (`git grep -l browser/harness.rs crates`). Each start is found and listed in the commit.
  - **(b) It never passes vacuously.**
    - The site's `index.html`, and `/` where the harness first hands out a home page's address, carry `svrngs-ui-v2` if and only if the switch is V2.
    - A mismatch panics with its cause in words, such as a binary built without `test-support`.
    - **The baseline:** the first parity run, on the empty pages, is recorded in `demo/runs/page/2026-10-31-baseline/`, with its await budget at 3 h. A test passing there is listed apart, as "passes with no v2".
  - **(c) `scripts/parity.sh`** (rung).
    - It `exec`s under `lockf -k /tmp/svrngs-gate.lock`, and runs each harness target as check.sh runs it, with `SVRNGS_UI=v2`.
    - It writes `target/parity/verdicts.tsv`, per test: `passed`, `failed`, `could-not-judge` (from the files check.sh reads) or `never-ran`. never-ran is a test `cargo test -- --list` names on v1 with no verdict on v2, or a target that did not build.
    - It prints the counts and each not-passed test, and exits 1 on any not-passed.
    - check.sh does not run it until page-parity, since it would double the slowest steps (`check.sh:29`).
  - **(d) The v2-only target, `crates/svrngs/tests/page_v2/`.**
    - `main.rs` takes harness.rs by `#[path]` and holds `const UI: Ui = Ui::V2;`.
    - It has one empty `mod` file per feature row: `open`, `shelf`, `index`, `members`, `join`, `acts` and `rooms`.
    - It is a `[[test]]` with `test = false`, with its own check.sh step, `page v2` (+1 `.`).
  - **(e) The home page's driver waits for a redraw as for a reload** (`web/tests/home.mjs`).
    - `home.mjs:101`'s press without `stays` waits for `Promise.race([load, changed])`, where `changed` is the `stays` rule (`:96`) caught across a navigation.
    - It adds the outputs `loads` (1 means it never reloaded) and `lines`, and the flags `until`, `visible` and `after`.
    - It keeps unchanged `b:main.rs:2376`, `b:play.rs:92`, `b:guests.rs:267` and `:349`, `leave_page.rs:113` and `:391`, and `home_group.rs:187` and `:301`.
    - **Plant:** home.js's lend `change` (`h:283`) planted to redraw in place (`lending()`) for its reload: the base `home.mjs` turns `b:main.rs:2360` red on v1 (waiting for `load`), this one green. (Corrected: a `changed` with no catch stayed green, since Playwright's `waitForFunction` re-runs across a navigation; d847e479.)
  - **(f) `tabs.mjs` waits for a fresh answer** (`web/tests/tabs.mjs`).
    - `:45-47` shuts `#uses` and writes `#used.replaceChildren()`, a DOM write under Preact's tree. In its place is a wait both UIs satisfy that guards the same thing: the list read after the click is an answer given after it.
    - The part reads `a:329-345` and `:564-567` first. v2's `asking` draws `#used` empty.
    - **Plant:** a v1 page planted not to ask again on the toggle turns `the_group_s_page_in_two_tabs_reaches_the_group_from_one` red.
  - **(g) DESIGN, in place,** where §5 says what the gate runs: the switch, parity and `page_v2` (AGENTS.md 3).
  - **Plants:**
    - the empty pages with the switch on make the browser, join_page, scenes and conformance targets exit non-zero;
    - a harness planted to ignore the switch fails every test on the marker;
    - `SVRNGS_UI=v3` panics in its words;
    - one test planted failing makes `parity.sh` exit 1, naming it.
  - **Byte for byte:** every v1 test, its default run, and every Rust assertion. (e) and (f) are the only v1 test files changed in this queue.
  - **Moves:** the parity report and its baseline. +25 to +40 (rung), +1 (`.`).
- [x] page-open 99a42027 — depends [page-switch] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — the phone page opens into its group, and the home page reads its groups. After it, the browser tests can drive both v2 pages into a group:
  - **Owns:** CHARTER's page-open line.
  - **(a) `ui/copy.js`, the first component, and the phone's computer line.**
    - `COPY` is `{ copy: ['idle', 'copying', 'copied', 'unwritten'] }`, its io `write`, with `failed: 'unwritten'`.
    - `Copy({ id, text, words, beside })`. Without `beside`, the answer replaces the button's word (`a:611`). With it, each answer is said after the button, as `h:137` and `:202` append, the second " Copied." kept.
    - A Copy while copying is refused.
    - `web/v2/computer.js` is `offer()` and `line()` (`a:595-614`): `section#computer`, `#line` and `Copy` with `id="copy"`, from the fragment, with no dial.
    - `web/tests/copy.test.mjs` covers every move and refusal. `page_v2/open.rs`, `in_chromium!`, asserts:
      - the line;
      - "Copied";
      - the text written;
      - no `securitypolicyviolation`;
      - no page error;
      - the `COPY:` lines.
    - **Plants:**
      - `copy.js` setting a `style` attribute turns the violation assertion red;
      - a `copied` that answers `'done'` turns `copy.test` red.
    - **Byte for byte:** `#copy` "Copied" (`computer.mjs:51-53`), `#computer` and `#line` (`links.mjs:43-44`).
    - **The v1 Copy tests:** `the_page_offers_a_computer_one_line_and_sends_the_token_nowhere` and `the_home_page_makes_a_link_to_invite_someone` pass on v2 only once page-join lands. Until then, parity shows them failed.
  - **(b) The phone's enter, the spine every phone feature reads** (`web/v2/enter.js`).
    - **The regions:** `tab: [unheld, holding, elsewhere, taking]` and `list: [unasked, reading, read, unread]`. Together they own `group`, `lent`, `people`, `shelf`, `unanswered`, `unheard`, `refused`, `behind`, `removals`, `visiting` and `changed`.
    - **The routing:** the fragment's (`a:681-699`), `hold`, `elsewhere` with Choose's "Open it here" `Act`, and `enter`'s persist, unseal and endpoint.
    - **Choose is drawn in `#door`** (`tabs.mjs:79` finds it there), a place `web/v2/door.js` draws, which is page-members'. So door.js's place draws enter's `chosen(s, send)`, one line, and page-members keeps it (corrected, page-open (b)).
    - **The read:** its next-to-answer fallback (`a:433-436`); the refusals removed, said no and older (`a:438-451`); and the kept `computers` and `addresses` (`a:456-458`).
    - **At the edge:** the key and the endpoint are held by the io, never in state (`a:427-428`). `navigator.locks.request` with its hold-forever promise (`a:391`), the AbortError reload (`a:393-397`) and the no-locks path (`a:386`) stay there too.
    - **Red under the switch:**
      - `tabs.mjs:35`, `:79-80`;
      - `the_group_s_page_in_two_tabs_reaches_the_group_from_one`;
      - `the_group_s_page_tells_a_removed_person`;
      - `an_older_computer_is_said_as_that_on_both_pages`;
      - `the_phone_goes_on_to_a_computer_that_answers` (`carried.rs:709`);
      - `a_link_naming_no_relay_is_refused_and_n0_never_reached`.
  - **(c) The fix: the phone reads again on coming back.** Today the list is read once (`a:436`).
    - `visibilitychange` sends `shown`, and `reread` reads again through the bound endpoint, counted with `ask` and `late`.
    - A read again that fails keeps the list drawn, and says "What <group> lends cannot be read again just now: <why>".
    - **v2-only:** another person lends while the page is hidden, and coming back shows it in `list`'s lines. Red with `UI` flipped to `Ui::V1`.
  - **(d) The home's root and its groups** (`home/v2/home.js`, `home/v2/group.js`).
    - **`HOME`:** `groups: [reading, read, unread]` and `said: [none, made, joined, left]`.
      - It reads `location` once: `/ask` and `?screen=` (`h:358-359`).
      - It draws in v1's order: each section (`h:100-104`), the early returns (`h:113-114`), `p.none` (`h:373`), `section#left` (`h:386-388`), the feature slots in `h:378-427`'s order, `p.address` last, and `h:429`'s failure.
      - It routes one fixed message, `again`, to every region that reads from this computer.
    - **`GROUP`, one per group:** `people` and `lends`, each `[reading, read, unread]`, each counted.
      - The people are drawn before the lends (`b:main.rs:3589`, timed).
      - Its messages are `read`, `again` and `visible`.
      - A read refused after the session lapses (`home.rs:201`) is said in words.
    - **Stubs:** `home/v2/{shelf,index,running,lending,door,visits,leave,screen,join,make,links,open,acts,ask,wifi,play}.js`, each drawing nothing and refusing every take with `not built yet: <file>`. `play.js` and `wifi.js` export empty parts for `index.js` and `acts.js`.
    - **Red under the switch:** `b:main.rs:2589` (the address) and `:3227` (no group).
    - **Byte for byte:** `h:100-104`'s shape, `p.none`, `section#left`, `p.address` and `h:429`'s words.
  - **(e) The fix: the query notices** (`h:363-367`) become `HOME`'s `said`, said once (CHARTER 9). **v2-only:** one `/v1/groups` read, with people before lends, and a reload of `/?made=Trip` that does not say it again. Red with `UI` flipped.
  - **Moves:** scene `life` (Cy's phone in lake-trip), the computer line, and the computer's home page on v2 showing its groups. +153 to +196 [`a:355-369`, `:385-458`, `:595-614`, `:667-706`, about 120; `h:21-66`, `:347-429`, about 75].
- [x] page-shelf 97eac838 — depends [page-open] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — the first feature, and the bring bug on both pages:
  - **Owns:** `ui/shelf.js`, `web/v2/shelf.js`, `home/v2/shelf.js`, `page_v2/shelf.rs`, and `web/tests/v2/shelf.mjs`.
  - **(a) The move.**
    - `ui/shelf.js` `Shelf`, on `Act` and `Fold`, is `q:199-210` `table()`, which both pages call (`h:203`). It includes "Bring more to <name>", folded for someone who just joined.
    - Its region is `bring: [idle, bringing, brought, refused]`, owning `ticked`, taking `tick`, `bring`, `brought` and `unbrought`.
    - The answer goes where v1 puts it: the `p` above the boxes (`q:203`, `:207`).
    - The phone's `#shelf` and the home's group section each mount it.
    - **Red under the switch:** `the_phone_brings_things_to_the_table`; `b:carried.rs:439-469`; `money.rs:123`; `house_rules.rs:134-140`.
    - **Where it landed** (corrected, page-shelf (a), bf54bc80):
      - The phone brings through `web/v2/enter.js` `through(group, path, body)`, an edge function page-shelf added to page-open's file (02f78828): the key and the one endpoint are enter.js's, and a flow's own endpoint of the key would displace it at the relay (`a:352`). Every phone write after it uses it.
      - The home's newcomer fold reads `?joined=` as v1 does (`h:203`), as `home/v2/shelf.js` loads, before `home.js` forgets it: a group's place is given GROUP's state, not HOME's `said`, and `home.js` is page-open's.
      - Green on v2: `the_phone_brings_things_to_the_table`, and `the_home_page_brings_things_to_the_table` through every shelf assertion, red then at `b:carried.rs:511`, the index, page-index's. `money.rs` and `house_rules.rs` make and join their groups on the home page first, so stay red until page-join.
  - **(b) The fixes, each its own commit, both pages:**
    - the answer in the `Act`, beside the button;
    - the button and the boxes disabled while bringing, where today a second tap posts twice;
    - brought apps no longer offered;
    - `brought` sends `reread` on the phone and `again` on the computer.
  - **v2-only:** each red with `UI` flipped to `Ui::V1`:
    - the answer is the button's next sibling;
    - the button is disabled while bringing;
    - the bring is followed by a read whose answer holds the brought app (`SHELF: brought: …`, then `LIST:` and `GROUP: again:` lines, with `loads == 1`).

    That it is drawn in the index is page-index's test (CHARTER, "What the merge moved").
  - **Where (b) landed** (corrected, page-shelf (b), 0560da0d..5695e619):
    - The read right after a bring does not hold the app: the node lists a carried app once the folder has placed its files in the record (`carried.rs:234-246`, woken by file events, `synced.rs:286-289`), and the first run read "0 lent". So each read after a bring that does not list what it brought is followed by another, 1, 2, 4 and 8 s on, the wait at the edge, then refused with why (`SHELF: listed`). "<app> is on the table." is drawn from the read alone.
    - The trace is `SHELF: brought (`, then `PAGE: reread: ` on the phone, or `HOME: again: ` and `GROUP: again: ` on the home page; there is no `LIST:` machine.
    - While it brings, the shelf says " Bringing…" beside the held button: Act is silent while busy, so v1's `home.mjs` `settled` read the second press mid-bring (5695e619).
    - `web/tests/v2/shelf.test.mjs` (8 tests, `page laws`) holds the move to question.js `table` byte for byte and every SHELF step. One run on V1 (`target/page-shelf/red-b.log`) fails every v2-only check on both pages.
    - **Lines:** v2 at 521, +44 against +40 to +58; `.` unchanged.
  - **Byte for byte:**
    - `Bring the ticked ones to ${name}` (`carried.rs:433`);
    - "You brought … to house: everyone in it can open it in a few seconds." (`carried.rs:551-554`);
    - "You brought Money to your group: …";
    - `#shelf input`, `#shelf button` and `/You brought|already|Nothing was brought/` (`shelf.mjs:15-22`);
    - `#home label:has-text(…) input`;
    - "Tick something to bring first.";
    - the `details` for a newcomer, from `said: joined`.
  - **Moves:** the house demo's bring, on an iPhone and on the computer. +40 to +58 [`q:199-210` 12, `a:485-489`, `h:203`].
- [x] page-index 769b9e3f — depends [page-open] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — what the group lends, drawn from each fresh read, on both pages; and on the computer, what it runs and lends:
  - **Owns:** `web/v2/index.js`, `home/v2/index.js`, `home/v2/running.js`, `home/v2/lending.js`, `page_v2/index.rs`, and `web/tests/v2/index.mjs`.
  - **(a) The phone's index** (`a:537-541`, page-local).
    - Each row is `span > button`, named, then `agreed`, " Full now.", and `Keep` on a fine pointer, keyed by label.
    - `open` takes `{label, at}` from state's `lent`, never a closure's (`a:539`, `:495`). The pass and the hand-off stay at the edge (`a:234-284`).
    - The unlisted lines (`a:572-574`) are `response(s)`.
    - **Red under the switch:**
      - join.mjs `lent`, `kept` and `leads` (`:158-165`);
      - `reads.mjs:27`;
      - cold.mjs `keep` and `index`;
      - `a_lent_page_is_told_the_lends_it_reads_and_opens_one_at_a_path`;
      - `a_phone_opens_a_lend_that_answers_only_in_gzip`;
      - `the_group_s_page_draws_the_door_and_the_index`.
  - **(b) The home's wide index** (`h:118-119`, `:179-191`, page-local): its `thead`, each lend by an `a` or the `reach` form, `play.js`'s cell (a stub until page-home-rooms), and the computers that did not say as spanning rows. **Red under the switch:** `b:carried.rs:493`, `house_rules.rs:124`.
  - **(c) The fix: both indexes are drawn from each read.**
    - On the phone, "Full now." and the kept addresses are refreshed. On the computer, the index is drawn again on `again`, where today Money is missing until a reload.
    - **v2-only:**
      - a lend filled while shown reads "Full now." on coming back;
      - a bring shows in `#lent` and in `table.index tr` with no reload (`until: "Money Who owes"`, `loads == 1`).

      Each is red with `UI` flipped.
  - **(d) "Already running here"** (`running.js`): `RUNNING: [unlooked, looking, looked, unread]`, with a region per row. The look stays on its button and is never automatic (`h:229-236`). **Red:** `b:main.rs:4394`. **Fix, its own commit:** `lent` sends `again`, so the index lists "ada’s jellyfin" with `loads == 1`.
  - **(e) "What you lend"** (`lending.js`): `LENDING: [reading, read, unread]`, with each lend's `change: [idle, changing, refused]`. The secret from "Sign people in" is kept in `shown`, which a read again never clears (`h:261`). **Red:** `b:main.rs:2360`, `:2414`. **Fixes, each its own commit:** a read again in place of the reload (`h:283`), and the why said once (`h:285`). **v2-only:** three changes with `loads == 1`, and `visible` keeping the secret.
  - **Byte for byte:**
    - the button's parent `childNodes`, `small.keep`, and `table.index` before `#door`;
    - `table.index tr`;
    - "Look for programs running on this computer", "You looked just now", `input[name=name|kind]`, "Lend … to everyone in your groups" and "You lent …";
    - "Let in this many", "Let everyone in", "Stop lending", "You lend nothing yet." and `section#lent`;
    - a `sent` holding no secret.
  - **Moves:** scene `films`, the night demo, and Events or Money seen without a reload. +78 to +108 [about 62 phone, `h:118-119`, `:178-191`, about 15, and about 28 running and lending].
- [x] page-members 39674ee8 — depends [page-open] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — who is in, who left, who is visiting, a guest's own visit, leaving, and the screen:
  - **Owns:** `ui/door.js`, `ui/visitors.js`, `web/v2/door.js`, `web/v2/visit.js`, `home/v2/door.js`, `home/v2/visits.js`, `home/v2/leave.js`, `home/v2/screen.js`, `page_v2/members.rs`, and `web/tests/v2/members.mjs`.
  - **(a) The door, both pages.**
    - **The shared parts:** `ui/door.js` `Door` is `q:161-198`, judged by `unanswered`, with "asking…" and "not known now". `ui/visitors.js` is `q:43`.
    - **The phone:** `#door > p`, titled (`a:481`); `ul#removals`, with its line when the answer is `null` (`a:461-462`); and `section#visitors` (`a:467-469`).
    - **The home:** Door from `GROUP` (`h:122-124`); the folder lines (`h:126-132`); `ul.removals` when there are any (`h:154`); and a `decides` part `acts.js` reads through `GROUP`.
    - **Red under the switch:**
      - drive.mjs `drawn`, `door`, `removals` and `visiting` (`:203-210`);
      - `the_group_s_page_shows_every_removal_and_ask`;
      - `a_phone_is_shown_a_lend_s_computer_that_did_not_answer_as_its_window_dark`;
      - `b:main.rs:3697`, `:3058`, `:4097`, `:2876`, `:2523`, `:2566`, `:2616`, `:2690` and `:3834`.
    - **Where it landed** (corrected, page-members (a), fa07bb80):
      - The line numbers above had moved; the red set run was by name: both `every_removal_and_ask`, `the_door_draws_who_is_on_now`, `the_home_page_tells_two_people_of_one_name_apart`, `the_home_page_says_where_the_groups_folder_is`, `the_home_page_opens_the_group_s_folder`, `the_group_s_page_draws_the_door_and_the_index` and `whose_computer_did_not_answer_and_who_asked_are_told_by_key`.
      - Three stay red past every door assertion until page-index lands: `whose_computer…` at `table.index td[colspan]`, `…draws_the_door_and_the_index` at its index row, `the_door_draws_who_is_on_now` at home.mjs's `text` wait for `#lent h2 ~ *`.
      - The phone's "your guest" line needs the key's public spelling: `web/v2/enter.js`'s `read` answers `me`, `w.actor(key)`, a field enter owns (b5c4e397), as page-shelf added `through` there.
      - The home's folder button is FOLDER, a machine of its own in `home/v2/door.js`, which also exports `post` and `decides(g, rows, me)` for page-acts.
  - **(b) Visits, both pages.**
    - **The phone:** an end timer at the earliest `until` sends `reread`, its own commit.
    - **The home:** `visits.js` (not `guests.js`, a vocab noun). "Who you have over", `ul.yours`, and a Stop `Act` whose effect is `{run: 'wait', until: ts}`, then a read again, in place of `h:163`'s reload, its own commit.
    - **Red:** `the_phones_page_shows_sam_visiting_and_kims_stop_ends_it` (`guests.rs:323`), `b:guests.rs:268`, `:349`.
    - **Where it landed** (corrected, page-members (b), c642e7d2): both guests tests pass every visits assertion on v2, then wait in home.mjs for `#lent h2` after Kim's Stop, page-index's section, so stay red until it lands. The phone's end timer is `ends`/`end` in `web/v2/door.js`, asked by `Ends`, a part drawing nothing, since `web/tests/v2/frame.test.mjs` draws each place as a plain function. Its v2-only tests, `a_visit_ended_while_open_drops_on_the_phone` and `a_guest_s_stop_ends_the_visit_with_no_reload`, drive `web/tests/v2/members.mjs`.
  - **(c) A guest's own visit** (`visit.js`, `a:206-226`): `visit: [none, naming, redeeming, on, ended]`, NameForm as a guest, and its key kept apart. **Fix, its own commit:** an edge timer at `v.until` sends `ended`, and the guest's index empties. **Red:** `visit.mjs:52-56`, `:93-99`, and `sam_opens_the_link_kim_sent_and_watches_kims_film_as_kims_guest`.
    - **Where it lands** (corrected, page-members (c)): the guest's name is asked in `form#join`, in `web/v2/join.js`'s place (page-join's), and what the visit opens is listed in `#lent`, `web/v2/index.js`'s place (page-index's), so each place draws visit.js's part, `naming` and `listed`, one line each, as `#door` draws enter's `chosen`. page-join's NameForm takes a guest's form over, `typed name` with it. `sam_opens_the_link_kim_sent…` stops first at Kim's "Have a guest", `home/v2/links.js`, page-join's stub (target/page-members/red-c.log, `guests.rs:103`), so it stays red until page-join lands; the move's red is the v2-only `a_guest_at_their_link_is_asked_their_name_once_and_is_in`, against the stub.
  - **(d) Leaving and handing on** (`leave.js`): `leave: [idle, asking, handed, decides, refused]`, where `left` sets `said: left`. **Fixes, each its own commit:** the refusal read through `answer` (`h:28-31`, where `h:42` swallows it), and `handed` sending `again`. **Red:** `leave_page.rs:102-122`, `:391`.
    - **Where it landed** (corrected, page-members (d), f1188f42, b7f773b6, 8e0af161): HOME's `left` is taken by leave.js's `take` (group.js `offered`), which sets `groups: reading` with `said: left`, so the page says it once the groups are read. The refusal is read by LEAVE's own io, not by `answer`: removal.rs's outcomes are 409s that are read (`decides`), so a reply is its outcome, or a 401 said in group.js's words, or its `error`. `a_leave_reaches_the_leavers_other_computer` is green on v2. `leaving_on_the_home_page…` passes every leave assertion, then stops at `#join`, page-join's stub, so it stays red until page-join lands (target/page-members/green-d4.log). Its v2-only tests are `a_refused_leave_is_said_in_its_own_words` and `hand_on_then_leave_with_no_reload`.
  - **(e) A group on a screen** (`screen.js`), on `Door`: `SCREEN: [reading, read, unread]`, with `seen` and `newest` in state. The poll (`h:328`) becomes an edge timer answering `tick`, and the door is judged before it takes the last one's place. **Red:** `b:main.rs:3753`, xenia's `/?screen=` (`xenia/main.rs:265`). No visible fix.
    - **Where it landed** (corrected, page-members (e), 0f05e0fe, 650e8d6a): the group's link to a screen, `show`, the last of GROUP's places, is screen.js's too: `the_home_page_copies_a_lend_s_address_for_a_program` stopped at it (`main.rs:3403`) and is green on v2 with `the_screen_shows_what_is_lent_after_it_opened` (target/page-members/green-e.log). The page's class and title are SCREEN's `look` effect; ui/door.js gains the newest mark and `counted`. xenia's `/?screen=` is matched by its title, "House on a screen", which `look` sets as v1 did; xenia runs only by name, so it was not run here. The v2-only test is `ten_screen_ticks_never_read_asking_again`, its plant a screen.js drawing the door unjudged while a tick's read is out. **Lines:** screen.js and door.js +31, so the row is +176 (v2 847 of 1,207).
  - **(f) The home door's fixes, each its own commit:** the folder's answer said once (`h:130`), and the door judged again on `visible`.
    - **Where it landed** (corrected, page-members (f), c0f51b0f, e1f03249): (f) landed before (e), as it fit the row's room and (e) did not (demo/runs/page/2026-10-31-members-range/NEEDS_HUMAN.md). Judging the door again on `visible` was page-open's already (home.js and group.js `visible`), so its commit is a test alone, `the_door_is_judged_again_when_shown_again`. It draws a computer that went off as "did not answer", not one that came back on, as nothing in `page_v2` restarts a node in process. The folder's test is `two_folder_presses_are_said_once`.
  - **v2-only:** each red with `UI` flipped to `Ui::V1`:
    - a visit ended while open drops on the phone;
    - a Stop with `loads == 1` and `VISITS: waited`;
    - a 5 s visit ends while shown;
    - hand on, then leave, with no reload;
    - two folder presses said once;
    - `visible` relights a computer;
    - ten screen ticks never read "asking…" again.
  - **Byte for byte:**
    - `.door li` with `on`, `off` and `phone`;
    - "Open this folder Opened, in a window of its own.";
    - `#home .yours button`, "Who you have over", "Visiting now" and "Stop ${guest}’s visit";
    - "You’re invited over as a guest";
    - "Leave ${name}", "You left Trip.", "Groups you left", "Trip (Sam’s, since" and `p.leave`;
    - `body.screen`, "Since this screen opened", `mark`, and "3 people · 2 on now · 2 things lent".
  - **Moves:** scene `life`'s removal, and the guests scenes. +110 to +180 (the operator widened it from +152, 2026-11-01: leave 24, the screen v1's 22 with its machine, +24 to +32; demo/runs/page/2026-10-31-members-range/NEEDS_HUMAN.md) [about 31 phone, `q:43`, `:161-198` 19, about 62 home].
- [ ] page-join — depends [page-open] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — coming in by a link, and making and sharing links, on both pages:
  - **Owns:** `web/v2/join.js`, `web/v2/computer.js` (from page-open), `web/v2/among.js`, `home/v2/join.js`, `home/v2/make.js`, `home/v2/links.js`, `home/v2/open.js`, `page_v2/join.rs`, `web/tests/v2/join.mjs`, `home-join.test.mjs` and `home-make.test.mjs`.
  - **(a) The phone joins** (`join.js`): `join: [unasked, asking, naming, redeeming, taken, joined]`, owning `why`.
    - The ask, a group known already, a device link with no name, `redemption`'s `why`, and the 409 loop (`a:148-196`).
    - **NameForm:** `form#join`, `p#keep`, a `Field` with focus, `span#group` and `Said#why`. The Home Screen line shows only when `standalone === false` (`a:127-129`).
    - `crypto.getRandomValues` and `store.keep` stay at the edge.
    - `computer.js` is finished: which of `#joins`, `#founds` and `#opens` shows, and `#how` on a fine pointer once in (`a:525`).
    - **Red under the switch:**
      - join.mjs `__said`, every heading in order (`join_page.rs:321`);
      - `asked`, `why` and `hint`;
      - cold.mjs `how`;
      - both `join_page.rs` tests;
      - `the_page_offers_a_computer_one_line_and_sends_the_token_nowhere`;
      - `the_page_opened_with_no_link_offers_a_founder_s_line`;
      - `a_lent_app_writes_a_link_into_itself_that_opens_on_a_phone`.
  - **(b) `#open=` across groups, its aliases, and cold** (`among.js`).
    - The naming: by label, else alias, `missed`, `nor`, `anew` and its yes (`a:546-563`).
    - `among` (`a:581-587`).
    - `cold` (`a:638-646`) at the edge, never named `hop`.
    - The no-group refusals (`a:702-705`).
    - **Red:**
      - cold.mjs `landed`;
      - `a_lends_alias_opens_it_from_the_groups_page`;
      - `a_lends_alias_is_one_in_every_time_zone`;
      - `a_lends_address_opens_cold_in_a_browser_that_joined`;
      - `a_person_in_two_groups_opens_each_ones_lend_where_it_is_lent`;
      - `a_links_token_with_what_it_opens_opens_it_and_a_garbled_fragment_says_why`;
      - `a_lends_bookmark_opens_cold_on_a_computer`.
  - **(c) The home joins** (`join.js`): `JOIN: [typing, checking, checked, refused, joining]`, guarded by `ask` and `late` on `link`, where `h:411` compares the typed value. `joined` sets `said: joined`. **Fix, its own commit:** no reload (`h:409`). **Red:** `home_group.rs:258-320`, `leave_page.rs:167`.
  - **(d) The home makes a group** (`make.js`): `MAKE: [typing, checking, checked, making, refused]`, with a counter on `name` (`h:420`). **Fixes, each its own commit:** the refusal said once (`h:421`), and no reload (`h:364`). **Red:** `home_group.rs:123-250`.
  - **(e) The home's links** (`links.js`): `link: [none, making, made, refused]`, with `Copy` `beside`. **Fix, its own commit:** " Copied." said once (`h:137`). **Red:**
    - `b:main.rs:4031`, which with (a) closes page-open's two Copy tests;
    - `visit.mjs:66-74`;
    - `home_group.rs:188`.
  - **(f) "Open a link from your group"** (`open.js`): `within` (`h:335`) as a pure function, with `window.open` at the edge. **Fix, its own commit:** the refusal said once (`h:397`). **Red:** `b:main.rs:3616`, `links.mjs:46`.
  - **v2-only:**
    - `home-join.test.mjs` and `home-make.test.mjs`, whose plants remove `late` and the counter;
    - `join.rs`: joining with `loads == 1`, one " Copied.", one refusal on make and on open. Each is red with `UI` flipped.
  - **Byte for byte:**
    - "Adding this phone to … as sam’s…";
    - `#join input[name=link|person]`, "Join Trip as Dee", "Asking the computer that sent it what it is for…" and "You’re in Trip, as Dee.";
    - xenia's `includes('You’re in House')`;
    - `#make input[name=name|person]`, "Make a group", "You made Trip." and the namesakes words;
    - `p.link code`, `^Invite someone to `, "Add your phone or another computer", "Have a guest", "Make a link for your guest", the 48, 24 and 6 select, and `…Copy the link Copied.`;
    - "That is not a link to something lent in your groups.";
    - "This link opens in svrngs on your computer."
  - **Moves:** both `join_page.rs` tests, and joining and making a group on the computer with no reload. +120 to +160 [about 97 phone, about 36 home].
- [ ] page-acts — depends [page-open] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — "What acts as you", the yes and the Stop, what apps use, and the person asked, on both pages:
  - **Owns:** `ui/acts.js`, `ui/changes.js`, `ui/uses.js`, `web/v2/acts.js`, `web/v2/uses.js`, `web/v2/ask.js`, `home/v2/acts.js`, `home/v2/ask.js`, `page_v2/acts.rs`, `web/tests/v2/acts.mjs` and `acts.test.mjs`.
  - **(a) The shared parts.**
    - `ui/acts.js` is `q:237-245`.
    - `ui/changes.js` is `q:220-236`: "See what’s new", and "Open the new version", whose done replaces its `p`.
    - `ui/uses.js` is `q:77-88`, with `tag` (the phone's `li`, the home's `p`) and its fetch at the edge.
  - **(b) The phone.**
    - **`acts.js`:** a `doing` region keyed by act, `[idle, busy, done, refused]`, and Reading's Stop as an `Act` (`a:512-518`). The yes keeps `/runs`, `sign_line`, `line` and `unsent` before the send (`a:501-508`), with the signing at the edge.
    - **`uses.js`:** `uses: [shut, asking, asked]`, the `Fold` `details#uses` over `ul#used`, given to `opens` through `acts(s)`. `fold` asks `choices` per lend, `asking` draws `#used` empty, and a Stop asks again. It never reads `#used`'s DOM.
    - **`ask.js`:** `ask: [none, asking, sending, refused]`, `form#ask`, `blockquote#words`, Yes and `button#not`. `question()` is the heading, Yes and Not now send `open`, and `choose` dials at the edge.
    - **Where (b) landed** (corrected, page-acts (b), 8c8da32a):
      - **`choose` and `uses`' reads dial a lender through the one endpoint**, which is enter's alone (`web/v2/enter.js:93` `bound`); a second endpoint of the key displaces it at the relay (`:115`). So `web/v2/enter.js` gains one export, `dialled`, page-acts' for that line only.
      - **Nothing sends `ask`:** enter's `looked` sends `visit`, `join`, `among`, `offer` and `failed` (`enter.js:38-50`), and the read is enter's. So `form#ask` sends it once the list is read and the address asks.
      - **`acts(s)` gives no `send`** (`web/v2/page.js:111`), and every act here has a button. So `acts.js` draws the parts itself, `uses.js`'s Fold among them; the phone has no wifi or play part.
      - **After a yes or a Stop, enter is sent `opened` with the group as kept,** and reads again: enter's `keep` after each read writes `s.group` back (`enter.js:72`), so a stale `line` would sign a seq twice. `acted` keeps each answer, so a yes stays said after the read no longer lists the change.
      - **`me`**, this phone's key in the ask a link's issuer is sent (`a:509`), is asked of the edge once the tab holds the group (`whoami`).
      - **Red, green:** `acts.rs` passes to `:242`, whose last step opens `#open=<label>` after the read, page-join's. `carried.rs:640`'s phone half runs after its home half, (c).
      - **Lines:** v2 739, the row at +111 of +108 to +144 with (c) and (d) to come.
    - **Where (c) landed** (corrected, page-acts (c), 9b049558):
      - **Nothing of GROUP's reaches a HOME place:** group.js gives its places only `g`, `s`, `send`, `home` and `me` (`group.js:46`), and HOME's places get HOME's state alone (`home.js:40`). So `group.js`'s PLACES gains `told` (`:26`), acts.js's `Told`, which draws nothing and tells HOME each group's people and lends as GROUP reads them.
      - **The foot waits:** v1 drew `p.address` after every read (`h:326-328`), and home.mjs opens every `<details>` as it shows (`home.mjs:72-75`). So Home draws it once no feature's file `holds` it (`home.js:43`), and acts.js holds it until its section has read.
      - **`b:opens.rs:41` is green at the stub:** its yes is index.js's `Reach`, page-index's, so it is not this row's to earn.
      - **Lines:** v2 809, the row at +181 of +108 to +144, so it stops for its range (`demo/runs/page/2026-11-01-acts-range/NEEDS_HUMAN.md`). (d) is next.
  - **(c) The home.**
    - **`acts.js`:** the parts derived, in place of `add` pushes (`h:123`, `:204-206`, `:381-385`):
      - reads, from `GROUP`;
      - opens, from REACH and `ui/uses.js`;
      - knows;
      - `WifiPart` and `PlayPart` (stubs until page-home-rooms);
      - decides, through `GROUP`.

      The reach Stop stays a form (`acts.mjs:64`).
    - **`ask.js`:** from `lentIn` (`h:66`) and `h:338-345`. Yes leaves by `location.assign` at the edge.
  - **(d) The fixes, each its own commit:**
    - `unsent` is carried on each `read` while not empty (`a:506`);
    - a refused yes keeps `#ask` shown, with its why (`a:316`);
    - the home's uses Stop asks again, in place of `h:221`'s reload;
    - `visible` reads `/reaching` again (`acts.mjs:60`);
    - the home's ask refusal is said once (`h:343`).
  - **Red under the switch:**
    - runs.mjs; acts.mjs; `page.mjs:162`;
    - `#used li` and `#used button` (`ask.mjs:82-86`), and `tabs.mjs:43-50`;
    - `ask.mjs:66-68`, `:99-100` and `:113`;
    - `a_new_version_runs_as_each_person_only_after_their_yes`;
    - `the_group_page_says_what_acts_as_you_and_its_stop_holds_at_the_next_pass`;
    - `a_lent_page_asks_its_person_through_the_group_s_page_and_is_stopped`;
    - `b:acts.rs:68`, `b:carried.rs:640`, `b:opens.rs:41` and `b:main.rs:2315`.
  - **v2-only:**
    - `acts.test.mjs`: a `read` holding `unsent` answers `carry`;
    - `acts.rs`, each red with `UI` flipped: a planted failed carry is carried on the next open; a refused yes leaves the form; a yes in another tab is shown on `visible`; two refusals on `/ask` are said once.
  - **Byte for byte:**
    - `See what’s new`, `Open the new version`, `new version for you now` and "Money opens at its new version for you now.";
    - `#acts > details`, `summary > h2`, and "Each Stop holds from the app’s next request.";
    - `#used li`, `#used button` "Stop", `#ask:not([hidden])`, `#words` and `#home h2` as `question()`;
    - `UNLENT`.
  - **Moves:** the yes to a new version, and Stops that hold, on both pages. +108 to +190 (the operator widened it from +144, 2026-11-01: the shared parts 23 with no estimate, the home 70 against +28 to +42, and (d)'s five fixes; demo/runs/page/2026-11-01-acts-range/NEEDS_HUMAN.md) [about 72 phone, `q:77-88`, `:220-245` 25, about 22 home].
- [x] page-home-rooms 1372867c — depends [page-open] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — WiFi and Play on the computer, each one region feeding both of its lists:
  - **Owns:** `home/v2/wifi.js`, `home/v2/play.js`, `page_v2/rooms.rs`.
  - **(a) WiFi.**
    - `ROOM: [reading, read, unread, turning]`. It exports the foot (`h:202`, `ul.wifi`) and `WifiPart` for "Open to this WiFi" (`h:382`, `ul#room`), both drawn from one read of `/v1/room` (`h:377`).
    - **Red:** `b:main.rs:3392` (two runs), `:3467`, `:3325`.
    - **Fixes, each its own commit:** both lists from `ROOM`, where today a press in the foot leaves `#room` stale, and the copy said once.
  - **Where (a) landed** (corrected, page-home-rooms (a), eb251576..e781274e):
    - Nothing on v2 draws "What acts as you" yet: `home/v2/acts.js` is page-acts's, a stub (`acts.js:3`), and page-acts is parked. So `#acts #room li` cannot be read on v2 in this row, the same latent flaw CHARTER's "What the merge moved" closed for the shelf. This row asserts what it draws; drawing `part` is page-acts's (its (c) names `WifiPart` and `PlayPart`).
    - The v2-only test is **opened in one group's foot, said open in another's**: ada in house and flat, each foot listing her shelf, one ROOM feeding both, `loads == 1` (`page_v2/rooms.rs`). `part` is held without a browser by `web/tests/v2/rooms.test.mjs`: ROOM's steps, and "Open to this WiFi" listing what is open and what a press here closed.
    - `part` is a hook acts.js calls on each draw, unconditionally; `rooms(s)` its pure view.
    - ROOM's first read starts as wifi.js loads, as v1 reads the room before drawing a group (`h:377`): asked by the first foot, a run read the foot before the room answered.
    - The v1 tests under the switch go as far as other rows' places: `b:main.rs:3403` (the group's "on a screen" link, page-members's `show`), `:3495` and `:3536` (`#acts`).
    - **Lines:** wifi.js 26, v2 at 653, +25 against +18 to +26.
  - **(b) Play.**
    - `PLAY: [reading, read, unread, turning]`, with the clipboard at the edge. It exports the index's cell and `PlayPart`.
    - **Red:** `b:play.rs:57-111`, `ports.rs:84`, `:183`.
    - **Fix, its own commit:** one region and no reload, where the cell (`h:91`) and the acts Stop (`h:384`) tell each other nothing.
  - **Where (b) landed** (corrected, page-home-rooms (b), 565c6e05..7c2e2273):
    - The move (565c6e05) kept v1's faults: a `TURN` per cell and per Stop. The fix (d1704786) gives every turn to `PLAY`, `turning` in its region, what a press copied kept per lend (`copied`, key:name) and each refusal appended to its lend's (`said`); `TURN` is deleted.
    - As for (a), `#acts` is not drawn on v2 (page-acts parked), so the v2-only test is **Play in one group's index, said on in the other's**: cy in ada's house and flat lends `played`, a UDP port; Play in house's cell, flat's says "Connect to"; Stop in flat's, house's offers Play; `loads == 1` (`page_v2/rooms.rs`). `plays(s)`, the Stops' pure view, and PLAY's steps are held without a browser by `web/tests/v2/play.test.mjs`.
    - The v1 tests under the switch go as far as `#acts`: `b:play.rs`'s first run, and `ports.rs`'s two Plays, pass; its second run and `ports.rs:183` wait on page-acts.
    - **Lines:** play.js 22, v2 at 671, +18 for (b), the row +43 against +32 to +46.
  - **v2-only:** each red with `UI` flipped:
    - opened in the foot, shown in `#acts #room li` in the same page, `loads == 1`;
    - Play and its Stop in one page, `loads == 1`.
  - **Byte for byte:**
    - "Open it to this WiFi", "Close it to this WiFi", the `on.said` words, "Closed to this WiFi: …", `table.index ~ ul.wifi li`, `#room`, and "Copied: <origin>. Paste it…";
    - "Play", "Stop", "Connect to …", " Copied.", "Open on this computer: played (cy’s), while Play is on. Stop", and the clipboard's text.
  - **Moves:** the raw-port scene's Play on v2. +32 to +46 [about 19].
- [ ] page-parity — depends [page-shelf, page-index, page-members, page-join, page-acts, page-home-rooms] — ONE ROW, the full gate on v2, and the operator's iPhone run prepared:
  - **Owns:** check.sh's `parity` step and `demo/runs/page/`.
  - **(a) Every v1 test on v2.**
    - `scripts/parity.sh` reports every v1 test passed, or could-not-judge where check.sh allows it. Nothing is failed or never-ran.
    - Each test in the baseline's "passes with no v2" is named against the row that since earned it.
  - **(b) The whole gate on v2.**
    - `SVRNGS_UI=v2 lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh` exits 0, with `page v2` green.
    - `git diff 46ba8288 -- crates/*/tests web/tests` lists only `home.mjs`, `tabs.mjs`, `page_v2/`, `web/tests/*.test.mjs` and `web/tests/v2/`.
  - **(c) The record,** `demo/runs/page/<date>-parity/`: `verdicts.tsv`, each step's verdict on v1 and on v2 side by side, the commands and the commit. One run is not a rate (AGENTS.md 7), so a second is recorded, on another day or on the Fedora box's image (`scripts/gate/README.md`).
  - **(d) check.sh gains `parity`,** held until the switchover so that v2 cannot slip back (+1 `.`).
  - **(e) Leftovers:** `grep -rn "not built yet" web/v2 crates/svrngs-node/src/home/v2` finds none.
  - **(f) The count beside the bar.** v2's lines that stay are set against v1's 703 deleted (CHARTER, "Estimates"), and the switchover's pass is sized here if they are over.
  - **(g) The iPhone run, prepared, never run by the loop.**
    - `demo/runs/page/<date>-iphone/PREDICTIONS.md` is committed before any run.
    - The demo, on a v2 build served from this Mac: bring Events; the answer beside the button; a second tap refused; Events in the list with no reload. On the computer, the bring in the index with no reload.
    - Each bar has its chance written beside it.
    - `NEEDS_HUMAN.md` gives the operator's steps, then stop. The results are told to the watching session, which writes `README.md` there.
  - **Plant:** a v1 test left failing on v2 makes `parity.sh` exit 1, naming it, and the `parity` step fail.
- [ ] HUMAN-page-switchover — depends [page-parity] — THE OPERATOR:
  - reads the parity record and the iPhone run;
  - if they choose, uses v2 on their own computer (a test build's node with `--ui v2`) and on their iPhone (`node web/tests/serve.mjs` over `target/web/site-v2`);
  - says go, or names what is owed.

  The switchover's raise, if page-parity's (f) found one, is set here or after the pass.
- [ ] page-switchover — depends [HUMAN-page-switchover] — ONE ROW, the swap in ONE commit so no tree holds half, any economizing commits before it, and `scripts/check.sh` once, at the close — v2 becomes the pages, and v1 goes:
  - **(a) The pass first,** whenever page-parity's (f) is over.
    - Write `demo/runs/economize/<date>-page-switchover/SURVEY.md`.
    - Its first candidates are the primitives the feature rows wrote apart (PROMPT fence 8), and the near-duplicates across the two pages: `~/dev/commonwealth-ai/target/debug/sovereign-cli code dry-report --scope <dir>` for `web/v2` and `crates/svrngs-node/src/home`. Each is folded into `ui/parts.js` with its case in `parts.test.mjs`.
    - The cuts are in v2 only, each its own commit with its behavior kept, with `parity.sh` green after each batch.
    - Past the bar after the pass, write `NEEDS_HUMAN.md` with the raise, naming what the pass cut and why the rest could not be found, and stop.
    - **Plant:** a fold that changes a drawn word turns `parts.test.mjs` red.
  - **(b) The swap, one commit.**
    - **The phone page:** `web/v2/app.js` becomes `web/app.js`, and `web/v2/<flow>.js` move to `web/page/`. `body.html`'s contents become `web/index.html`'s body, and `git diff -U0` touches no line in `<head>`.
    - **The home page:** `home/v2/index.html` and `home.js` take v1's paths, and `home/v2/<flow>.js` move to `home/page/`, served at `/page/`.
    - **Deleted:** v1's `app.js`, `home.js` and bodies, and question.js's DOM builders (`visiting`, `uses`, `el`, `keeping`, `people`, `door`, `table`, `changes`, `acts`, `row`). Its words stay.
    - **The build:** `build-web.sh` takes the one UI, with `web/v2/build.sh` folded in, `SHELL` still the build's, `sw.js`'s hand list gone, and 404.html kept.
    - **home.rs** serves the files through one `macro_rules!` over their names on (a)'s table. `serve.rs` is deleted.
  - **(c) The switch removed, in the same commit:**
    - `SVRNGS_UI`, `Ui`, `ui()` and `site_of`;
    - `--ui` in cli.rs and build-web.sh, and the builder method;
    - the markers, and build-release.sh's refusal of them;
    - `window.__ui`, and v1's trace sink;
    - `v1.test.mjs`, `policy.test.mjs`, and `release.test.mjs`'s `--ui` case. Its `test-support` case stays: it holds of every release;
    - `parity.sh` and its step;
    - `page_v2`'s `const UI`. The target becomes `tests/pages/`, and its step `pages`;
    - the rung, from `BUDGET`.
  - **(d) The tests default to the only UI**, every one unchanged by name.
  - **(e) The documents, rewritten in place** (AGENTS.md 3):
    - DESIGN §5's `svrngs-web` row (`docs/DESIGN.md:1203`);
    - PAGE_STATE §3's rules, each with its gate;
    - §3's home page no longer reloading;
    - page-switch's paragraph removed;
    - `ARCHITECTURE.md:482` and `:734`, and PLAN §1;
    - a closing line in svrn-docs `PAGE_STATE.md`.
  - **(f) Lines re-measured.** `.` is at or under its cap, 19,481 or the operator's raise, and the cap is lowered to one past where it lands.
  - **Plant:** `web/tests/one-ui.test.mjs` fails on `SVRNGS_UI`, `svrngs-ui-v2`, `web/v2/`, `home/v2/`, `--ui` or `page_v2` left in the tree. It is watched red with harness.rs's `ui()` left in, then green.
  - **Moves:** the live pages, at the operator's deploy and release.
- [ ] HUMAN-page-deploy — depends [page-switchover] — THE OPERATOR:
  - deploys the page and cuts a release from the switchover's commit (a detached worktree, `npm ci --prefix web`, `target/debug` dropped, `CARGO_INCREMENTAL=0 scripts/deploy-page.sh`);
  - reads it back: the page's version, the `/page/` routes from a released binary, and `svrngs-ui-v2` in no tarball;
  - plays the house demo once on their iPhone against the live page.
