# The economizing pass before the page switchover, 2026-11-01

page-switchover (a) (`ralph/next/page/STATE.md`). page-parity (f) put `.`
at **19,486 to 19,514 after the swap**, 364 to 392 over its cap of 19,122
(`../../page/2026-11-01-parity/README.md`, "The count beside the bar").
So the pass comes first, in v2 only, each cut its own commit with its
behavior kept, and `parity.sh` green after the batch. v1 goes whole in
the swap, so none of it is cut here.

## What the instruments found

- **The clone report: nothing to cut.** A corpus of HEAD's `web/v2`,
  `web/vendor` and `crates/svrngs-node/src/home` (`git archive e3fc4f62`,
  `sovereign-cli code index . --corpus-id ps-e3fc4f62`, 291 chunks in 64 s,
  `data/index.out`). Then `sovereign-cli code dry-report --corpus-id
  ps-e3fc4f62 --scope <dir> --threshold <t>` for `web/v2` and
  `crates/svrngs-node/src/home`, at 0.95 and 0.90 (`data/dry-*.md`).
  - The exact tier could not judge: "no SCIP function symbols in scope",
    as in every pass over JavaScript before (I4).
  - Every near cluster is one span inside its neighbor in one file:
    `LEAVE`/`Leave` (leave.js), `LINK`/`said`/`Links` (links.js),
    `Found`/`Running` (running.js), `Group`/`Read` (group.js),
    `stay`/`visiting` (visit.js), and v1's `running`/`look` (home.js,
    deleted by the swap). None is two copies of one thing.
- **Imports: 185 lines** of v2's 1,076 that stay, as page-parity (f)
  counted. The flows were split by owner so rows could run in parallel
  (CHARTER, "Who owns each file"), and each named every shared module it
  read from.
- **POST helpers on the home page:** `door.js` and `running.js` each
  exported the same `post`, and `shelf.js` wrote it again inline.
- **Names defined more than once** (`grep` of each flow's top-level
  `const` and `function`):
  - `titled` on the phone, the same in among.js, enter.js and door.js.
    Two of the three sit on a line with another name, so folding it into
    `ui/parts.js` cuts 1 and adds 1: net 0, not done.
  - `trace` and `wasm`, each `globalThis.__trace?.(l)` or one dynamic
    import, in index.js, uses.js, enter.js, acts.js and visit.js. Each
    shares its line with another name, so there is nothing to cut.
    `trace` is local because ui/trace.js sets `window.__lines` as it loads,
    which node, running web/tests, does not have.
- **The phone's dialled POST** (join.js `posted`, enter.js `through`,
  uses.js `door`, visit.js `visiting`): each encodes a JSON body with its
  `content-length`, fetches through the dial, and closes it. One helper
  cuts about 3 lines and adds 2, since join.js would import enter.js for
  it. visit.js reads its answer after the close, where the other three read
  it before, so it would not fold without a change in behavior. Not done:
  net -1.
- **Duplicated primitives** that fence 8 put in each row's own files: none
  found beyond the above. No flow draws its own `Act`, `Said`, `Fold`,
  `Field`, `Row`, `Keep` or `Heading`.

## Cut, behavior kept

| Commit | Where | What | Lines |
|---|---|---|---:|
| 48cabcea | `home/ui/kit.js`, 31 flows | the core, the parts and question.js's words imported from one module, in one line per flow | -65 |
| 134e7dc2 | `home/v2/group.js`, 12 flows | one `post`, beside `get`; shelf's `bring` through it | -6 |

**-71 in all**: v2's rung goes from 1,192 to 1,121 (`lean_check.py`).
Imports go from 185 lines to 104. What stays is a flow's imports from its
own page's flows, and the phone's `store.js`, which are each by name.

Behavior was watched kept in three ways:
- `node --test` over `web/tests/*.test.mjs`: 78 passed, 0 failed, after
  each commit;
- every name a flow imports from the kit is one it exports
  (`target/ps-econ/kitcheck.mjs`: "kit names 56 missing 0");
- `parity.sh` on v2 after the batch, at 134e7dc2 (`CARGO_INCREMENTAL=0
  bash scripts/parity.sh`, which takes the gate lock itself; exit 0):

      parity: 142 passed (39 of them passed with no v2, in the baseline), 0 failed, 0 could-not-judge, 0 never-ran

  The same counts as page-parity's run at 668dbb66. Its `verdicts.tsv` is
  `data/parity-v2.tsv`.

No primitive was folded into `ui/parts.js`, so `parts.test.mjs` has no new
case, and the row's plant (a fold that changes a drawn word turns
`parts.test.mjs` red) has nothing to watch.

## Where `.` lands after the swap

The swap's sum, with page-parity (f)'s terms, at 134e7dc2:
- `.` now: **19,121**.
- **S, v2's lines that stay: 1,008 to 1,011.** That is the rung's 1,121,
  less `scripts/parity.sh` (37), `home/v2/serve.rs` (62) and
  `web/v2/build.sh` (19), plus the 5 to 8 lines of build.sh that fold into
  build-web.sh. Counted by lean_check's rule
  (`target/ps-econ/count.py`).
- **T, home.rs's table of the pages' files:** +8 to +30, as before. It is
  one `macro_rules!` over 32 names now, with kit.js.
- **H, the both-live hooks the swap deletes:** 13 to 16.
- **D, v1 deleted:** 703 (app.js 412, home.js 231, question.js's DOM
  builders 59, sw.js's hand `SHELL` 1).

So `.` after = 19,121 − H + S + T − 703 = **19,418 to 19,446**. That is
**296 to 324 over** the cap of 19,122.

## Why the rest could not be found

- **The fixed cost:** `ui/` (the core, the parts and the shared parts,
  now 175 lines with kit.js) and the vendored libraries (3). v1 had none of
  it.
- **The home page's regions:** `home/v2/`'s modules are 374 counted lines
  (serve.rs aside), against v1's home.js at 231. Each redraw that v1 did by loading
  the page again is a region with its states, its reads counted by `ask`,
  and its refusals (PAGE_STATE §3). That is the queue's purpose, and
  CHARTER's estimate named it.
- **What a further fold would cost:**
  - A second hub per page that re-exports its own flows would cut about
    10 more. But it is one more indirection, with its cycles to keep out
    by hand.
  - The phone's frame could import its 11 flows dynamically, as home.js
    does, for about -9. That changes when they load on WebKit, where a
    module's loading has wedged the tab before (web/app.js:659-662), so it
    is a change in behavior and not a pass's.

So the raise is the operator's: `NEEDS_HUMAN.md` in `ralph/next/page/`.
