# events-wait (b) — a phone told within about the cap: predictions, before any data

Row events-wait (`ralph/next/events/STATE.md`), part (b): the capped take's
C3 again (`demo/runs/asleep/2026-10-14/CAPPED.md`, 3). Written at cb71f8e,
part (a), which gives a dial to a lend's computer up at 4 s
(`dial::DIAL_WITHIN`), before this run's driver, before the fix, and
before any data.

## The setup

As events-asleep's (`demo/runs/asleep/2026-10-14/PREDICTIONS.md`, "The
setup"): a debug build, n0's relays, every computer and phone on this host.
Sam's computer makes the group and lends a probe; Bo's computer joins by
the link; Cy's phone is WebKit as an iPhone, events-asleep's `phone.mjs`
unchanged, joined by the page's address of the link.

## The two takes

The same driver, each at its own commit:

- **plant**, before the fix, as now: the phone's page holds its dial to a
  lender with iroh's own idle timeout, 30 s, and keep-alive, 5 s
  (iroh-1.0.2 `endpoint/quic.rs:156`, noq's default idle timeout).
- **wait**, at the fix: the page's dial to a lender is given up after
  `DIAL_WITHIN` of hearing nothing from the computer (its idle timeout),
  with a keep-alive under it, so a computer that answers is never silent
  that long; and a request on a held dial given up that way is told at
  once that the computer did not answer, not dialled again.

Each take runs, in order:

- **sit**: Cy's phone opens the probe, then the page sits open 60 s with
  nothing asked. The bytes in and out of the phone's browser's network
  process (Playwright WebKit's `com.apple.WebKit.Networking`, which carries
  the worker's WebSocket to n0's relay) and of Sam's svrngs, read by
  `nettop` before and after. Then Cy opens the probe again.
- **frozen**, 3 rounds: Bo and Cy each see the probe; Sam's svrngs is
  stopped by SIGSTOP, which sends nothing, as a lid closing sends nothing;
  Bo and Cy ask at once, each timed to being told. Then SIGCONT, and once
  Sam answers, Cy opens the probe again and Bo asks.

## Predictions

0. **n0 answers.** At least one of n0's relays answers `/ping`. If none
   does, the take is could-not-judge, and nothing below is judged.
1. **plant, the red run: Cy waits past the cap.** Cy's page says the
   computer did not answer about **34 s** after asking, each round, 31–38
   s: the held dial waits out iroh's 30 s, then a new dial the cap's 4 s.
   Bo is told within 5 s, the cap and a second. Falsified by Cy within 5 s,
   which would say the plant is not what holds the phone.
2. **wait: Cy told within about the cap.** Cy's page says the computer did
   not answer **2.5–5 s** after asking, each round: the held dial is given
   up 4 s after the last thing Sam's computer sent, which came at most a
   keep-alive and a round trip before the freeze, and no second dial
   follows. Bo within 5 s, as in 1. Falsified by Cy past 5 s.
3. **Both takes: a working lend is never said away.** Every open before a
   freeze, after the sit and after each wake shows the probe, and Bo's
   every ask answers 200. After a wake the held dial is gone, given up
   while Sam was frozen, so Cy's open dials again and is shown. Falsified
   by any `away` to a computer that answers.
4. **The keep-alive's cost, measured, not a bar.** In the plant take, the
   page sitting open moves the relay's own traffic and a QUIC keep-alive
   every 5 s. In the wait take, a keep-alive every second: a PING and its
   ACK, each in a relay frame inside the WebSocket's TLS, about 100–200
   bytes each way. So the phone's network process moves **50–400 bytes a
   second more**, in and out together, than in the plant take, and Sam's
   svrngs about as much more. Falsified by a difference outside that
   range. Whether that cost is worth the wait it saves is the operator's.

One host, one take each: not a rate (AGENTS.md 7). A take that could not
finish is could-not-judge, never a pass.
