# The grapevine scene under svrngs, on one host — 2026-10-16

Row gv-run (b) (`ralph/next/grapevine/STATE.md`): GRAPEVINE_DEMO §10's
scene with Sam, Jo and Kim each on a computer of their own, each
computer a data dir and svrngs of its own on this host, each person's
grapevine (`apps/grapevine/`) in record mode on their own computer. The
setup and predictions are `PREDICTIONS.md`, committed (69d3171e) before
the driver and any data. Every person is a program: no people bar is
judged (AGENTS.md 7). One host, one run of each: not a rate.

The `gv-*-check*.out` files in this folder are earlier rows'
`scripts/check.sh` outputs, not this run's.

## How it ran

```
W=/tmp/gv-run python3 demo/runs/grapevine/2026-10-16/run.py                       # data/
PLANT=<unsealed|spread|uncorrected|upstream> W=/tmp/gv-run python3 demo/runs/grapevine/2026-10-16/run.py   # data/plants/<plant>/
python3 demo/runs/grapevine/2026-10-16/judge.py <its summary.json>                # its verdicts.json
```

- **The clean take** at 6e3f327b, nothing dirty (`data/events.jsonl`,
  `start`), 47.6 s from start to end (`summary.json`, `run_s`); the
  1-minute load 11.9 at the start and 14.4 at the end on 12 cores.
- **The planted takes** at 6e3f327b, each just after the clean one, data
  in `data/plants/<plant>/`.
- **Takes before these**, against the uncommitted driver, their data
  cleared, found three of the judge's own faults and one of the app's
  (6e3f327b's body, and "What it found" below). In that commit's body,
  "9e7fd0b0's parent" is wrong: the app's fix is 60d663c2.
- n0's public relays, the default; all three computers on this host.

## Against the predictions

Verdicts are `data/verdicts.json` (clean) and
`data/plants/<plant>/verdicts.json`. Times are from Sam's press of
**Ask** (`summary.json`, `steps`, `s`), each act's arrival on each
computer from the 0.2 s reads (`seen`).

| | predicted | as it ran | clean |
|---|---|---|---|
| 1 | each computer's person per group from svrngs; each page those groups | svrngs: Sam in 1400 Page, Jo in both, Kim in Waller block (`town`); each page opened at the address its start line printed showed exactly those (`steps`, `open`) | passed |
| 2 | each press read back within 2 s | 7 presses, **58–187 ms** from the press to the page read again (`pressed_ms`) | passed |
| 3 | Jo sees it within 10 s; Sam offered no carry | Jo's page showed **Carry to Waller block** at **0.835 s**, 0.543 s after Sam's press landed, one reload; no Carry button on any page of Sam's | passed |
| 4 | Kim sees it within 10 s, carried by Jo, never Sam; only Kim told | Kim's page at **1.467 s**, 0.534 s after Jo's press: "carried by Jo", "You might have this"; no page of Kim's before her yes says Sam; neither Sam's nor Jo's says "You might have this" | passed |
| 5 | the offer at Sam's within 15 s; Jo told one went by, none of it | Sam's page showed "offered: when Saturday morning; where Waller and Pierce" at **4.739 s**, 3.184 s after Kim's press; Jo's: "A sealed message went by toward the person who asked: 96 bytes, Fri 16 Oct, 14:39." and no sealed word on any page of Jo's | passed |
| 6 | no name before two yeses; then both within 15 s | Kim's page holding Sam's yes (9.565 s) named no one; after Kim's yes (9.639 s) her page named Sam in 4 ms (both yeses on her own computer), Sam's named Kim at 14.932 s, **5.293 s** after | passed |
| 7 | gone from Kim's page within 20 s; Jo's correction within one loop, 6 s | Kim's page dropped the ask at 20.387 s, **5.383 s** after Sam's press; Jo's computer held Sam's withdrawal at 15.027 s and Jo's correction at 19.839 s: **4.812 s** (`seen`) | passed |
| 8 | the scene under three minutes, predicted under 90 s | **20.4 s**, Sam's Ask to the ask gone from Kim's page; the start, from no svrngs to three pages open, 18.3 s | passed |
| 9 | bar 1 on the records | one carry in each copy of Waller block, Jo's (`on_behalf_of` and `person` Jo), holding §3's fields alone, none of Sam, 1400 Page's id or name, or the ask's handle; no act in Waller block is Sam's (`copies`) | passed |
| 10 | bar 3: no sealed word or secret key outside a state file; `tile saw` seen | 44 files, 3,289,069 bytes under `W` (every data dir, svrngs log, grapevine trace and state file) and every record kept: no sealed word, no secret key outside a state file. The search's own check: `tile saw` found in each computer's `ring_oplog.jsonl` (Jo 2, Kim 1, Sam 1) | passed |
| 11 | bar 2 as Jo's page says it | under "Where it can’t go", after the carry: "Waller block: Waller block already has it" | passed |
| 12 | no word to look up; the shortcut on every conversation | 19 pages read, every word of each (closed parts too): none of key, member, record, act, hash, handle, hop or seal; 12 conversations drawn, each with the shortcut | passed |
| 13 | each plant fails its bar | below | below |
| 14 | not run | bar 6, GRAPEVINE §7 bars 4 and 7, its product bars and kill bar, a second host, a phone | never-ran |

**The plants**, each the whole scene against a copy of the app with one
change (`run.py`, `PLANTS`):

| plant | predicted | as it ran |
|---|---|---|
| `unsealed` (bar 3's) | bar 10 failed, the offer in the open in both groups; stalls at step 4 | **failed** bar 10: "Saturday morning" and "Waller and Pierce" in 4 files, every computer's copy of both groups (`ring_oplog.jsonl`); stalled at step 4, Sam's page never opening the offer (bar 5 failed, 6–8 could-not-judge). Every other bar held |
| `spread` (bar 1's) | bar 9 failed | **not as predicted**: the app's own validator refused the carry before it was written, "a carry saying more than this version reads (id, asked, in, carriers, live, show, notice), so not counted", said on Jo's page in place; Jo's trace `refused: it would not read`, `unread_field`. Bar 2 failed (the press refused), bar 9 could-not-judge: no carry reached a record |
| `uncorrected` (bar 5's) | bar 7 failed; the ask still on Kim's page at the bound | **failed** bar 7: no correction of the carry by Jo in any copy, the ask on Kim's page 60 s after Sam took it back; bar 8 failed with it. Bars 1–6 and 9–12 held |
| `upstream`, added after the predictions | bar 9 failed | **failed** bar 9: "the carry holds the ask's handle" in Jo's and Kim's copies; stalled at step 4 (Kim's offer addressed to a handle no route of Jo's answers) |

`upstream` was added after the first takes, before the driver's commit:
`spread` never reaches a record, so without it no take put a break of
bar 1 in the records for bar 9 to catch. Its prediction above is made
after seeing `spread`'s refusal, not before any data.

## What it found

1. **The page said "key"** (fence 9), in the `unsealed` take before the
   driver's commit: Sam's and Kim's pages listed "a message that does not
   read: it does not open with your key" under "1 thing here couldn’t be
   read". Six of the fold's and validator's sayings reached a page that
   way. Fixed in the app, with its test red first (60d663c2).
2. **The one validator holds bar 1 at the write**: a carry spread with the
   ask is refused before svrngs sees it (`spread`). Its refusal names the
   fold's own fields (`id, asked, in, …`) on the page. Only an app
   changed to write that way reaches it; not fixed here.
3. **Time is the loop's.** Each act reached the other computer holding a
   group within the 0.2 s read's resolution (`seen`: the ask on Jo's
   computer 3 ms after Sam's, the carry on Kim's 9 ms after Jo's). Every
   wait over a second in the scene is a turn of a carrier's loop: the
   offer 3.2 s, Kim's sight of Sam's yes 4.2 s, Sam's of Kim's 5.3 s, the
   carry corrected 4.8 s. RELAY_EVERY, 5 s, is the scene's floor, not
   svrngs.
4. **Reads are cheap at this size.** Each pass read each group's whole
   record in at most 10 ms, the median 3–4 ms on each computer
   (`data/*.trace`, `a pass`, `reads`), 9 and 10 acts a
   group by the end (`copies`).
5. **The run's own record holds what each person read** (`summary.json`,
   `steps`, the page text): Sam's page shows Kim's offer, so the run's
   summary does. That is the screen as the person saw it, not a record,
   log or trace of the app's or svrngs's; the search above covers every
   file the app and svrngs wrote.

## Bars the operator runs

GRAPEVINE.md §7's product bars (three real rings of neighbours, a
cross-ring tool borrow at one tap a hop, the median ask to a yes within a
day) and its kill bar (the same asks on the neighbourhood's Buy Nothing
or Nextdoor group filled no slower): **never-ran**. They are runs with
real neighbours, the operator's. Bars 4 and 7 of §7 (a carrier's
computer off; the group of two) are not in this app (GRAPEVINE_DEMO §8);
bar 8 is out (taps only, §12.1).
