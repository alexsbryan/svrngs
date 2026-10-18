# The 25-computer run

The demo evening's race, run first by programs on two machines: 25 svrngs
computers join one group by one link, each lends a greeter that says
`Hi, <X-Svrngs-Person>`, and the founder's home page reads the whole group
back to back, as the projector would. The bars and their numbers, written
before any data, are `PREDICTIONS.md`.

## What is here

| file | what it does |
|---|---|
| `run.py` | Runs everything, from the Mac. It starts each machine's keepers, the wall, the probes and the visits, then collects every line and log into `<run-dir>/data/` and runs `judge.py`. |
| `fleet.py` | One machine's part, stdlib only, the same on macOS and Linux. Each computer gets its own `HOME`, so the user's own `~/.svrngs` and `~/svrngs` are never touched. `stop` and `clean` touch only what it made. |
| `greeter.py` | The app every person lends. |
| `judge.py` | `PREDICTIONS.md` §2, line by line, with the four verdicts. It writes `data/verdicts.json`. |
| `bin/` | The release's `SHA256SUMS` and `RECORD.txt`, the sha256 of each tarball and binary the runs used. The binaries aren't committed (`bin/.gitignore`): fetch them as below. |
| `runs/` | Each run's `data/`, without the raw logs (the burst's founder log alone was 366 MB). The burst's `summary.json` is gzipped, since it holds all 54,844 wall reads. |

The binaries are V1.1, `a555a225`, as group.svrnme.sh served it on
2026-10-17. For each of `darwin-aarch64`, `linux-x86_64` and
`linux-aarch64`, fetch `https://group.svrnme.sh/get/svrngs-<platform>.tar.gz`
into `bin/`, check it against `bin/SHA256SUMS`, and unpack it into
`bin/<platform>/`. Its binary's sha256 is in `RECORD.txt`. The site serves
whatever was published last, so after the next release these files come
from that release's commit instead.

## Run it

From this directory on the Mac. Fedora is `alexbryan@fedora` (Tailscale);
`--host` changes it.

**All 25 on Fedora.** This puts no load on the Mac, so it can run while a
ralph loop runs here:

```
python3 run.py --mode burst   --layout linux --run-dir runs/linux-burst
python3 run.py --mode stagger --layout linux --run-dir runs/linux-stagger
```

**Split: the founder and 11 on the Mac, 13 on Fedora.** It refuses while a
ralph loop runs on the Mac, unless you pass `--mac-load-ok`:

```
python3 run.py --mode burst   --layout split --run-dir runs/split-burst
python3 run.py --mode stagger --layout split --run-dir runs/split-stagger
```

To watch the wall fill when the founder is on Fedora: `run.py` prints the
line once the founder holds House, `ssh -N -L <port>:127.0.0.1:<port>
alexbryan@fedora`. Then open `http://svrngs.localhost:<port>/` on the Mac.
The port is the founder's own, so the page's Host matches. To keep it as a
rehearsal of the projector, screen-record that window.

`--mac N --linux M --founder mac|linux` sets any other split. `--keep`
leaves each machine's work dir after collecting (`w-mac/` here,
`~/twentyfive/w` on Fedora); without it, both are removed once their
lines and logs are home.

A burst run is about 5 minutes: the wall's ceiling is 240 s past the last
join, it stops 20 s after it first reads everything settled, then come
the matrix probes, ceiling 180 s, and the visits. A stagger run adds 2
minutes.

## What it reads, and from where

- **Join, in and lent**: each machine's `events.jsonl`, written by
  `fleet.py` as each step happens.
- **The wall**: the founder's `/v1/rail/people` then `/v1/lends`, back to
  back, as `home.js` reads them at V1.1. Lit is the door's rule (V1.1
  `home/question.js:115`).
- **The matrix**: every computer's own `/v1/lends`.
- **Visits**: through each computer's own door, as a program does. That
  means loopback, the lend's `Host` (`<label>.localhost:<port>`, its
  origin as `/v1/lends` gives it, V1.1 `surface.rs:261`) and no
  `Sec-Fetch-Site`, which the door takes as a program's (V1.1
  `origin.rs:79-84`).
- **Paths**: iroh's `path::selected` debug event in each log (iroh-1.0.2
  `remote_state.rs:661-667`). The `remote` field is the first 10 hex
  digits of the peer's key, matched against the keys the wall saw.
  `network_path` is `Ip(local->remote)` or `Relay(url, id)`
  (`transports.rs:1126-1137`).
- **CPU and RSS**: `ps` over every process of this run's binary, in
  back-to-back 10 s windows.

## What it cannot tell you

- **The venue's WiFi.** Many venue and guest networks keep their clients
  apart. Every connection between two laptops then goes through n0's
  relay, which `split` on a home LAN does not show. Only a few laptops on
  the actual WiFi will.
- **Tailscale.** It runs on both machines, so iroh may pick a 100.x
  address as a direct path. The venue has no such thing: `judge.py` counts
  Tailscale paths apart from LAN ones, and neither is evidence for the
  room.
- **Fedora's firewall.** From the Mac, firewalld answers LAN port 22 with
  "no route to host", so inbound UDP may be filtered too. If `split` shows
  relay paths where LAN ones were expected, that is the likely cause.
  `sudo` needs a password there, so the run doesn't change it.
- **Real laptops.** Corporate firewalls, VPNs, sleep, and 25 different
  CPUs. Here one machine carries 12 or 25 computers, which over-states CPU
  contention and under-states network variety.
- **Debug traces.** They add CPU to every process; the CPU line is
  measured with them on.
- **Phones.** No phone joins in this run.
- **A people bar.** Every person is a program (AGENTS.md 7).

## What the runs gave

Three runs on 2026-10-17, all `linux` and `burst`, 25 computers on Fedora,
V1.1's binaries. `judge.py runs/<run>/data` writes each run's
`verdicts.json` (gunzip the burst's `summary.json.gz` first). One run of
each is a spread, not a rate.

- **`runs/linux-burst-take1-hung/`: never ran.** The keeper's `nohup …
  &` backgrounded the whole `mkdir && …` list in a subshell, which held
  SSH's stdout, so the sampler's SSH never returned (`run.log`,
  `subprocess.TimeoutExpired`). `run.py`'s `keeper()` braces the command
  now.
- **`runs/linux-burst/`: the wall reading back to back.**
- **`runs/linux-burst-wall1s/`: the wall reading once a second**
  (`--wall-every 1`), as the projector's would.

| line | burst, back to back | burst, once a second |
|---|---|---|
| 0. preconditions | passed | passed |
| 1. every join in | **failed**: 24 of 24, median 3.33 s against a guess of under 3 s | **failed**: 24 of 24, median 3.32 s, max 3.43 s |
| 2. lit on the wall | passed: median 3.26 s, max 3.30 s | passed: median 3.79 s, max 4.24 s |
| 3. lend on the wall | passed: max 2.76 s | passed: median 0.48 s, max 1.47 s |
| 4. wall settles | passed: 3.32 s after the last join | passed: 3.33 s |
| 5. wall reads, settled | passed: p50 1 ms, max 4 ms, 12,259 reads | passed: p50 2 ms, max 7 ms, 20 reads |
| 6. matrix whole | passed: 27.3 s after the last share | passed: 27.1 s |
| 7. greetings right | passed: 75 of 75 | passed: 75 of 75 |
| 8. CPU and memory | **failed**: founder 180% steady, 442 MiB | passed: founder 10% at most, 65.5 MiB |
| 9. Mac to Fedora | never-ran: one machine | never-ran: one machine |

- **Joining is the joiner's wait for n0's relay.** In `Aiko.log`, the
  joiner had waited for a relay 3.09 s after it started. Then the
  founder admitted it within 0.13 s. The guess of under 3 s was off by
  0.3 s.
- **Line 6 is a ceiling, not a measure.** The matrix's first pass was
  already whole (`rounds: 1`), so every computer saw all 25 by the time
  the matrix first asked, 27 s after the last share.
- **Line 8 is the wall's own load.** Back to back, the wall read about
  700 times a second, and the founder's RSS kept growing under it. That
  growth is `demo/runs/leak/2026-10-17/`: the release's Linux allocator,
  not svrngs's code.
- **Lines 5 and 7 have no network in them.** All 25 were on one machine.
