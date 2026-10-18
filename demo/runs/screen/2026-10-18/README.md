# A group on a screen: the evening's rehearsal

The screen (`home.js` `screen`, `/?screen=<group>`) on a projector-sized
page while 25 computers join one group in a burst and each lends a greeter.
Then the computer that started the group stops answering, the way a
closed lid leaves it. Everyone is a program, so no people bar is judged
here (AGENTS.md 7), and two takes are a spread, not a rate.

## What ran

- **The group:** the 25-computer burst of `../../twentyfive/2026-10-17/`,
  all 25 on Fedora (x86_64, 32 cores). Its `run.py` and `fleet.py` are used
  as they are, by `rehearse.py` here. The svrngs binary is built on Fedora
  from the screen's commits.
- **Two screens, both on the Mac,** in headless Chromium at 1920x1080 through
  `ssh -L`, read every 250 ms by `watch.mjs`:
  - **Priya's.** She started House. Her screen opens before anyone joins.
  - **Aiko's.** She's a member, and her screen opens once she's in. The
    finale is read on hers, since Priya's computer is the one stopped.
- **The wall reads once a second,** as the screen does: `fleet.py wall
  --every 1`.
- **The finale:** once both screens read "25 people · 25 on now · 25 things
  lent", Priya's svrngs is frozen with SIGSTOP, as a closed lid leaves it.
  Then one visit, through Aiko's computer, opens Bo's greeter.

To run it again, from here on the Mac, with svrngs built on the Linux
machine:

```
python3 rehearse.py --bin <svrngs on Fedora> --run-dir <take>
```

## The takes

| | take 1 | take 2 |
|---|---|---|
| built from | 7a385472 (`a91495c3…`) | 2bebba22 (`29b4f3a8…`) |
| Priya's screen opened | 8.0 s before the first join | 8.0 s before |
| each window on Priya's screen, after that join started | median 4.15 s, at most 4.65 | median 4.07 s, at most 4.57 |
| each lend on Priya's screen, after its `share` returned | median 0.83 s, at most 1.38 | median 0.79 s, at most 1.24 |
| each window on Aiko's screen | median 4.41 s, at most 5.67 | median 4.35 s, at most 5.02 |
| each lend on Aiko's screen | median 0.98 s, at most 2.23 | median 1.01 s, at most 1.67 |
| both screens read 25 · 25 · 25, after the first join started (the last started at +9.6 s) | +13.9 s, +14.0 s | +13.9 s, +14.2 s |
| Priya drawn "did not answer" on Aiko's screen, after her computer froze | 3.2 s | 3.5 s |
| Bo's greeter opened through Aiko's computer, Priya frozen | 200, `x-svrngs-person: Aiko` | 200, `x-svrngs-person: Aiko` |
| Priya's window | **24th of 25** | first |

Aiko's screen opened 5.0 s after the first join started, in take 2, so
the windows of those who joined before it show its opening time rather
than their join.

Almost all of a window's 4 s is the joiner's own wait for n0's relay. In
the burst run it was 3.09 s of a 3.3 s join
(`../../twentyfive/2026-10-17/README.md`). The rest is the screen's
one-second read.

**Take 1 found a bug.** The door sorted people by `joined.at`, and the
founder's `joined` is the string `"started"`, whose `.at` is a string's
method. Every comparison with Priya came out NaN, and she was drawn 24th. The
test now wants the one who started the group first (58446306), and the fix
asks for `started` by name (2bebba22). Take 2 is the fixed screen, and its
pictures are the rehearsal's.

## The pictures, take 2

- `take2/data/shots/founder-opened.png`: Priya alone, before Go. "Nothing
  new yet."
- `founder-mid.png`: 14 people, the newest marked.
- `founder-settled.png`: 25 people, 25 on now, 25 things lent.
- `member-finale.png`: Aiko's screen once Priya's computer stopped
  answering. Priya's window is dashed, "did not answer", and 24 are lit.

`member-opened.png`, `member-mid.png` and `member-settled.png` are Aiko's
screen at the same moments.

## What this does not show

- **A real network.** All 25 run on one machine, so no connection crosses
  a real network, and venue WiFi is not in it.
- **A loaded Mac.** The Mac drawing the screens was at a load of 13 to 25,
  from other work, during both takes.
- **The count line wraps.** At 1920x1080, "lent" goes onto a second line
  once there are 10 or more people. That's a CSS size in `look.css`, left as
  it is.
