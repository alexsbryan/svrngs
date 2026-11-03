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
