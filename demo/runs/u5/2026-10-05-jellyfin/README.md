# Stock Jellyfin through a lend, measured — 2026-10-05

u5-3 (DESIGN §3, "Lent apps", calls 2, 4 and 5; §11 risk 7): what stock
jellyfin-web does through the door before anything is built on it, and
which way past its sign-in u5-4 builds. This section is committed before
any run; the data, and the decision it gives, come after it in later
commits.

## The rule, stated before the data

**Zero sign-in** — the lender's computer makes each person their own
Jellyfin user, through Jellyfin's API with a key the lender gives once,
and the door signs each person in as theirs — **is decided if and only if
both hold**:

1. *It fits in at most two rows*, u5-4 counted among them. A row is one
   piece of work with a check of its own landing in one place
   (`apps/jellyfin/`, the door, the page or the lend's worker, the
   command). Every piece the data shows a person needs to get from the
   list to a playing film with nothing typed is counted, and Demo A's
   "Jellyfin's list of users unchanged" clause, which users made during a
   run break (DESIGN §3, call 4), counts as one more unless it is the
   operator's alone.
2. *It adds no Jellyfin noun outside `apps/jellyfin/`*: every piece outside
   it is general — any app could use it — and names nothing of Jellyfin's
   API (`apps/jellyfin/nouns.txt`, and any of its routes, headers, storage
   keys or fields).

**Else sign in once**: the lend carries no secret, each person signs in
with an account the lender made, and zero sign-in goes to DESIGN §9 with
the trigger that brings it back. If sign in once also fails its bars
below, neither is decided and the question goes to the operator
(`ralph/next/u5/ctl/NEEDS_HUMAN.md`).

## What is measured, and the bar each is read against

Against a scratch Jellyfin 12.0.0 on this host (as U3's dry run,
`demo/runs/a/dry-2026-10-04/dry.sh`), lent with `svrngs share 8096` and
**no secret**, two accounts the lender made, from Alex's computer (its
browser at the lend's origin on Alex's loopback) and from the page (a
phone joined by the group's link), each in Playwright's Chromium and
WebKit:

- **(a) Sign in, play, seek; two people at once.** Two people, each in a
  browser of their own, each signs in with their own account and plays a
  different film. Plays: the `<video>` moves within 30 s of play. Seeks:
  one seeks five times, each playing again within 5 s (Demo A's bar);
  then the other seeks three times while the first plays. Left alone: the
  person not seeking has no `emptied` or `loadstart` on their `<video>`,
  never goes back more than 2 s, and never stands still 5 s, while the
  other seeks; and both are still signed in afterwards. One film plays as
  it is; the other is in a form no browser plays, so Jellyfin converts it,
  and the play it converts is the one watched while the other seeks.
- **(b) The live socket.** jellyfin-web's socket opens through the lend,
  and a message the lender's Jellyfin sends that person's session arrives
  on their page within 10 s.
- **(c) A secret in a byte received.** Every response header and body
  each browser receives is searched for the lender's password, the
  lender's own token, the other person's password and token, and every
  token the lend holds. With no secret, the lend holds none; so the same
  search is also run on a lend that carries a token, asking Jellyfin's
  playback info and its converted stream's playlists through the door,
  which is what zero sign-in would put through it (e0-bot's unmeasured
  claim, DESIGN §8, U5's inventory).
- **(d) What zero sign-in would take.** A user made per person through
  Jellyfin's API with the lender's key, signed in, and that person's token
  carried by the door on a lend opened with nothing typed: whether
  jellyfin-web reaches a playing film, and if not, what more it needs,
  each piece sized as a row under the rule above.

One host, one run of each: not a rate.

## How it ran

Everything above this heading was committed before any run (`9a637e5`).
The instrument is `measure.sh` and what it calls, as committed at
`4b23bb4`; the data is `data/`, committed at `f42f25f`; product code as at
`a24e2ed`. Jellyfin 12.0.0 (the macOS app's server); Playwright 1.59.1,
Chromium 147.0.7727.15 and WebKit 26.4 as an iPhone 15; both computers'
`svrngs` reached each other, and the phones reached Sam's, through iroh's
public relays (Sam's door: `relayed=Some(true) rtt_ms=Some(79)` for a
phone), since the command takes no relay of its own.

```
cargo build -p svrngs
D=demo/runs/u5/2026-10-05-jellyfin
$D/measure.sh up /tmp/u5-3 $D/data
$D/measure.sh pair /tmp/u5-3 $D/data computer chromium   # and webkit; phone chromium; RUN=<n> for phone webkit
$D/measure.sh probe /tmp/u5-3 $D/data
python3 $D/scan.py /tmp/u5-3 $D/data
$D/measure.sh down /tmp/u5-3 $D/data
```

`data/steps.log` is the order. Each pair is Ana, who plays the film
Jellyfin plays as it is (H.264 in MP4, `stream.mp4?Static=true`) and
seeks it five times, and Ben, who plays the one it converts for every
browser (MPEG-2 in Matroska, `master.m3u8`) and seeks it three times;
each signs in as `ana` or `ben`, accounts the lender made.

**The instrument, failing first.** In the first phone and WebKit pair
(`pair-phone-webkit-1.json`) and two more (`-2`, `-3`), Ben's converted
film never started: his `<video>` held the film buffered, paused at 0
(`-4`: buffered 0–120 s, `readyState` 4). Alone, on a phone in WebKit, it
started in 5.9 s. Those runs waited for playing with Playwright's
`page.waitForFunction`; a watch on every `play()` (`drive.mjs`) caught
WebKit refusing jellyfin-web's with `NotAllowedError` under that wait
(`-9-oldwait.json`, run again after the fix to reproduce it), and with
the wait done by `page.evaluate` every 250 ms the film started in all four
runs (`-5` to `-8`; `-8` without the `play()` watch, so the watch changes
nothing). Why the one wait leads WebKit to refuse is not shown; the
instrument is the second, and the pairs on a computer and on a phone in
Chromium were run again with it (`steps.log`, 01:43–01:45).

## What it found

1. **(a) With the lend carrying no secret, stock jellyfin-web works for
   two people at once, each signing in with their own account.** From
   Alex's computer (`pair-computer-chromium.json`, `-webkit`) and from a
   phone joined by the link (`pair-phone-chromium.json`,
   `pair-phone-webkit-5..8.json`), each person is asked "Please sign in"
   (`asked`), signs in within 2.9 s, and their film plays within 15 s of
   play (1.4–3.4 s on a computer). Seeks through jellyfin-web's own seek
   bar played again within 5 s in 55 of 56 (on a computer 353–575 ms;
   on a phone up to 2,185 ms but one, Ana's fourth in `-5`, 7,871 ms). The
   person not seeking was left alone in all 14 phases: no `emptied`, no
   `loadstart`, no new element, never back, never still 5 s; the
   converted play included. Each browser keeps its own device
   (`devices_apart`), and both were still signed in after.
2. **(b) The live socket works, both ways.** On a computer each page opens
   `/socket` through the lend and receives Jellyfin's frames
   (`received.sockets`, `frames`); on a phone it goes by the shim. A
   message the lender's Jellyfin sent each session appeared on that
   person's page within 10–61 ms in every pair that counts (`socket`).
3. **(c) No byte a person received holds anyone else's secret.** In all
   12 pair runs, each browser's every header, body and socket frame
   (`scan.json`, up to 76 MB a person) holds none of the lender's password
   or token, nor the other person's password or token, and neither
   computer's `svrngs` output holds any (`scan.json` `logs`). Each person
   receives their own token: once in the sign-in's answer, and for the
   converted film 43 more times, in `PlaybackInfo`, `master.m3u8` and
   `main.m3u8`. **So a token the lend carries reaches the browser**: on a
   lend carrying zed's token, it was in 43 places of what the browser
   received, in Chromium and in WebKit (`scan.json`,
   `keyed-*-placeholder`), and the door, which drops only a response
   header holding the secret (`crates/svrngs-node/src/door.rs:555-560`),
   reads no body. e0-bot's claim holds for Jellyfin 12.
4. **(d) What zero sign-in would take** (`probe.json`,
   `probe-*.json`). The lender's token, or an API key the lender makes,
   makes a non-admin Jellyfin user per person (200 each). A lend carrying
   that person's token signs the door's `/Users/Me` in as them; yet
   jellyfin-web, opened with nothing typed, asks "Please sign in" in both
   engines, because what it is signed in as lives in its own storage at
   the lend's origin (`jellyfin_credentials`). Written there as U3's dry
   run's driver wrote it, it plays and seeks. And Jellyfin then sees the
   lend's device, not the browser's: the lender's message found no
   session of the browser's (`socket.theirs_by_device`:
   `svrngs-lend-zed`). The pieces, each a row:
   - a Jellyfin user per person, made by the lender's computer with a key
     the lender gives once: `apps/jellyfin/` (U5's inventory,
     `mesh_media/viewer.rs` `provision`);
   - the door carrying a secret per person, not per lend: today a lend has
     one secret (`door.rs:501-520`, `lend::secret_dir` per lend), so every
     person is one device, and one person's sign-in logs it out for
     everyone (U3's dry run). General;
   - jellyfin-web's stored sign-in written at the lend's origin before its
     page loads, with nothing typed: a script an app supplies, run at its
     lend's origin, on a computer (`crates/svrngs-node/src/origin.rs`)
     and on a phone (`web/lend.js`, `web/open.js`). General in the infra,
     the script in `apps/jellyfin/`;
   - the person's token kept out of the bodies Jellyfin writes it into
     (finding 3): the door rewriting a streamed body, and the addresses it
     rewrites still working. General.
   Four rows, and Demo A's "Jellyfin's list of users unchanged", which
   users made during a run break, is the operator's bar (DESIGN §3, call
   4), not a row.

One host; seven pairs that count, four of them on a phone in WebKit; one
probe. Not a rate.

## What the rule gives

**Zero sign-in is not decided**: it fails the rule's first condition,
four rows where at most two are allowed (finding 4). Its second condition
could hold: every piece outside `apps/jellyfin/` is general.

**Sign in once passed every bar but one seek**: in `pair-phone-webkit-5`,
Ana's fourth seek played again in 7,871 ms; the bar is 5 s. Every other
bar held in every run (findings 1–3). The rule, as committed before the
data: "If sign in once also fails its bars below, neither is decided and
the question goes to the operator." That seek is direct play through the
phone's lend over iroh's public relay, bytes zero sign-in would carry the
same way; DESIGN §11 risk 4 has WebKit's seeks through n0 at up to 3.4 s.
Whether it counts against sign in once is not this row's to judge after
the data: it is asked in `ralph/next/u5/ctl/NEEDS_HUMAN.md`, with the
seat's recommendation, sign in once.
