# A newcomer's computer from the line — this host, 2026-10-11

v1-onboarding (f) (`ralph/next/v1/STATE.md`). One newcomer on this Mac
(macOS 27.0.1, arm64; `data/run.json`, `host`) gets svrngs from the group's
page by the one line, and is in the group with its folder on their disk.
Predictions first, in `PREDICTIONS.md`, committed before any data
(`be2200f`). One host, one run: not a rate (AGENTS.md 7).

## How it ran

```
scripts/build-release.sh --out /tmp/onb-f-warm      # the caches, first: data/build-release-warm.txt
demo/runs/onboarding/2026-10-11/run.sh /tmp/onb-f-run demo/runs/onboarding/2026-10-11/data
```

`run.sh` at `be2200f`, its redaction of Sam's `svrngs://join/` line fixed
after the run (below). The page host is `scripts/build-web.sh`'s site, with
`scripts/build-release.sh`'s `get/` in it, served on localhost
(`web/tests/serve.mjs`). Sam makes Trip there (`svrngs new Trip --as Sam
--origin <page>`, a debug build of this tree) and keeps Trip's folder with
one photo in it, `beach.jpg`, 78,817 bytes. Chromium opens the group's link
on a computer's viewport and reads the line off the page (`data/line.json`,
shown 329 ms after the link opened). The line runs with a `HOME` of its own
and no `SVRNGS_DIR`, so nothing of this host's svrngs is touched; `open` is
stubbed to write what it was given, and Chromium then loads that page until
it lists Trip and its folder. The one thing of this host's the line
touches, the LaunchAgent `sh.svrnme.svrngs` in the person's launchd domain,
was absent before (`run.sh` refuses otherwise) and booted out at the end
(`data/steps.log`, 05:08:29 UTC).

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | all four tarballs build; each matches `SHA256SUMS` | all four built, cold, in 11:31 (`build-release-warm.txt`); in the run, warm, 15 s; `shasum -c` OK for each (`sums-checked.txt`); 11,231,192–12,801,252 bytes (`tarballs.txt`) | held |
| 2 | the line exits 0 within 15 s, saying each step | exit 0 at 4,642 ms: fetched, checked, installed, said not on PATH, joined, kept running at login, home page given to `open` at 4,585 ms (`install.out`, `run.json`) | held |
| 3 | the home page lists Trip and says where its folder is within 20 s | at 6,029 ms: "Trip’s folder is on this computer at /tmp/onb-f-run/newcomer/svrngs/Trip" (`home.json`) | held |
| 4 | Sam's photo in that folder, byte for byte, within 60 s | at 6,489 ms (`steps.log`, `cmp`) | held |
| 5 | all of it well inside Demo C's five minutes | 6,489 ms of 300,000 | held, here |
| 6 | the Linux tarballs never run here | built and summed; never run: no Linux host, no docker | could-not-judge |

Prediction 2 also said the join went through n0's lookup; nothing in
`data/` shows the path it took, so that clause is not judged.

## What the run leaves out, each said

- **The person.** Nobody read the page or found a terminal; the clock
  starts at the paste. Demo C's clock starts at the link in the chat.
- **The network.** Every byte came over loopback. The darwin-aarch64
  tarball is 12,111,448 bytes; from a real host it is a download.
- **macOS's dialogs.** None was shown here, and none was looked for. The
  installed binary carries no quarantine attribute (only
  `com.apple.provenance`) and the linker's ad-hoc signature
  (`installed-binary.txt`); a person's Mac was not tried.
- **Linux**, prediction 6.

## What it found

1. **The newcomer is named by their computer's login.** The line gives
   no name, so `join` took `$USER`: "You're in Trip, as alexsbryan."
   (`install.out`; `crates/svrngs/src/cli.rs:1369`), and the home page
   lists "People: Sam, alexsbryan" (`home.json`). In Demo C the others see
   that name. Not changed here: a name typed would break the gate's "one
   line, nothing else typed"; a name in the link would be Sam's to type
   for each person. The operator's call.
2. **A tarball is not the same bytes twice.** The warm build's sums and
   the run's differ for each platform (`build-release-warm.txt`,
   `SHA256SUMS`): `tar` and `gzip` write times. Each `SHA256SUMS` matches
   its own tarballs, which is all `install.sh` asks; publishing ships the
   pair together.
3. **The run's first record of Sam's link held its token** in the
   `svrngs://join/` form: `run.sh` redacted only the `#` form. Redacted
   before anything was committed, and `run.sh` fixed to redact both, as
   `apps/jellyfin/check/check.sh` does. Sam's node was stopped at the run's
   end, so the token answered nothing after.

## Publishing: the operator's

The row ends with the command, never runs it:

```
scripts/deploy-page.sh        # builds the page and the four tarballs into one site, then vercel deploy --prod
```
