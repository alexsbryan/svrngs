# arch-stream (a): the browser suite's streaming test under load — predictions, before any data

Row arch-stream (`ralph/next/arch/STATE.md`), part (a). Written at 472e06e2,
before the runner exists and before any run. Nothing in the product changes
in this part.

## The test measured

`a_page_speaks_http_through_the_door_streaming_both_ways`
(`crates/svrngs-node/tests/browser/main.rs:457`), alone, both engines at once
as the suite runs them. At HEAD its asserts that failed in arch-peer's checks
are :509 (the range not 206), :530 (the past-the-end range not 416; the row
named it "the body handed whole", but at HEAD :530 is
`assert_eq!(past["status"], 416, ..)` — the chunks assert is :523) and :548
(the upload not 200).

Each run is read at the assert it failed, by the page's own lines: a
measurement instrument (its own commit, behaviour kept) stamps every line the
page traces during the HTTP checks with its time since they began, and the
test prints each engine's `http` answer whole. Beside them, the relay's debug
log (`target/traces/browser.log`, the harness's `traced`), kept per run, and
the host's load at each run's start and end.

## The setup

- **Two builds**: 1acbba2, the driver's own `connect` (its own endpoint), and
  HEAD, the driver's `endpoint(..).dial_link()` (9543e52d), each a debug build
  in its own tree, the instrument applied to both.
- **Two loads**: idle (nothing started on purpose; the host's load recorded,
  other sessions' named when seen), and **burn24**: 24 processes of `yes >
  /dev/null`, twice this host's 12 cores, started by the runner before the
  run's first test and killed when it ends — the 20–30 arch-peer failed at
  (`demo/runs/arch/2026-10-15-peer/close-check-loaded.out`,
  `start-browser-loaded.out`).
- **Ten runs a cell**, each run both engines: 2 builds × 2 loads × 10.

## What I expect, and what would falsify it

1. **Idle, both builds: green.** At most 1 of 10 runs red in either engine.
   Falsified by 2 or more.
2. **burn24, HEAD: red in at least 3 of 10 runs**, in either engine. Falsified
   by 2 or fewer: then (a) has not reproduced it, which is could-not-judge and
   NEEDS_HUMAN.md by the row.
3. **The builds alike under burn24**: 1acbba2's red count within 3 of HEAD's.
   #172 did not cause it. Falsified by a difference of 4 or more.
4. **Where it stops**: the reds are mostly :509, the range failing with "no
   stream could be opened to the computer: timed out" — `open_bi` on a
   connection already closed by its idle timeout — and with the later steps
   failing as the connection is gone; the range's `ms` in those runs near 30 s
   (iroh's idle timeout, the dial being held as iroh holds one,
   `web/src/lib.rs` `reach`, `held` for the issuer).
5. **Why: two endpoints of bo's key on one relay.** The page holds an
   endpoint of its own as bo once it lists the lends (`web/app.js` `enter`,
   `bound`, never closed), and the driver binds a second as bo in the same
   page (`drive.mjs` `httpChecks`, `m.endpoint(key, token)`; at 1acbba2 its
   own `connect`). The relay keeps one active client per key and sets the
   older aside (`iroh-relay-1.0.2/src/server/clients.rs:80-92`), and sends
   packets for that key only to the active one. Under load a relay ping
   goes unanswered in time and the page's own endpoint reconnects, takes the
   key back, and the driver's connection hears nothing until it times out.
   Seen in the relay's log: in a red run, bo's key registered again after the
   driver's binding ("multiple connections found, deactivating old
   connection" twice or more for bo within the run); in a green run at most
   once. Falsified by a red run with one or none, or a green run with a
   second taking the key back before the HTTP steps end.

If 5 holds, the change is the test's own: no page a person opens holds two
endpoints of one key (the page binds one, `app.js` `dialler`; a lend's
worker binds one as a pass's holder, a key of its own, `lend.js:137`), so
the driver's second endpoint is a thing only the test has. If 5 fails and
the reds are a step's 60 s ran out with the connection alive, the wait is
the test's (`drive.mjs` `step`) or the page's, and (b) re-derives it by
`dial::DIAL_WITHIN`'s rule from the slowest working step here.
