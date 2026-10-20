# A house's rules on one host — 2026-10-17

Row canon-run (`ralph/next/canon/STATE.md`). Elm is a house of four: Ana,
Bo and Cy on computers, Dee on a phone. Ana brings House rules and answers
its question. The house makes a rule and loses two people, and still
decides. Then Ana replaces House rules' files. The setup and predictions
are in `PREDICTIONS.md`, committed (8baf3e4c) before the driver (2b2107aa)
and before any data (a6f71450). Every person is a program, so no people bar
is judged (AGENTS.md 7). These are takes on one host, not a rate.

## How it ran

```
W=/tmp/elm-run python3 demo/runs/canon/2026-10-17-house/run.py                          # data/
TAKE=<2|3> W=/tmp/elm-run python3 demo/runs/canon/2026-10-17-house/run.py               # data/takes/<n>/
PLANT=<via|now|count|pin> W=/tmp/elm-run python3 demo/runs/canon/2026-10-17-house/run.py  # data/plants/<plant>/
python3 demo/runs/canon/2026-10-17-house/judge.py <its summary.json>                    # its verdicts.json
node demo/runs/canon/2026-10-17-house/same_second.mjs                                   # "What it found", 2
```

- **The clean take** ran at 2b2107aa with nothing dirty
  (`data/events.jsonl`, `start`). It took 91.2 s from start to end. The
  1-minute load was 6.1 at the start and 4.8 at the end, on 12 cores
  (`start`; `summary.json`, `end_load`).
- **Two more clean takes** ran at a6f71450, which is f25a46cb with the
  first take's data added (`data/takes/<n>/events.jsonl`, `start`). 9ab38f04's
  body says "at f25a46cb", which is wrong: the head is a6f71450. f25a46cb
  only adds `TAKE`. They were run because the trial takes met a race that
  the first clean take did not (see "What it found", 1).
- **The planted takes** ran at 9ab38f04 (`data/plants/<plant>/events.jsonl`,
  `start`). Each built svrngs and the page with its one change, then put
  the tree back before the story started (run.py `build`; each `built`
  line names the file planted). House rules is embedded from `apps/canon/`
  by the shelf (`crates/svrngs-node/build.rs`), so a plant in `house.js` is
  in the build.
- **Two trial takes came before these**, and their data is not kept. They
  found faults in the driver, not in svrngs (2b2107aa's body). The driver
  read the page once after an Agree that passes a change, but the page draws
  again only when asked. It also stopped at the phone's page after a leave,
  where a person would open the page again.
- canon 0.5.0, as canon-share vendored it (faf50750,
  `apps/canon/vendor/README.md:11`). The default is n0's public relays, and
  the phone reaches every computer through them. Every computer and the
  phone ran on this host.

## Against the predictions

The verdicts are in `data/verdicts.json` (clean),
`data/takes/<n>/verdicts.json` and `data/plants/<plant>/verdicts.json`.
Times are from the clean take. Each counts from the press, or from when the
driver asked after the page call returned.

| | predicted | as it ran (clean take) | clean | takes 2, 3 | its plant |
|---|---|---|---|---|---|
| 1 | Ana's page says House rules was brought within 10 s, and nothing else was; Bo's and Cy's computers list it within 60 s, with its build | "You brought House rules to Elm: everyone in it can open it in a few seconds." in **221 ms**. Ana's computer listed only `House-rules`. Bo's and Cy's listed it at 803 and 806 ms, and held the 5,679,979-byte build, the same bytes as `apps/canon/vendor/canon.js`, at 810 and 813 ms (`listed`, `build`, `kinds`) | passed | passed, passed | — |
| 2 | the question first, three choices, none ticked; answered, the rule said within 5 s | "How does your house decide its rules?", 3 choices, 0 ticked. "A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them." in **221 ms** | passed | passed, passed | — |
| 3 | the proposal under "What’s proposed" within 5 s; on Bo's and Cy's within 30 s | "Quiet hours are 10pm to 7am", then "You proposed it on 17 Oct.", in 235 ms. Bo's computer held it 10 ms after the page call returned, Cy's 2 ms after (`reached_3`) | passed | passed, passed | — |
| 4 | the phone draws House rules within 120 s; its agreement written by another person's computer and said whose on every page | The phone's Elm page listed House rules in 825 ms, and House rules drew **4,760 ms** after the press. Its agreement was written by Ana's computer (`dee_act`: `person` Ana, `on_behalf_of` Dee). The phone said "Agreed: you, through Ana’s computer."; Cy's page and Bo's page each said "Agreed: Dee, through Ana’s computer." | passed | passed, passed | `via`: **failed**, "Agreed: Dee." on Cy's page, "Agreed: you." on the phone |
| 5 | "Object" asks why; Cy's objection said, the rule still proposed | "Why do you object?"; "You object: “I work late on Fridays”" in 227 ms; still under "What’s proposed" | passed | passed, passed | — |
| 6 | three of four agree and it holds on every page within 30 s | Before his press, Bo's page showed Dee's agreement said whose and "Cy objects: “I work late on Fridays”". After it, Bo's, Ana's ("you proposed it") and Cy's computers and Dee's phone each listed "Quiet hours are 10pm to 7am (Ana proposed it on 17 Oct)" under "Your house’s rules". The last page, the phone's, was read **6,029 ms** after the press | passed | passed, passed | — |
| 7 | Bo, who agreed, leaves; held out within 30 s; the rule still holds; who has a say is Ana, Cy and Dee; Bo's leave said | Ana's and Cy's computers held him out at **522** and **523 ms**. Ana's, Cy's and Dee's pages still listed the quiet hours as a rule, said "Everyone in your house has a say:" with Ana, Cy and Dee, and said "Bo had a say from 17 Oct until 17 Oct, when they left." | passed | passed, passed | `now`: **failed**, the quiet hours under "Not adopted", "Too many of you objected or left for it to be agreed." |
| 8 | Cy's change to "everyone agrees" proposed; once Ana agrees, every page says it | proposed in 219 ms; once Ana agreed, her page and Cy's said "A rule holds once everyone in your house when it was proposed agrees, and one objection stops it." Ana's page drawn right after her press still said the old rule, with nothing proposed; opened again, the new one, 2,496 ms after the press ("What it found", 2) | passed | passed, passed | `count`: **failed**, the rule said as "A rule holds once 3 of you agree, …" |
| 9 | Cy leaves under "everyone agrees"; held out within 30 s; Ana and Dee have a say | held out at **522 ms**; "Everyone in your house has a say: Dee and you." under "everyone agrees" | passed | passed, passed | `now`, `count`: **failed** with them |
| 10 | the two left change their rule back to "most of you agree" within 30 s | The phone's proposal, then Ana's Agree: both agree, and it holds. Ana's page opened again said "more than half" **2,341 ms** after her press, and so did Dee's phone. Right after the press, Ana's page drew the old rule again ("What it found", 2) | passed | passed, passed | `count`: **failed**, the change still proposed, "Agreed: you", two of a count of three |
| 11 | after Ana replaces `vendor/call.mjs`, nobody runs it until they say yes | Ana's computer listed the change 580 ms after the file was written. Dee's phone asked "Ana changed House rules’s files on 17 Oct. Open the new version?" **5,845 ms** after. Before any yes, Dee's phone and Ana's computer showed "Ana keeps the big room" under "What’s proposed". Dee pressed "Yes, use the new version of House rules": "House rules opens at its new version for you now.", and Dee's House rules listed the big room under "Your house’s rules". Ana's, with no yes from her, still listed it as proposed | passed | passed, passed | `pin`: **failed**, the big room a rule on Dee's phone and on Ana's computer before any yes |
| 12 | no page read says scope, ratification, commitment, act, actor, canon, key or record | none of the 86 reads did, across home pages, the phone's Elm page and House rules | passed | passed, passed | — |
| 13 | every page drawn, no page error | 26 steps, none failed, no page error | passed | passed, passed | `count`: **failed**, its own words |
| 14 | a house with real people, a second host, a phone on cellular | not run | never-ran | never-ran | — |

Each plant failed its own bar. Where it failed another, the plant itself
is the cause:

- **`via` and `pin`** each failed only their own bar.
- **`now`** failed 7 differently than predicted. The quiet hours were "Not
  adopted", not proposed again. With Bo's agreement dropped and Bo unable to
  agree again, canon judged that the share could no longer be reached. It
  also failed 9: Cy's change was dropped once she left, so the house was
  back under "most of you agree".
- **`count`** failed 10 as predicted, Dee's change stuck at two of three. It
  also failed 8 and 9, because "Everyone agrees" was said as "3 of you
  agree", and 13, because the driver's waits for those words timed out.

## What it found

1. **After a leave, the phone's page can reach the leaver's computer and
   give up.**
   - The page dials the link's computer and each computer it last heard of,
     all at once, and goes on with the first to answer
     (`web/app.js:322-334`, `reachGroup`). A computer whose person has left
     still runs svrngs and still answers the dial.
   - Its `/people/lends` then fails, and the page says "This browser could
     not open the page." (`web/app.js:392`, `:97`). It never tries the next
     computer. Take 2's console, step 7-dee: "reached Elm through Bo's
     computer", then "GET /people/lends — REFUSED — the computer's answer
     is not HTTP: connection error".
   - House rules listed only when the page was opened again, **10,937 ms**
     after the first try (`data/takes/2/summary.json`). The trial takes, whose
     data is not kept, met it after both leaves, for 52.7 s and 134.1 s
     (2b2107aa's body).
   - Takes 1 and 3 did not meet it. It is a race, not a rate.
   - A person a bit afraid technically is told their browser failed, while
     Ana's computer is on: #205.
2. **For about a second after the agreement that passes a change, the page
   shows the change neither proposed nor in force.**
   - The clean take's steps 8 and 10 did this: right after Ana's Agree, her
     page said the old rule with nothing proposed.
   - The page asks canon at the browser's clock (`apps/canon/index.js:53`).
     svrngs times a new act after every act it holds
     (`crates/svrngs-core/src/command.rs:243-250`), so an agreement written
     in the same second as the proposal is timed a second ahead.
   - Asked before that second, canon calls the change ratified and keeps the
     old rule in force. `same_second.mjs` prints:

     ```
     asked at 2099: the change ratified since 2100; the rule in force majority
     asked at 2100: the change ratified since 2100; the rule in force unanimous
     ```

   - The take kept no act's time, so the cause is reproduced, not observed.
     Opened again, the page says the new rule. #206.
3. **What a person waits for here is under a second, but not on a phone.**
   In the clean take, every act reached the other computers within 14 ms of
   the driver asking (`reached_*`). Every leave reached them within 0.6 s,
   and House rules' 5.7 MB build within 0.9 s (1, 3, 5, 6, 7, 9). The phone took 4.5 to 4.8 s
   to draw House rules, across the three clean takes (`drawn_ms`), going
   through n0's relays. That was on one host, and is not a rate.

Neither finding is fixed here; this row carries nothing to the product but
the driver's own fixes.

## The close

`scripts/check.sh` ran once at 52a9fc0d, after this README. It exited 0 in
about 16 minutes. All fifteen gates passed; none failed, could not be
judged or never ran. One WebKit seek through a lend took 13,632 ms. That
was a rate line, inside a gate that passed.

## After the fix

canon-fix fixed both findings: #206 in 5eb3d563 and #205 in 3666d2a4
(`ralph/REVIEW_FINDINGS.md`). `scripts/check.sh` ran once at 6c7b8296 and
exited 0 after 15m36s. All fifteen gates passed. None failed, could not be
judged or never ran.
