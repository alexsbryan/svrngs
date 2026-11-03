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
