# page-scaffold: the `v2` rung is past the row's range

**The question.** page-scaffold's `v2` rung lands at +250. Its range is +149
to +208 (ralph/next/page/STATE.md:225, CHARTER.md:243). Do you widen the
range to +250 so the row can go on to (g) and its gate? Or do you name a
cut?

The rung's cap, 1,150, is not in question: v2 is 250 of 1,150.

## Where it lands

`python3 scripts/lean_check.py` at b6d3f3e2:

    crates/svrngs-core     3241 / 3242 lines
    .                     19118 / 19122 lines
    v2                      250 / 1150 lines

Each part against the CHARTER's estimate (each file counted as
`lean_check.py` `code_lines` counts it):

| Part | Files | Lines | Estimate |
|---|---|---|---|
| vendor | web/vendor/{preact,hooks,htm}.js | 3 | +3 to +6 |
| machine | ui/machine.js | 23 | +18 to +25 |
| primitives | ui/parts.js | 21 | +30 to +45 |
| view, trace, build, serve | ui/view.js 17, ui/trace.js 8, web/v2/build.sh 19, home/v2/serve.rs 62, home/v2/home.js 5 | 111 | +68 to +92 |
| phone frame | web/v2/app.js 23, page.js 43, eleven stubs 26 | 92 | +30 to +40 |
| **total** | | **250** | **+149 to +208** |

- The phone frame is +52 over its top. 0d9e1c95's body says why: `compose`
  holds the three refusals, the heading's order, the responses' join and the
  acts' merge.
- serve and home are +19 over theirs.
- (g) adds no `v2` lines: its checks are `.test.mjs` and `tests/home.rs`,
  which are free, and build-release.sh's refusal counts on `.`.

## What was tried

- **page.js and app.js, which the last session's note named first.** There
  is nothing to cut that keeps what they do. page.js's 43 lines are `compose`
  and `Page`, and each of `compose`'s statements is one of the refusals or
  orders the row asks for. app.js's 23 lines are its 17 imports and the edge
  the row lists.
- **serve.rs, made lean** (7156bbd2): its 404 in one statement (−2), and one
  matcher for the paths. What is left:

| Lines | What |
|---|---|
| 13 | `Ui::asked`: `--ui v1\|v2` taken out of the binary's arguments |
| 7 | `Node::ui`, the builder method |
| 4 | the `Ui` enum |
| 1 | the `OnceLock` choice |
| 4 | `use` lines |
| 27 | the layer: which file, the trace, pass on, the content type, read or 404 |
| 6 | `named`: the `^[a-z0-9-]+\.(js\|css\|html)$` check |

- **A cut that would change a premise, not done:** let the binary read its
  choice from `SVRNGS_UI` (the variable page-switch's `ui()` reads) in place
  of `--ui v2`.
  - It drops `Ui::asked`, −12 `v2` lines, and costs `.` nothing.
  - It lands at about +238, still past +208.
  - The row says `--ui v2` (STATE.md:205, CHARTER "What is settled" 5).

## The ready diff, if you widen the range

```diff
--- a/ralph/next/page/STATE.md
+++ b/ralph/next/page/STATE.md
@@ -225 +225 @@
-  - **Moves:** `page laws`, and both v2 pages served empty. +149 to +208 (rung), +9 to +11 (`.`), against (a)'s cuts [`a:47-117`, about 40].
+  - **Moves:** `page laws`, and both v2 pages served empty. +149 to +250 (rung; the operator widened it from +208, 2026-10-31: the phone frame 92 against +30 to +40, serve 62; demo/runs/page/2026-10-31-scaffold-range/NEEDS_HUMAN.md), +9 to +11 (`.`), against (a)'s cuts [`a:47-117`, about 40].
```

**What it costs the queue.** The rung's estimate summed to +815 to +1,112,
with 38 lines of slack under 1,150 (CHARTER, "Estimates"). Landing 42 over
this row's top leaves the later rows −4 against their summed tops. So they
must land at least 4 under their tops in all, or the cap is raised at the
switchover's economizing pass.

## Where the row stands

- (a) to (f) are committed, b50592ca..b6d3f3e2.
- Next is (g), the guards, then `scripts/check.sh` once, under the gate lock.
- `.` has 4 lines left. (g)'s refusal takes 1, and page-switch's `page v2`
  step takes 1.

## The director's check, 2026-10-31

**Reproduced.** At 174038cf, `python3 scripts/lean_check.py` prints
`v2 250 / 1150`. The tree is clean. Counting each file under lean_check's
`V2` paths (`scripts/lean_check.py:358`) by its rule (`:384-387`) gives the
same parts:
- vendor 3;
- machine 23 and parts 21;
- view 17, trace 8, build.sh 19, serve.rs 62 and home.js 5;
- app.js 23, page.js 43 and the eleven stubs 26.

The total is 250.

**Why it is yours.** CHARTER "Leave for the operator" names "a row past
its range" under the budget. So the director does not widen the range,
and does not cut a settled premise (`--ui v2`, settled 5) to get under
it. The `SVRNGS_UI` cut would land at about +238, still past +208.

**The recommendation: widen to +250 (the diff above).**
- The cap, 1,150, does not move.
- page.js's 43 lines are the refusals and orders that settled 4 and the
  row ask for (read at `web/v2/page.js`). I found none of them redundant.
- The cost is 4 lines against the later rows' summed tops. The rows that
  aim below their range (CHARTER, "Estimates": each home row) carry that,
  or the switchover's pass does.

**What would make this wrong:** a later row landing past its own top for
the same reason this one did, which is a frame the estimates never sized.
In that case, the rung's estimates should be resized as a whole, not
widened row by row.
