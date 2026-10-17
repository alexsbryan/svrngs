# events-wait (b) — a phone told within about the cap, 2026-10-14

Row events-wait (`ralph/next/events/STATE.md`), part (b): events-asleep's
capped take's C3 again (`demo/runs/asleep/2026-10-14/CAPPED.md`, 3), red
first. The setup and predictions are `PREDICTIONS.md`, committed (8b79587)
before the driver (a4a9153), the fix (a07a9cc) and any data. One host, one
take each: not a rate (AGENTS.md 7).

## How it ran

```
TAKE=plant W=/tmp/wait-run python3 demo/runs/wait/2026-10-14/run.py < /dev/null   # data/plant/
TAKE=wait  W=/tmp/wait-run python3 demo/runs/wait/2026-10-14/run.py < /dev/null   # data/wait/
```

- **plant** at 9fa3075, before the fix, nothing dirty, 17:52:28 to
  17:55:36 UTC; the 1-minute load 27.3 at the start and 46.4 at the end on
  12 cores.
- **wait** at a07a9cc, the fix, nothing dirty but the plant's data,
  committed, 17:59:06 to 18:01:05 UTC; the load 34.1 and 39.5.
- A debug build, n0's relays, every computer and phone on this host; the
  phone events-asleep's `phone.mjs`, unchanged.
- Two takes before these found two faults of the driver's, each named in
  its fix's body: the sit's bytes read per process, which fell as flows
  closed (4a9beba), and `svrngs share` reading the driver's standard input
  for a password (9fa3075). The first, at a4a9153, ran its frozen rounds
  whole: Cy 34,037–34,354 ms, as the plant below.

## Against the predictions

| | predicted | plant (as before the fix) | wait (the fix) | verdict |
|---|---|---|---|---|
| 0 | some n0 relay answers | all four | all four | passed |
| 1 | plant: Cy 31–38 s; Bo within 5 s | Cy **34,068, 34,042, 34,067 ms**, 502 `away`: the held dial given up at iroh's 30 s, then a new dial refused at 4,001–4,006 ms. Bo **4,008–4,013 ms** | — | passed: red, as planted |
| 2 | wait: Cy 2.5–5 s; Bo within 5 s | — | Cy **4,010, 4,036, 4,271 ms**, 502 `away`, "The computer lending this did not answer. Try again later.", and no dial begun in the window (each round's `dials` empty). Bo **4,003–4,006 ms** | passed |
| 3 | a working lend never said away | every open before a freeze, after the sit and after each wake shown; Bo 200 each time | the same; after each wake Cy shown in 191–287 ms, the held dial gone and dialled again | passed |
| 4 | 50–400 B/s more while the page sits open | phone's relay flow **106.7 B/s** (2,318 in, 4,103 out, 60.2 s); Sam's two relay flows 222.0 B/s | phone's **324.2 B/s** (7,146 in, 12,339 out, 60.1 s); Sam's 355.9 B/s | measured: **+217.5 B/s** on the phone, +133.9 on Sam's |

Data: `data/plant/summary.json` and `data/wait/summary.json`, each round's
`bo`, `cy`, `after` and the sit's `moved`; `events.jsonl` as it ran.

## What it shows

- **A phone holding a dial to a computer that froze is told within about
  the cap.** The held dial is given up once Sam's computer has sent
  nothing for 4 s (`dial::held`), and the request on it is said asleep
  without a second dial (`Dialled::fetch`, `web/lend.js`). Each round came
  at about 4 s, not less: the request's own packets restart the idle timer
  when nothing was sent since the last heard.
- **A computer that answers is not said away.** The keep-alive each
  second holds the dial while Sam answers, and after a wake the dial given
  up while Sam slept is dialled again.
- **The cost** is a keep-alive a second through the relay while the page
  holds a dial: here 217.5 bytes a second more on the phone's relay flow,
  about 0.8 MB an hour, and 133.9 more on Sam's. The flows read are TCP
  flows to port 443 open for the whole sit (`nettop -p <pid> -L 1 -x -n`);
  others that opened or closed meanwhile are listed in `moved.others`, not
  summed. Sam's two are to his own relay (49.13.207.244) and to the
  phone's (5.78.69.43); why his difference is smaller than the phone's
  was not looked into.

## What this run does not show

- **A phone on cellular, a second host, a release build**: not run,
  could-not-judge; the operator's (CHARTER.md).
- **A real phone's worker suspended and resumed** with a held dial: its
  idle timer may fire on the first request after, which would be said
  away. Not run (a07a9cc's body).
- **A phone's battery** under a keep-alive a second: not measured.
