# capability-keys: two people of one name stay two

svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §7.4, "Yes, keys
everywhere"; `demo/runs/authorship/2026-10-24/DESIGN.md` §8.

## (a) Red

The tests, red at e872f9e0, are in 5c7e8b76; its body pastes each red run:
a yes kept per person, each act's writer by first key, Money's and Events'
corrections by the same person's key, and the home page telling a namesake
apart ("ada (joined …)").

## (b) Green

- a809c6d5: a yes and an act's writer by first key; `cargo test -p
  svrngs-core` green, golden.rs's first test unchanged (no byte, id,
  signature or line version moves).
- 1bd20af4: Money and Events; `node --test $(find apps -name '*.test.mjs')`,
  141 passed, 0 failed.
- f7b28d01: the pages; the browser test green in Chromium and WebKit.
- 61d488e2, bb2353c5: the economizing cut, then the caps to one past where
  the row lands, 18,285 and 3,299, inside the operator's ceiling.

## The close

`scripts/check.sh` exited 0 at bb2353c5, 838 s in all: tests, browser,
joining in a browser, and every scene (doc, events, films, grapevine,
house_rules, life, money, night, pantry) passed in Chromium and WebKit
(`ralph/next/capability/ctl/log/sessions/capability-keys-2.await.log`, its
last line "note took 838 s in all").
