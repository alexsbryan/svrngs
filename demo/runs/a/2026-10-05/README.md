# Demo A's dry run in stock Jellyfin — this host, 2026-10-05

u5-9 (`ralph/next/u5/STATE.md`). Demo A's three play stock Jellyfin through
a lend at once, on this one host. A scratch Jellyfin is lent with
`svrngs share 8096` as u5-4 left it (sign in once). Three browser profiles
join by the link. Alex is on a computer in Chromium, at the address the
bare command prints. Bo, in WebKit as an iPhone, and Cy, in Chromium, are on
phones: through the page by the group's link, then n0's relay. Each plays a
title of their own for ten minutes, all at once, and seeks. Demo A's seek
bar is judged here as a rate (operator, 2026-10-05). The last section is the
package for the operator: three homes on three networks, and the
catalogue's five acts on a real phone.

## How it ran

Instrument: `apps/jellyfin/check/` at `760215c` — `check.sh films`, `up`,
`trio`, `down`; `trio.mjs` drives the three, timing each seek with
`person.mjs`'s `seek()`, u5-4's own (moved, behaviour kept, `024ed19`);
`trio.py` reads the bars and Sam's door lines; `scan.py` searches every
byte. svrngs built from this tree (no Rust changed since `d77d61d`).
Jellyfin 12.0.0, the macOS app's server, with its own data dirs. Playwright's
Chromium 147 and WebKit (Safari 26.4's user agent, iPhone 15). This host has
12 cores.

```
cargo build -p svrngs
C=apps/jellyfin/check D=demo/runs/a/2026-10-05/data
$C/check.sh films /tmp/u5-9-films 1200              # One, MP4; Two, MPEG-2, converted; Three, Matroska
networkQuality -s >$D/upload.txt                     # Uplink capacity: 50.594 Mbps
MEDIA=/tmp/u5-9-films PEOPLE="alex bo cy" SHARE_FLAGS="--upload 50" $C/check.sh up /tmp/u5-9-run $D
$C/check.sh trio /tmp/u5-9-run $D                    # run 1, 06:23:21–06:33:58 UTC
RUN=2 $C/check.sh trio /tmp/u5-9-run $D              # run 2, 06:35:01–06:45:36 UTC
$C/check.sh down /tmp/u5-9-run $D
```

Sam's group has no limit flag, so the lend lets in three at once, the
default. Sam is told "Each is told their share of the 50 Mbit/s upload you
stated." Alex plays One, which Jellyfin sends as it is. Bo plays Two, which
Jellyfin converts for WebKit at 3,730,555 bit/s ("ContainerNotSupported,
VideoCodecNotSupported, AudioCodecNotSupported"). Cy plays Three, which
Chromium plays as it is (`streams_mid` in each `trio*.json`). The films
are 1,200 s each. Once all three are playing, each seeks through the
window: Alex 10 times, Bo and Cy 22 each, spread evenly over its 600 s, to
5–85 % of the film, forward and back in turn.

Not shown to the trio: run 1's `trio.json` was printed before `trio.mjs`
redacted a person's own `ApiKey`. It rode in Cy's film URLs, which the
worker's trace keeps, so it was redacted after the run by the same pattern
(142 places). Run 1 also predates the run's `from` and `until`, so
`trio-summary.json` read Sam's log between the times `steps.log` gives.

## Every seek, in ms

Demo A's bar: playing again within 5 s.

| run | Alex, computer, Chromium | Bo, phone, WebKit, converted | Cy, phone, Chromium |
|---|---|---|---|
| 1 | 405, 441, 500, 552, 402, 406, 418, 389, 559, 405 | 430, 3,833, 1,796, 2,903, 2,373, 2,616, 2,469, 2,589, 1,822, 2,708, 2,946, 4,004, 3,910, 1,895, 1,056, 4,440, 3,238, 2,617, 2,394, 2,327, 2,188, 2,373 | 1,035, 3,311, 850, 990, 1,077, 1,300, 1,028, 964, 853, 966, 1,055, 953, 699, 841, **19,293**, 2,708, 1,776, 883, 806, 826, 715, 1,023 |
| 2 | 592, 512, 410, 470, 412, 375, 418, 468, 423, 386 | 4,921, 1,632, 2,589, 2,407, 2,251, 2,639, 2,640, 2,475, 2,159, 2,586, 2,492, 2,398, 2,349, 2,615, 2,734, 2,881, 2,661, 2,419, 2,587, 3,333, 2,614, 3,599 | 2,059, 890, 817, 941, 879, 2,153, 1,295, 1,036, 812, 1,111, 992, 887, 935, 1,072, 1,777, 1,743, 1,080, 1,037, 1,152, 1,134, 2,346, 1,542 |

**The rate, on the carriage.** The phones went through n0's relay
(`relayed: Some(true)` on every door line for Bo and Cy). Their seeks:
**88, of which 1 missed 5 s**. That miss was Cy's 15th in run 1, 19,293 ms.
Alex's computer: 20 seeks, none missed. All 108: 1 missed. Beside them,
u5-3's one miss, 7,871 ms
(`demo/runs/u5/2026-10-05-jellyfin/data/pair-phone-webkit-5.json`, the
fourth of `first_seeks`). u5-4's one miss was 6,389 ms, on a computer at a
load of 33.6 (`5c323f8`). The catalogue's misses were Immich's in WebKit,
15,007 and 7,408 ms (`demo/runs/catalogue/2026-10-05/RESULTS.md`).

**The run before these, kept** (`data/before/smoke-trio.json`). First
the instrument ran with a 60 s window, 06:03 UTC. The seeks were scheduled
2.6 s apart, so each began as soon as the last ended. Bo's first came 3 s
into the window, just after Bo's film began to play. Of Bo's 22 seeks there,
**21 did not play again within 30 s**, and the 22nd took 29,592 ms. Of Cy's
22, one missed: 5,693 ms. Alex's 10 played again in 395–1,014 ms. The 1-min
load at those seeks was 3.1–20.6, so this was a loaded host part of the
time (below). These are not counted in the paced rate above, which has its
own conditions, but they are not dropped either: they are the finding
below.

**The host's load.** `uptime` at each run's start and end: run 1 8.14 and
8.50, run 2 9.77 and 2.55, under the 12 cores. The 1-minute load at each
seek, which `trio.mjs` reads as it seeks (`load1`), ran 4.2–13.5 in run 1
and 2.5–13.9 in run 2. Fourteen seeks came above 12 (8 in run 1, 6 in run
2), and none of them missed. Cy's miss came at 7.92.

## The other bars, as read here

- **Each plays through the ten minutes, the other two playing.** Each
  person's film went on through the window. Time it moved forward: Alex
  598.9 s in both runs; Bo 552.4 and 544.2 s; Cy 566.8 and 581.7 s. The
  rest is the time their own seeks took. Leaving out a person's own seeks
  (and a second after each), there was no emptied, loadstart, new element,
  error or ended in either run, nor in the 600 s any still of 5 s or more,
  with one exception: Bo in run 2 was still for 6,104 ms from 0.2 s into
  the window, just after starting to play the converted film. `trio.py`
  marks that run's "bo plays through the window" failed, and it stays
  failed (`trio-2-summary.json`).
- **Signed in once, as themselves, after.** Each signed in once with the
  account Sam made: run 1, 254, 840 and 868 ms; run 2, 321, 2,850 and
  2,321 ms. Each was still signed in as themselves at the end (`Users/Me`).
  The three browsers were three devices to Jellyfin.
- **Sam's password and token, in nothing any of them received.** No secret
  of the run was in any of the 1,211,551,491 bytes the three browsers
  received over both runs: every header, body and socket frame. Searched
  for: Sam's password and token, each person's password, each person's
  token in the other two's. Nor was any in either computer's svrngs output.
  The lend keeps none (`scan.json`, `lend_kept: []`).
- **Jellyfin's users unchanged by the run.** `users-made.json`, the export
  after Sam made the three, and `users-after.json` are both `["alex", "bo",
  "cy", "sam"]`. Before Sam made them: `["sam"]`.
- **Open to playing**, ms, open + sign in + play: run 1, Alex 227 + 254 +
  1,485, Bo 5,525 + 840 + 14,600, Cy 6,372 + 868 + 9,200; run 2, Alex 209
  + 321 + 849, Bo 5,418 + 2,850 + 13,216, Cy 3,213 + 2,321 + 4,389. On the
  phones, opening the link to the group's list first took 8,337 and 8,318
  ms in run 1, and 5,727 and 5,270 ms in run 2.

## Each stream's share, from the lender's log

`sam-run.log.gz`, the door's "let in, and told their share of the declared
upload" lines, summarised in each `trio*-summary.json` under `door`:

| run | Alex | Bo | Cy |
|---|---|---|---|
| 1 | 176 requests let in as away, told 50,000,000 bit/s with 1 person using it; then 96 guessed home, told none | 1,605 let in, told 16,666,666 (3 using it) every time; relay round trip median 166 ms, max 2,088 | 489 let in, told 16,666,666–25,000,000 (2–3 using it); round trip median 994 ms, max 2,034 |
| 2 | 276 guessed home, told none | 1,559 let in, told 25,000,000 (2 using it) every time; median 174 ms, max 2,104 | 500 let in, told 25,000,000–50,000,000 (1–2 using it); median 757 ms, max 1,875 |

Run 1 and run 2 split the same upload differently, for one reason. In run
1 the door saw no path for Alex's first requests, on Sam's own computer. It
took Alex to be away ("door: no path observed, so the person is taken to be
away", `crates/svrngs-node/src/door.rs:803`) and counted Alex. Alex's film,
asked as it is, held that place for the whole run (each of Bo's 1,605
lines says 3 using it), so Bo and Cy were each told a third. In run 2 the path was known from Alex's first request, Alex
was guessed home and not counted, and each was told a half. In Demo A Alex
is in another home, so Alex would be counted, and each of the three told
a third.

## What it found

1. **Through the lend, WebKit stopped asking for the converted film after
   a seek.** This came in the run before these: unpaced seeks, three at
   once. Bo's film requests after the first seek (to 216 s) were
   `hls1/main/69`, `70` and `71`, each 200, the last 8.7 s in; then none
   (`data/before/smoke-trio.json`; the dump's index). Sam's door saw Bo's
   requests only at the player's 10 s progress pace from 06:03:58 on
   (`data/before/smoke-sam-run.log.gz`). Jellyfin restarted the conversion
   once, at 207 s, and converted to the film's end by 06:05:23 UTC. It
   never restarted it for a later seek (`data/before/smoke-jellyfin.log.gz`).
   So the segments existed and were never asked for. The same WebKit and
   film straight at Jellyfin, no svrngs: 5 seeks, 428–886 ms
   (`data/before/control-direct-webkit.json`). Through the lend with Bo
   alone: 4 seeks, 1,009–1,152 ms (`data/before/repro-phone-webkit-alone.json`).
   Paced, three at once: 44 of 44 within 5 s. Not reproduced, and not
   explained. The carriage is where it differs, and the paced runs did not
   bring it back. The instrument now keeps each film request and each
   phone's worker trace, which the smoke run lacked.
2. **Cy's miss was the first part after the seek, held about 18 s before
   Sam's door saw it.** Cy's browser asked `bytes=295436288-` 316 ms after
   the seek, and it was answered 19,915 ms after. The parts before and after
   it took 0.8–4.1 s (`trio.json`, Cy's `media`). Sam's door logged no
   request of Cy's from 06:30:24 to 06:30:46, across the seek. Cy's relayed
   path's round trip then was mostly 1.1–1.3 s, up to 1.9 s (the door's
   `rtt_ms`, 06:30:20–06:31:10). This host's uplink measured
   "Uplink Responsiveness: Low (958.604 milliseconds | 62 RPM)"
   (`upload.txt`), and on this host every byte both phones receive goes up
   it to n0's relay and back. That may be the cause. It is not shown.
3. **The door counts a computer on the lender's own network as away until
   it sees a path** (the share table above). Its share then holds for as
   long as the stream does.

## The package: the operator's run

`operator-only: anything that needs a service this host cannot run` —
three homes on three networks, people new to Jellyfin, and a real phone.

**Decide first.** Finding 1 left a WebKit phone that never played again
after seeks, on one unpaced run of three. Run Demo A as below, or have a
row instrument and fix it before. Demo A's gate is on computers, and every
computer seek here played again within 592 ms.

### Demo A, as `demo/a.md` has it

On Sam's computer, at home, with Jellyfin and at least three films:

```sh
cargo build --release -p svrngs
networkQuality -s                    # write down "Uplink capacity", whole Mbit/s
target/release/svrngs share 8096 --upload <that number>
```

`share` prints how to make each person an account. Make `alex`, `bo` and
`cy` that way, each with a password and every library ticked. Then, in
Jellyfin, Dashboard → API Keys → + (name it "export"), and export the
users:

```sh
curl -s http://127.0.0.1:8096/Users \
  -H 'Authorization: MediaBrowser Client="export", Device="Sam", DeviceId="export", Version="1", Token="<the key>"' \
  | python3 -c 'import json,sys;print(json.dumps(sorted(u["Name"] for u in json.load(sys.stdin))))' >users-before.json
```

Alex, Bo and Cy each have svrngs on their computer, put there beforehand
(`cargo build --release -p svrngs`, or the binary copied). Then the run,
as `demo/a.md` says. Sam:

```sh
RUST_LOG=info target/release/svrngs new House --as Sam 2>sam-run.log
```

Sam sends each person the `svrngs://join/…` link and their Jellyfin name
and password. Each person, on their own computer:

```sh
target/release/svrngs join '<the link>' --as <their name>
RUST_LOG=info target/release/svrngs 2><their name>-run.log
```

They open the address it prints after "Sam's jellyfin, in your browser:".

**What to record**, as `demo/a.md`'s "What is watched" and "After" say:
each screen recorded and each browser's HAR; each seek's time to play again,
read from the recording; Sam's `sam-run.log`, whose door lines carry each
stream's share; Jellyfin's Dashboard → Playback while all three play
(direct or converted, and the bitrate); `uptime` on Sam's computer at the
start and end. After: search the HARs and the three logs for Sam's
password; export the users again and compare; write `runs/a/<date>.json`
as `demo/README.md` says, before anything else.

### The catalogue's five acts, on a real phone

`demo/runs/catalogue/2026-10-05/RESULTS.md`'s acts ran only in Playwright's
engines. On a real phone, Safari on an iPhone and Chrome on Android, over
cellular, with the page on svrnme.sh, which gives each app an address of
its own. On a Mac with podman (scratch under `$HOME`):

```sh
R=~/u5-catalogue-phone
demo/runs/catalogue/2026-10-05/run.sh apps $R          # the five in podman, Bo's accounts made
export SVRNGS_DIR=~/.svrngs-catalogue
RUST_LOG=info target/release/svrngs new House --as Sam --origin https://www.svrnme.sh/g/ 2>sam-run.log
# beside it, once each, pressing Enter at any password question:
for ap in home-assistant:8123 immich:2283 nextcloud:8080 actual:5006 paperless:8000; do
  target/release/svrngs share "${ap##*:}" --kind "${ap%%:*}" --name "${ap%%:*}" </dev/null
done
```

Bo's password is in `$R/secrets/bo_pw`; it is also Actual's server
password, and Bo signs in to Immich as `bo@house.test`. Open the `https://www.svrnme.sh/g/#…`
link on the phone, give a name, and do each act from the group's list:

| App | The act | Checked by Sam, on the Mac |
|---|---|---|
| home-assistant | Bo signs in; Sam turns the Lamp on, then off | Bo's page shows each change without a reload |
| immich | Bo uploads a video of about 100 MB, plays it, seeks three times | the video in Sam's Immich, whole; each seek's time to play again |
| nextcloud | Bo uploads a file | the file in Sam's Nextcloud, the same size |
| actual | Bo types the server password, makes a budget, opens it | the budget on Sam's server |
| paperless | Bo signs in and uploads a PDF | a new document in Sam's Paperless |

Record the screen; for each act, record did or did not, and where it
stopped; each Immich seek's time; the phone, its OS and browser. If one
stops, also save the app's `https://<label>.svrnme.sh/svrngs-trace` from
the same browser. `run.sh apps-down $R` removes the five.
