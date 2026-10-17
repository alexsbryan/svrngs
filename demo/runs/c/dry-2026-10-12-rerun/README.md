# Demo C's dry run again — this host, on what the bash queue landed, 2026-10-12

Row bash-rerun (`ralph/next/bash/STATE.md`): Demo C's dry run
(`demo/runs/c/dry-2026-10-12/`) again, after bash-first-minute, bash-limit,
bash-person and bash-devices landed. Its steps on one Mac, every person a
program. Predictions first, in `PREDICTIONS.md`, committed before any data
(`834a7ea`); the driver committed before it ran (`e83aecc`). It is not
Demo C's run: no person did anything, so no people bar is judged, and no
record goes in `runs/c/` (AGENTS.md 7). One host, one run: not a rate.

## How it ran

```
demo/runs/c/dry-2026-10-12-rerun/media.sh /tmp/c-rerun-media
W=/tmp/c-rerun MEDIA=/tmp/c-rerun-media python3 demo/runs/c/dry-2026-10-12-rerun/run.py   # data/, 14:22:47–14:27:30 local
W=/tmp/c-rerun MEDIA=/tmp/c-rerun-media python3 demo/runs/c/dry-2026-10-12-rerun/name.py  # data/name.json
```

`run.py`, `c.mjs`, `name.py` and `media.sh` at `e83aecc`, as they ran, in
one take. The build is `e83aecc`'s (`data/events.jsonl`, `start`). macOS
27.0.1, 12 cores, **the 1-minute load 39.76 at the start and 99.55 at
the end** (`events.jsonl`, `start`; `summary.json`, `end`), 31.5–38.6 as each
newcomer finished and 108.1 as Eve did (`newcomers.*.load`,
`asleep.eve.load`): another project's builds, not this run's. Every number
below is under that load. The setup is `PREDICTIONS.md`'s:

- **The page and the binary** built from this commit and served on this
  host, nothing published: `scripts/build-web.sh` and
  `scripts/build-release.sh --out <W>/site` (exit 0 both, warm),
  `web/tests/serve.mjs` serving it at `http://localhost:56384/`; the
  darwin-aarch64 tarball 12,228,105 bytes, its sum matching the
  `SHA256SUMS` served beside it (`summary.json`, `published`); Sam's and
  Ana's svrngs unpacked from it, every newcomer's line fetching it from
  there and checking it ("Checked: svrngs-darwin-aarch64.tar.gz matches
  SHA256SUMS", `data/*-install.out`).
- **Sam's library**: Jellyfin 12.0.0 with Demo A's three films,
  `apps/jellyfin/mw` on 8097, `svrngs share 8096 --sign-in 8097`. **Ana's
  music**: Homebrew's Navidrome 0.64.2 (`summary.json`, `apps`,
  `jellyfin_version` and `navidrome_version` apart now) with three albums
  of three tracks, first with no ExtAuth setting, then started again with
  what `svrngs share 4533` printed (`ana.navidrome_set`:
  `ND_EXTAUTH_USERHEADER=X-Svrngs-User`,
  `ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32`). **The trip**: 30 JPEGs,
  146,488,940 bytes, by `media.sh`.
- **Each person** a `HOME` of its own with `USER` their login; one line at
  a time, each person's svrngs taken out of launchd and run by the driver
  once their clock was done; **asleep** is `SIGSTOP` to Ana's svrngs and
  Navidrome.

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | the link on the local page; its preview the invitation; the line within 10 s; the tarball's sum matches | `new` printed the link on `http://localhost:56384/` (`data/sam-new.out`); fetched as Discord's preview fetches it, 200, "You’re invited to a group" and its description (`summary.json`, `preview`); the line shown 348–953 ms after the link opened (`newcomers.*.line.shown`, `asleep.eve.line.shown`); the sum matched | held |
| 2 | Sam's folder from `new`; Ana in as `ana` by the line; `share 4533` lends `navidrome`, asks nothing, prints both settings; each `share` says it lets everyone in at once | `new` said "lake-trip's folder is /tmp/c-rerun/sam/svrngs/lake-trip: what anyone in lake-trip puts there reaches this computer, …", and it was there (`sam.folder`); Ana's line exited 0 in 5,692 ms, Ana's folder made (`ana`); `svrngs share 4533`, given nothing on its input, said "Lent navidrome (port 4533) to everyone in your groups, as many at once as want it. …" and "… ND_EXTAUTH_USERHEADER=X-Svrngs-User and ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32 in its environment, then start it again." in 19 ms (`ana.share`); Sam's `share 8096 --sign-in 8097` the same "as many at once as want it" (`sam.share`) | held |
| 3 | each of Alex, Bo, Dee within 5 min; the line within 10 s, done within 30 s; **both lends listed within 5 s of the line's exit**; film and album within 20 s each, no sign-in; the photo within 120 s | **13,762, 12,831 and 12,736 ms** (against the dry run's 63,987, 59,609 and 58,935, `dry-2026-10-12/data/summary.json`, `newcomers`). The line done at 4,272–4,553 ms; the home page listing the group, its folder and both lends at 4,675–4,899 ms, **346, 357 and 403 ms after the line's exit** (`newcomers.*.listed_after_line_ms`; the dry run's 45–50 s); the film playing 1,861–2,004 ms after its play, the album 1,408–1,431 ms; neither app asked anyone to sign in; Ana's `IMG_0001.jpg` on each disk, the same bytes, opened as a 4032×3024 image (`newcomers`) | held, the machine's floor |
| 4 | the logins as names; Navidrome signs each in as `<name>-<16 hex of their key>`, never `ana`, never an admin | the people read "alex, ana, Sam", "alex, bo, ana, dee, Sam" and on Eve's "eve, alex, ana, dee, Sam, Cy" (`newcomers.*.home.said`, `asleep.eve.home.said`) — the logins, in an order not the predicted one (finding 1); Navidrome signed in `alex-6b954c8924e2ad87`, `bo-ae3d6397c2f1296e`, `dee-b24ca0d4ca376120` (`newcomers.*.music.as`), each its person's name and the first 16 hex of the `person_key` `/v1/rail/people` gives (`accounts.x_svrngs_user`) | held |
| 5 | Cy's phone: the group within 10 s; a name asked and typed; film and album within 2 min; Navidrome as `Cy-<16 hex>` | the name asked at 3,618 ms; in, "You’re in lake-trip, as Cy", at 6,514 ms, both lends listed at 6,525 ms; the film playing at 26,715 ms (12,255 ms after its play), the album at 36,024 ms, signed in to Navidrome as `Cy-c061d0f7bbef6a1b`, neither asking a sign-in (`cy`) | held, the machine's floor; the typed name is the operator's (the dry run's finding 5) |
| 6 | removed within 30 s; Sam's photo after, in `new`'s folder, reaches all but Bo; Bo's remove counts nowhere; Bo keeps the trip | `svrngs remove bo --group lake-trip` said "bo is no longer in lake-trip. …" in 66 ms; Bo's computer's address for Sam's jellyfin and Ana's navidrome answered 403 `removed` at the first ask, **0.089 s** and **0.092 s** after; Bo's home page said "You are no longer in lake-trip. …" at 1,163 ms; Sam's `after.jpg` on Ana's, Alex's and Dee's disks in 0.7–0.9 s, on Bo's not in 60 s; Bo's `svrngs remove Sam` refused, "You are no longer in lake-trip." (exit 1), every standing list after Cy, Sam, alex, ana, dee; Bo's disk kept all 30 photos (`removed`) | held |
| 7 | asleep: all 30 on Eve's disk within 2 min; Eve's first look and Alex's say "ana's computer did not answer"; no running computer said not to answer, not heard or refused | Eve's line done at 6,118 ms; **all 30 on Eve's disk, each the same bytes, 9.6 s after** (`asleep`); Eve's home page at its first look listing the group, 9,541 ms after Eve's link (3,423 ms after Eve's line), said "ana's computer did not answer, so what they lend is not listed." and named no one else; Alex's said the same; across every look the driver kept, "did not answer" is said of `ana` alone and "has not heard" and "answered without" of nobody (`summary.json`, `events.jsonl`); Eve's film playing 1,948 ms after its play; Sam's jellyfin answered 200 to Alex's, Dee's and Eve's computers | held |
| 8 | Jellyfin: `sam` and one account each who played; Navidrome: `ana` and one each who played an album; each person's two accounts one name | Jellyfin `["sam"]` before, after `sam` and `Cy-c061…`, `alex-6b95…`, `bo-ae3d…`, `dee-b24c…`, `eve-0e6a…`; Navidrome `["ana"]` before, after `ana` and `Cy-c061…`, `alex-6b95…`, `bo-ae3d…`, `dee-b24c…` — the same four names as in Jellyfin (`accounts`); no newcomer was asked for anything but Cy's name | held |
| 9 | no newcomer refused `full`; the count could-not-judge on one host | no `full` refusal in either lender's log (`summary.json`, `door`: Sam's 0, Ana's 0); Sam's door guessed a computer in Sam's home 680 times and Ana's 148, counting none of them, and counted Cy's phone, relayed, as `people=1` (`data/sam.log`, "door: let in") | held; the count could-not-judge |
| 10 | a second `ana` at the line refused, told how to add a device; no LaunchAgent; the people Sam and `ana`; Navidrome `ana` alone; a member signed in as themselves | the line exited 1 after 3,048 ms: "svrngs: someone in Lake already goes by ana, so you were not let in as them. If that's you, add this computer from one of yours: `svrngs link Lake --device` there makes a link it joins as you by. Else join under another name, with `--as <your name>` after the link", then "svrngs is in …, and did not join the group: it said why above."; no LaunchAgent kept, no home page opened; the people Sam and `ana`; Sam, opening Ana's Navidrome through the lend, signed in as `Sam-f7f70ba9d76b657c`, `isAdmin: false`, Sam's `x-svrngs-user`; Navidrome's users after it `Sam-f7f70ba9d76b657c` and `ana` (`data/name.json`) | held; the users were read after Sam's open made Sam's account, so `ana` is not alone — no second `ana` |
| 11 | Apart | every computer on this host | never-ran |
| 12 | every people bar | no person | never-ran |

**The newcomer's minute is now the line's.** Of each newcomer's
12,736–13,762 ms, the line took 4,272–4,553 ms, the home page listed both
lends 346–403 ms after it, and the film, the album and the photo took the
rest. The dry run's
45–50 s of waiting for the issuer's round is gone, as
bash-first-minute (b) measured on three computers
(`demo/runs/first-minute/2026-10-12/`, "After (b)").

## What it found

Nothing here is fixed in this run's commit; each is written for a row.

1. **The people are listed in key order.** "alex, bo, ana, dee, Sam": the
   order of each person's `person_key` (`6b95…`, `ae3d…`, `af03…`,
   `b24c…`, `b4bb…`, `accounts.x_svrngs_user`), neither by name nor by when
   they joined. The home page joins `/v1/rail/people` as it comes
   (`crates/svrngs-node/src/home/home.js:64`). Harmless to read, but an
   order nobody chose; bash-same-name, which makes the people list tell two
   people of one name apart, is where it would be decided.
2. **The taken-name refusal ends without a full stop** ("… with `--as <your
   name>` after the link", `crates/svrngs/src/cli.rs:467`), where every
   other refusal the line prints ends with one.
3. **The load was 40–108 on 12 cores**, against the dry run's 4.9–5.4.
   Every number above is a floor under that load; the phone's film played
   12,255 ms after its play (the dry run's 13,902 ms,
   `dry-2026-10-12/data/summary.json`, `cy.film.playing`), the computers' 1,861–2,004 ms, and one run does not say how
   much of either is the load.

## What it leaves out, each said

- **People**, every bar's. The clocks start when Chromium opened the
  link, and nothing waited between steps.
- **The network.** Everything, the page and the tarball included, came over
  this host's loopback; the published page was not touched. Apart and the
  two homes never ran.
- **A chat.** The preview was fetched as Discord's fetcher names itself.
- **macOS's dialogs and the LaunchAgent at login**: each line's LaunchAgent
  was booted out once its person was done.
- **The full name.** These logins have no account on this host (`id -F
  alex`: "no such user"), so `join` took `$USER`; that a person's full name
  is taken where the system shows one is bash-person (e)'s test, not this
  run's.
- **A lend's limit at six at once**: on one host the door counts no
  computer (row 9).
- **Ana in Ana's own Navidrome through svrngs**: as `ana-af03…`, an
  account beside the admin `ana`; not opened here.

## For Demo C

Every finding of the dry run the queue took on held here: the lends listed
within a second of the line, nothing up said not to answer, each person
their own Navidrome account by the door's name, a second `ana` refused and
told how to add a device, the founder's folder from `new`, no lend refusing
anyone with no limit, and Ana's Navidrome set as `share` prints. What
stands between this and Demo C is its last unit, the two homes (DESIGN
§8), the phone's typed name (the operator's), and publishing this build:

```
scripts/deploy-page.sh        # the operator's: builds the page and the four tarballs into one site, then vercel deploy --prod
```

written for the operator, never run here.
