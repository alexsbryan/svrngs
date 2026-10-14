# The primitive event on one host — 2026-10-14

Row events-run (`ralph/next/events/STATE.md`): Sam's dinner drawn from The
House and dWeb, through the reference app (`apps/events/`) lent by Sam's
computer, answered from computers and phones. The setup and predictions
are `PREDICTIONS.md`, committed (f5d443e) before the driver and any data.
Every person is a program: no people bar is judged (AGENTS.md 7). One
host, one run: not a rate.

## How it ran

```
W=/tmp/events-run python3 demo/runs/events/2026-10-14/run.py                  # data/
PLANT=<cross|guest|no> W=/tmp/events-run python3 demo/runs/events/2026-10-14/run.py   # data/plants/<plant>/
python3 demo/runs/events/2026-10-14/judge.py <its summary.json>              # its verdicts.json
```

- **The clean take** at d97a5ea, nothing dirty (`data/events.jsonl`,
  `start`), 87.5 s from start to end; the 1-minute load 9.2 at the start
  and 9.6 at the end on 12 cores (`start`; `summary.json`, `end_load`).
- **The planted takes** at 79e8cdf (`data/plants/<plant>/events.jsonl`,
  `start`). Between the two commits only what a clean run clears changed:
  the clean take at d97a5ea removed `data/` whole, the planted takes run
  just before it with it, so they ran again (79e8cdf's body).
- **Takes before these**, against the uncommitted driver, found the
  judge's own faults (d97a5ea's body). One is the run's: the app posts to
  the groups in the order the computer lists them, by id (`/v1/groups`),
  and lands on the first — dWeb's, in one take where dWeb's id sorted
  first. Prediction 1 named The House's; the judge reads where it landed
  as evidence, and each page's sections by their group.
- n0's public relays, the default; every computer and phone on this host.

## Against the predictions

Verdicts are `data/verdicts.json` (clean) and
`data/plants/<plant>/verdicts.json`.

| | predicted | as it ran | clean | its plant |
|---|---|---|---|---|
| 1 | one `event` in each record under one id; lands within 5 s | one `event` by Sam in each record, both `deb4107f252b3153` (`summary.json`, `copies.folded`); the page landed on The House's event in **145 ms** (`event`) | passed | — |
| 2 | each answer within 5 s on a computer, 15 s on a phone; a phone's event open within 30 s; each act in its own group | Bo's, Di's and Fay's presses **64–89 ms**, phones' **196–208 ms** (`presses`); Cy's phone on the event **1,604 ms** after `#open=`, Eve's **1,627 ms** (`views`, step 3). The House's acts by Bo, Cy, Fay and Sam; dWeb's by Di, Eve and Sam (`copies.folded`) | passed | — |
| 3 | Bo's `coming` names Maya; only Bo's `came` for Maya; no "Maya came" on Cy's page | Bo's live `coming` `{"guests": ["Maya"]}`, and the one `came` for Maya Bo's; Cy's page offered "Save", "Take my name off", "I came" (`cy_maya`); Sam's view "Bo + Maya" and "Maya, Bo's guest" | passed | `guest`: **failed** — Cy's page offered "Maya came", and the `came` it wrote was Bo's, which Bo never pressed |
| 4 | changes of mind are corrections; Eve nowhere but invited | Bo's first `coming` (`0551d749`) corrected by Bo into the one with Maya; Eve's (`604f9781`) voided by her own (`2c71c7d4`) (`copies.folded`, Sam's dWeb). No page read from then on names her but under Invited | passed | `no`: failed with it — her `coming` never taken back, so shown coming |
| 5 | Sam's view: both groups, invited, coming, came | The House: invited Bo, Cy, Fay; coming "Bo + Maya", Cy, Fay; came Bo, Cy, "Maya, Bo's guest". dWeb: invited Di, Eve; coming Di; came Di (`views`, Sam, step 10) | passed | `no`: failed with it (Eve coming) |
| 6 | each other person's view their own group's alone; the other group's address refused | Bo, Cy and Fay one section, The House; Di and Eve one, dWeb. Cy at dWeb's address: 403, "You're not in dWeb, so this event isn't yours to see."; Di at The House's: 403, the same of The House (`views`, steps 10, 10x) | passed | `cross`: **failed** — all five pages held both groups |
| 7 | every copy folds the same, within 30 s of the last act; Di's within 60 s of waking | The House: 16 ops, 8 acts, on Sam's, Bo's and Fay's computers; dWeb: 10 ops, 5 acts, on Sam's and Di's; each copy folding to the same answers and no gaps (`copies.folded`), held 19 ms and 8 ms after being asked (`copies.held`), asked 3.0 s after Fay's answer (`events.jsonl`, 20.05 s and 23.04 s from start). Di's computer slept 0.5 s (`di_asleep_s`), through Eve's correction; awake, its copy held all 9 of Sam's dWeb ops in **854 ms** (`di_woke`; its key `ops_missed` counts the ops it had to hold, not the one it missed) | passed | — |
| 8 | three kinds, read fields only; no page says a refusal | every payload `event`, `coming` or `came` with read fields only; no page read says not coming, declined or no-show | passed | `no`: **failed** — `{"kind": "not-coming"}` by Eve in dWeb's record |
| 9 | every act its own person's press | each act in each record matched to a press by the person it names, in its group | passed | `guest`: **failed** — a `came` for Maya by Bo, unpressed; `no`: failed with it |
| 10a | with Sam's computer asleep, said as that, never as no answers | Bo's browser: 502, `x-svrngs-refused: away`, `{"error":"the computer that lends this did not answer; try again later","kind":"away"}`; Cy's phone: 502, "The computer lending this did not answer. Try again later."; the app asked on loopback for Sam: 502, "This group can't be read on this computer right now." No event page with no one on it. Awake again, Bo's page held the same answers 420 ms after Sam's svrngs started (`asleep`) | passed | — |
| 10b | told within 30 s | Bo **30,019 ms**, Cy **30,024 ms** (`asleep`): Bo's computer dialled Sam's for 30 s before it said so (`data/bo.log`: the request at 08:36:44.54, "dial failed … error=timed out" at 08:37:14.55) | **failed** | — |
| 11 | Fay, joined after the post, sees it, answers, is invited | Fay joined at 1791967001 (`people`), the post at 1791966993 (The House's `event` act, `ts`): 8 s after. Her page listed Dinner, her coming was written and counted on every copy, and Sam's Invited for The House names her | passed | — |
| 12 | no page says key, member, record, act, namespace or origin | none does (`views`, every page read) | passed | — |
| 13 | a phone on cellular, a second host, real people | not run | could-not-judge; people bars never-ran | — |

Each planted take held every other bar but 10b, and for `no` but the
bars its unvoided answer brings down with it, 4, 5 and 9
(`data/plants/*/verdicts.json`).

## What "invited" means: whoever is in the group now

The charter leaves it to the run ("Decide"): the people in the group when
the event is posted, or now.

- **What the run shows.** Fay joined The House 8 s after the post (11). She
  saw the dinner, said she's coming, and every copy counts it (7). The app
  lets her because it asks who stands in the group now (`server.mjs`
  `standing`, `/v1/rail/people`).
- **"At posting" would contradict the page.** Fay would be coming and not
  invited. Or the app would have to refuse her answer, which is a machine
  deciding she may not speak. `/v1/rail/people` carries each person's
  `joined`, so the app could compute it; it would then keep a second rule
  for who may answer beside the one for who may see.
- **So invited is whoever is in the group now**: one read decides who may
  see, who may answer and who is invited.
- **The cost.** Someone who joins a year later is listed as invited to
  last year's dinner. Someone removed drops off Invited while their
  answers stay. Neither was run here.
- **What would falsify it**: a run with people where "who was invited"
  the next morning misleads someone because of a later joiner.

## What it found

1. **A person waits 30 s to be told the host's computer is asleep** (10b).
   The wait is the iroh bridge's dial timing out (`data/bo.log`,
   "transport: iroh bridge: dial failed … error=timed out"), then
   `crates/svrngs-node/src/origin.rs` `away`. svrngs's own two 30 s
   constants, the limit's window (`door/limit.rs:39`) and the sign-in's
   ask (`door/sign_in.rs:58`), are not on this path. It is svrngs's,
   written in `NEEDS_HUMAN.md` here.
2. **On a computer, svrngs's words arrive in braces** (10a): Bo's browser
   shows the JSON. start-run found the same of the door's limit words
   (`demo/runs/start/2026-10-13-story/README.md`, prediction 11). Not new.
3. **The app cannot give anyone the event's address.** The driver built
   the phone's `#open=<label>&at=<path>` itself (`run.py` `phone_at`),
   from the label in a computer's `/v1/lends` origin. The app is told who
   asks and nothing of where it is lent from (the door's headers,
   `crates/svrngs-node/src/door.rs:114-180`). And
   a computer's address for the lend is that computer's own port. svrngs's,
   in `NEEDS_HUMAN.md`. events-doc needs a way a person reaches the event;
   the group's page and the app's own list ("Dinner" on Fay's page, step 9)
   is the way this run did not take.
4. **While Sam's computer sleeps no one reaches the event**, though Bo's
   and Fay's computers hold every answer (7). EVENTS.md §2's "up while the
   host sleeps" needs a second computer lending the app; this run lent it
   from Sam's alone.
5. **The post lands on the first group by id** (How it ran). The app's own
   choice, harmless. A follow-up for the app if the host should land
   where they began.

## The shots take

events-doc asked for the run's pictures and a way in, and the clean take
had neither (finding 3). `SHOTS.md` holds its setup and predictions,
committed (f474069) before its driver (d7c39ba) and its data (a984c1b,
whose subject names its commit wrongly: it ran at f474069 with the driver
d7c39ba commits as its only dirt, `data/shots/events.jsonl`, `start`).
106.9 s from start to end, the 1-minute load 5.0 at the start and 7.6 at
the end on 12 cores. Its numbers are this take's; the run's are the clean
take's, above.

| | predicted | as it ran | verdict |
|---|---|---|---|
| S1 | Bo's home page lists "Open Sam’s events" under The House within 30 s | listed under The House, beside "Copy its address for a program", 31 ms after the address was asked (`way_in.bo[0]`, `03-bo-2w.png`) | passed |
| S2 | it opens the app's list; Dinner lands on The House's event within 5 s | the app's list, "The House", "Dinner Friday 7", 45 ms; Dinner on The House's address of the event, `way_in.path`, 17 ms (`way_in.bo[1-2]`, `04`, `05`) | passed |
| S3 | Cy's phone, no link, lists Sam’s events; the app's list within 30 s; Dinner lands on The House's | "You’re in The House, as Cy" and "Sam’s events", 667 ms; the app's list 1,046 ms after the press; Dinner on The House's address, 91 ms (`way_in.cy`, `06`-`08`) | passed |
| S4 | every bar as the clean take's, 10b included | the same verdict on every bar; 10b failed again, Bo 30,019 ms, Cy 30,026 ms (`data/shots/verdicts.json`) | passed |
| S5 | no picture shows a link's token | no step shot shows a link; the home page's "Make a link" was not pressed (`03-bo-2w.png`) | passed |

Two things the pictures show:

- **The phone's group page shows its hidden forms** to someone already
  in: a name box, "Join", "Yes" and "Not now" above "Sam’s events"
  (`06-cy-3w.png`). The same cause start-run found on a computer,
  `web/index.html:53`, `form { display: grid; }` over `hidden`; svrngs's,
  and known.
- **The post landed on dWeb's event** this take (`summary.json`,
  `event.landed`; `02-sam-1.png`), The House's in the clean take: finding
  5, by which group's id sorts first.
