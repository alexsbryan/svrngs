# Demo C's dry run with n0 out of the path — this host, one svrngs relay, 2026-10-13

Row relay-run (`ralph/next/relay/STATE.md`): Demo C's dry run again,
`demo/runs/c/dry-2026-10-12-rerun/`'s steps, with every computer and the
phone's page on one `svrngs relay` started on this host by its own
`relay::run` at 127.0.0.1, and n0 out of the path. Predictions first, in
`PREDICTIONS.md`, committed before any data (`419590a`); the driver
committed before it ran (`82b21f5`); the data after (`6356e3c`). It is not
Demo C's run: no person did anything, so no people bar is judged, and no
record goes in `runs/c/` (AGENTS.md 7). One host, one run of each take: not
a rate.

## How it ran

```
demo/runs/c/no-n0-2026-10-13/media.sh /tmp/c-relay-media && \
  W=/tmp/c-relay MEDIA=/tmp/c-relay-media python3 demo/runs/c/no-n0-2026-10-13/run.py   # data/, exit 0, 13m05s
```

Started by the loop (`ralph-result await`), 01:01:52–01:14:34 local
(`data/events.jsonl`, `start` and `end`). `run.py`, `c.mjs` and `media.sh`
at `82b21f5`, as they ran, in one take; the build is `82b21f5`'s
(`events.jsonl`, `start`). macOS 27.0.1, 12 cores, **the 1-minute load
34.4 at the start and 17.6 at the end** (`start`; `summary.json`, `end`),
33.7–40.8 as each newcomer finished and 20.1 as Eve did
(`newcomers.*.load`, `asleep.eve.load`) — the rerun's was 31.5–38.6 and
108.1. The setup is `PREDICTIONS.md`'s:

- **The relay**: `the_verbs_relay` (`crates/svrngs/tests/n0_off.rs`, built
  `--release`, binary `n0_off-b3848139f63e6439`, `summary.json`,
  `published.built.relay`) serving `relay::run` at
  `http://127.0.0.1:57771/`, a process of its own, the one port for both
  takes (`summary.json`, `relay.at`).
- **The page and the binary** built from this commit and served on this
  host at `http://localhost:57849/`; the darwin-aarch64 tarball 13,140,753
  bytes, its sum matching the `SHA256SUMS` beside it (`published`).
- **Sam's library, Ana's music, the trip**: the rerun's — Jellyfin 12.0.0
  through `apps/jellyfin/mw`, Navidrome 0.64.2 started again with what
  `svrngs share 4533` printed, 30 JPEGs of 146,488,940 bytes (`apps`,
  `ana.navidrome_set`, `ana.photos`).
- **n0's addresses** by `dig` at the start: its four relays, two staging
  relays and two lookup hosts, sixteen addresses, with `relay.svrnme.sh`'s
  two beside them (`summary.json`, `n0` and `ours`).

## Against the predictions

| | predicted | as it ran | |
|---|---|---|---|
| 1 | the relay's words; the link within 10 s; every computer's trace names that relay alone | "Relaying at http://127.0.0.1:57771/ for every computer and phone that reaches it, each sending at most 70 Mbit/s. Leave this running." (`relay.open`); `new` printed the link on `http://localhost:57849/` (`data/sam-new.out`), its preview 200, "You’re invited to a group" (`preview`); Ana's, Alex's, Bo's, Dee's and Eve's traces say "the relays this computer meets its groups at, n0's none of them relay_urls=["http://127.0.0.1:57771/"]" (`data/<who>.log`, its first `cli:` line); Sam's says the same in its whole trace (`/tmp/c-relay/sam/.svrngs/svrngs.log`; `data/sam.log` keeps the last 4,000 lines of its filter, which begin after it) | held |
| 2 | the rerun's steps 1–8 on its bounds | Ana in by the line, exit 0 in 9,730 ms (the rerun's 5,692; `ana.line_ms`); each `share` "as many at once as want it" in 16–17 ms (`sam.share`, `ana.share`); each newcomer done well inside 5 min (row 3); Navidrome naming each `<name>-<16 hex>` (`newcomers.*.music.as`); Bo removed, refused 403 `removed` at the first ask, 0.036 s after (`removed.refused_removed_s`), told on Bo’s home page at 591 ms; ana asleep said of `ana` alone (row 5); one account per person per app (`accounts`) | held |
| 3 | each newcomer done within 20,600 ms; the line done within 7,000 ms; both lends listed within 1,000 ms of the line's exit | done at **20,548, 18,519 and 17,933 ms** (Alex, Bo, Dee; the rerun's 13,762, 12,831, 12,736); the line done at **8,593**, 6,678 and 6,910 ms (the rerun's 4,272–4,553); the lends listed **538, 991 and 976 ms** after the line's exit (the rerun's 346–403); the film playing 2,144–2,660 ms after its play (the rerun's 1,861–2,004), the album 1,554–1,869 (1,408–1,431); no sign-in asked (`newcomers`) | Alex's line missed, by 1,593 ms; the rest held. Each step slower than the rerun's, near half again; finding 4 |
| 4 | Cy: the name asked and in within 10 s; both lends; the film sooner after its play than the rerun's 12,255 ms; the album within 2 min; `Cy-<16 hex>` | the name asked at 2,169 ms, "You’re in lake-trip, as Cy" at 3,069, both lends listed at 3,075; **the film playing 3,200 ms after its play**; the album playing, Cy done at 14,529 ms; signed in to Navidrome as `Cy-70d804bd8735711b` (`cy`) | held |
| 5 | `after.jpg` on Ana's, Alex's, Dee's disks within 5 s, never Bo's; all thirty on Eve's within 30 s, Ana stopped | 0.52, 0.58 and 0.62 s, never Bo's (`removed.after_jpg_s`); Bo's disk kept all 30 (`removed.bo_keeps`); **all thirty on Eve's 9.7 s after Eve was in** (`asleep.all_thirty_s_after_eve_in`; the rerun's 9.6); Eve's and Alex's home pages said "ana's computer did not answer, so what they lend is not listed." and named nobody else (`asleep.unanswered`) | held |
| 6 | no n0 address in any sample, nor `relay.svrnme.sh`'s; a `svrngs` off this host only to the router; the phone asks nothing off this host; no trace names n0 | **561 samples, `n0_hits` and `ours_hits` empty** (`summary.json`, `lsof`; each (process, protocol, remote) once in `data/lsof.jsonl`). Off this host, each `svrngs` reached only `192.168.1.1:5000` (iroh's port mapper); the relay, the page's server, the middleware and WebKit nothing; Jellyfin four addresses and Navidrome three, each on 443, their own lookups; Playwright's Chromium helper two of `142.251.219.x` on 443; the other `?` rows are this host's programs the run did not start, sampled with the rest. Playwright listed no request of the phone's refused in either take (`cy.away`, `group.cy.away`, `group.gus.away`: `[]`). No trace matched `iroh\.link\|pkarr\|relay\.n0\|relay\.svrnme\.sh` across the 18,841 lines `traced_lines` counts, summed (`n0_in_traces`) | held |
| 7 | the group's relay's words; within 90 s it admits Sam, ana, alex, dee, eve and refuses Bo | "Relaying at http://127.0.0.1:57771/ for the computers and phones of the groups this computer is in. Leave this running." (`group.said`); within 0.3 s of it, "relay: admitted — the key stands in a group held here" for Sam, alex, ana, dee and eve, each with their person, and "relay: refused — the key stands in no group held here" for Bo's key `d525…` (`group.first`, `group.decisions`; `data/relay-group.log.gz`), refused 28 times in all | held |
| 8 | `after2.jpg` on four disks within 60 s, never Bo's; Alex's home lists both lends; Cy's phone, opened again, admitted and playing Sam's film within 2 min | 0.96–0.98 s on Alex's, Eve's, Ana's and Dee's, never Bo's (`group.after2_jpg_s`); Alex's home page listed Sam's jellyfin and ana's navidrome at 841 ms (`group.alex_home`); Cy's phone admitted (`70d8…`, Cy, `group.decisions`), in, "You’re in lake-trip, as Cy" at 2,330 ms, both lends listed at 2,364 — **and the film never played**: its address showed "Open this page from your group’s link. What’s your name? … Join", the music "Opening…" and no more (`group.cy.film.stopped`, `music.stopped`), while the relay refused a key `2b0c…` 14 times from 0.7 s after Cy's admission to 65 s later (`relay-group.log.gz`) | **failed**: finding 1 |
| 9 | Fay's key refused; svrngs blames the sender; exit non-zero, Fay not in. Gus refused; the page's words within 70 s, Gus never in | Fay's key `bfd6…` (by Fay’s `identity`) refused 11 times; the line said "svrngs: this computer has not reached http://127.0.0.1:57771/ in 10 s, where its group reaches it; it goes on, and keeps trying.", then "svrngs: the computer that sent this link cannot be reached; it may be off, so try again later", exit 1 at 24,385 ms; the people after, Cy, Sam, alex, ana, dee, eve (`group.fay`). Gus's phone: a key `d892…` refused 17 times while it ran; the page said "The computer that sent this link did not answer through 127.0.0.1:57771. Try again later." at 61,555 ms; the people after the same (`group.gus`) | held; both words blame Sam's computer, not the relay that refused them (finding 2) |
| 10 | Apart, every people bar, the network; the public half | every computer and the relay on one host; no person | never-ran; the public half written below, never run here |
| 11 | a packet capture | `sudo` | never-ran, the operator's |

## What it found

Nothing here is fixed in this run's commit; each is written for a row.

1. **A relay for the group alone lets the phone in and plays it nothing.**
   The page opens each lend from a fresh key holding a pass the person's
   key signed (`web/app.js:22-25`, `web/lend.js:9`), so no lend's origin
   holds the person's key; that fresh key stands in no group, so
   `relay::Admits` refuses it (`crates/svrngs/src/relay.rs:83-99`). The
   open take shows the keys: Cy's `70d8…` and, as the film and the album
   opened, `98a2…` and `f7f5…` (`data/relay-open.log.gz`, 08:04:09.9 and
   08:04:17.0 UTC), keys of nobody's. In the group's take, `2b0c…` refused
   14 times as Cy's film opened. GROUP_RELAYS §4.3 names a newcomer's key
   not yet synced; a pass holder is the case it does not name. And the
   phone is not told: the film's address showed the group's join page,
   "What’s your name?", in the film's place (AGENTS.md 6), and its page
   logged "FetchEvent.respondWith received an error: TypeError:
   importScripts cannot be used if worker type is "module"" and "Cannot load
   http: /apkp…localhost:57849/." (`group.cy.logs`), neither seen in the
   open take (`cy.logs`: `[]`); whether the refusal is what led there is
   not traced in this run. svrngs's own relay is `--open`, so a group on
   the default is untouched; a group that runs its own relay admitting
   only the group plays nothing to a phone.
2. **A newcomer refused at the group's relay is told the sender is off.**
   Fay's line and Gus's page each say the computer that sent the link did
   not answer, while the relay refused their keys by name, traced. As
   predicted (prediction 9), and words a person would act on wrongly:
   wait for Sam, when Sam's computer was up the whole time.
3. **n0 is out of the path.** No process the run started opened an
   address of n0's, nor any address off this host but iroh's port mapper on
   the router, in 561 samples across both takes; no trace names n0.
   lsof cannot see a DNS name asked; a capture would, and needs `sudo`.
4. **Every step was slower than the rerun's**, the loads near each other
   (33.7–40.8 here, 31.5–38.6 there): the line done 2.2–4.0 s later with
   the paste in (Alex's line shown at 1,843 ms, the rerun's 410), the
   lends' listing 192–634 ms more after it, the film's start 0.3–0.7 s
   more, each newcomer 5.2–6.8 s more in all (each against the same
   person's in `dry-2026-10-12-rerun/data/summary.json`, `newcomers`). One run of each:
   not a rate, and not traced to the relay; the phone's film, opposite,
   started in 3,200 ms against the rerun's 12,255 through n0's relay.

## What it leaves out, each said

- **People**, every bar's; the network, Apart and the two homes; a chat;
  macOS's dialogs and the LaunchAgent at login; `name.py`'s second `ana`
  (the predictions' "Left out"). As in the rerun.
- **The verb as typed**: `svrngs relay <name>` serves 80, 443 and 7842 on
  every interface with Let's Encrypt's certificate, a port opened on this
  host's network (fence 7); the run served its `relay::run` on loopback.
- **A DNS name asked**: lsof sees sockets, not lookups.

## The public half — the operator's run, never run here

`relay.svrnme.sh` today answers with Vercel's wildcard, `216.198.79.1` and
`216.198.79.65` (`summary.json`, `ours`): nothing of ours is there yet. In
order:

```
scripts/deploy-relay.sh --ssh-key <fingerprint>     # the droplet, svrngs relay relay.svrnme.sh --open, its A/AAAA, waits until it answers
scripts/deploy-page.sh                              # refuses until relay.svrnme.sh answers; then the page and the four tarballs
svrngs new lake-trip --as Sam                       # on a computer at home: meets its group at relay.svrnme.sh, the default
```

Then a real phone with Wi-Fi off, on cellular, opens the link Sam's
`new` printed, joins with its name and plays Sam's film, while on the
droplet `journalctl -u svrngs-relay` shows the phone's key admitted (open)
and Sam's computer's `svrngs.log` "door: let in" the phone. Its bars are
this run's prediction 4 on a carrier's network — in within 10 s, the film
within 2 min — and they are judged there, not here.
