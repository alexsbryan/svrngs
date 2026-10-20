# The 25-computer run: setup and predictions, before any data

The demo evening's race, run by programs on two machines before people
run it in one room: 25 computers join one group by one link, each
lends a small app that greets whoever opens it by name, and the
founder's home page, which the projector would show, reads the whole
group back to back. Written 2026-10-17, before the driver first ran and
before any data. `README.md` here says what the run gave against each
line below. Every person is a program, so no people bar is judged
(AGENTS.md 7). One run of each layout and mode is a spread, not a rate.

## 1. Setup

- **svrngs** is V1.1, the release at group.svrnme.sh, built from
  `a555a225`. Its tarballs are fetched as `install.sh` fetches them and
  checked against the release's `SHA256SUMS` (`bin/`). Its help is
  byte-for-byte V1.1's `USAGE` (`crates/svrngs/src/cli.rs:113` at
  `a555a225`). It is not HEAD: House rules is not in it.
- **Each computer** is a `svrngs` process with a `HOME` and `SVRNGS_DIR` of
  its own under the machine's work dir, as
  `demo/runs/first-minute/2026-10-12/run.py:32-33` runs three. Debug
  traces are on where the timings are read, plus iroh's
  `path::selected` event (iroh-1.0.2 `remote_state.rs:661-667`). Every
  computer uses n0's relays, as V1.1 ships (`cli.rs` `SHIPPED`).
- **The founder**, Priya, runs `svrngs new House --as Priya`, takes the link
  it prints, and lends her greeter.
- **Everyone else**, Aiko to Wren (24 people), at their planned second:
  - starts a greeter (`greeter.py`, which answers `Hi, <X-Svrngs-Person>`);
  - runs `svrngs join <link> --as <name>`;
  - once their computer holds House, runs `svrngs share <port> --name
    hello-<name> --kind hello`, answering its password prompt with Enter
    alone.
- **Layouts:**
  - `split`: Priya and 11 more on the Mac (Apple M2 Max, 12 cores,
    64 GiB), 13 on Fedora (x86_64, 32 cores), over one LAN with
    Tailscale on both.
  - `linux`: all 25 on Fedora, the wall seen from the Mac by `ssh -L`.
- **Modes:**
  - `burst`: the 24 joins start within 10 s, interleaved across machines,
    the way the room starts on "Go!";
  - `stagger`: one join every 5 s.
- **The wall** is the founder's home page's reads, as `home.js` makes them
  at V1.1: `/v1/rail/people`, then `/v1/lends`, back to back, each read
  starting when the last lands. Lit is the door's rule (V1.1
  `home/question.js:115`): a person with a computer who is not in
  `unanswered`.
- **The matrix** is every computer's own `/v1/lends`, asked until each lists
  all 25 lends, all 25 people, and nobody unlisted.
- **Visits:** each computer opens 3 others' greeters through its own svrngs,
  as a program (V1.1 `origin.rs:79-84`, `app_of`: no `Sec-Fetch-Site` is a
  program's), one of them on the other machine where there is one.
- **CPU and memory**: every svrngs process sampled by `ps` in back-to-back
  10 s windows, on both machines.
- **Times** are on the Mac's clock. Fedora's lines are shifted by the
  offset from the clock reading with the shortest round trip; its error
  is half that trip, recorded in `summary.json` as `pre.linux_clock`.

## 2. Predictions

Numbers marked *guess* have no earlier run behind them. They are written
down so the run can falsify them.

0. **Preconditions.** At least one of n0's four relays answers `/ping`; SSH
   reaches Fedora; the founder prints a link and holds House within 60 s.
   If any is absent, the run never ran, and nothing below is judged.
1. **Every join gets in.** All 24 joins hold House:
   - each within 10 s of its start, *guess*;
   - median under 3 s. With three computers the founder wrote the
     newcomer's admission 0.71 s after their join began
     (`demo/runs/first-minute/2026-10-12/README.md`, "Where the minute
     goes"). Twenty-four redeems in 10 s queue on one founder.

   Falsified by any join not in within 120 s, or one past 10 s.
2. **Each person is lit on the wall** within 15 s of their join's start,
   *guess*.
3. **Each lend shows on the wall** within 5 s of its `share` returning, the
   median under 2 s. Basis: the wall asks every computer live, all 24 at
   once, each ask cut at 3 s (V1.1 `node.rs:625-630`; `ANSWER_WITHIN`,
   `node.rs:109`).
4. **The wall settles**: 25 lit, 25 lends, nobody unlisted.
   - burst: within 30 s of the last join's start, *guess*;
   - stagger: within 15 s of the last join's start, *guess*.
5. **Wall reads, once settled.** `/v1/lends` at a p50 of 400 ms or less and a
   p95 of 1,500 ms or less, none at 3,000 ms or more. Basis: an awake
   lender answered in 90-324 ms (`demo/runs/mnp/2026-10-09/README.md`, cited
   at V1.1 `node.rs:103-108`); a read waits on the slowest of 24.
6. **Every computer sees the whole group** (the matrix) within 60 s of the
   last `share` returning, *guess*. The risk is the "has not heard of yours
   yet" row (`unheard`). A redeem wakes the founder's round at once (V1.1
   `redeem.rs:139-146`), but the other 23 hear each newcomer only through
   a round. Falsified by a computer still short at 180 s.
7. **Every greeting names the right person.** 75 of 75 visits answer 200
   from the lender's greeter, with `x-svrngs-person` the visitor's name.
   - Visit time p50: under 100 ms on one machine, under 200 ms across the
     two, *guess*.
   - A single wrong name fails it: identity is the point.
8. **CPU and memory**, *guess*, the first time this is measured.
   - While the wall reads, once settled: the founder at most 25% of one
     core; every other computer at most 5%; each at most 80 MiB.
   - During the burst: the founder at most 100%.
9. **Mac to Fedora** (`split` only): every pair that dialled across the
   two machines selected some path (`path::selected`). It is read off the
   same lines, saying for each pair whether it went over the LAN
   (192.168.x), Tailscale (100.64.0.0/10) or n0's relay.
   - Prediction: mostly direct.
   - Tailscale runs on both machines and will not be at the venue, so a
     path over it says nothing about the room. Treat it as direct for
     reach only, never for the room.
   - The `linux` layout has no such pairs: never-ran.

## 3. What a verdict is

- **passed** means every number of the line was within its bound.
- **failed** means one was not.
- **could-not-judge** means the run went but the reading is absent, such
  as no wall read after settling, or no `path::selected` line.
- **never-ran** means a precondition was absent (AGENTS.md 6).

`judge.py` holds these bounds as constants beside each line's number
here. It writes `data/verdicts.json`.
