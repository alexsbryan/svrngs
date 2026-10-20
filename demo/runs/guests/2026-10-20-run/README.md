# A visit on one host — 2026-10-20

Row guests-run (`ralph/next/guests/STATE.md`). Oak is a group: Ana's, Kim's
and Bo's computers. Sam, Kim's friend, has only a browser. Kim sends Sam a
link, and Sam watches Kim's film as Kim's guest. The house sees the visit,
a third guest is past the house's bound, and three visits end: by Kim's
Stop, on time, and by Kim's leave. The setup and predictions are in
`PREDICTIONS.md`, committed (376f813a) before the driver (e0020a16) and
before any data (43c2fde3). Every person is a program, so no people bar is
judged (AGENTS.md 7). These are takes on one host, not a rate.

## How it ran

```
W=/tmp/oak-clean python3 demo/runs/guests/2026-10-20-run/run.py                      # data/
TRIAL=1 W=/tmp/oak-run python3 demo/runs/guests/2026-10-20-run/run.py                # data/trial/
PLANT=<host|bo|count|lapsed> TRIAL=1 W=/tmp/oak-run python3 demo/runs/guests/2026-10-20-run/run.py  # data/plants/<plant>/
python3 demo/runs/guests/2026-10-20-run/judge.py <its summary.json>                  # its verdicts.json
```

- **The clean take**, the whole story with step 8's wait for the visit
  until tonight, runs under `ralph-result await` from the commit after
  this README. Its data and verdicts are not here yet.
- **The trial take** ran the story without that wait at e0020a16, with
  only the plants' data uncommitted (`data/trial/events.jsonl`, `start`).
  It took 225 s, the 1-minute load 3.5 at the start and 5.5 at the end, on
  12 cores (`start`; `summary.json`, `end_load`).
- **The planted takes** ran the same way at e0020a16. Each built svrngs
  with its one change, then put the tree back before the story started
  (run.py `build`; each `built` line names the file planted).
- **Two trial takes came before these**, and their data is not kept: the
  driver ran uncommitted. They found faults in the driver, not in svrngs
  (e0020a16's body), and met the three things "What it found" says.
- Kim's and Bo's Jellyfin are a stand-in (`jellyfin.mjs`), a page with one
  `<video>` and a 60 s film, lent with `svrngs share <port> --kind
  jellyfin`. svrngs uses n0's public relays, its default. Every computer
  and browser ran on this host.

## Against the predictions

The verdicts are in `data/trial/verdicts.json` and
`data/plants/<plant>/verdicts.json`. Times are from the trial take. Each
counts from the press, or from when the driver asked after the page call
returned.

| | predicted | as it ran (trial take) | trial | its plant |
|---|---|---|---|---|
| 1 | Kim's link at 48 hours, offered 48, 24 and 6, within 5 s | "48 hours" chosen of "48 hours", "24 hours", "6 hours"; the link on the page's address, `#visit=`, **222 ms** after the press | passed | — |
| 2 | Sam watches within 5 minutes, told "Sam (Kim's guest)" with the visit's key | asked a name once, then "You’re Kim’s guest until 10/22/2026, 11:14:53 AM", 172,800 s from the act; "Kim’s jellyfin" alone listed. The film played **6,129 ms** after Sam opened the link. All 16 of Sam's requests at Kim's stand-in said "Sam (Kim's guest)", with the visit's key and the account `Sam-6158c7419894e5b6.guest` | passed | `host`: **failed**, all 16 told "Kim", account `Kim-719f06ea17d97c81` |
| 3 | Bo's door refuses Sam's pass, said which fact | Bo's computer listed Sam **49.3 s** after Sam got in (the driver waited 45,095 ms for it, from Sam's film playing); the pass Sam's browser then signed for Bo's jellyfin was refused: "Jellyfin is not Kim's to open to you." No request of Sam's reached Bo's stand-in | passed | `bo`: **failed**, Bo's film opened to Sam, 16 requests |
| 4 | every home page shows Sam within 30 s, Stop on Kim's alone, one `until` | each said "Sam, Kim’s guest, until Thursday 11:14 AM", with one `until` on every computer, Stop on Kim's alone; the last page **53,660 ms** after Sam got in. The pages were read after step 3, which waited for Bo's computer to hold the visit, so the 53.7 s holds that wait; Bo's computer held it after 49.3 s, past the 30 s either way | failed | — |
| 5 | Kim's visit until tonight, 6 hours, Jo in and on every page within 30 s | "6 hours" offered; Jo's visit 21,600 s; the film played in 1,351 ms; every home page listed Jo within **3.3 s** | passed | — |
| 6 | Max, the third guest, is a guest on no copy, then or later | Max's page: "This link lets nobody in. Ask for a new one."; Ana's log: written, `counted=false`. Kim's computer held Max's act after **56,380 ms**. After Kim's leave, Ana's and Bo's computers each listed "Max, Ana’s guest, until Thursday 11:15 AM" ("What it found", 1) | failed | `count`: **failed**, Max in as "You’re Ana’s guest", listed on every copy from step 6 |
| 7 | Kim's Stop ends Sam's visit on every copy within 30 s | Kim's and Bo's computers dropped Sam in 4 and 3 ms, Ana's in **58,970 ms** ("What it found", 2); Jo still visiting; Sam's film page, opened again: "Kim ended your visit." | failed | `lapsed`: **failed**, Sam's film page opened again |
| 8 | tonight's visit ends then, on every copy | not run under `TRIAL` | never-ran | — |
| 9 | Kim's leave ends Lee's visit on every copy within 30 s | Lee watched Kim's film. Ana's and Bo's computers held Kim out in 5 and 4 ms, Lee no longer listed; Lee's film page: "Kim is no longer a member, so your visit ended." But both home pages then said "Visiting now" with Max ("What it found", 1) | failed | `lapsed`: **failed**, Lee's film page opened again |
| 10 | no page says record, key, fold, standing, signer or actor | none of the 61 reads did | passed | — |
| 11 | every page drawn, every step finished, no page error | 31 steps, none failed, no page error | passed | `lapsed`: **failed**, its own: the film pages that opened said no words |
| 12 | a visit with real people, the operator's measure | not run | never-ran | — |

Each plant failed its own bar. The four other failures, 4, 6, 7 and 9,
came in every planted take too, from the two things "What it found" says
first. Bar 5 failed in the `host` and `bo` takes, where Jo reached Bo's
computer after 57.4 and 54.8 s.

## What it found

1. **A leave that reaches a copy before the leaver's last act makes a
   refused visit a visit there.**
   - Kim's computer wrote Lee's visit 2.7 to 4.0 s before Kim's press of
     **Leave Oak** ended (`summary.json`: `lee`, `steps` `9-leave`). The leave reached Ana's
     and Bo's computers in milliseconds, told as the Remove alone
     (`crates/svrngs-node/src/removal.rs:211-222`, "the cut, `lines` and
     nothing else"). Lee's visit, the leave's `prev`, waits for a round,
     and after the leave no round carries Kim's acts.
   - On a copy without it, the leave covers only itself
     (`crates/svrngs-core/src/membership.rs:515-556`, a `prev` chain this
     copy does not hold). Kim's earlier acts then count nowhere: each
     such copy's `/v1/rail/log` says four of Kim's acts, Sam's and Jo's
     visits among them, came "after they were removed from the group", and
     names the missing act, Lee's visit, as an `unheld_cut`
     (`summary.json`, `end_record`, beside `sam`, `jo` and `lee`).
   - With Sam's and Jo's visits gone from that copy's fold, Max's visit,
     judged again in record order, has no guests before it, and counts:
     "Max, Ana’s guest" on the home page. Max's page was told "This link
     lets nobody in", and Max's visit key was never kept, so Max opens
     nothing. But the house is shown a guest who is not one, and the copies
     disagree: Kim's copy, holding every act, lists nobody.
   - In all five takes, Ana's computer lacked Lee's visit and listed Max.
     Bo's lacked it in the trial, `count` and `lapsed` takes, and listed
     Max; in the `host` and `bo` takes it held Lee's visit, and did not.
   - A Visit is where the run met it. Any act a leaver makes in the last
     round before leaving can do the same to the leaver's earlier acts on
     another copy: #211.
2. **A visit and its Stop reach the house only at the next round.** A
   round runs every 60 s (`crates/svrngs-node/src/ring_sync.rs:107`). A
   join wakes it at once (`crates/svrngs-node/src/redeem.rs:157-166`), and
   so does a removal (`removal.rs:205`). A Visit (`redeem.rs:186-226`) and
   a Stop (`crates/svrngs-node/src/home.rs:437-451`) do not. Bo's computer held Sam's visit
   49.3 to 50.9 s after Sam got in, in all five takes (`summary.json`, `steps`
   `2-visit` to `3-page`); Kim's Stop reached Ana's computer
   after 51.8 to 59.5 s in all five. Until then, a guest the host stopped
   is still listed on the others' pages, though every door already refuses
   them where they are lent: a door asks its own copy
   (`crates/svrngs-node/src/door.rs:979-981`), and Kim's computer dropped
   Sam in 2 to 4 ms.
   #212.
3. **A visit past a bound is reported as an act of no member.** Max's act
   is Ana's, a member's. Every computer's sync says of it, again and
   again, "this answer covers a subset — the rail could not account for
   every op … NotAMember", from the moment it is written: 7 to 19 times
   in each computer's kept log in each take, and 0 to 2 in `count`'s,
   where Max's visit counts (each take's `<computer>.log`). Bo's
   `/v1/rail/log` names it a `not_a_member` gap where it holds every act
   (`data/plants/host/summary.json`, `end_record`). GUESTS §3.7 has a visit
   past a bound "simply not a visit"; the record calls its signer no
   member. #213.

Nothing here is fixed: this row carries nothing to the product but the
driver's own fixes.

## The close

Not yet run: once the clean take is in.
