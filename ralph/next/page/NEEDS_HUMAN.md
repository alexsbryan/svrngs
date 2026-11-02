# page-switchover: the raise after (a)'s pass, 2026-11-01

The operator's go (STATE.md, HUMAN-page-switchover) said "the raise is set
after (a)'s pass, from `NEEDS_HUMAN.md`". The pass is done, so this is that
raise. Two questions follow, each with a ready diff.

## What the pass cut, and where `.` lands

The pass is in `demo/runs/economize/2026-11-01-page-switchover/SURVEY.md`.
- **Cut: -71 in v2,** in two commits, each with its behavior kept:
  - 48cabcea: every flow imports the core, the parts and question.js's
    words in one line, from `ui/kit.js` (-65);
  - 134e7dc2: the home page has one `post`, in group.js beside `get` (-6).
  - v2's rung went from 1,192 to 1,121.
- **The clone report found nothing to cut.** Every near cluster is one span
  nested in its neighbor.
- **After the swap, `.` lands at 19,418 to 19,446**, which is 296 to 324
  over the cap of 19,122. The sum:
  - `.` now: 19,121;
  - less the both-live hooks: 13 to 16;
  - plus v2's lines that stay: 1,008 to 1,011;
  - plus home.rs's table of the files: 8 to 30;
  - less v1: 703.
- **Why the rest could not be found** (SURVEY, last section):
  - `ui/`'s fixed cost: 175 lines that v1 never had.
  - The home page's regions: 374 lines against home.js's 231. Every redraw
    v1 did by reloading the page is now a region with its reads and
    refusals (PAGE_STATE §3).
  - The two folds left would cut about 19 lines. One adds a second hub of
    indirection, and the other changes when the phone's flows load on
    WebKit. Neither is a pass's to make.

## 1. The raise (yours, from the operator seat)

Raise `.` to one past the estimate's top, and lower `v2` to one past where
the pass left it (AGENTS.md "Done": a pass's cuts lower the cap). Then the
swap lowers `.` to one past where it really lands (STATE, page-switchover
(f)).

```diff
--- a/scripts/lean_check.py
+++ b/scripts/lean_check.py
@@
 # 2026-11-01-join-range, 2026-11-01-acts-range). v2 671 landed and those
 # three tops come to 1,206, so the cap is one past it.
-BUDGET = {"crates/svrngs-core": 3242, ".": 19122, "swift": 1, "v2": 1207}
+# The operator's raise for the switchover (2026-11-01, after page-switchover
+# (a)'s pass, demo/runs/economize/2026-11-01-page-switchover/SURVEY.md):
+# v2's imports in one line from ui/kit.js and one `post` (-71, v2 at 1,121,
+# so its cap is one past it); `.` to one past the swap's estimate, 19,418 to
+# 19,446, where v2's 1,008 to 1,011 lines that stay are counted again and
+# v1's 703 go. The swap lowers it to one past where it lands.
+BUDGET = {"crates/svrngs-core": 3242, ".": 19447, "swift": 1, "v2": 1122}
```

What would falsify the estimate: the swap lands `.` over 19,446. Then the
swap stops with that number, and nothing over the cap is committed.

## 2. The swap's own BUDGET hunk (your yes, for the worker to make)

The swap has to edit `BUDGET` in its one commit. It deletes the `v2` rung
and the `V2` paths, and lowers `.` to one past where it lands (STATE,
page-switchover (c) and (f)).
- Without that edit, `.` would still skip `web/vendor/` and `home/ui/`,
  which stay after the swap. That leaves 178 lines uncounted, a quiet
  bypass.
- A worker's edit to `BUDGET` is refused as a CI bypass (fence 11). But the
  swap is one commit, so the operator seat cannot land this edit beside it.

The hunk only removes and lowers. It never raises:

```diff
-BUDGET = {"crates/svrngs-core": 3242, ".": 19447, "swift": 1, "v2": 1122}
-V2 = ("web/v2/", "web/vendor/", "crates/svrngs-node/src/home/v2/", "crates/svrngs-node/src/home/ui/", "scripts/parity.sh")
+BUDGET = {"crates/svrngs-core": 3242, ".": <one past where the swap lands, at most 19447>, "swift": 1}
```

`code_lines`' `v2` parameter and its `rel.startswith(V2)` test go with them,
along with the rung's comment, which is rewritten to say that `.` took v2
back.

**The worker's recommendation:** say yes to 2 in STATE's
HUMAN-page-switchover line, in words the swap's session can cite. For
example: "the swap's commit may delete the v2 rung and lower `.`". With
that line, the hunk is an approved edit and not a bypass.

## After your answers

Land 1 from the operator seat, and write 2's answer in STATE. The loop
resumes page-switchover at (b), the swap, in one commit.
