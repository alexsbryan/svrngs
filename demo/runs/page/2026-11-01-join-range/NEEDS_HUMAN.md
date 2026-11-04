# page-join: the row is past its range

**The question.** page-join lands at +163 on the `v2` rung after its last
move, and about +165 once its three fixes are in. Its range is +120 to +160
(ralph/next/page/STATE.md, the page-join row's "Moves"; CHARTER.md,
"Estimates"). The charter leaves a row past its range to you. Do you widen
the row to about +165 so it can finish its fixes and close?

The rung's cap, 1,150, is not in question: v2 is 791 of 1,150.

## Where it lands

`python3 scripts/lean_check.py` at 82b021a6 (ralph/page-join):

    crates/svrngs-core     3241 / 3242 lines
    .                     19120 / 19122 lines
    v2                      791 / 1150 lines

At `git merge-base HEAD main` (d8009598, main's tip) v2 is 628, so all +163
is this row's. By file, counted as `lean_check.py` `code_lines` counts:

| Part | File | Base | Now | Lines | Charter's estimate |
|---|---|---|---|---|---|
| phone join | web/v2/join.js | 3 | 55 | +52 | +40 to +50 |
| among, cold, `#open=` | web/v2/among.js | 1 | 38 | +37 | +22 to +28 |
| | web/v2/computer.js, enter.js | 117 | 120 | +3 | |
| home join | home/v2/join.js | 1 | 20 | +19 | +16 to +22 |
| home make | home/v2/make.js | 1 | 19 | +18 | +14 to +20 |
| home links | home/v2/links.js | 1 | 21 | +20 | +20 to +28 |
| home open | home/v2/open.js | 1 | 15 | +14 | +8 to +12 |
| **total** | | | | **+163** | **+120 to +160** |

- The home's four parts, +71, are inside their +58 to +82. The worker's
  note said they were over "about 36 home", but that is the Moves bracket,
  v1's lines deleted, not the estimate.
- The excess is the phone's, +92 against +62 to +78: join.js's NameForm,
  the 409 loop and the device link, and among.js's naming by label, alias,
  `missed`, `nor` and `anew` (7b78b2a2).

## What is left, and its cost

- (c)'s fix, joining with no reload: +1 in join's take.
- (d)'s second fix, making with no reload: +1.
- (e)'s fix, " Copied." said once: +1.
- shelf.js's `JOINED` line goes: −1. This comes with (c). The director
  decided that HOME's `q.joined` goes through home.js and group.js to
  shelf.js, so a group just joined is still folded (`home_group.rs:331-335`).
  That decision is in this row's commit.
- So the row closes at about +165.

## What was tried

The four home files were read whole (`grep -vnE '^\s*(//|$)'`). Each is
already a machine with its view in one or two statements. The only cuts
left would join two lines into one, which games the count without making
the code smaller. None was made.

## The options

1. **Widen the row to +165 (recommended).** The row finishes its three
   fixes and the v2-only checks it promises: `join.rs` joining with
   `loads == 1`, and one " Copied.". Then it runs `scripts/check.sh`.
   - Cost: 5 lines past the range, about 3%, with 359 left under the
     rung's cap.
   - The five rows merged so far (scaffold, switch, open, shelf, index) sit
     at 628. Their summed estimates are +445 to +652, so the rung is on its
     course.
   - The switchover's raise, already expected and already yours, grows by
     about 5.
2. **Close at +163, with the three fixes moved to page-parity or the
   switchover.**
   - This is still past the range, so it needs your yes anyway.
   - It drops the row's "Moves" ("joining and making a group on the
     computer with no reload") and two of its v2-only checks. That weakens
     the row's bar for 2 lines.
3. **Name a cut.** The phone's among.js (+37) is the part furthest over
   its estimate. Any cut there changes what (b) moved, so it would need to
   reopen a part that is already green.

## What a worker does on your yes

Apply (c)'s fix with the prop, as recorded in the row. Then (d)'s second
fix and (e)'s fix. Write the `join.rs` checks `loads == 1` and one
" Copied.", each red with `UI` flipped to `Ui::V1`. Then run
`scripts/check.sh` under the gate lock.
