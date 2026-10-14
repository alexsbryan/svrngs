# The dinner's link, and Sam's computer asleep — 2026-10-14

Row events-doc-link (`ralph/next/events/STATE.md`): events-run's link and
asleep steps again, since svrngs tells a lend where it is (events-links)
and gives up a dial to a lend's computer at 4 s (events-asleep,
events-wait). The setup and predictions are `PREDICTIONS.md`, committed
(fda8c19) before the driver (41ce3cb) and the data (ac576cf). Every person
is a program: no people bar is judged (AGENTS.md 7). One host, one take:
not a rate.

## How it ran

```
W=/tmp/doclink-run python3 demo/runs/events/2026-10-14-links/run.py < /dev/null
```

- At 41ce3cb, nothing dirty (`data/events.jsonl`, `start`), 18:22:59 to
  18:25:22 UTC, 142.7 s; the 1-minute load 15.9 at the start and 24.1 at
  the end on 12 cores (`start`; `summary.json`, `end_load`).
- A debug build, n0's public relays; every computer and phone on this
  host. Computers' browsers Chromium keeping no group, phones WebKit as an
  iPhone (`people.mjs`).
- Every step a person saw is in `summary.json` `views`, its step, its
  `ms`, its words, and its picture `data/shots/<shot>.png`.

## Against the predictions

| | predicted | as it ran | verdict |
|---|---|---|---|
| P1 | one link in each group's part, `<page>#open=<label>&at=/g/<group>/e/<event>`, no token; Copy says "Copied", the clipboard holds it | dWeb's and The House's parts each "To send in <group>'s chat:", the box `http://localhost:62272/#open=sk3fz4…bfh&at=/g/<group>/e/14eb8f7dfd40865a`, the page served here, the label `/v1/lends` gives (`summary.json`, `lend`), the event posted (`event`). Copy said "Copied" both times; the clipboard read back each box (`05-sam-L1`, `06-sam-L1`) | passed |
| P2 | phones land on their group's dinner within 10 s, their group alone; "I'm coming" counted | Cy on The House's dinner **1,753 ms**, Eve on dWeb's **1,801 ms**, each page one part, their own; coming Cy, coming Eve (`07`–`10`) | passed |
| P3 | computers told "This link opens in svrngs on your computer.", the link shown; pasted on the home page, the dinner within 10 s; their group alone; counted | Bo's and Di's browsers said it, the link shown beside (`11-bo-L3`, `14-di-L3`); pasted, the dinner's tab, at the lend's address on their own computer, in **36 ms** each; one part, their own; coming Bo, Cy and Di, Eve (`12`–`16`) | passed |
| P4 | the other group's link: told it is not theirs, no answers shown | Cy's phone: 403, "You're not in dWeb, so this event isn't yours to see." (`17-cy-L4`); Di's tab: 403, the same of The House (`18-di-L4`) | passed |
| P5 | stopped: Bo 502 `away` in braces, 4.0–5 s; Cy's dinner again 4.0–5 s; Bo's home page within 10 s | Bo **4,036 ms**, 502 `away`, `{"error":"the computer that lends this did not answer; try again later","kind":"away"}` (`20-bo-L5`); his computer's log "the dial was not answered in time, so the computer is taken as asleep … within=Some(4s)" (`data/bo.log`, 18:23:52.41). Cy **4,018 ms**, 502 `away`, "The computer lending this did not answer. Try again later." (`22-cy-L5`). Bo's home page under The House "Sam's computer did not answer, so what they lend is not listed." in **3,210 ms** (`21-bo-L5`) | passed |
| P6 | stopped, the link anew: at the group's page, "Your group’s computer did not answer through <relay>", 29–35 s | **30,497 ms**, the group's page: "Your group’s computer did not answer through usw1-1.relay.n0.iroh.link. Try again later.", the name box, Join, Yes and Not now below it (`23-cy-L5`) | passed: the wait is past the cap, as predicted |
| P7 | awake again within 90 s: each shown the dinner, every answer | Bo's pasted link the dinner 534 ms after Sam's svrngs started, Cy's link 2,366 ms after; coming Bo, Cy (`24`, `25`) | passed |
| P8 | frozen: as P5 and P6; Cy, her dial held, 4.0–5 s | Cy, the dinner opened 90 ms before the freeze, **4,221 ms** (`27-cy-L6`); Bo **4,038 ms** (`28-bo-L6`); his home page **3,058 ms** (`29-bo-L6`); Cy's link anew **30,522 ms**, the same words (`30-cy-L6`). After SIGCONT, Bo 1,366 ms, Cy 3,385 ms (`31`, `32`) | passed |
| P9 | no page says key, member, record, act, namespace or origin; no part with no one on it while asleep | none does (every `views` text); while asleep no dinner page was shown at all, only the words above | passed |
| — | a phone on cellular, a second host, real people, a release build | not run | could-not-judge; people bars never-ran |

## What it shows

- **Sam has a link to send.** Each group's part of the host's page holds
  its own link and a Copy button, and only the host is offered it. The
  link carries the lend's label and the dinner's address in that group,
  never a token: it opens only for someone already in the group, and for
  someone in the other group it says the dinner is not theirs (P4).
- **On a phone the link opens the dinner** in under 2 s; **on a
  computer** it takes one paste more, on the home page, since the browser
  does not know where that computer's svrngs is (db36fec's body).
- **On the dinner's own address, Sam's computer asleep is said within
  about the cap**, 4.0–4.2 s, stopped or frozen, on a computer and a phone
  (P5, P8), where events-run waited 30,019 ms (its README, 10b).
- **A phone opening the link anew still waits 30 s** (P6, P8). Before it
  opens anything lent, the group's page dials the computer that made the
  link it joined by, Sam's here (`web/app.js:314`, `enter`), and that dial
  waits as long as iroh does (`web/src/lib.rs:155-160`, `reach`'s doc;
  `dial`, lib.rs:307-308, passing no cap). 5c097eb left it so: "the
  measurement did not cover it, and it broke a working join". svrngs's,
  written in `NEEDS_HUMAN.md` here.
- **On a computer, svrngs's words still arrive in braces** (P5, P8), as
  events-run found (its README, finding 2). Not new.
- **The phone's group page shows its hidden forms** under the words (P6),
  as events-run's shots take found (`web/index.html:53`). Not new.

## What this take does not show

- **A phone on cellular, a second host, a release build, real people**:
  not run, could-not-judge; the operator's (CHARTER.md).
- **A computer lending the app while Sam's sleeps** (events-run's finding
  4): not run; while Sam's computer sleeps no one reaches the dinner.
- **The link sent through a real chat app**: the driver handed the box's
  text to each browser, as a chat would.
