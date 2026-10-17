# The grapevine scene under svrngs, on one host — setup and predictions, 2026-10-16

Row gv-run (b) (`ralph/next/grapevine/STATE.md`): GRAPEVINE_DEMO §10's
scene, three people on three computers, each computer a data dir and
svrngs of its own on this host, each person's grapevine
(`apps/grapevine/`, at 88b814b7) in record mode on their own computer.
Written before the driver and any data; `README.md` here says what the
run gave, against each line below. Every person is a program: no people
bar is judged (AGENTS.md 7). One host, one run: not a rate.

## 1. Setup

`W=<scratch> python3 demo/runs/grapevine/2026-10-16/run.py`, data in
`data/`; with `PLANT=<name>`, the same scene against a planted copy of
the app, data in `data/plants/<name>/`. `judge.py` reads a take's
`summary.json` and writes its `verdicts.json`, one verdict a bar
(AGENTS.md 6: passed, failed, could-not-judge, never-ran).
`scripts/check.sh` runs none of this.

- **svrngs** as the Events run built it (`demo/runs/events/2026-10-14/`):
  `cargo build -p svrngs` at this commit; each computer a `HOME`, `USER`
  and `SVRNGS_DIR` of its own under `W`, each svrngs run by the driver,
  none under launchd. n0's public relays, the default.
- **The town** (§10).
  - Sam makes 1400 Page (`svrngs new "1400 Page" --as Sam`, which runs
    on); Jo joins by its link (`svrngs join <link> --as Jo`).
  - Jo's svrngs stops and makes Waller block the same way (`new` claims
    the data dir, as Sam's made dWeb in the Events run); Kim joins by its
    link. Jo's computer then holds both groups, Sam's 1400 Page alone,
    Kim's Waller block alone.
- **The start**, each person's one line, as `demo/grapevine.md` will give
  it: a config `{"record": <their svrngs's api>, "data": <a folder>}` and
  `node apps/grapevine/server.mjs <config>` (the app copied to `W/app`,
  a plant applied there when one is named), `GRAPEVINE_DEBUG=1` to a
  trace of its own. Each start line prints `{"port", "at"}`; the person's
  browser opens that `at`, `http://127.0.0.1:<port>/` (§13.2). No
  terminal after this.
- **The people.** Sam, Jo and Kim each a program in Chromium of their
  own (`people.mjs`, after the Events run's), pressing only buttons
  their page shows, by their words, and filling fields by their labels.
  A person waiting for something reloads their page every half second,
  as someone refreshing it would, up to a bound (60 s); a wait that ends
  at its bound is said, the step's later bars could-not-judge, never
  passed.
- **The scene**, each step timed from Sam's press of **Ask**:
  0. (before the clock) Kim lists a tile saw in Waller block, under
     **What you have**, **List it**.
  1. Sam asks, under **Ask your groups**, `tool`: What `tile saw`, When
     `Saturday`, In 1400 Page, How far "One ring away", **Ask**.
  2. Jo's page shows **Carry to Waller block**; Jo presses it.
  3. Kim's page shows the ask "carried by Jo" and "You might have this";
     Kim offers When `Saturday morning`, Where `Waller and Pierce`,
     **Offer**.
  4. Sam's page shows the offer; Jo's says a sealed message went by.
  5. Sam says **Yes** as `Sam Reyes`; once Kim's page shows Sam said yes,
     Kim says **Yes** as `Kim Okafor`; each page then names the other.
  6. Sam presses **Take it back**; Kim's page no longer shows the ask.
- **The clock on the loop.** While the scene runs, the driver reads
  Jo's computer's copy of 1400 Page and of Waller block every 0.2 s
  through Jo's svrngs (`/v1/rail/log`), for when Jo's computer first
  holds Sam's withdrawal and when Jo's correction of the carry is first
  in Jo's Waller block.
- **After.** Each computer's copy of each of its groups' records, read
  through its svrngs and kept in `data/`. Then every process stopped,
  and every file under `W` (each data dir, each svrngs's log, each
  grapevine trace and state file) and every record kept, searched for
  each sealed word (`Saturday morning`, `Waller and Pierce`, `Sam
  Reyes`, `Kim Okafor`) and each secret key in the state files; the
  same search for `tile saw`, which the ask holds in the open, is the
  instrument's own check: it must be found in each computer's data dir,
  or the search could not have seen a plaintext there.
- **Plants**, each the whole scene against a copy of the app with one
  change, the change checked to have applied once:
  - `unsealed`: `seal.mjs` returns the body as its box (bar 3's plant,
    `laws.test.mjs`);
  - `spread`: the carry copies the ask whole (bar 1's plant);
  - `uncorrected`: the fold never tells the loop to correct a carry whose
    ask was withdrawn (bar 5's, the canary's plant).

## 2. Predictions

1. **The town.** Each computer's `/v1/groups` names its person in each
   group it holds (Sam in 1400 Page; Jo in both; Kim in Waller block),
   and each page shows exactly those groups: "me" read per group from
   svrngs (§13.3), never a config's.
2. **Each press lands** within 2 s on the presser's own page (an append
   on loopback; Events' were 64–89 ms).
3. **Jo sees the ask** within 10 s of Sam's press, with **Carry to Waller
   block**; Sam's page offers Sam no carry (one group).
4. **Kim sees it** within 10 s of Jo's press, "carried by Jo", never
   Sam's name, with "You might have this"; Jo's and Sam's pages never say
   it.
5. **The offer reaches Sam** within 15 s of Kim's press: Waller to Jo's
   computer, one turn of Jo's loop (5 s), 1400 Page to Sam's. Jo's page
   says a sealed message went by, with its size and time, and none of its
   words.
6. **Names after two yeses.** Kim's page, holding Sam's yes, names no
   one; after Kim's yes, both pages name the other, each within 15 s.
7. **The withdrawal.** Kim's page drops the ask within 20 s of Sam's
   press; Jo's correction is in Jo's Waller block within one loop of
   Jo's computer holding the withdrawal: at most 6 s (RELAY_EVERY, 5 s,
   and 1 s for the pass and the 0.2 s read).
8. **The whole scene**, Sam's Ask to the ask gone from Kim's page, under
   three minutes; predicted under 90 s.
9. **Bar 1 on the records.** In every copy of Waller block, the carry is
   written as Jo's (`on_behalf_of` or `person`), holds no field naming
   Sam, 1400 Page's id or name, or the ask's handle, and no act there is
   Sam's.
10. **Bar 3 on the records.** No file under `W` but the state files, and
    no record kept, holds a sealed word or a secret key; `tile saw` is
    found in every computer's data dir.
11. **Bar 2** as the pages say it: once carried, Jo's page says Waller
    block already has it, under "Where it can’t go".
12. **No page** says key, member, record, act, hash, handle, hop or seal
    (fence 9), and every conversation says the shortcut (§6).
13. **The plants:** `unsealed` fails bar 10 (the offer in the open in
    every record of both groups) and stalls at step 4; `spread` fails bar
    9 (Waller's carry holds Sam's ask's handle and seal key, and names
    Sam if the fold's ask does); `uncorrected` fails bar 7, the carry
    never corrected and Kim's page still showing the ask at the bound.
    Each holds every bar its change does not touch, up to where it stalls.
14. **Not run here:** bar 6 (two carriers), the GRAPEVINE §7 bar 4 (a
    carrier's computer off) and bar 7 (the group of two); GRAPEVINE §7's
    product bars and its kill bar, which are runs with real neighbours,
    the operator's: never-ran. A second host and a phone: not this run.
