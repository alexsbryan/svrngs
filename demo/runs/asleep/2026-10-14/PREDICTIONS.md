# events-asleep (a) — how long a dial to a lend's computer takes: predictions, before any data

Row events-asleep (`ralph/next/events/STATE.md`), part (a): measure first,
then cap (the operator, 2026-10-14). Written at 9ac4a4c, before the driver
exists and before any data. Nothing in the product changes in this part.

## The setup

- This commit's `svrngs`, a debug build, n0's relays (`SHIPPED = MeetAt::N0`,
  `crates/svrngs/src/cli.rs:246`), every computer and phone on this host,
  each computer a HOME and data dir of its own.
- **Sam's computer** makes a group with the page built and served here as
  its origin, and lends a probe: a static page served on loopback,
  `svrngs share <port> --kind probe --name probe --upload 100`. The upload
  is declared only so Sam's door traces each request's path, relayed or
  not, with its RTT (`door.rs`, the `In` and `Home` verdicts' traces).
- **Bo's computer** joins by the link `new` prints.
- **Cy's phone** is WebKit as an iPhone (`web/tests/page.mjs`), joined by
  the page's address of the link.

## What is measured

**W, a working dial**, in rounds. Each round restarts Sam's svrngs (Ctrl-C,
then started again and answering), so every dial in it is a new one:

- **from a computer** — Bo asks the probe at Bo's own origin for it, once
  right after the restart (*cold*) and four times more (*warm*). Each
  request is a dial of its own: `origin.rs` `carried` opens a new TCP
  connection to the bridge (`door::send_to`), and the bridge dials Sam's
  computer for each one it accepts (`guest.rs` `spawn_preferring`). Timed
  from the request to the probe's whole answer. Both computers are on this
  host, so I expect iroh to carry it on loopback, not the relay; Sam's
  door says which.
- **from a phone's page** — Cy's phone opens the probe's page at its own
  origin. The worker's dial is to Sam's last run, now closed, so it dials
  again (`web/lend.js` `dialFor`, the second try). The page dials only
  through a relay (`web/src/lib.rs` `reach`: the address is
  `TransportAddr::Relay` alone), so this is the path through n0's public
  relays from this host. The dial is timed from the worker's own trace,
  "dialling <key>" to "QUIC connection established" (`/svrngs-trace`), and
  the page from the address given to the probe's words shown.

**A, the dial to an asleep computer, today**, each round once Sam is
asleep, Bo and Cy asking at once:

- *stopped*: Sam's svrngs stopped by Ctrl-C, as events-run did (step 12);
- *frozen*: Sam's svrngs stopped by SIGSTOP, which sends nothing, as a lid
  closing sends nothing. Woken by SIGCONT after.

Rounds: 12 of W, 3 of A stopped, 3 of A frozen. One host, one run: a
spread on this host, not a rate (AGENTS.md 7).

## Predictions

0. **n0 answers.** At least one of n0's relays answers `/ping`
   (`scripts/relay-answers.sh`). If none does, the run is could-not-judge,
   and nothing below is judged.
1. **A computer's working dial, on loopback.** Every one of Bo's 60
   requests answers 200. Cold, each within 2 s, median under 500 ms; warm,
   each within 250 ms. Sam's door names a direct path, not a relayed one,
   for the warm ones. Falsified by any answer but 200, or any time past
   those.
2. **A phone's working dial, through n0's relay.** Every one of Cy's 12
   opens shows the probe. The dial, from the trace, each within 3 s,
   median under 1 s; the open, each within 10 s. Falsified by any open not
   shown, or a dial past 3 s.
3. **An asleep computer today, stopped.** Bo is answered 502, `away`,
   **30 s ± 1 s** after asking, each round, as events-run's 30,019 ms
   (`demo/runs/events/2026-10-14/data/bo.log`); Bo's log says the bridge's
   dial "timed out". Cy's page says the computer did not answer, also
   30 s ± 1 s. Falsified by either told in under 29 s or past 31 s.
4. **An asleep computer today, frozen.** Bo the same as 3, 30 s ± 1 s: his
   dial is a new one whatever Sam sent. Cy's page longer, **about 60 s**:
   the worker's dial to Sam's run is still open, and with no goodbye it
   waits for that connection to be given up before dialling again, and
   that dial waits its own 30 s. Falsified by Cy told in under 35 s.

## The cap, decided by a rule written now

Part (b) caps the dial to a lend's computer by one constant. Its value is
decided by this rule, before the numbers it reads exist:

- **the slowest working dial measured** in 1 and 2, cold or warm, computer
  or phone, **times 3, rounded up to a whole second**.
- The margin, 3 times, is for a path slower than this host's: a relay
  farther than the one n0 gives this host, a busier one, a loaded
  computer. It is not for cellular, which this host cannot measure:
  that stays the operator's, could-not-judge.
- If a working dial in 1 or 2 fails, or the slowest one comes within 3
  times of the asleep wait in 3, the rule gives no cap, and part (b)
  stops for the operator.
