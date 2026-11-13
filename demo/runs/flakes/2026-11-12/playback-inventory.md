# Media-playback waits in the svrngs test suite

Read-only survey, 2026-11-12, of the working tree at `fc357324` (plus the
other session's uncommitted core/door/rail edits, which touch no test).
"Playback wait" means a check that passes only when a browser's media decoder
runs: `currentTime` moving, `!paused` after `play()`, `!seeking` plus movement
after a seek, `readyState >= 3`, or the continuous `<video>` watch.

## 0. The instruments (JS wait sites)

| id | wait site | what it waits for | timeout | on miss |
|---|---|---|---|---|
| P | `web/tests/page.mjs:561` `play({at, within})` | `at===null`: `muted=true; play()`; else `currentTime=at`. Then `!seeking && !paused && currentTime > from+0.25` (page.mjs:579) | `within` (15 s at every caller) | `{ms:null, at, seeking, paused, error}`; the script still exits 0 |
| C | `apps/web-conformance/checks.js:69` `seeks` route | `<audio src=big.wav>` (64 MiB WAV); `play()`; `currentTime > 0.5` (checks.js:89, 20 s); 20 seeks 250 ms apart; then `!seeking && !paused && currentTime > last+0.25` (checks.js:104, 30 s). `ok = again+250 <= 5000` | 20 s / 30 s, whole check 60 s (`judged`) | `{ok:false, saw}` |
| J1 | `apps/jellyfin/check/person.mjs:375-400` (`enter`, `play_ms`) | jellyfin-web's Play clicked; poll `ct>0.5 && !paused`; a second tap if paused with `rs>=3` for 3 s | 30 s | `play_ms = null`, `person.stopped` set, `person.video` dumped |
| J2 | `person.mjs:109` `seek(pct)` | drag `.osdPositionSlider`; then `!paused && !seeking && readyState>=3` and `currentTime > from+0.3` | 30 s | `{ms:null, …}` |
| J3 | `person.mjs:27` `watch` + `:66` `judge(since)` | samples the `<video>` every 200 ms: `emptied`, `loadstart`, new element, `back_s`, `still_ms` | continuous | numbers; judged by the caller |
| N | `apps/jellyfin/check/night.mjs:56-66` `fresh`/`playing`, loop :131-142; `until` (tv.mjs:126) at :169 `paused`, :194-207 `sought`, :209 `played_on` | TV `<video>` new since Play and `ct>0.5 && !paused` (plus Jellyfin's session naming the film); paused; `ct` within 5 s of half the duration **or** Jellyfin's session position there; `!paused && ct > from+0.3` | 60 s; 30 s each | `stopped` (play); `{ms:null}` (others) |
| H | `apps/holyarrs/check/scene.mjs:290-297` | `!paused && currentTime > 1` (`waitForFunction`) | 45 s | throws → `seen.stopped` |

Out of the gate (no Rust test runs them): `apps/holyarrs/check/newcomer-smoke.mjs:250-276`
(`readyState>=3` 30 s, playing 45 s, then J2), `apps/jellyfin/check/screen.mjs:84,226,240,251`,
`apps/jellyfin/check/trio.mjs` (J3 over a window): Demo A/B measurement tools run by
`apps/jellyfin/check/check.sh`/`screen.sh`, and the xenia world (`crates/svrngs/tests/xenia/film.rs:86`
only reads Jellyfin's `/Sessions` on the server side; no browser wait in svrngs).

## 1. The table

Steps are `scripts/check.sh` steps. Engines come from `in_webkit!`/`in_chromium!`/
`in_both_engines!` (`crates/svrngs-node/tests/browser/harness.rs:23-70`) and `scene!`
(`crates/svrngs/tests/scenes/main.rs:36`, Chromium only; each person still gets the engine
of their device).

| # | Rust test (engine) | step | JS wait → Rust consumer | timeout | verdict on a miss | what the test needs to claim for itself (decoder aside) | network observations already recorded |
|---|---|---|---|---|---|---|---|
| 1 | `a_lent_film_plays_and_seeks_under_the_worker_which_reaches_each_lender_itself::webkit` (main.rs:838-1050) | browser | P ×5: `first` runtime.mjs:93 → main.rs:930; seeks@40,20 runtime.mjs:96 and seek@52 after the worker stop runtime.mjs:122 → `seeks_judged` main.rs:936; `replayed` (after the browser reopens) runtime.mjs:119 → main.rs:956 | 15 s each; seek bar 5 s (load.rs:52) | `first`, `replayed`: **fail**. Seeks: >5 s **fail**, or **could-not-judge** if the load at the start was past `BUSY`×cores (load.rs:37; written to target/could-not-judge/browser.txt) | the lend opens at its own origin and the lend's worker serves `title.mp4` from ada's door by Range: 206 `video/mp4`; each range reaches the door as a closed part ≤ 2 MiB naming bo's key; **a seek makes a new Range past the bytes already fetched**; after the worker stops (browser reopened) the lend's address is served again and a new range reaches the door | `film_heads` (every request head at the film origin, in order: `Range`, `x-svrngs-key`; door/origins.rs:94), `keys()`/`ranges()` (main.rs:748,767). The page records only console logs. No response list |
| 2 | `a_lend_worker_that_reads_no_store_dials_with_the_pass_its_start_page_hands_it::webkit` (main.rs:1176-1335, `handed_over(_, false)`) | browser | P ×2: `first` handover.mjs:120 → main.rs:1266; `replayed` handover.mjs:152 → main.rs:1300 | 15 s | **fail** | the lend's worker dials with the pass its start page handed it, and again after a restart through the start page: a Range of `title.mp4` answered 206 by the worker, both times; the door names bo | `film_heads` keys (main.rs:1330); the worker's trace (`/svrngs-trace`, `handedTrace`/`stoppedTrace`); logs |
| 3 | `a_lend_worker_that_loses_every_key_handed_it_takes_the_holders_bytes_in_memory::webkit` (`handed_over(_, true)`) | browser | same as #2 | 15 s | **fail** | as #2, with the holder's bytes handed in memory and, after the restart, a pass signed to a new holder | as #2 |
| 4 | `sam_opens_the_link_kim_sent_and_watches_kims_film_as_kims_guest::webkit` (guests.rs:15-204) | browser | P ×1: `first` visit.mjs:126 → guests.rs:144 | 15 s | **fail** | Sam, a guest, reaches Kim's lend at its origin, and every request reaching Kim's "Jellyfin" says `x-svrngs-person: Sam (Kim's guest)` and Sam's visit key (guests.rs:150-202) | `film_heads` (guests.rs:150, `!heads.is_empty()` :151 already depends on the `<video>` having asked for bytes; `preload="auto"` makes it ask without play); `requested` (every URL any page asked, visit.mjs:41) |
| 5a | `on_a_computer::chromium` (conformance.rs:699-705) | conformance canary | C ×2 per lend (sam, bo) → `verdicts`/`verdict` conformance.rs:244 → `failed.is_empty()` :610 | 20 s + 30 s, 60 s whole | **fail**, or **could-not-judge** when the start's load was past `BUSY` (conformance.rs:249-252) | the canary's media scrub carried whole: `big.wav` asked by Range through the lend, each 206, the 20th seek's range answered without first reading a long range through (#53) | `seen.requests` (`<time> GET <url> <status> by the worker / not`, conformance.mjs:69-76), `seen.trace` (each range in the lend worker's trace, phone only, :157), `seen.asked` (page host, phone), `seen.memory` |
| 5b | `on_a_computer::webkit` | conformance canary | C ×2 per lend, plus `rate()` → `web/tests/control.mjs:89` (same C straight at the canary, 2 runs) | same | **never fails** (`a_rate`, conformance.rs:235): verdict `passed/failed, a rate`; kept in target/conformance/seeks-computer.tsv and target/seeks-webkit.log; check.sh:163-168 prints the rate. control.mjs must exit 0 (:441) | as 5a | as 5a, plus control.mjs's `log` of each Range asked per connection |
| 5c | `on_a_phone_with_the_wildcard::chromium` (:707-713) | conformance canary | as 5a, through the lend's worker at the wildcard (Chromium, no device) | same | as 5a | as 5a, through the lend's worker | as 5a, plus `trace` |
| 5d | `on_a_phone_with_the_wildcard::webkit` | conformance canary | as 5b, through the lend's worker | same | as 5b: a rate | as 5a | as 5c |
| 6 | scenes `films::chromium` (films.rs:156-311) — Alex and Bo, both Chromium, `way: computer` (each their own computer's loopback origin for Sam's lend, `home/session-worker.js` in the path) | scenes | J1 ×2 via drive.mjs `enter` → films.rs:248 `play_ms.is_number()`; J2 ×3 per phase ×2 phases (drive.mjs:51) → `seeks_judged` films.rs:268; J3 per phase (drive.mjs:58) → films.rs:286 (`emptied==0 && loadstart==0 && back_s<=2`) and :291 (`still_ms>=5000` → `late`) | 30 s; 30 s per seek, bar 5 s; 5 s still | play: **fail**. Seeks over 5 s: **fail**, or the scene's **could-not-judge** (`scene.could_not_judge`, :309). `emptied`/`loadstart`/back: **fail**. `still_ms`: `late` → **could-not-judge** past BUSY, else fail (people.rs:44) | the invariant (films.rs:15-18): a lend carries nothing of its lender's sign-in — each in Jellyfin as themselves (`signed_in_after`, :250), devices apart (:255), Sam's password in nothing received nor in logs (:302-306). For the stream: each person's `/Videos/<their film>/…` asked through their own computer's address and answered 206/200 `video/*`, both at once | `record()` (person.mjs:169): every response to DUMP/<user>/index.jsonl with url, status, all headers, body file; `tally.from_worker`; Jellyfin `/Sessions` per device (`sessions[].playing`, drive.mjs:85); `socket()` message carried |
| 7 | scenes `night::chromium` (night.rs:53-211) — TV = Chromium at Dee's room address (tv.mjs:21, no key, no lend worker); Ana's phone = Chromium as a Pixel 7 through the group's page; Dee's phone (tv part) WebKit; the phones never play | scenes | N play (night.mjs:131-142, also needs Jellyfin's session naming the film) → `stopped` → night.rs:189; N `paused` :169, `sought` :194-207, `played_on` :209 → `pause_seek_within_10s` :211 → night.rs:198 `late`; `within_3_min` (:154, from the TV play time) → night.rs:192 `late` | 60 s play; 30 s each; bars 3 min and 10 s | TV never plays in 60 s: **fail** (hard; not load-judged). 3 min / 10 s: `late` → **could-not-judge** past BUSY, else **fail** | invariant (night.rs:18-22): a device in the room reaches the lend through the computer it was opened on; the TV holds no key; nobody makes an account (`users()` :206). Stream claim: after Play on the phone, the TV asks `/Videos/<film>/stream…` at the room address and gets 206/200; the phone's remote pause/seek reach the TV (Jellyfin session `IsPaused`, a new request from the seek point) | `out.films` (night.mjs:222-241: every TV `/Videos/` request: path, Range, status, content-range, length), `screen()` log of every TV request by host and `asked.elsewhere` (tv.mjs:28-55), Jellyfin session (`session()`: `playing`, `paused`, `play_method`, PositionTicks via `position()`) |
| 8a | scenes `holyarrs::chromium` (holyarrs.rs:67, 79-257) — Bo Chromium, computer way | scenes | H scene.mjs:290 then J2 `seek(50)` scene.mjs:298 + assert `ms!==null && |at-target|<3` :299-302 → `seen.stopped` → holyarrs.rs:206 | 45 s; 30 s | **fail** (any throw is `stopped`; not load-judged). Never-ran without Jellyfin/podman | H5: Bo plays the imported film from Movie Night's link: Jellyfin's `/Sessions` has Bo's session on that item (scene.mjs:303-313). Network: a `/Videos/<itemId>/…` request through Bo's lend origin answered 206/200 `video/*`, and a seek's later Range past what was fetched | `record(bo.context…)` (scene.mjs:21) dump; Jellyfin `/Sessions` |
| 8b | scenes `holyarrs::phone::webkit` (holyarrs.rs:68-77) — Bo WebKit as an iPhone through the group's page, **every Jellyfin byte through the lend's service worker** | scenes | as 8a | same | as 8a | as 8a, through the lend's worker (`from_worker`) | as 8a, plus `tally.from_worker` |

Not in the gate but consuming the same instruments:

| Rust test | how it runs | wait | verdict |
|---|---|---|---|
| `a_films_peak_through_a_relay` (main.rs:1059-1170), Chromium | `#[ignore]`, by `RELAY_RATE_OUT=… --ignored` | P via runtime.mjs (first, 3 seeks; Chromium so no `replayed`) → main.rs:1162 `first.ms` | fail on no first play; seeks only written out. Claim: relay's peak bytes/s (sampled `bytes_recv`) — the play just drives traffic |
| `upload_after_seeks::with_the_wildcard` (conformance.rs:742), WebKit | `#[ignore]`, by name | C (ROUTES default `seeks,upload`) | seeks only printed; fails on host-answered upload or memory ≥ 1024 MB |

## 2. Note per test (non-playback assertions to keep)

**#1 runtime (main.rs:838).** Non-playback asserts: opened at `label(1,"page")` (:929);
three seeks recorded (:935); WebKit `reopened == opened` (:955); a `range: bytes=` head
reached the film's service (:964); every film head names bo (:973); every film range a
closed part ≤ 2 MiB (:984); cy's shelf range 206 + content-range + length + sha (:991-1009);
open-ended range a bounded 206 part, sha-exact (:1010-1037); refusals `[403 null, 403 path]`
(:1041 block); shelf heads non-empty and all bo (:1041-1049). Chromium's
`stopped_and_ran_again` (main.rs:786) is dead code: the test is `in_webkit!` only, so the
CDP stop path in runtime.mjs:101-109 never runs in the gate (only via the ignored relay test).
The `replayed` wait is the only evidence that WebKit's reopened worker opened a new endpoint
(main.rs:950-953 comment) — a 206 Range fetched in the reopened page would carry it instead.
Seeks: what the seek proves at network level is a Range starting past what was fetched,
which `ranges(&film_heads)` already lists in order; it is not asserted today.

**#2/#3 handover (main.rs:1221).** Non-playback asserts: `unsealedAfter` = bo's key (:1256);
opened at its origin (:1261); handedTrace contains "the page handed over the pass" (:1262);
`unheld` = `{503, pass}` (:1270); stoppedTrace "is not kept here" (:1285); goes through start
page (:1289); new-holder log only when keys lost (:1294); reopened == opened (:1299); bytes
taken only when keys lost (:1309); "message was lost" log only when keys lost (:1315); kept
key is a CryptoKey (:1320) with no `seed` (:1324); film heads name bo (:1330). Here `first`
and `replayed` stand for "the worker dialled with the pass"; `handedTrace` is read *after*
`first`, so the play also serves as the wait for the dial. A Range fetch of `title.mp4`
answered 206 and sha-exact (runtime.mjs `range()` already does this for `other`) would
replace both waits.

**#4 guests (guests.rs:35).** Non-playback asserts: 48 hours kept (:116), no length over
48 (:119), secret never asked (:123), link shape and issuer (:126, :130), not asked again
(:133), "Kim’s guest" status (:134), opened at the lend's origin (:140), heads non-empty
(:151) and each `x-svrngs-person: Sam (Kim's guest)` and not Kim's key (:153-162), Ana
refused twice (:170-171), again/ended (:172-180), groups `["own"]` (:181), one visit in Kim's
record with its fields and 48 h length (:187-196), heads name the visit key (:197). The
film is only the vehicle for "a request reaches Kim's lend as Sam".

**#5 conformance (conformance.rs).** Every other route (root, websocket, range, upload,
worker, isolated, host, signin+cookie, guarded, storage) and the cookie/key/secret/leak/
unsealable/at/record-post asserts (:619-697) are untouched by the media. Only the `seeks`
route uses the decoder, and in WebKit it is already a rate, not a gate. In Chromium it is a
gate with load-based could-not-judge. Its claim (back-to-back seeks answered quickly
through the lend, #53) could be expressed as: after the 20th seek, the worker's trace /
`seen.requests` show a Range of `big.wav` starting at the last target's byte offset,
answered 206 by the worker within 5 s, with no unclosed range read through first.
`seen.requests` lacks the Range header today (only method, URL, status, by-worker); the
trace (phone) has each range. On a computer neither has the Range.

**#6 films scene.** The scene's invariant needs no decoder. The decoder claims here are
Demo A's people bars (seek within 5 s, plays on while the other seeks, never still 5 s):
a measured instrument (`trio.mjs`, demo/runs/a) rather than a lend-carrying claim.
Non-playback asserts: no `stopped` (:244), signed in as themselves (:250), devices apart
(:255), password held nowhere (:302-306). Note `phase()` and the seeks run only `if
(playing)` (drive.mjs:66), so without J1 the seek phases would not exist.

**#7 night scene.** The TV is Chromium straight at the room address; no service worker
and no WebKit in its media path. Non-playback asserts: Sam makes the room's account (:99),
TV signed in by Quick Connect as `room` (:170-177), no `stopped` (:186, :189 — the TV play
is inside this), nobody made an account (:206). `tv_session_ms` (Jellyfin naming the film as
the TV's) is reported by jellyfin-web, likely after its player starts, so it is not
decoder-free on its own; `out.films` is.

**#8 holyarrs scene.** H1-H4, H6 (removal cuts a live stream within 5 s, stranger and
removed refused), H7 (no owner credential reached Bo), H8 (fixture, no page errors) need no
media. H5 alone plays and seeks; its independent check (`/Sessions` names Bo on the item,
scene.mjs:303-313) is client-reported. 8b is the one gate test that asserts **playback in
WebKit of a real Jellyfin stream through the lend's service worker**, and it fails hard.

## 3. WebKit through a lend's service worker

Tests that assert decoder playback in WebKit with the lend's worker (web/lend.js, at the
wildcard origin) in the media path, and fail when it does not play:

- #1 runtime (`::webkit`; first, seeks, replayed), browser step — hard fail, except the
  seeks' load-judged could-not-judge.
- #2, #3 handover (`::webkit`; first, replayed), browser step — hard fail.
- #4 guests (`::webkit`; first), browser step — hard fail.
- #8b holyarrs `phone::webkit` (play 45 s, seek 30 s), scenes step — hard fail.
- #5d conformance wildcard `::webkit` (audio/WAV) — a rate, never fails; `control.mjs`
  (no worker) must run.

All of these run wherever `scripts/check.sh` runs: there is no platform gate on them.
The only platform split is `web/tests/page.mjs:208-214` (`starting`): WebKit on macOS
starts through webkit.sh; on Linux Playwright's own WebKit (GStreamer media) starts as is.
The repo records nothing about GStreamer failing through a worker (no mention under
crates/, web/, apps/, docs/, scripts/, demo/); that comes from the caller.

On the Mac the browser step's WebKit tests are the only gate evidence that WebKit's
decoder plays bytes the lend's worker answers. Chromium with the lend's worker in the
media path: #5c only (audio, Chromium). No in-gate test plays a `<video>` in Chromium
through the lend's worker: #1-#4 are WebKit only, and films/holyarrs-chromium go through
the computer's loopback origin.

## 4. Duplicates and a recommendation

Same claim, several tests:

- "a lent film plays at its lend's origin through the lend's worker (WebKit)": #1 `first`,
  #2 `first`, #3 `first`, #4 `first` — four tests, the same synthetic 60 s H.264 film
  (`film()`, main.rs:637) behind the same `one_origin` + `kept_range_origin`, the same P wait.
- "it plays again after the worker restarts": #1 `replayed`, #2 `replayed`, #3 `replayed`.
- "a seek plays again within 5 s": #1 (WebKit, worker), #5a/c (Chromium, audio, gate),
  #5b/d (WebKit, rate), #6 (Chromium, Jellyfin, computer), #7 (TV Chromium, Jellyfin, room),
  #8a/b (Jellyfin, both engines).
- "Jellyfin's player starts": #6, #7, #8a, #8b.

Recommendation: one contained browser-step test owns the decoder claim, per engine —
`a_lent_film_plays_and_seeks_through_its_worker` as `in_both_engines!` (runtime.mjs's first
play, three seeks judged by `seeks_judged`, and play again after the worker is stopped:
Chromium through CDP, which revives the dead `stopped_and_ran_again`; WebKit by reopening).
WebKit where its decoder cannot play through a worker reports could-not-judge with the reason,
never a pass. Every other test asserts the network claim it owns:

- #1 (the rest of runtime): `title.mp4` Range fetched in the lend page (runtime.mjs `range()`)
  206, `video/mp4`, sha-exact; the door's heads (all parts ≤ 2 MiB, bo's key); after the
  worker stop/reopen, a new 206 and a new head at the door. Seeks become: a range beginning
  past what was fetched reached the door after `currentTime` was set (ordered `film_heads`).
- #2/#3: a 206 sha-exact Range of `title.mp4` from the lend page, before and after the
  hand-over, plus the traces already asserted.
- #4: one Range fetch of `title.mp4` from the lend page, 206; the heads' person/key asserts
  stand.
- #5: drop `seeks` from the gate's ROUTES (keep it for control.mjs and the
  `upload_after_seeks` runs), or keep it a rate in both engines; the `range` route already
  carries the 206 claim.
- #6 films: the invariant asserts plus, from `record`'s index.jsonl, each person's
  `/Videos/<their film>/` answered 206/200 `video/*` through their own computer's address.
  Demo A's seek bars belong to the recorded instrument (trio.mjs, demo/runs), not the gate.
- #7 night: from `out.films`, the TV asked its film at the room address and got 206/200;
  `asked.elsewhere` empty; Jellyfin's session names the film as the TV's; the 3-min and
  10-s bars stay recorded runs.
- #8 holyarrs H5: from Bo's `record` dump, `/Videos/<itemId>/` answered 206/200 `video/*`
  through Bo's lend (by the worker on the phone), plus the `/Sessions` check, which is
  client-reported.
