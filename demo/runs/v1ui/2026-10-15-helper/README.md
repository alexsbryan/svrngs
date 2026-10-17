# Jellyfin lent from the home page beside its helper — 2026-10-15

Row v1ui-polish (d) (`ralph/next/v1ui/STATE.md`): v1ui-run's driver,
with Priya lending Jellyfin from her home page beside its sign-in helper,
and the planted take with no helper. The setup and predictions are
`PREDICTIONS.md`, committed (c6872dbc) before the driver (5f95a890,
29c859bb) and before any data (ad5e75b4). A program is not a stranger:
**the people bar never-ran**, and it is the operator's to run with real
people (`demo/README.md`). One host, one take of each: not a rate
(AGENTS.md 7).

## How it ran

```
PLANT=nohelper W=/tmp/v1ui-helper FILMS=/tmp/v1ui-films python3 demo/runs/v1ui/2026-10-15-helper/run.py   # data/plants/nohelper/
W=/tmp/v1ui-helper FILMS=/tmp/v1ui-films python3 demo/runs/v1ui/2026-10-15-helper/run.py                  # data/
python3 demo/runs/v1ui/2026-10-15-helper/judge.py <a take's summary.json>                                 # its verdicts.json
```

Both from the loop's await, in that order, each exiting 0
(`ralph/next/v1ui/ctl/log/sessions/v1ui-polish-1.await.log`, not kept).

- **The planted take** at 29c859bb, nothing dirty
  (`data/plants/nohelper/events.jsonl`, `start`). The load was 7.4 at the
  start on 12 cores and 52.2 at the end (`summary.json`, `end_load`). No
  middleware was started, so `mw.log` is absent.
- **The clean take** at 29c859bb, the planted take's data its only dirt
  (`data/events.jsonl`, `start`). The load was 52.2 at the start and 11.1
  at the end. The middleware ran on 8097, debug (`data/mw.log`).
- Everything else is v1ui-run's (`../2026-10-15/README.md`, "How it ran"):
  a release build fetched by install.sh, Jellyfin 12.0.0 scratch on 8096
  with three 120 s films, n0's relays, a `launchctl` stub.

## Against the predictions

Times are in ms, from each take's `verdicts.json`, each prediction's `read`.

| | predicted | as it ran (clean) | clean | plant |
|---|---|---|---|---|
| 1 | the line shown within 5 s | 37–48 ms after the link opened, on each of the four computers | passed | passed |
| 2 | the line exits 0 within 30 s, asking nothing | exit 0 in 5,751–8,163 ms; each said "You're in Elm Street, as <name>." and last the home page's address | passed | passed |
| 3 | the door within 30 s of the line, everyone joined lit, Eve "in from a phone" | 57–62 ms after the line for Ann, Bo and Cy; 8,310 ms for Di, Eve's join and Sam's stop between. The index listed Money and "Open Priya’s jellyfin" | passed | **failed**: Di's door 32,698 ms after the line (below) |
| 4 | Money open within 10 s; square within 30 s; one act a press | open in 76–93 ms on the computers; square in 83–282 ms after 2–5 presses, each one act by its presser | passed | passed |
| 5 | the library with no sign-in within 15 s; playing within 30 s of the card | Jellyfin's home in 2,580–2,701 ms with no sign-in; the film's title to playing in 1,198–1,428 ms | passed | **failed**: every newcomer stopped at "Please sign in" |
| 6 | Eve asked her name within 10 s; in within 30 s of Join | asked at 1,485; in 1,836 after Join | passed | passed |
| 7 | the issuer stopped: through someone else's computer within 15 s; Sam drawn off | "…as Eve, through Ann’s computer" in 3,886 ms, then through Bo’s; Sam "did not answer" | passed | passed |
| 8 | Eve's Money square and film playing, each within 60 s | Money 3,120 ms over 6 presses; the library 12,921 ms (Jellyfin's home 6,542, then the title to playing 6,379) | passed | **failed**: the library, as in 5 |
| 9 | Di, the issuer stopped: as 3–5 | as above; Sam stopped before Di's home page was opened | passed | **failed**: 3 and 5 |
| 10 | each of the five done within 180 s of the link; the slowest within 120 s | Ann 10,261, Bo 10,071, Cy 10,172, Di 20,918, Eve 35,844 | passed | **failed**: none done |
| 11 | no password box or sign-in shown; one Jellyfin account each, made by the middleware | 0 shown to each; Jellyfin's accounts were Ann-, Bo-, Cy-, Di- and Eve-<16 hex>, and priya | **passed** | **failed**: 6 sightings each, and no account but priya's |
| 12 | the door after: Sam off, five lit, Eve a phone | so on each of Priya's, Ann's, Bo's, Cy's and Di's computers | passed | passed |
| 13 | Money square everywhere; the same acts on every copy | square on all five; 21 Money acts on each copy, 20 presses and Gas bill | passed | passed |
| 14 | none of key, member, record, act, hash, origin, port or relay on a svrngs page | none, over 6–12 pages a person | passed | passed |
| 15 | the plant fails 5, 8, 10 and 11, and holds Money's 4 and 13 | so; it also failed 3 and 9, which 15 did not name | passed | never-ran (it is the plant) |
| 16 | real people, cellular, a second host | not run | could-not-judge; **the people bar never-ran** | could-not-judge |
| 17 | the page offers the helper, in the words before Lend and in the declaration | before Lend: "Its helper is running on this computer, so everyone is let in as themselves, with no account to make."; `lends/jellyfin.json` `"sign_in": 8097`. In the plant: "It keeps its own accounts, so make each person one in it: …" and "Or start its helper first, and nobody needs an account: here is how.", no `sign_in` | passed | passed |

The row's bar is 11 passing in the clean take, and it did: lent from
Priya's page with no terminal, the library let in all five as themselves,
each with an account the middleware made (`summary.json`,
`jellyfin_users`). v1ui-run's finding 3, "the bar's library is the
terminal's, not the page's", no longer holds on this host.

## What it found

1. **The plant's Di waited on the driver, not on svrngs.** Di's line
   ended at 1792090143.99 (`data/plants/nohelper/events.jsonl`, Di's
   `open` at 1792090129.014 plus `line.at` 14,976). Eve's phone browser
   then took until 1792090162.255 to open (`open`, Eve), and her join
   until 1792090172.667; Sam was stopped at 1792090173.596. Di's home
   page itself answered in 3,092 ms. The 30 s of 3 is measured from the
   line's end, so it counted the driver's 18 s of opening a browser,
   with the load climbing to 52.2 by the take's end. It is failed, as the
   judge reads it, and 9 fails with it and with 5.
2. **The page offers the helper itself as something to lend.** After
   Priya lent Jellyfin, her page listed "Something at
   http://localhost:8097. To lend it, give it a name and say what it is"
   beside every other program (`summary.json`, `lent.seen.words`). That is
   the middleware, which only Priya's own svrngs should ask. Nothing here
   lent it, and what lending it would let in is not shown by this run. It
   is a follow-up, and nothing here changes it.
3. **Eve's library was slower than the computers'.** Jellyfin's home in
   6,542 ms and the title to playing in 6,379, against 2,580–2,701 and
   1,198–1,428 on the computers (5, 8). v1ui-run's take 2 was the same
   (6,694 and 6,978), so it is the phone's way through another person's
   computer, not the helper.

## What this run does not show

- Real people, a phone on cellular, a second host, launchd itself: not
  run. They are could-not-judge, and the operator's. **The N+1 people
  bar never-ran.**
- A lender building and starting the helper by `apps/jellyfin/README.md`:
  the driver started it, given its key, as a lender would.
- The planted page's "here is how" link followed: the driver read the
  words, not where the link goes.
