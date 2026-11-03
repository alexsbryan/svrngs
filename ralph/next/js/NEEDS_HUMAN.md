# svrngs js: for the operator

## A refused Stop on the home page is never said beside it (js-types, 2026-11-02)

Found while typing `crates/svrngs-node/src/home/page/acts.js` (6f18511e).
Kept as it runs: saying it is a change a person would see, which this
queue does not make (fence 3).

**What happens.** On the home page's "What apps use", Stop sends
`{ type: 'stop', lend, app }`. ACTS runs the `stop` effect with
`failed: 'unstopped'` (home/page/acts.js:270). When the post fails,
`machine` makes the failed message as `{ type, n, why }` (ui/machine.js:191),
so it carries no `lend` and no `app`. The `unstopped` step files the why
under `` `${m.lend} ${m.app}` `` (home/page/acts.js:274), which is the key
`"undefined undefined"`. `Uses` reads `said[`${lend} ${app}`]` beside each
Stop (ui/uses.js), so the refusal is never drawn. It is traced to
`window.__lines`, but no person sees it.

**The phone does it right.** Its io answers its own refusal with
`{ type: 'unstopped', of: `${l.label} ${app}`, said, why }`
(web/page/uses.js:188), and its step files it under `of` (:97).

**The choice.** Do it the phone's way: the home page's `stop` io catches
its own failure and answers `unstopped` with `lend` and `app`. A refused
Stop is then said beside its button. A person would see that, so it is
yours to choose. The fix is a few lines, and its test is a browser test
that refuses a Stop.

**Fixed 2026-11-03, the operator's "Fix the bugs":** the phone's way, at
dd7a6b60, outside this queue; its test is crates/svrngs/tests/pages/acts.rs
`a_refused_stop_on_what_apps_use_is_said_beside_it`. The two the typing
found in code comments went with it: web/lend.js's split head (cc24f9ae,
and a 127-byte frame, 00f75721) and web/shim.js's DataView (adef516a).

## A phone's write waits out a round's slowest dial (js-webkit, 2026-11-03)

Found while reading js-webkit's `carried` failure
(`demo/runs/webkit-wall/2026-11-03/README.md`, "The `carried` failure, read
again"). Not this queue's: it is the node's round, not the pages' code.
js-webkit fixes its test's fixture instead (STATE.md, js-webkit (c)).

**What happens.** A write that lands while a round runs, such as a yes
carried from a phone (`POST /carry`), calls `notify_one`, which wakes only
the round after the one running (`crates/svrngs-node/src/ring_sync.rs`,
`spawn_ring_sync_loop`, the comment above its `tokio::select!`). A round
dials every member computer, and one that is asleep holds the round for
its whole dial: 4003-4054 ms per round in `carried-trace.txt`. With no
round running, the same line reached the other computer 18 ms after it
was carried.

**What a person meets.** In a group with one computer asleep, a yes or a
write made on a phone reaches the other computers up to 4 s late. An app
opened at once on one of them runs as before; reopened a little later, it
runs as the yes asked.

**The choice.** A row in a node queue: a write nudged mid-round goes out
without waiting for that round's slowest dial (for one, a push to the
members already answered, apart from the round). Its test: a group with
one computer asleep, a phone's yes, and the app opened on the other
computer at once.

**Decided 2026-11-03 by the operator** ("A now, B as a row"): js-webkit's
fixture change (c) now, and this as a row of its own, `docs/PLAN.md` §1.

## The home page is read before a joined group is (js-webkit, 2026-11-03)

Found in js-webkit-4's gate at 48d09878
(`demo/runs/webkit-wall/2026-11-03/README.md`, "The gate after (c)";
`home-group-gate.txt`). Not this queue's: the fix either changes what the
page draws while it reads, which a person would see (fence 3), or changes
when a test harness reads the page.

**What happens.** `home_group`'s
`the_home_page_joins_another_group_from_its_link_while_svrngs_runs::webkit`
failed once in the gate's `tests` step (`home_group.rs:334`, no "Bring
more to Trip"). It passed three times alone. `web/tests/home.mjs` reads
the page after a press at the first frame whose words changed and hold no
"…". That frame came before Trip's group was read, and by the time the page
was read it said "Reading your groups…" under "What acts as you". The page
queue's `page-parity-4` and `-5` gates failed this test as well.

**The choice.** One of these, and its row:
- The page says a joined group is being read in every frame until it is
  read. A person would see that.
- The harness waits for a signal the page gives when it is done, rather
  than for the first frame with no "…".

Either way, the first step is a traced run that names the frame (AGENTS.md 2).

**Again, on the make path** (js-webkit-5's gate at af3fd04a;
`demo/runs/webkit-wall/2026-11-03/home-group-gate-2.txt`).
`the_home_page_makes_a_group_another_computer_joins_with_no_restart::webkit`
failed at `home_group.rs:205`: the read after "Make a group" had "You made
Trip.", Trip's heading with no folder line under it, and "Reading your
groups…". Every other step of that gate passed, `browser` in 162 s. Both
gates since js-webkit (c) failed only on this race, so js-webkit's pass
(`scripts/check.sh` exits 0) waits on this choice.

**Decided 2026-11-03 by js-webkit's director: the harness fix, in js-webkit
as its part (d)** (`STATE.md`). The harness reads a press once the page is at
rest, judged in the same frame as the read, from the words and the page's
own trace (`window.__lines`). The page draws nothing new, so a person sees
no change. Closing js-webkit with `check.sh` red was refused, since it would
weaken the row's pass bar. This entry stays open for the operator for one
reason: whether a person should see a made or joined group as being read
until it is. That is a page change, and no queue holds it.

**Fixed 2026-11-03** (the operator: "Of course things should show as
loading until they're loaded"): a group whose people aren't read yet says
"Reading <group>…" in its own section (e1ca7cb2), and a group drawn before
a make or join no longer loses its section when "You made …" is first said
(e1d3615a, a '' Preact drew as words). Test: crates/svrngs/tests/pages/
join.rs `a_group_made_or_joined_says_it_is_being_read_until_it_is`;
check.sh exit 0 at e1d3615a.

## `.` is at its cap by one gate line (js-lint, 2026-11-03)

**What happens.** js-lint (d1) runs the pages' type-aware lint as a step
of its own, `step "typed lint" node scripts/typed-lint.mjs`
(scripts/check.sh:108, 94192670), after `types`, since it reads the same
wasm declaration the browser step builds. That one shell line is counted
in `.`, which main holds at 18,860 against a cap of 18,861
(scripts/lean_check.py:403). The row takes `.` to 18,861, and js-lint-6's
gate failed `lean and RUST_ARCH` on it: "BROKEN . has 18861 lines of
code, budget 18861" (ctl/log/sessions/js-lint-6.await.log). Every other
step of that gate passed, apart from one browser test that read a renamed
local's name (edd6b030, fixed).

**The pass.** Nothing to cut was found in the code `.` holds. No function
in scripts/*.py or *.sh is defined and never called, and no bare `return`
or `pass` is left. Every line of check.sh is a step or a verdict, so
folding the typed lint into `types` or `lint` would pack two verdicts
onto one line (fence 1), and lose the separate verdict (AGENTS.md 1). The
line is CHARTER "Answered" 4's agreed type-aware lint, and the charter's
waived pass is `js`'s only.

**The ready diff** (scripts/lean_check.py, above and at `BUDGET`):

```
+# js-lint's type-aware lint (CHARTER "Answered" 4), the operator's raise:
+# +1 in `.`, the gate's `typed lint` step (check.sh, 94192670), at 18,861.
+# The pass found nothing to cut: every check.sh line is a step or a verdict.
-BUDGET = {"crates/svrngs-core": 3478, ".": 18861, "js": None, "swift": 1}
+BUDGET = {"crates/svrngs-core": 3478, ".": 18862, "js": None, "swift": 1}
```

Once it lands, js-lint closes with `lockf -k /tmp/svrngs-gate.lock bash
scripts/check.sh`.

**Decided 2026-11-03 by the operator** ("Approve +1"): `.` to 18,862,
landed from the operator seat in scripts/lean_check.py.

## The js jubilee (js-read (e), 2026-11-03)

The closing `lockf -k /tmp/svrngs-gate.lock bash scripts/check.sh` exited
0 at 8e00b6e8 in 1,384 s. The run and the queue's counts are recorded in
`demo/runs/economize/2026-11-03-js-read/CLOSE.md`. The doc scene is
explicitly `could-not-judge`, as the gate reports; it is not called passed.

**The ready diff, for the watching session to land from the operator
seat.** CHARTER "Answered" 2 sets the cap to one past the queue's final
count: `js` 7,338, cap **7,339**. This worker has not changed the cap.
Removing the uncapped branch also removes three counted Python lines:
`.` 18,861 → 18,858, with its cap kept at 18,862. No JavaScript changes.

```diff
diff --git a/scripts/lean_check.py b/scripts/lean_check.py
--- a/scripts/lean_check.py
+++ b/scripts/lean_check.py
@@ -392,3 +392,2 @@
-# this rung's own 4 came in, so `.` is one past 18,505. `js` has no cap (None) until the js queue's
-# jubilee sets one past where the queue lands; until then it is printed as
-# uncapped, never passed in silence.
+# this rung's own 4 came in, so `.` is one past 18,505. The js queue's
+# jubilee sets `js` one past 7,338 (ralph/next/js/NEEDS_HUMAN.md).
@@ -408 +407 @@
-BUDGET = {"crates/svrngs-core": 3478, ".": 18862, "js": None, "swift": 1}
+BUDGET = {"crates/svrngs-core": 3478, ".": 18862, "js": 7339, "swift": 1}
@@ -478,3 +477,0 @@
-        if cap is None:
-            print(f"{rung:<20} {n:>6} lines, no cap until the js queue's jubilee")
-            continue
```

The extracted diff passes `git apply --check --unidiff-zero -` at
8e00b6e8; apply it with `git apply --unidiff-zero` from the repository
root. After applying, run `python3 scripts/lean_check.py`: `js` must report
`7338 / 7339`, and `.` `18858 / 18862`. As the positive control, plant
`web/page/jubilee-plant.js` with `export const jubileePlant = true;`:
`lean_check` must fail on `js` at 7,339. Remove the plant and require
exit 0 before committing. These are instructions for landing the diff,
not claims of a run against an applied cap.

**Landed by the watching session, 2026-11-03, under CHARTER "Answered"
2's authorization.** `js` is capped at 7,339. The temporary uncapped branch
is gone. Its three-line cut lowers `.` to 18,858, so its cap is **18,859**,
rather than retaining the ready diff's 18,862 (AGENTS.md "Done": a pass's
cuts lower the cap to where they land).

The actual positive control, with `web/page/jubilee-plant.js` containing
`export const jubileePlant = true;`, failed `lean_check` at `7339 / 7339`.
The core stayed `3477 / 3478` and `.` stayed `18858 / 18859`. The plant
was removed; `python3 scripts/lean_check.py` then exited 0 with
`js 7338 / 7339`. The run is recorded in the close report.
