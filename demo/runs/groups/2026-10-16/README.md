# Leaving a group on one host — 2026-10-16

Row groups-run (`ralph/next/groups/STATE.md`): Ana leaves Trip from her
home page, Sam tries to leave before handing on the order of removals,
and Ana is sent a new link. The setup and predictions are
`PREDICTIONS.md`, committed (f407b7a4) before the driver (9c91879f,
5e2dee5b) and any data. Every person is a program: no people bar is
judged (AGENTS.md 7). One host, one run: not a rate.

## How it ran

```
W=/tmp/groups-run python3 demo/runs/groups/2026-10-16/run.py                     # data/
PLANT=<untold|deleted|removed|keeper|rejoin> W=/tmp/groups-run python3 demo/runs/groups/2026-10-16/run.py   # data/plants/<plant>/
python3 demo/runs/groups/2026-10-16/judge.py <its summary.json>                 # its verdicts.json
```

- **The clean take** ran at 5e2dee5b with nothing dirty
  (`data/events.jsonl`, `start`). It took 59.2 s from start to end. The
  1-minute load was 1.7 at the start and 4.4 at the end, on 12 cores
  (`start`; `summary.json`, `end_load`).
- **The planted takes** ran at 1b6e4c85, which is 5e2dee5b with the clean
  take's data added (`data/plants/<plant>/events.jsonl`, `start`). Each
  built svrngs and the page with its plant applied, then put the tree back
  before the story started (run.py `build`; each `built` line names the
  file planted).
- **Takes before these** found faults in the driver, not in svrngs.
  `svrngs share` was asked before its page's server listened (9c91879f's
  body). The lenders' acceptor traced at `node`, which the driver left at
  info, so the first clean take's refusals were missing from the logs
  (5e2dee5b's body). Neither take's data is kept.
- n0's public relays, the default. Every computer and phone ran on this
  host.

## Against the predictions

Verdicts are in `data/verdicts.json` (clean) and
`data/plants/<plant>/verdicts.json`.

| | predicted | as it ran | clean | its plant |
|---|---|---|---|---|
| 1 | Ana's page says what stays and that this computer can't rejoin, beside "Leave Trip"; pressed, it says she left and lists Trip under "Groups you left", within 5 s | Before the press, the predicted sentence was beside "Leave Trip", with no hand-on. After, "You left Trip. Rejoining needs a new invitation, and this computer can’t rejoin yet.", "Groups you left", and "Trip (Sam’s, since 16 Oct): what you have of it stays on this computer, its folder at /tmp/groups-run/ana/svrngs/Trip, and nothing new of it reaches you." No "Leave Trip". The whole browser call took **435 ms** (`ana_leave`) | passed | — |
| 2 | every lender holds her out within 30 s, no round asked | Sam's and Bo's computers each listed `{"by": "Ana", "of": "Ana", "said": "left"}`, and Ana was not among their people. Both held at the first look, **437 ms** and **439 ms** after her page was asked (`held_out`). The driver looks only once her page call returns | passed | `untold`: **failed**, neither within 60 s |
| 3 | nothing lent reaches her, nor hers them | Before the leave, `notes`, `photos` and `list` each answered 200 with its page (`before`). After it, each answered 404, `unlent`, "nothing is lent here". Sam's computer listed Bo's photos and Sam's notes, not Ana's list (`after`) | passed | — |
| 4 | her computer as it was before the leave is refused by each lender | `ana-copy`, started 313 ms before it was asked, got 403 for `notes`, `removed`, "You are no longer in this group, so nothing of it is open to you." That answer is Sam's door, and sam.log:12-14 holds its refusals at 16:55:44. It got 404 for `photos`, `unlent`: by then its first sync had been told the leave, and bo.log:10-11 holds Bo's refusals. Its `/v1/groups` said Trip left at **503 ms** (`copy`) | passed | `untold`: **failed**, both lends served their pages |
| 5 | her copy kept; nothing new reaches it | Her copy went from 8 acts to 9: all 8 kept, plus her leave. `before.txt` was in her folder, the same bytes, having reached it in 576 ms. `after.txt` reached Bo's folder in **503 ms** and was not in Ana's 20 s later (`ana_copy`, `after_txt`) | passed | `deleted`: **failed**, her folder and `before.txt` gone |
| 6 | "Ana left on …" on Sam's, Bo's and Cy's pages within 30 s; never "removed" | Sam's and Bo's showed "Ana left on Oct 16, 2026, 9:55 AM". Cy's phone showed "Ana left on Oct 16, 2026 at 9:55 AM" on its first read, which came 22,485 ms after Ana's page was asked (`cy_ms`); the phone was not asked sooner. Bo's computer held the leave at 439 ms (2) | passed | `removed`: **failed**, "Ana removed Ana" on all three |
| 7 | a keeper's leave refused until they hand on | Sam's page showed the hand-on with Bo to choose. "Leave Trip" added "You keep the order of removals in Trip: choose who keeps it after you, then leave.", and Sam was not left. `svrngs leave Trip` exited 1 with the predicted words, after "svrngs: ". "Hand it on" said "Bo keeps the order of removals in Trip now, so you can leave it." "Leave Trip" then left. Bo's computer held "Sam left" at **381 ms** and said he keeps it (`keeper`) | passed | `keeper`: **failed**, Sam's first press left and Bo kept nothing |
| 8 | Ana told she can't rejoin from this computer | Her page, with the new link pasted, said "You’re no longer in Trip on this computer, and you can’t join it again from here yet. Join from your phone or another computer instead.", with no "Join Trip". `svrngs join` exited 1 in 4,028 ms with the predicted words. Sam's people held no Ana (`rejoin_page`, `rejoin_command`) | passed | `rejoin`: **failed**, the command said "You're in Trip already, on this computer." and ran on, and the page said the link's computer could not be reached |
| 9 | Ana rejoins from her phone | "You’re in Trip (Sam’s, since 16 Oct), as Ana" in **2,807 ms**. Sam's people listed Ana as a browser key `afb5630e…`, not her computer's `3b11f2ac…`. Sam's page still said "Ana left on …" | passed | `removed`, `untold`: failed with them |
| 10 | no page or command says key, member, keeper, record, act or relay | none of the 193 lines read does (`10`, `read`, `hits: []`); given "Your key is kept", the judge fails it (9c91879f's body) | passed | — |
| 11 | a second host, a phone on cellular, real people | not run | could-not-judge; people bars never-ran | — |

Each plant failed its own bar, and no other bar failed without a cause
the plant itself makes:
- **`deleted`, `keeper` and `rejoin`** each failed only their own bar.
- **`removed`** failed 6, and 9 with it, because 9 reads Sam's page for
  "Ana left on".
- **`untold`** failed 2, 4 and 6, and 7 to 9 with them. With no cut told
  to anyone, Sam's leave reached nobody either. Ana's computer still
  stood at the issuer when her phone joined, so the phone's page asked
  her for another name (`data/plants/untold/sam.log`, "a standing person
  goes by the name; answered 409"). The driver waited 90 s for a status
  that never came.

## What it found

1. **The leaver is told "nothing is lent here", not that she left** (3).
   - Ana's computer answers every old lend address with origin.rs's
     `dialled` refusal (origin.rs:240-246, `Unlent`, 404). That refusal is
     for a lender who stands in no group held here.
   - The lenders' own words for a cut computer are "You are no longer in
     this group" (door.rs:297-302, prediction 4). Her own computer never
     says them.
   - The bar held, because nothing reached her. The words, though, are
     fence 2's substitution, and its own finding: #182.
2. **A lender's acceptor traces a cut dialer as "stands in no group",
   then tells it its cut untraced** (4).
   - sam.log:12-14 says "node: refused — the dialer stands in no group
     held here" (node.rs:939). The acceptor then serves that dialer the
     `gone` router (node.rs:371-374).
   - Nothing traces that branch, which tells the dialer "You are no longer
     in this group". The log reads as a refusal, and the answer was the
     cut said in words. AGENTS.md 1: #183.
3. **What a person waits for here is under a second.** The leave reached
   both lenders, the keeper's leave reached Bo, and a copy of the
   pre-leave computer was told its leave, each within 0.6 s (2, 4, 7).
   That is on one host, with the leave told directly (removal.rs `tell`),
   and is not a rate.

None of this changes svrngs; this row carries nothing to product but the
driver's own fixes.

## The close

`scripts/check.sh` ran once at 214aa196, after the README: it exited 0 in
13m44s, fourteen gates passed, none failed, could-not-judge or never-ran.
