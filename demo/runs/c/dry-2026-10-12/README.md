# Demo C's dry run — this host, against the published page, 2026-10-12

The operator, 2026-10-12, once `scripts/deploy-page.sh` had published the
page: "can you do a run of the demo?" Demo C (`demo/c.md`) is five people
in their own homes; this is its steps on one Mac, every person a program,
against https://group.svrnme.sh/. Predictions first, in `PREDICTIONS.md`,
committed before any data (`b1d8f40`). It is not Demo C's run: no person
did anything, so no people bar is judged, and no record goes in
`runs/c/` (AGENTS.md 7). One host, one run: not a rate.

## How it ran

```
W=<scratch>/c-dry MEDIA=<scratch>/c-media python3 demo/runs/c/dry-2026-10-12/run.py   # data/, 06:34–06:45 local
W=<scratch>/c-dry MEDIA=<scratch>/c-media python3 demo/runs/c/dry-2026-10-12/name.py  # data/name.json
```

`run.py` and `c.mjs` at this file's commit, as they ran, but for one
change between two takes (below). macOS 27.0.1, 12 cores, the 1-minute
load 5.4 at the start and 4.9 at the end (`data/events.jsonl`, `start`,
`end`). The setup is `PREDICTIONS.md`'s:

- **The page and the binary** are the published ones: `SHA256SUMS` and
  the darwin-aarch64 tarball fetched from the page, 12,112,144 bytes, its
  sum matching (`data/summary.json`, `published`); Sam's and Ana's
  svrngs unpacked from it; every newcomer's line fetched `install.sh` and
  the same tarball from the page and checked it ("Checked:
  svrngs-darwin-aarch64.tar.gz matches SHA256SUMS", `data/*-install.out`).
- **Sam's library**: Jellyfin 12.0.0 (the macOS app's server) with Demo
  A's three films, `apps/jellyfin/mw` on 8097, `svrngs share 8096
  --sign-in 8097`. **Ana's music**: Homebrew's Navidrome 0.64.2 with three
  albums of three tracks, `ND_EXTAUTH_USERHEADER=X-Svrngs-Person`,
  `ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32`, Ana its admin. **The trip**:
  30 JPEGs, 144,567,678 bytes, made here.
- **Each person** a `HOME` of its own with `USER` their login; one
  `install.sh` at a time, each person's svrngs taken out of launchd and
  run by the driver once their clock was done (`taken-over` in
  `events.jsonl`, 426–3,398 ms each); **asleep** is `SIGSTOP` to Ana's
  svrngs and Navidrome.

**Two takes.** The first stopped at Ana's lend: `svrngs share 4533 --name
music`, as the predictions had it, was refused (finding 2). The driver was
changed to try that, keep its answer (`share_first`), then lend as svrngs
said, `--kind navidrome`, and the music's link words to match; the second
take is every number here. The first take's data was overwritten by the
second's, which repeats its refusal word for word.

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | the link on the page; its preview the invitation; the line within 10 s; the tarball's sum matches | `new` printed `https://group.svrnme.sh/#<the token>` (`data/sam-new.out`); fetched as Discord's preview fetches it, 200, title "You’re invited to a group" and its description (`summary.json`, `preview`); the line shown 657–1,401 ms after the link opened; the sum matched | held |
| 2 | Ana in as `ana` by the line, the folder at `~/svrngs/lake-trip`; `share` asks a password and prints nothing of Navidrome's settings | the line exited 0 in 3,921 ms, the folder made; `share 4533 --kind navidrome` asked "If navidrome takes a password on each request, paste what it takes …" (Enter given) and named neither setting (`summary.json`, `ana.share`). Not predicted: `--name music` alone was refused (finding 2) | held, with finding 2 |
| 3 | each of Alex, Bo, Dee within 5 min, from the link to film, album and photo; neither app asking a sign-in | **63,987, 59,609 and 58,935 ms.** The line done at 5,271–6,020 ms; the home page listing the group, its folder and both lends at 50,948–55,875 ms; the film playing 1,923–2,048 ms after its play, the album 1,422–1,432 ms after its play; neither app asked anyone to sign in; Ana's `IMG_0001.jpg` on each disk, the same bytes, opened as a 4032×3024 image by the time it was looked for (`summary.json`, `newcomers`) | held, the machine's floor |
| 4 | the logins as names; Navidrome's users named so | the home pages' people: "Sam, ana, alex, Cy, eve, dee" (`asleep.eve.home.said`); Navidrome signed in `alex`, `bo`, `dee` and `Cy` as themselves (`music.as`) | held |
| 5 | Cy's phone: the group within 10 s; a name asked and typed; film and album within 2 min | the name asked at 4,386 ms; in, "You’re in lake-trip, as Cy", at 6,779 ms, both lends listed with it; the film playing at 28,983 ms, the album at 39,730 ms, signed in to Navidrome as `Cy`, neither asking a sign-in (`summary.json`, `cy`) | held, the machine's floor; the typed name is the operator's (finding 5) |
| 6 | removed within 30 s: both lends refuse Bo, Bo's home page says so; Sam's photo after reaches all but Bo; Bo's remove counts nowhere; Bo keeps the trip | `svrngs remove bo --group lake-trip` said "bo is no longer in lake-trip. …" in 16 ms; Bo's computer's address for each lend answered 403 `removed`, "You are no longer in this group, so nothing of it is open to you.", at the first ask, **0.031 s** after; Bo's home page said "You are no longer in lake-trip. …" at 677 ms; Sam's `after.jpg` on Ana's, Alex's and Dee's disks in 0.82–0.83 s, on Bo's not in 60 s; Bo's `svrngs remove Sam` refused, "You are no longer in lake-trip." (exit 1), and every standing list after reads Cy, Sam, alex, ana, dee; Bo's disk kept all 30 photos (`summary.json`, `removed`) | held |
| 7 | asleep: Eve has all 30 within 2 min of being in; Eve's home page says Ana's computer is not answering; Sam's jellyfin answers everyone | Eve in at 5,891 ms; **all 30 on Eve's disk, each the same bytes, 8.7 s after**; Eve's film playing at 14,357 ms; Eve's and Alex's home pages said "ana's computer did not answer, so what they lend is not listed."; the jellyfin lend answered 200 to Alex's, Dee's and Eve's computers (`summary.json`, `asleep`) | held; when the pages first said it is not measured (below) |
| 8 | Jellyfin: `sam` and one account the middleware made for each who played; Navidrome: `ana` and one made from the header for each; no sign-in, no key asked | Jellyfin `["sam"]` before, after `sam` and `alex-…`, `bo-…`, `Cy-…`, `dee-…`, `eve-…`, each `<name>-<16 hex of a key>`; Navidrome `["ana"]` before, after `ana`, `alex`, `bo`, `Cy`, `dee` (`summary.json`, `accounts`); no newcomer was asked for anything but Cy's name | held |
| 9 | Apart | every computer on this host | never-ran |
| 10 | every people bar | no person | never-ran |

**The removal is Bo's own computer's, as measured here.** After the cut no
request of Bo's reached a lender's door: the scratch dir's debug logs
carry Bo's key last at 13:38:37.634 UTC on Sam's door and .636 on Ana's,
and Sam's told Bo's computer its cut at .657 ("removal: the removed
computer told its cut", `data/sam.log`); every ask after was refused at
Bo's own door. That a lender's door refuses a removed key that dials
it anyway is the door test's,
`crates/svrngs-node/tests/door/main.rs:1164-1173`, not this run's.

**When "did not answer" was first said is not measured.** `c.mjs read`
waited for "ana’s computer did not answer", with a typographic
apostrophe; the home page writes it with a plain one
(`crates/svrngs-node/src/home/home.js`, "'s computer did not answer"), so
the wait ran its 120 s and the words above are its last look. That the
pages said it, held; how soon after the lid shut, could-not-judge.

## What it found

1. **A name is not a person, and Navidrome makes it one** (`name.py`,
   `data/name.json`). A second member joining Sam's group `--as ana` is
   let in — "You're in Lake, as ana.", the people then "Sam, ana, ana" —
   and opening Ana's Navidrome from that computer is signed in as `ana`,
   `isAdmin: true`. The door sends the person's name
   (`crates/svrngs-node/src/door.rs:1185-1200`, `X-Svrngs-Person`), a
   name is whatever the joiner gives (`--as`, `$USER`, or what a phone
   types), and nothing refuses one already standing; DESIGN defers
   "another device for the same person" to §9
   (`docs/DESIGN.md:1812-1813`). So any app that takes the person from
   the header — DESIGN §1's Navidrome and Calibre-Web — gives whoever
   holds the link and types a member's name that member's account, the
   lender's included. Jellyfin's middleware does not: it names each
   account `<name>-<16 hex of a key>` (`accounts` above). In Demo C the
   link sits in a Discord channel. The operator's call before Demo C:
   a group refusing a name already standing, the header naming a key's
   person uniquely, or Navidrome left out.
2. **Navidrome is lent as svrngs says, not as `demo/c.md` says.**
   `svrngs share 4533 --name music` is refused: "say what is on port
   4533: `svrngs share 4533 --kind <kind>`" (`summary.json`,
   `ana.share_first`); `--kind navidrome` lends it, listed as "ana’s
   navidrome". It asks Ana a password on each request (Enter alone for
   none) and prints nothing of `ND_EXTAUTH_USERHEADER` or
   `ND_EXTAUTH_TRUSTEDSOURCES`: `crates/svrngs/src/well_known.txt` has
   Jellyfin's row alone, and `demo/c.md`'s "as `svrngs share` prints
   them" was never built. Ana has to know both settings from elsewhere.
   `demo/c.md` is corrected to say so in this commit.
3. **Three at once.** Each lend took the default, "up to 3 people at
   once" (`summary.json`, `sam.share`; `crates/svrngs/src/share.rs:36`).
   Here the newcomers went one at a time; in Demo C six people may open
   Sam's library in one evening, and `demo/c.md` names no `--limit`.
   Untested here.
4. **The names people see are logins.** "Sam, ana, alex, Cy, eve, dee":
   the two who gave a name capitalised, the four the line named by
   `$USER` not; Sam removes `bo` as Bo's computer's login, which Sam reads
   off the home page (onboarding's finding 1, `runs/onboarding/2026-10-11/`).
5. **The phone asks a name, and it is typed.** The gate's "one line …
   the only thing typed" is the computer's; on the phone the page asked
   Cy's name at 4.4 s (`cy.asked_name`). Whether that is inside Demo C's
   "nothing typed" is the operator's.
6. **"Did not answer" is said of computers that are up.** Eve's home
   page, the first time it listed the group 9.2 s after the paste, said
   alex's and dee's computers did not answer — both were running — beside
   ana's, which was asleep (`asleep.eve.home.said`); its later looks said
   ana's alone. A newcomer's first look can tell them a member's computer
   is off when it has not been asked yet.
7. **The founder has no folder from `new`.** `join` places the group's
   folder (`crates/svrngs/src/cli.rs:1203`, `folder_at_join`); `new` does
   not, so Sam needed `svrngs share --folder ~/svrngs/lake-trip --group
   lake-trip` before step 4's "a photo Sam puts in the folder". Not in
   `demo/c.md`'s steps.
8. **The newcomer's minute is mostly waiting for the lends.** Of 59–64 s,
   the line took about 5 s and the three things about 8 s; the home page
   listed both lends at 51–56 s, as u7's joiner did (52.6 s,
   `runs/u7/2026-10-11/take-2/events.jsonl:12`). Eve, who needed Sam's
   alone, had it at 9.2 s. Where the rest goes is not split here.

## What it leaves out, each said

- **People**, every bar's. The clocks start when Chromium opened the
  link, and nothing waited between steps.
- **The network.** The page, `install.sh` and the tarball came over the
  internet; every svrngs byte between computers came over this host's
  loopback. Apart and the two homes never ran.
- **A chat.** The preview was fetched as Discord's fetcher names itself;
  Discord itself never showed the link.
- **macOS's dialogs and the LaunchAgent at login**: each line's
  LaunchAgent was booted out once its person was done.
- The driver wrote Navidrome's version over Jellyfin's in
  `summary.json`'s `apps.version` (both keyed `version`); Jellyfin is the
  macOS app's 12.0.0, as Demo A's runs.

## For Demo C

Navidrome works as Ana's app through a lend — every person signed in as
themselves from the header, nothing typed — with finding 1 to decide
first. Then finding 2's settings in Ana's instructions, a `--limit` for
six (3), and the two homes, Demo C's last unit (DESIGN §8).
