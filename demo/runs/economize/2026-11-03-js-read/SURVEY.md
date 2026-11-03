# The survey before the pages are read, 2026-11-03

js-read (a) (`ralph/next/js/STATE.md`). It lists what (b) to (c) change,
at HEAD 4b7160c0, over the pages' code (`ralph/next/js/CHARTER.md`,
"Scope"), each with the cut or move it takes. Nothing here changes code.

## How it was measured

- `python3 -I demo/runs/economize/2026-11-03-js-read/survey.py` writes
  `data/survey.out`: each file's templates and lines over 100, each view
  and whether it is a named component, comments with few words once their
  citations are out, and runs of four normalized lines found twice.
- `python3 -I demo/runs/economize/2026-11-03-js-read/survey.py more`
  writes `data/more.out`: each called-where-written arrow (an IIFE), each
  local written as a parameter's default, each comment line citing a row's
  letter, a `file@commit` or v1, and each top-level name defined in two
  files.
- `npx --prefix web oxlint -c demo/runs/economize/2026-11-03-js-read/data/sequences.oxlintrc.json --format unix web/*.js web/page/*.js crates/svrngs-node/src/home`
  writes `data/sequences.txt`: the comma sequences the gate's
  `no-sequences` lets through because they sit in parentheses. 15 of them.
- `data/table.md` is all of it, per file.

`sovereign-cli code dry-report` **never ran.** No build on this Mac has it:
`~/dev/commonwealth-ai/dist/sovereign-aarch64-apple-darwin/sovereign-cli
code dry-report --help` answers "not available in this build", and the
same for `~/dev/wt-rel070/dist/`. The developer build the last passes
ran, `$SOVEREIGN` in `~/.zshrc`
(`~/dev/commonwealth-ai/sovereign/target/release/sovereign-cli`), is gone
("No such file or directory"). No `sovereign-cli-dev` binary is anywhere
`mdfind` finds. Building one in commonwealth-ai is outside this
lane's `target/` (fence 11). Its last run over these pages could not judge
the exact tier either ("no SCIP function symbols in scope",
`../2026-11-01-page-switchover/SURVEY.md`). The line-window and name
lists above stand in for it. A near clone that differs by its names is not
caught by either. Each part's session reads its flow whole, and folds what
it finds.

## Lines before

`python3 scripts/lean_check.py` at 4b7160c0: **`js` 6,535**, no cap until
the jubilee; **`.` 18,861** of 18,862. This part adds only files under
`demo/runs/`, so both stay.

## What the pages hold

Over 55 files and 12,177 lines (`wc -l`):

| What | Count | Where |
|---|---:|---|
| Views packed into an expression: a `view` entry that is a template, or a lowercase helper answering one | 51 | `data/table.md`, column 6 |
| Named components, `function X` or `const X = (…) =>` | 38 | column 7 |
| Modules holding a machine beside its views | 28 | column 3 |
| Lines over 100 characters, mostly templates the formatter leaves whole (`embeddedLanguageFormatting: "off"`) | 214 | column 5 |
| Arrows called where they are written, `((a, b = f(a)) => …)(x)` | 27 | `data/more.out`, `iife` |
| Locals written as a parameter's default, `(state, you = state.people.find(…)) =>` | 53, by a heuristic each part confirms | `defaults` |
| Comma sequences in parentheses, `(trace(…), location.pathname)` | 15 | `data/sequences.txt` |
| Comment lines citing a row's letter, a `file@commit` or v1 | 236 | `cites` |

The worst, by reading:
- `home/home.js:258` `Home` draws the whole page in one template of about
  600 characters, five conditions in it.
- `home/page/wifi.js` `Line` and `Copy` and `home/page/play.js` `Turn` are
  each an IIFE whose parameters are locals, `((state, open = on(state,
  lend)) => html`…`)(useStore(room))`.
- `web/page/enter.js` holds the phone's dial, its tab locks and its open
  flow in 709 lines, with 6 of the 15 sequences, `(answer(…), new
  Promise(…))` among them.

## The shape each part moves to

CHARTER's "The shape the code takes", held to what the tests read. Fence 3
leaves every test unchanged, so a module a law imports keeps every name it
imports from it, where it is:

| Law | Imports |
|---|---|
| `web/tests/pages/frame.test.mjs`, `enter.test.mjs`, `acts.test.mjs` | `page.js` `compose`, `ORDER`; each of the 11 phone flows by name, whole, composed |
| `web/tests/pages/home-join.test.mjs` | `home/page/join.js` `JOIN`, `take` |
| `web/tests/pages/home-make.test.mjs` | `home/page/make.js` `MAKE`, `take` |
| `web/tests/pages/play.test.mjs` | `home/page/play.js` `PLAY`, `plays` |
| `web/tests/pages/rooms.test.mjs` | `home/page/wifi.js` `ROOM`, `rooms` |
| `web/tests/pages/members.test.mjs` | `home/ui/door.js` `Door`, `home/ui/visitors.js` `Visiting` |
| `web/tests/pages/shelf.test.mjs` | `home/ui/shelf.js` `SHELF`, `drawn` |
| `web/tests/pages/acts.test.mjs` | `home/ui/acts.js` `Acts`, `ui/changes.js` `Changes`, `ui/uses.js` `Uses` |
| `web/tests/copy.test.mjs` | `home/ui/copy.js` `COPY`, `drawn` |
| `web/tests/pages/frame.test.mjs` | `home/ui/copy.js` `Copy`, `drawn`; `home/ui/machine.js` `machine`, `go` |
| `web/tests/parts.test.mjs` | `home/ui/parts.js` `Act`, `Field`, `Fold`, `Heading`, `Keep`, `Row`, `Said` |
| `web/tests/machine.test.mjs`, `copy.test.mjs` | `home/ui/machine.js`, by name |
| `web/tests/fallback.test.mjs` | `home/page/group.js` `Group` |

Where a file is served from also binds it:
- **The phone** loads `web/page/*.js` flat: `scripts/build-web.sh` copies
  `$APP/page/*.js`, and `web/sw.js`'s SHELL is built from that glob, so a
  new file there is served and kept offline with no edit. No
  subdirectory.
- **The home page** serves only the names in `home.rs` `served`
  (`crates/svrngs-node/src/home.rs:136-137`), and
  `tests/home.rs` `the_home_page_s_modules_are_served_each_as_its_file_and_no_other`
  fails on a file it leaves out. Each new file adds its name there, on the
  line that lists its directory, so `.` does not move.
- **`import/no-cycle` is on.** A component's file never imports the
  module that imports it for its `view` table.

So each part moves its flow this way.

**A phone flow, `web/page/<flow>.js`,** stays the module `compose` reads,
since the laws compose them by name:
- the machine: its typedefs, `owns`, `states`, `takes`, `heading`,
  `response`;
- its `io` table last, apart from the steps, each entry calling the edge
  module below, never a step;
- `view`, a table naming components, each imported from its own file.

A component's file, `web/page/<what-it-draws>.js`, holds one component and
its props' `@typedef`. It imports `ui/`, `question.js` and pure helpers,
never its flow.

**A home feature, `home/page/<feature>.js`,** stays what `home.js` reads
by `FILES` and the laws import:
- its machine, its typedefs, `step` a `switch`, and `io`;
- `take` and `holds`;
- every name a law imports from it.

A component that runs the feature's machine imports the feature, so the
feature cannot import it back. Its file exports the component and
`view = { <place>: Component }`, and is named in `home.js` `FILES`, which
reads every module's `view` already (`home.js:278`,
`Object.assign({ left }, ...mods.map((flow) => flow.view))`). A
component no machine runs needs nothing of the kind.

**The kept exceptions.** `plays` draws `Turn`, and `rooms` draws `Line`,
each reading its module's store and helpers. `drawn` is the same in
`ui/copy.js` and `ui/shelf.js`. A law pins each where it is. Moving the
component out would make it import back the module that draws it. So
each part either moves the pure helpers both read into a module both
import, and the component with them, or keeps the component beside its
machine and says which law keeps it. `ui/parts.js` keeps its seven parts
for `parts.test.mjs`.

**Templates** are broken up by hand: one element or one condition per
line, and each condition named by a component or a `const` before the
`return`. No IIFE, no local as a default, no sequence. What a person
reads, its words, ids, classes and element kinds, stays byte for byte, and
so do the spaces between elements: `' Copied.'`, `${' '}`.

**Comments.** Each of the 236 citing lines loses its row's letter,
`file@commit` and v1. v1 is gone since the switchover (`one-ui.test.mjs`),
so a reader cannot check "as v1 drew it". Where the citation was the why
("v1 drew it after", "a driver's `text` waits on it"), the comment says
the reason itself: what a person sees, or which test reads it, by its
path. A comment that only says what the next line does goes. The 23 short
comments in `data/survey.out` are each read with their line. Most say why
a cast holds, "`split` answers one part at least", and stay.

## Duplicates, and the cut each takes

| What | Where | Cut |
|---|---|---|
| `trace`, the trace sink as a flow can reach it in node | `web/page/among.js:151`, `enter.js:478`, `hand.js:40`, `join.js:241`, `uses.js:138` | One, in the phone's edge module (below). It stays local to `web/page/` and does not come from `ui/trace.js`, which writes `window` as it loads, so the laws could not import it. |
| `wasm`, the runtime's one dynamic import | `acts.js:266`, `among.js:150`, `enter.js:477`, `hand.js:41`, `join.js:240`, `visit.js:152` | One, in the edge module. |
| `PATIENCE = 60_000`, how long a dial waits | `enter.js:475`, `join.js:238` | One, in the edge module: one decider (AGENTS.md 8). |
| The dial (`through` and its POST, `content-length`, close) | `enter.js` around :475-700; used by `shelf.js`, `acts.js`, `uses.js`, `visit.js` | Moves out of `enter.js` into `web/page/dial.js`, the phone's edge module, with `trace`, `wasm` and `PATIENCE`. Each flow's `io` calls it. `visit.js` reads its answer after the close where the others read it before (`../2026-11-01-page-switchover/SURVEY.md`), so its call stays its own. |
| `sending`, a `Sender` | `among.js:50`, `enter.js:135`, `join.js:74` | One, exported by `page.js` beside the `Sender` type it makes. No cycle: `page.js` imports no flow at run time. |
| `titled`, a group's name with whose it is | `among.js:48`, `door.js:43`, `enter.js:139` | One, in `question.js` beside `whose`, which it calls. |
| `hex` | `web/authority.js:167`, `web/sign.js:23`, `web/store.js:389` (exported) | `store.hex` for all three. Both `sign.js` and `authority.js` import from `store.js` already. `sign.js`'s takes an `ArrayBuffer`, so its caller wraps it in `Uint8Array`. |
| `pkcs8` | `web/sign.js:29`, `web/open.js:62` | One, in `authority.js`, which both import, if the bytes are the same. `sign.js` builds them from a seed, `open.js` from hex, so the part compares them on one key before folding them. |
| `decode`, base64url JSON | `web/sign.js:21`, `web/open.js:49` | **Not the same.** `open.js` decodes UTF-8 (`TextDecoder`), and `sign.js` takes `atob`'s Latin-1 as it is. Folding them would change what a non-ASCII field reads as on `sign.html`. That is a change someone could see, so it is not this queue's. Kept, and named in `NEEDS_HUMAN.md` at (b)'s close if a field there can carry non-ASCII. |
| PLAY and ROOM: `answer`, `same`, `keyOf`, `on`, and the `read`/`turn` step | `home/page/play.js:78-88`, `:120-135`, `:182`; `home/page/wifi.js:92-111`, `:130-145`, `:192` (the line-window match, `play.js:126 = wifi.js:136`) | `same` and `keyOf` are the same on both: one, in a module both import. The steps differ in their words, regions and what a turn answers, so they are folded only if one machine with those as parameters reads better and costs fewer lines. (c) judges it in play's commit. |
| `Copy` on the home page | `home/page/links.js:106` (its last answer only), `home/page/wifi.js:247` (with `Keep`), `home/ui/copy.js:74` | Each draws `ui/copy.js`'s `drawn` differently, so they stay three. Each becomes a component of its own file, with no IIFE. |
| `get`/`post`, the home page's I/O | `home/page/group.js:134`, `:154`; `leave.js:154` writes its own `fetch` | One edge module for the home page, which `group.js` and `leave.js` import. `leave.js`'s `fetch` reads its answer's status itself, so it moves as it is, or onto `post` if `post` answers the same. |

The names that are defined twice but do two things (`answered`, `FIRST`,
`said`, `reading`, `PLACES`, `on` in `ui/shelf.js`) are listed in
`data/more.out` and are not duplicates.

## Each part, in order

All of (b) and (c) keep fences 1 to 8. Each commit reports `js` and `.`
before and after it.

**(b) The phone page, one commit per flow:**
1. `web/page/dial.js`: the edge module. It takes the dial out of
   `enter.js`, then `trace`, `wasm`, `PATIENCE`, `sending` and `titled`
   folded as above. That makes every later flow's commit smaller.
2. `shelf.js`, then `door.js`: four packed views, `visitors` and `Ends`.
3. `computer.js`, `index.js`, `ask.js`, `uses.js`: one or two views each.
4. `visit.js`, `among.js`, `join.js`: `NameForm` and `listed`, and the
   sequences.
5. `acts.js`, then `enter.js`: `chosen` becomes the tabs' component, and
   enter's 18 long lines and 3 IIFEs go.
6. `page.js` `Page` and `compose`, and `app.js`.
7. `web/*.js` that draws nothing, one commit each: `store.js`,
   `authority.js` with `sign.js` and `open.js` (the `hex` and `pkcs8`
   folds), `shim.js`, `sw.js`, then `lend.js`, which is 1,665 lines with
   24 over 100.

**(c) The home page, one commit per feature:**
1. The edge module, `get`/`post`.
2. The leaves, whose components no machine runs: `shelf`, `door`,
   `visits`, `index`, `open`, `screen`, `running`, `lending`.
3. The features whose component runs their machine: `ask`, `join`,
   `make`, `links`, `leave`, `acts`, each component to a file named in
   `FILES` and `served`.
4. `play` and `wifi`, with the PLAY/ROOM fold judged, and the kept
   exceptions.
5. `group.js` (`Group`, `below`, `Read`), then `home.js` (`Home`, `left`,
   `unread`, the sequence at :292).
6. `ui/`: `door.js`, `changes.js`, `copy.js`, `shelf.js`, `acts.js`,
   `visitors.js`, `uses.js`, and `parts.js`'s templates broken up, none of
   them moving, since each is a law's import. Then `question.js` and
   `session*.js`, comments and lines only.

**(d)** writes CHARTER's shape into PAGE_STATE §3 and DESIGN §5, with
this survey's two additions: a law's import stays where it is, and a
component a machine runs is the module that names its place.

**(e)** is the jubilee, after the close's gate.
