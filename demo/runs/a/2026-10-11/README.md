# Demo A's dry run with nothing typed — this host, 2026-10-11

v1-onboarding (e) (`ralph/next/v1/STATE.md`; the mw README §7, items 5
and 7, `demo/runs/mw/2026-10-08/README.md`). Demo A's steps as
`demo/a.md` now has them: Sam lends Jellyfin through its middleware
(`apps/jellyfin/mw`), which makes each person an account as they come, and
keeps one library, Family, back with `--keep-back Family` (`e877024`).
Alex, Bo and Cy each join on a computer of their own, all on this one host;
nobody types a name or a password into Jellyfin. Predictions first, in
`PREDICTIONS.md`, written before the run.

## How it ran

Instrument: `apps/jellyfin/check/` at `8b70f09` — `check.sh films`, `up`,
`mw`, `trio` with `MW=1`, `down`. svrngs built from this tree (`cargo build -p
svrngs`), the middleware by `check.sh` from its own manifest. Jellyfin 12.0.0, the
macOS app's server, with its own data dirs. Playwright's Chromium 147 and
WebKit 26.4. This host has 12 cores. The steps, as `data/steps.log` has
them (04:24:38–04:41:40 UTC):

```
C=apps/jellyfin/check D=demo/runs/a/2026-10-11/data
$C/check.sh films /tmp/onb-e-films 1200        # One, MP4; Two, Matroska, converted for WebKit; Three, Matroska
networkQuality -s >$D/upload.txt               # Uplink capacity: 37.660 Mbps
MEDIA=/tmp/onb-e-films PEOPLE="" SHARE_FLAGS="--upload 37" KEEP_BACK=Family $C/check.sh up /tmp/onb-e-run $D
COMPUTERS="alex bo cy" MW_SHARE_FLAGS="--upload 37" $C/check.sh mw /tmp/onb-e-run $D keep-back-off   # the red run
KEEP_BACK=Family ... $C/check.sh mw /tmp/onb-e-run $D keep-back   # twice: the first is mw-keep-back-unforgotten.json
KEEP_BACK=Family MW=1 $C/check.sh trio /tmp/onb-e-run $D          # 04:26:07–04:36:29 UTC
KEEP_BACK=Family MW=1 RUN=2 $C/check.sh trio /tmp/onb-e-run $D    # stopped at 04:41:40 UTC
$C/check.sh down /tmp/onb-e-run $D
```

`up` with `PEOPLE=""` makes nobody an account (`users-made.json`:
`["sam"]`), and the plain lend it makes is unshared before the
middleware's (`sam-unshare.out`). Sam is told by `share --sign-in 8097`:
"Each person is signed in to it as themselves by what answers on port 8097,
with nothing typed" (`steps.log`, 04:25:08). Each film is 1,200 s; Alex
plays One as it is, Bo's WebKit gets Two converted at 3,730,555 bit/s
("ContainerNotSupported, VideoCodecNotSupported, AudioCodecNotSupported"),
Cy plays Three as it is (`mw-trio-summary.json`, `streams_mid`). Each seeks
10 times through the 600 s window.

## Against the predictions

1. **Held.** The red run, the middleware not told: "failed  Family kept
   back from each: {'Alex': ['Family', 'Movies'], 'Bo': ['Family',
   'Movies']}", every other bar passed (`mw-keep-back-off.json`).
2. **Held, on the second try.** With `--keep-back Family`, every bar
   passed: "{'Alex': ['Movies'], 'Bo': ['Movies']}" (`mw-keep-back.json`).
   The first try was the instrument's fault, kept as
   `mw-keep-back-unforgotten.json`: Sam's door still held Alex's and Bo's
   sign-in answers from the red run, whose accounts `mw_up` had deleted, so
   the first `/Users/Me` answered 401, 401 and the second asks made each a
   second account. `check.sh` forgets them since (`mw_forget`, `8b70f09`).
3. **Failed.** Nobody was asked to sign in (`mw-trio.json`, each person's
   `asked`: false, `signed_in_ms` 0), and all three played through the
   window: 596.5, 565.1 and 589.5 s of 600, no still of 5 s or more outside
   their own seeks. But of 30 seeks, **3 missed 5 s, all Bo's**: 16,534,
   5,349 and 10,242 ms. Every seek, ms:

   | Alex, Chromium | Bo, WebKit, converted | Cy, Chromium |
   |---|---|---|
   | 1,169, 941, 835, 1,778, 595, 531, 453, 453, 944, 451 | **16,534**, **5,349**, 1,706, 2,849, **10,242**, 571, 450, 474, 454, 617 | 3,668, 2,106, 1,752, 1,800, 1,521, 963, 632, 799, 948, 810 |

   The host was loaded far past its 12 cores: `uptime` 38.66 at the start
   (`steps.log`, 04:26:07), and the 1-minute load at Bo's three misses
   54.72, 67.4 and 74.54 (`load1` in `mw-trio-summary.json`). Bo's last
   five seeks, at 45.42 down to 11.59, took 450–617 ms. Alex's and Cy's
   seeks at the same moments played again within 3,668 ms. Bo's film is
   the one Jellyfin converts, on this same host. That the load is the
   cause is likely and not shown; the run is kept as it failed.
4. **Held.** Jellyfin's users after: `["Alex-6df0191cc56d133f",
   "Bo-1e424b00582c07cd", "Cy-9eecfe26e7c6d0b5", "sam"]`
   (`users-after.json`), against `["sam"]` before (`users-before.json`):
   three the middleware named, none anyone made. Each account opens Movies
   alone (`views-mw-trio.json`).
5. **Held.** None of the run's secrets (`admin_token`, `mw_key`,
   `sam_pw`) in any of the 177,127,873 body bytes the three browsers
   received (11,872,836, 147,588,487 and 17,666,550), nor in any of the 19
   logs of the four computers and the middleware (`scan.json`).

**The second trio could not judge.** It began at 04:37:11 and was stopped
at 04:41:40 by an operator stop of the session running it ("page.waitForTimeout:
Target page, context or browser has been closed", `mw-trio-2.json`); it is
kept, and counted in nothing above.

## Recorded, not judged

- **The door saw them home.** All three were on Sam's own computer, so
  the door guessed most of their requests home and told them no share:
  Alex 255 home, none let in as away; Bo 1,629 home, 3 let in; Cy 259 home,
  4 let in (`mw-trio-summary.json`, `door`). In Demo A each is in another
  home, and each is counted.
- **Open to playing**, ms, open + the list shown + play: Alex 5,553 +
  2,382 + 4,778; Bo 7,308 + 1,750 + 7,694; Cy 7,960 + 2,823 + 4,625
  (`mw-trio.json`, `open_ms`, `shown_ms`, `play_ms`).

## For the operator's run

`demo/a.md`'s steps, as they now are, on three computers in three homes:
Sam runs `jellyfin-mw --listen 127.0.0.1:8097 --keep-back <library>`,
pasting the key from Jellyfin's API Keys (`apps/jellyfin/README.md`), then
`svrngs share 8096 --sign-in 8097 --upload <Mbit/s>` and `svrngs new
House --as Sam`, and sends the link and nothing else. Prediction 3 says
what to watch: a converted film's seeks on a loaded lender. Write `uptime`
on Sam's computer at the start and end, as `demo/a.md`'s "After" says.
