# v1ui-reach (a) — how long a phone's page takes to reach the computer that made its link: predictions, before any data

Row v1ui-reach (`ralph/next/v1ui/STATE.md`), part (a): measure first, then
cap (the operator's rule for a dial, "Measure first, then cap"; groups-join-wait,
`demo/runs/events/2026-10-14-links/NEEDS_HUMAN.md`). Written at aae56f8,
before the driver exists and before any data. Nothing in the product
changes in this part.

## The dial measured

Opening the group's page, a phone dials the computer that made the link it
joined by before it lists anything (`web/app.js:314`, `enter`,
`connect(key, group.token)`). That dial, on the members' protocol, waits as
iroh does (`web/src/lib.rs:307-308`, `dial` passing `reach` no cap): with
that computer asleep a person was told after 30,497 ms (stopped) and
30,522 ms (frozen) (`demo/runs/events/2026-10-14-links/README.md`, P6 and
P8). It is not the redeem dial (`connect_to_join`), which a computer just
started answered in 3.03 s and which 5c097eb kept uncapped; this run does
not time that one.

The dial is timed from the page's own trace, each line stamped in the page
by `performance.now()` as the page logs it: from "LAYER tls/alpn+relay-hop:
dialing" to "ok — QUIC connection established" (`web/src/lib.rs` `reach`),
the span a cap on `reach` would bound. Beside it, the endpoint's bind and
the relay's handshake before it (from "LAYER page: wasm module loaded" to
"LAYER relay-connect: ok"), and the whole open, from the address given to
the page's status shown.

## The setup

- This commit's `svrngs`, a debug build; every computer and phone on this
  host, each computer a HOME and data dir of its own.
- **Two paths**, each a group of its own made anew:
  - **loopback**: iroh's relay on 127.0.0.1, plain HTTP
    (`crates/svrngs/tests/n0_off.rs` `the_relay`), Sam's group made
    `new --relay` there, so the link and the phone's every dial name it;
  - **n0**: n0's relays, as svrngs ships (`SHIPPED = MeetAt::N0`), the
    link naming the one Sam's computer reached.
- **Sam's computer** makes the group, `--origin` the page built and served
  here, as its link's computer.
- **Bo's phone**: WebKit as an iPhone (`web/tests/page.mjs`), one browser
  for each path, joined by the page's address of the link.

## What is measured, each path

- **W, warm**: 20 opens of the group's page with Sam's computer running,
  each a new page load, so each a new endpoint and a new dial.
- **C, cold**: 5 opens, each the moment Sam's svrngs, stopped by Ctrl-C and
  started again, answers on its loopback API: a computer just woken.
- **A, asleep**: 2 opens with Sam's svrngs stopped by Ctrl-C: today's wait.

One host, one run: a spread on this host, not a rate (AGENTS.md 7).

## Predictions

0. **n0 answers.** At least one of n0's relays answers `/ping`
   (`scripts/relay-answers.sh`). If none does, the n0 path is
   could-not-judge and nothing of it below is judged.
1. **Warm, loopback.** Every one of the 20 opens says "You’re in". Each
   dial within 300 ms, the median under 100 ms. Falsified by an open not in,
   or a dial past 300 ms.
2. **Warm, n0.** Every one of the 20 says "You’re in". Each dial within
   1 s, the median under 300 ms (a phone's dial through n0's relay took
   66–103 ms, `demo/runs/asleep/2026-10-14/README.md`). Falsified by an open
   not in, or a dial past 1 s.
3. **Cold, both paths.** Every one of the 5 says "You’re in"; each dial
   within 3.5 s, as the redeem dial to a computer just started took 3.03 s
   (5c097eb). Falsified by an open not in, or a dial past 3.5 s.
4. **Asleep, both paths.** Each told the computer did not answer
   30 s ± 2 s after the address is given, in words naming the relay's host
   ("did not answer through …"), as the links take (P6, P8). Falsified by
   either told in under 28 s or past 32 s.
5. **The cap fits the constant.** The slowest working dial of 1–3, times
   3, is within `dial::DIAL_WITHIN`, 4 s: under 1,334 ms. Falsified by a
   slower one; the cap is then the rule's, below.

## The cap, by the rule written now

Part (b) caps this dial by the same rule as `dial::DIAL_WITHIN`
(`crates/svrngs-node/src/dial.rs`; `demo/runs/asleep/2026-10-14/PREDICTIONS.md`,
"The cap"):

- **the slowest working dial measured** in 1–3, warm or cold, either path,
  **times 3, rounded up to a whole second**; `DIAL_WITHIN` itself when that
  is within it, so a lender's dial and this one are one decider.
- The margin, 3 times, is for a path slower than this host's: a relay
  farther than n0 gives this host, a busier one, a loaded computer. It is
  not for cellular, a second host or a release build, which this host does
  not measure: those stay the operator's, could-not-judge.
- If a working open in 1–3 fails, or the rule's cap comes within 3 times of
  the asleep wait in 4, the rule gives no cap, and part (b) stops for the
  operator.
