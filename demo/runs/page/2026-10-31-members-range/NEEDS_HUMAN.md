# page-members: (e), the screen, does not fit the row's range

**The question.** page-members stands at +145 of its range, +110 to +152
(CHARTER.md:248, STATE.md:428), with (a) to (d) and (f) landed. That
leaves 7 lines for (e), a group on a screen, which is v1's 22 counted
lines (`home.js:307-329`). Do you widen the range to +180 so the row can
land (e) and run its closing gate? Or do you name a cut?

The rung's cap, 1,150, is not in question: v2 is 773 of 1,150.

## Where it lands

`python3 scripts/lean_check.py` at e1f03249:

    crates/svrngs-core     3241 / 3242 lines
    .                     19120 / 19122 lines
    v2                      773 / 1150 lines

The row was +122 at 750 (the last session's note, 53c68f39). This
session's (d) added +23 (f1188f42), and its fixes and (f) added none
(b7f773b6, 8e0af161, c0f51b0f, e1f03249). Each of the row's files, counted
as `lean_check.py` `code_lines` counts it:

| Part | File | Lines | Estimate |
|---|---|---|---|
| (a) shared door | ui/door.js | 16 | |
| (a), (b) shared visitors | ui/visitors.js | 3 | |
| (a), (b) phone door | web/v2/door.js | 21 | +22 to +30 |
| (c) a guest's visit | web/v2/visit.js | 37 | +20 to +26 |
| (a), (f) home door | home/v2/door.js | 26 | +20 to +30 |
| (b) visits | home/v2/visits.js | 23 | +12 to +18 |
| (d) leave | home/v2/leave.js | 24 | +18 to +24 |
| (e) screen | home/v2/screen.js | 1 (the stub) | +18 to +24 |

- **(d) cost its estimate's top.** v1's leave is about 9 lines
  (`home.js:40-42`, `:145-153`). v2's is LEAVE: five states, its answers
  each said beside its button, HOME's `left` taken here (`take`) in place of
  v1's reload, and an io that reads an outcome, a 401 and an `error` apart.
- **(e) at v1's count, and v2's overhead on it.** v1's screen is 22 lines.
  The row asks of v2 a SCREEN region (`reading`, `read`, `unread`), `seen`
  and `newest` in state, the poll as an edge timer answering `tick`, and
  the door judged before it takes the last one's place. It also needs the
  page's title and body class set at the edge, not by a view, and the
  newest mark in ui/door.js's window. The estimate is **+24 to +32**, so the
  row lands at **+169 to +177**.

## What was tried

An economizing pass over the row's own files (the last note named
visit.js, door.js, home/v2/door.js and visits.js; leave.js too). It found
no cut that keeps what each does:
- **The three end timers** (web/v2/door.js `io.end`, visit.js `io.over`,
  visits.js `io.wait`) are each one line, `new Promise((r) =>
  setTimeout(r, t * 1000 - Date.now()))` inside its effect. One helper
  leaves each call site a line: 0 lines. It is page-switchover's fold into
  parts.js.
- **leave.js's reader against door.js's `post`.** `post` rejects every
  non-ok answer, but removal.rs's outcomes are 409s that must be read
  (`decides`), so `post` would need a new contract for /visits and
  /folder. That is 0 lines, and it would change their 401's words.
- **LEAVE's `sent`.** Each effect answers one message (PAGE_STATE §3.3),
  so the step that takes HOME's answer stays.
- **No statement was joined to another line** only to lower the count.
- **Fixes landed with no lines.** (d)'s two fixes rewrote lines in place,
  (f)'s folder fix too, and (f)'s `visible` was page-open's already, so
  its commit is a test.

## The ready diff, if you widen the range

```diff
--- a/ralph/next/page/CHARTER.md
+++ b/ralph/next/page/CHARTER.md
@@ -248 +248 @@
-| page-members | +110 to +152 | phone door +22 to +30, visit +20 to +26, home door +20 to +30, visits +12 to +18, leave +18 to +24, screen +18 to +24 |
+| page-members | +110 to +180 (the operator widened it from +152, 2026-10-31: leave 24, screen v1's 22 with its machine; demo/runs/page/2026-10-31-members-range/NEEDS_HUMAN.md) | phone door +22 to +30, visit +20 to +26, home door +20 to +30, visits +12 to +18, leave +18 to +24, screen +24 to +32 |
--- a/ralph/next/page/STATE.md
+++ b/ralph/next/page/STATE.md
@@ -428 +428 @@
-  - **Moves:** scene `life`'s removal, and the guests scenes. +110 to +152 [about 31 phone, `q:43`, `:161-198` 19, about 62 home].
+  - **Moves:** scene `life`'s removal, and the guests scenes. +110 to +180 (the operator widened it from +152, 2026-10-31; demo/runs/page/2026-10-31-members-range/NEEDS_HUMAN.md) [about 31 phone, `q:43`, `:161-198` 19, about 62 home].
```

**What it costs the queue.** page-scaffold landed 42 over its top, which
left the later rows −4 against their summed tops (2026-10-31-scaffold-range).
At +177, page-members lands 25 over its top, so the rows not yet landed
must come in about 29 under their tops in all. If they do not, the cap is
raised at the switchover's economizing pass. The `v2` rung itself is about
805 of 1,150 after (e).

## Where the row stands

- (a), (b), (c), (d), (f): landed, each part and fix its own commit, each
  fix with its v2-only test watched failing on `Ui::V1`
  (target/page-members/red-*.log, green-*.log).
- Seven v2-only tests in `page_v2/members.rs` pass
  (target/page-members/green-f1.log, green-f2.log).
- Still red under `SVRNGS_UI=v2`, past every assertion of this row:
  `leaving_on_the_home_page_is_said_as_a_leave_everywhere` stops at `#join
  input[name=link]`, page-join's stub (target/page-members/green-d4.log).
- Left: (e), then `scripts/check.sh` once, at the close.
