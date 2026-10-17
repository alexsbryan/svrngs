# On air: the three primitives, proved together

onair-run. ON_AIR §7's proof of concept, whole, on one host: three svrngs
in one group (Ana the station, Bo and Cy listening and carrying), Dee
removed, and a stranger's headless Chromium on a page served from a
directory on 127.0.0.1. Nothing outward-facing. One host, one run of
each: not a rate (AGENTS.md 7).

The predictions (`PREDICTIONS.md`) were committed in `5e65112`, before
any plant was applied or the harness ran at that commit. Then each code
plant, and then the whole run, at `43724d7` (its code `db6ba51`'s; the
harness as of `6440fc3`):

    W=/tmp/onair-run-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13

It ran 137 s, exit 0, the load 7.2 at its start and 5.3 at its end. Every
number below is in `data/summary.txt`, printed from `data/` by
`summary.py`, unless it names another file. `data/` is what the harness
writes (see `../2026-10-13-red/README.md`); beside it, by hand after the
run: `bo-heard.mp3` and `sound.txt`, `lines.txt` (by `lines_at.py`) and
`summary.txt`.

## The bars

Each bar's plant ran red first, on its own bar, before the whole run
(`plants/`, committed in `43724d7`). Every bar then passed, as predicted.

| bar | its plant, red | the bar, green |
|---|---|---|
| 4.1 | — ON_AIR §7 names none. Its red is `../2026-10-13-red/`: no `/v1/blob`, Bo held 0 of 10 | **passed**: Bo held 10 of 10 pieces, 23 to 108 ms after each line arrived, against the 4,000 ms bound set before the run (`BOUND_MS`, `run.py`, since `a92ae3f`) |
| 4.1, carried | `plants/bo-not-serving`: **failed** — with Ana stopped, Cy's get of Y 502, "no holder answered", Ana's "did not answer within 3s", Bo's `never_held`; no empty piece | **passed**: Cy's get of Y 200, its 4,112 bytes Y's, in 3,017 ms, from Bo, named in Cy's trace (`verdicts.json`, `cy_trace`); the plant Z 502 `unanswered` in 3,003 ms |
| 4.1, lapses | `plants/keep-ignored`: **failed** — 65.1 s after the last put, Bo's and Cy's gets of piece 0 and Y 200, said lapsed none, 10 pieces in each of Bo's and Cy's stores | **passed**: 65.1 s after the last put, 410 `lapsed` of piece 0 and Y at Bo and at Cy, 502 of a hash never held, told apart; no piece in Bo's or Cy's store; Ana's, stopped, held 10, and none 1.3 s after it started again |
| 4.1, scoped | `plants/group-check-out`, on svrngs-node `tests/folders.rs` `bytes_an_app_put_in_one_group_are_refused_to_a_member_of_another`: **failed**, "Ana refuses Cy her films bytes: Ok(50000)" (`red.txt`, exit 101); taken out, ok (`green.txt`, exit 0) | **passed**: Dee, removed, refused Y by her own computer, 403 "this computer does not stand in" the group. The harness cannot show the provider's own check: Dee's computer refuses before it dials one, so this bar would pass on its plant, as predicted |
| 4.2 | three plants in the run itself, each refused for its own reason (below) | **passed**: the page on `evening` played 10 of 10 in Chromium, refused none |
| 4.3 | `plants/drain-without-sender`: **failed** — every drained payload named "the drain names no sender", Bo's note among them; the six other bars passed | **passed**: every line Bo (11) and Cy (12) drained named Ana's key and person key; Bo's note in Cy's drain named Bo's (`verdicts.json`) |
| off air | `no-ended`, in the run: **said "cannot reach"**, "no line 2 in 12 s", never "ended" | **passed**: Bo said "ended" 52 ms after the station's, Cy 53 ms, the page 259 ms — each within one piece, 4,000 ms |

## The page, in Chromium

| tab | played | refused | said |
|---|---|---|---|
| `evening` | 0-9 | none | "ended", after 10 pieces |
| `flipped` (byte 1,000 of piece 1 flipped) | 0, 2-9 | 1: "its BLAKE3 is afdedf6b6058…, not the line's c225526d54ff…" | "ended" |
| `act-sig` (one of Ana's acts' signatures, from her journal) | 2-9 | 0 and 1: "its signature does not check under the station's key 5299633c…" | "ended" |
| `bo-signed` (Bo's computer signed lines 0, 1 and the end) | none | 0, 1, 2: "…does not check under the station's key 5299633c…; it names 5b438c87…" (Bo's) | "cannot reach", "no line 3 in 12 s" |
| `no-ended` | 0, 1 | none | "cannot reach", "no line 2 in 12 s" |

No console error in any tab (`errors []`). A refused `ended` is not ended:
`bo-signed`'s page says "cannot reach".

## Each piece, from its line's arrival to Bo holding it

| piece | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| Bo, ms | 88 | 23 | 25 | 33 | 28 | 29 | 42 | 50 | 108 | 78 |
| Cy, ms | 102 | 32 | 30 | 33 | 28 | 30 | 42 | 39 | 108 | 78 |

The bound, 4,000 ms, was one piece's time on air; the slowest was 108.
On loopback, so it says what svrngs adds, not what a network would.

## Carrying, and what it spent

Bo's 11 fetches: 10 from Ana's computer, 1 (piece) from Cy's. Cy's 11:
10 from Ana's, 1 (Y) from Bo's. Ana's computer sent 1,221,209 bytes in
20 gets, 1.91 times the 640,731-byte show, 243 kbit/s over its 40.2 s on
air; Bo's sent 4,112, Cy's 64,365. onair-blob's run split its listeners'
fetches 6 to Ana and 5 to Cy (`../2026-10-13-blob/README.md`). Two
listeners asking the same piece at the same moment both find only the
station holding it whole, so most fetches go to it. One run each: not a
rate. What it means for ON_AIR §5 is said below.

## What the stores held after

Bo's and Cy's stores held no piece 65.1 s after the last put. Ana's,
stopped at the carried step, kept its 10 pieces' files on disk (a stopped
computer runs no sweep and serves nothing); started again, it held none
1.3 s later, what apps placed being kept in memory alone.

## What the plants found

- **bo-not-serving failed 4.1, lapses as predicted, but not for the
  reason predicted.** Bo said piece 0 `lapsed` (410), not 502: the
  harness's own `bo-signed` plant puts pieces 0 and 1 from Bo's computer
  (`plants/bo-not-serving/data/bo.trace`, "blob: put … keep=60"), so Bo
  held piece 0 and it lapsed. And Cy's store kept all 10 pieces.
- **What Cy's store kept is a gap in svrngs, for a follow-up row.** With
  the plant, each fetched entry was dropped from what the store keeps at
  once, so between two sweeps nothing it keeps changed, and every sweep
  stopped at "the store's sweep drops nothing — no record and no
  directory moved since what it keeps was read"
  (`crates/svrngs-node/src/folder.rs:216-220`). Bytes that enter the store
  with no change to what it keeps stay on disk until something else
  moves. The plant made that happen every time. Unplanted, the same path
  runs when a get's fetch fails (`blob.rs`, `get`, the `Err` arm:
  `carry(…, None)` after `carry(…, Some)`) within one sweep, 1 s,
  leaving what the fetch wrote. Not fixed here (this row changes nothing
  but the harness); the next audit or the operator places it.

## The sound, the operator's

40 s generated here by ffmpeg's `aevalsrc`: a sine stepping up a fifth
every 2 s from 220 Hz, mono, 128 kbit/s MP3, 640,731 bytes. Nobody holds
rights to it. What Bo's listener appended, piece by piece, is
`data/bo-heard.mp3`, byte for byte the station's MP3, and Cy's the same
(`data/sound.txt`: `cmp` exit 0, one sha256 for all three; ffprobe 40.05 s,
mono, 44.1 kHz). Nobody heard it in this run, and no bar asks that.
Hearing it once is the operator's:

    afplay demo/runs/onair/2026-10-13/data/bo-heard.mp3

## What 10,000 listeners would need that this run did not show

Each is the operator's (CHARTER.md, "Leave for the operator").

- **A CDN and a bucket.** The station writes its pieces and lines to a
  directory (`station.mjs --public`), served here on 127.0.0.1. For
  strangers, that directory goes to an object store behind a CDN, on the
  operator's account and its keys (Cloudflare R2, or S3 and CloudFront,
  for example), copied as it is written, e.g.
  `rclone copy <public>/air <remote>:<bucket>/air`, with the page served
  from the same place. ON_AIR §5's 1.28 Gbit/s and 576 GB an hour are the
  CDN's to carry, and a load test against it (not the bucket) is what
  would show it.
- **A second machine.** Every computer here ran on one host, on loopback.
  Bo on another computer on another network, joined by the same link
  (`svrngs join <link> --as Bo` there), is what would give the 4.1 bound
  a network to cross, n0's relay and NAT included. The harness starts
  every computer itself, so a run across two machines needs a driver of
  its own.
- **HLS and Safari.** The page plays MP3 pieces by Media Source, and says
  "cannot play" where `MediaSource` or `audio/mpeg` is missing
  (`apps/on-air/page/page.js:32`). Only Chromium ran it. Safari on a Mac
  and on an iPhone, each opening the page, is the operator's, on their
  devices. Whether iPhone Safari offers `MediaSource` at all was not
  checked here. If it does not, the station would also cut HLS
  (`ffmpeg -i <show> -f hls -hls_time 4 …`) and the page would check each
  segment the same way.
- **Phones.** A phone opening the page needs a public address, which is
  the CDN above. No phone ran here, and none carries for the group: a
  phone is a stranger with a browser.

## For svrn-docs `rings/current/ON_AIR.md` §6-§8, for the operator to carry there

This queue commits only in svrngs. What those sections should now say:

- **§6, size, counted, not guessed** (`data/lines.txt`, by
  `scripts/lean_check.py`'s rules, at each row's commits):
  - 4.3, the live lane naming who spoke (onair-live): +11, against "a few
    lines".
  - 4.2, a signature for an app: +28 in svrngs-core (onair-sign (b)) and
    +25 for its route (c), 53 in all, against "some tens … a few".
  - 4.1, bytes by hash with a lifetime (onair-blob (a)): +222, against
    "perhaps a hundred or two".
  - The three: +286 over 15,950 (`e114b96`) to 16,236, svrngs-core +28.
    That is 51 lines under the operator's ceiling of 16,287 and 33 under
    svrngs-core's 3,182. The "19 left" in §7's cost is now this.
- **§7, the proof of concept, as it ran.** Every bar passed, each plant
  red first (above). Where the run parts from §7's letter:
  - The act-sig plant's signature comes from Ana's journal: `/v1/rail/log`
    gives no signatures (`crates/svrngs-core/src/admit.rs:309`).
  - The public directory is `apps/on-air/harness/public.py`, http.server
    with a listen backlog of 128, because `python3 -m http.server`'s 5
    reset Chromium's module loads (`2a37770`).
  - 4.1, scoped's provider check is a test, not the run: a removed
    computer refuses itself first.
  - 4.1, lapses' "no store holds a piece" is true of running stores. A
    stopped computer keeps the files until its first sweep after it
    starts again. A 192-byte hash tree per piece (`.obao4`) is left in
    the store (`../2026-10-13-blob/README.md`; DESIGN §9).
  - §7's bound is set: 4,000 ms, met at 108 at most.
- **§5, Ana's own upload.** "Near 0.5 Mbit/s, whatever the audience"
  assumes the carriers fetch from each other. As built, members asking at
  the same moment fetch mostly from the station: 19 of 20 piece fetches
  here, 1.91 times the show for two listeners. So the members' tier costs
  the station about one stream per member who listens live. The
  strangers' tier, through the CDN, is unchanged. Whether that needs a
  change (say, a listener waiting one piece behind) is for a run that
  shows it needed, not a number chosen now (onair-blob (b)).
- **§8, the questions:**
  1. The three primitives and no public door are built, and passed here.
  2. The queue ran as §8 asked: red first, a row per primitive, then this
     run, within the raise the operator gave (+300 and +60, of which
     +286 and +28 were spent).
  3. 4.2's key is the computer's own (CHARTER.md). The stranger checks
     each line against the key the link carries. "This key is Ana's"
     stays with people inside her groups. Publishing the group's binding
     so a stranger could check it too is still open, and the operator's.

## check.sh, at the close

`scripts/check.sh`, once, at `5cd62bb`: exit 0. All 14 steps passed, from
its own summary lines: principles pinned, lean and RUST_ARCH, app nouns
gate, app nouns, rustfmt, clippy, tests, canaries, ceiling canary, web
rustfmt and clippy, browser, joining in a browser, conformance canary,
app laws. None failed. Lean at `5cd62bb`: crates/svrngs-core 3149 / 3150,
the repository 16236 / 16237 (`scripts/lean_check.py`), the same as before
this row, which changed no product code.
