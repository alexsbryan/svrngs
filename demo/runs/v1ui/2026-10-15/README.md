# The N+1 bar's steps on one host — 2026-10-15

Row v1ui-run (`ralph/next/v1ui/STATE.md`): `ONE_PAGER.md`'s N+1 bar,
"Four of five strangers settle an expense and open a title in the library
within three minutes of joining the chat, with zero accounts and zero key
prompts" (V1_UI_BUILD §7), its steps run with every person a program. Five
newcomers open one chat's link to Elm Street: Ann, Bo, Cy and Di on
computers, by the line the page gives, and Eve on a phone. Each says they
were in on Priya's Gas bill in Money and pays their share, then plays a
film from Priya's lent Jellyfin. Sam's computer, the link's, is stopped
after the last join.

The setup and predictions are `PREDICTIONS.md`, committed (aed3d2d,
corrected in 6c555d5 to count five computers still on, not six) before the
driver (3f10f8b) and before any data (912c078 on). A program is not a
stranger: **the people bar never-ran**, and it is the operator's to run
with real people (`demo/README.md`). One host, these takes: not a rate
(AGENTS.md 7).

## How it ran

```
PLANT=accounts W=/tmp/v1ui-run FILMS=/tmp/v1ui-films python3 demo/runs/v1ui/2026-10-15/run.py   # data/plants/accounts/
W=/tmp/v1ui-run FILMS=/tmp/v1ui-films python3 demo/runs/v1ui/2026-10-15/run.py                  # data/
TAKE=2 W=/tmp/v1ui-run FILMS=/tmp/v1ui-films python3 demo/runs/v1ui/2026-10-15/run.py           # data/takes/2/
python3 demo/runs/v1ui/2026-10-15/judge.py <a take's summary.json>                              # its verdicts.json
```

- **The planted take** at 3f10f8b, nothing dirty (`data/plants/accounts/
  events.jsonl`, `start`). The load was 2.7 at the start on 12 cores.
- **The first clean take** at 912c078, nothing dirty (`data/events.jsonl`,
  `start`), 221.9 s from start to end. The load was 3.2 at the start and
  4.4 at the end (`summary.json`, `end_load`). It exited 1, because
  Jellyfin had stopped by the time the driver read its accounts (below).
- **The second clean take** at 3e06099, nothing dirty (`data/takes/2/
  events.jsonl`, `start`), 119.1 s from start to end. The load was 8.3 at
  the start and 7.7 at the end. Between 912c078 and 3e06099, only the
  judge changed (6219f75, below) and where a take writes its data
  (3e06099). The steps are the same.
- **Every take is judged by judge.py at 6219f75 or later**, which is the
  same judge. 9c13027's subject is right, but its body names that commit
  "9b7e0bd's successor", which is wrong.
- **Takes before these**, two of them against the driver before it was
  committed, at 6c555d5, found the driver's own faults. 3f10f8b's body
  says what they were. Neither take is kept.
- svrngs ran as a release build, served under the site's `get/` as
  install.sh fetches it, and the page was served on this host. Jellyfin
  was 12.0.0, from the macOS app, scratch, on 8096, with three 120 s MP4
  films. Its middleware ran on 8097. The relays were n0's. A `launchctl`
  stub stood in for launchd, and the driver started what each plist
  named (PREDICTIONS.md §1).

## Against the predictions

The numbers are the second clean take's (`data/takes/2/verdicts.json`,
each prediction's `read`), with the first's in brackets where they differ
in kind. Times are in ms.

| | predicted | as it ran (take 2) | take 2 | take 1 | plant |
|---|---|---|---|---|---|
| 1 | the line shown within 5 s | 35–39 ms after the link opened, on each of the four computers (`summary.json`, `people.*.link.ms`) | passed | passed | passed |
| 2 | the line exits 0 within 30 s, asking nothing; "You're in Elm Street, as <name>." | exit 0 in 5,622–5,936 ms, stdin closed; each said "You're in Elm Street, as Ann." (and Bo, Cy, Di), and last "Your home page, the same address each time svrngs starts: …", the address `open` was asked for | passed | passed | passed |
| 3 | the door within 30 s of the line, everyone joined lit, Eve "in from a phone" | 71–74 ms after the line for Ann, Bo and Cy. For Di it was 8,244 ms, because Eve joined and Sam's computer was stopped in between. Every person joined was "on now". On Di's door Sam "did not answer" and Eve was "in from a phone". The index listed Money and "Open Priya’s jellyfin" | passed | passed | passed |
| 4 | Money open within 10 s; square within 30 s; one act a press | open in 76–97 ms; square in 82–249 ms after 2, 3, 4 and 5 presses. Each press is one act by its presser in Priya's copy (`acts` beside `presses`) | passed | passed | passed |
| 5 | the library with no sign-in within 15 s; playing within 30 s of the card | Jellyfin's home in 2,548–2,605 ms with no sign-in; from the film's title to playing in 1,171–1,381 ms | passed | **failed**: Di's tab waited 60 s and read `{"error":"jellyfin could not sign you in","kind":"sign-in"}` | **failed**: every newcomer stopped at "Please sign in" |
| 6 | Eve asked her name within 10 s; in within 30 s of Join | asked at 1,492; "You’re in Elm Street (Sam’s, since 15 Oct), as Eve" 1,821 after Join | passed | passed | passed |
| 7 | the issuer stopped: through someone else's computer within 15 s; Sam drawn off | "You’re in Elm Street (Sam’s, since 15 Oct), as Eve, through Cy’s computer" in 3,887 ms; Sam "did not answer". Take 1 went through Ann's computer, then Bo's | passed | passed | passed |
| 8 | Eve's Money square and film playing, each within 60 s | Money 3,275 ms over 6 presses; the library 13,672 ms (Jellyfin's home 6,694, then the title to playing 6,978) | passed | **failed**: the library, as in 5 | **failed** |
| 9 | Di, the issuer stopped: as 3–5 | as above; Sam stopped before Di's home page was opened | passed | **failed**: 5 | **failed**: 5, not named in prediction 15 |
| 10 | each of the five done within 180 s of the link; the slowest within 120 s | Ann 10,297, Bo 9,983, Cy 10,012, Di 18,109, Eve 36,610 | passed | **failed**: Ann 10,097, Bo 12,523, Cy 10,062; Di and Eve never | **failed** |
| 11 | no password box or sign-in shown; one Jellyfin account each, made by the middleware | 0 shown to each; Jellyfin's accounts were Ann-, Bo-, Cy-, Di- and Eve-<16 hex>, and priya | passed | could-not-judge: 0 shown, but Jellyfin's accounts could not be read | **failed**: 6 sightings each, and no account but priya's |
| 12 | the door after: Sam off, five lit, Eve a phone | so on each of Priya's, Ann's, Bo's, Cy's and Di's computers | passed | passed | passed |
| 13 | Money square everywhere; the same acts on every copy; none unpressed | "You’re square with everyone." and "Nobody owes anybody." on all five; 21 Money acts on each copy, 20 presses and Gas bill | passed | passed | passed |
| 14 | none of key, member, record, act, hash, origin, port or relay on a svrngs page | none, over 6–12 pages a person | passed | passed | passed |
| 15 | the plant fails 5, 8, 10 and 11, and holds Money's | as said | passed | passed | — |
| 16 | real people, cellular, a second host | not run | could-not-judge; **the people bar never-ran** | | |

On this host, in take 2, five of five settled and played within 37 s of
opening the link. In take 1, three of five did. Neither is the bar: the
bar is strangers, and these were programs.

## What it found

1. **Jellyfin 12.0.0 stopped when a tab playing it closed** (take 1). Cy's
   tab was closed at 1792085871.918 (`data/events.jsonl`, `back`). Jellyfin
   then logged "WS 127.0.0.1 closed" and "[FTL] Main: Unhandled Exception
   … ObjectDisposedException … WebSocketController", and exited
   (`data/jellyfin-stopped.log`). Ann's and Bo's tabs had closed the same
   way before it, and so had all five in take 2 and in the planted take,
   with no stop. That makes it a race in Jellyfin, met once in three
   kept takes. This is Jellyfin's fault, not svrngs's. But on Priya's one
   computer the library has no second host, so nobody had it until Priya
   started Jellyfin again.
2. **A library that has stopped is told as a sign-in, in braces.** This
   is svrngs's. Di's and Eve's tabs read `{"error":"jellyfin could not
   sign you in","kind":"sign-in"}` (take 1, `events.jsonl`, `library`).
   The middleware had logged "GET /Users could not be sent … 127.0.0.1:
   8096" (`data/mw.log`), so Jellyfin was down and was refusing nobody.
   The door says any failure of the sign-in program as "could not sign
   you in" (`crates/svrngs-node/src/door.rs:1333`), and a computer's tab
   shows svrngs's JSON as it is. The events run found the same braces
   (`demo/runs/events/2026-10-14/README.md`, finding 2). Fence 2's line
   is that a computer that did not answer is said as that. This is a
   follow-up row, and nothing here changes it.
3. **Lent from the home page, Jellyfin asks everyone to sign in** (the
   plant). The page's "Lend jellyfin" said "It keeps its own accounts, so
   make each person one in it: …" (`plants/accounts/summary.json`,
   `lent`). Every newcomer then met "Please sign in", with nothing of
   their own to type. Zero accounts holds only by the terminal's
   `svrngs share 8096 --sign-in 8097` and the middleware
   (`apps/jellyfin/README.md`), which the page does not offer. So the
   bar's library is the terminal's, not the page's: V1_UI_BUILD §3.2's
   "lent without a terminal" stops short of zero accounts for Jellyfin.
4. **Money is named with its group's id, and as the reader's own.** On
   Ann's computer the index said "Open Ann’s Money-ce985a0bbef0a4d7
   (Money)", and Eve's phone said "Ann’s Money-ce985a0bbef0a4d7 (Money)"
   (take 2, `people.Ann.home.index`, `people.Eve.group.index`). Sixteen
   hex letters in a person's words, and the group's app called one
   person's. The name is `<kind>-<16 of the group's id>`
   (`crates/svrngs-node/src/lend.rs:228`). It is drawn by `named`, which
   adds the kind when name and kind differ
   (`crates/svrngs-node/src/home/question.js:14`), and the "Whose" is
   the copy listed first (`once`). Fence 3 forbids nothing of this by
   name, so judge.py's words passed it. It is a follow-up row.
5. **Settling grows with the group.** Each newcomer settled by paying
   everyone who had already paid, since `transfers` splits what they owe
   among the people owed. Ann pressed 2 buttons, Bo 3, Cy 4, Di 5 and Eve
   6, the last five "Say you paid Ann $2.00" through "… Priya $2.00", in
   1,644 ms (take 2, `people.Eve.money.presses`). It was quick here.
   Whether a person reads six $2 debts as settling one gas bill is for
   people to say.
6. **The phone's computer changed between opens.** Take 1's Eve went
   through Ann's computer, then Bo's (7). Whichever answers first is the
   one used (`web/app.js` `reachGroup`), and the page says whose, as
   fence 2 asks.

## What this run does not show

- Real people, a phone on cellular, a second host, launchd itself: not
  run. They are could-not-judge, and the operator's. **The N+1 people
  bar never-ran**. It is the operator's to run with real people, and to
  record by `demo/README.md`.
- The chat. The link was opened from the driver, not from a chat app's
  preview or in-app browser.
- Jellyfin from the page with no accounts (finding 3).
