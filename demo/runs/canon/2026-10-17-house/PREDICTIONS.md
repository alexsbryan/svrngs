# A house's rules on one host — setup and predictions, 2026-10-17

Row canon-run (`ralph/next/canon/STATE.md`): a house of four on three
computers and a phone brings House rules, answers its question, makes a
rule, loses two people and still decides, and one of them replaces House
rules' files. What canon-house (a0b3b5e5a), canon-share (c2b48b4ef, canon
0.5.0 vendored by faf50750) and canon-pin (59348697c) built, run as people
meet it, every computer and phone on this host. Written before the driver
and any data; `README.md` here says what the run gave, against each line
below. Every person is a program: no people bar is judged (AGENTS.md 7).
One host, one run: not a rate.

## 1. Setup

`W=<scratch> python3 demo/runs/canon/2026-10-17-house/run.py`, data in
`data/`; with `PLANT=<name>`, the same story against a build of svrngs with
one planted change, data in `data/plants/<name>/`. `judge.py` reads a take's
`summary.json` and writes its `verdicts.json`, one verdict a bar (AGENTS.md
6: passed, failed, could-not-judge, never-ran). scripts/check.sh runs none
of this.

- **svrngs** as groups-run built it (`demo/runs/groups/2026-10-16/`):
  `cargo build -p svrngs` at this commit, House rules embedded from
  `apps/canon/` by the shelf (`crates/svrngs-node/build.rs`), each computer
  a `HOME` and data dir of its own under `W`, run with
  `RUST_LOG=…,folder=debug,removal=debug`. The page built by
  `scripts/build-web.sh` and served on this host by `web/tests/serve.mjs`.
  n0's public relays, the default; the phone reaches computers through
  them, as every phone does.
- **The cast.** Elm, a house: Ana's computer, Bo's computer, Cy's
  computer, Dee's phone. Ana makes Elm (`svrngs new Elm --as Ana --origin
  <page>`); Bo and Cy join by its link (`svrngs join <link> --as …`); Dee's
  phone, WebKit as an iPhone (`web/tests/page.mjs`), opens its page address
  and types "Dee". Ana keeps the order of removals, so Bo and Cy are the
  ones who leave.
- **Pages.** One program, `people.mjs`, keeps a browser for each person for
  the whole run: Chromium for each computer, WebKit as an iPhone for the
  phone. A computer opens its home page at `svrngs.localhost:<port>` and
  House rules at the address its own `/v1/lends` lists for it. The phone
  opens Elm's page and presses "House rules" in its list, as a person does.
  Every press is a click on what the page shows, by its words.
- **What's read.** Each page's words after each press; each computer's
  `/v1/rail/log` and `/v1/groups` for when an act or a leave reached it.
  Whose computer wrote the phone's entries is read from the record's
  `person` beside its `on_behalf_of`.
- **The story, in order**, each step timed from its press:
  1. *Bring.* Ana's home page: under Elm's "Bring things to the table" she
     unticks Money and Events and presses "Bring the ticked ones to Elm".
  2. *The question.* Ana opens House rules, ticks "Most of you agree" and
     presses "Make this how your house decides".
  3. *Propose.* Ana writes "Quiet hours are 10pm to 7am" and presses
     "Propose this rule to everyone in your house".
  4. *The phone agrees.* Dee's phone opens House rules from Elm's page and
     presses "Agree".
  5. *An objection.* Cy's computer opens House rules, presses "Object",
     writes "I work late on Fridays" and presses "Send your objection".
  6. *It holds.* Bo's computer opens House rules, reads it, and presses
     "Agree". Ana's, Cy's and Dee's pages read.
  7. *One who agreed leaves.* Bo's home page: "Leave Elm". Ana's and Cy's
     computers watched until each holds him out; Ana's, Cy's and Dee's
     House rules read.
  8. *To "everyone agrees".* Cy opens "Change how your house decides",
     ticks "Everyone agrees" and presses "Propose this change to everyone
     in your house"; Ana presses "Agree".
  9. *Under "everyone agrees", one leaves.* Cy's home page: "Leave Elm";
     Ana's computer watched until it holds her out.
  10. *The house still changes its rule.* Dee's phone opens "Change how
      your house decides", ticks "Most of you agree" and presses "Propose
      this change to everyone in your house"; Ana presses "Agree".
  11. *The plant the pin closes.* Ana proposes "Ana keeps the big room".
      Then she replaces `apps/House-rules/vendor/call.mjs` in Elm's folder
      on her computer with one that reads every proposal as adopted, canon's
      answer rewritten after the build gives it. Dee's phone reads Elm's
      page and House rules; Ana's computer reads House rules; then Dee
      presses "Yes, use the new version of House rules" and opens House
      rules again; Ana's House rules read once more.
- **Plants**, each the whole story against a build with one change, the
  change checked to apply once, the tree put back after the build:
  - `via`: an entry another computer wrote said as its person's alone
    (house.js `recorded`, `via` always null).
  - `now`: only the people in the house now counted, an entry by anyone who
    left dropped before canon is asked (house.js `fold`).
  - `count`: "Everyone agrees" a count of the house on the day it chose,
    three, as canon-house (b) wrote it (8772c1c4; house.js `CHOICES`).
  - `pin`: each computer serving an app's files as its folder holds them
    now, never the version that runs as the person asking (carried.rs
    `file`), as canon-pin (b) planted it.

## 2. Predictions

1. **Ana brings House rules from her home page.** Within 10 s of the press
   her page says "You brought House rules to Elm: everyone in it can open it
   in a few seconds."; nothing else is brought. Bo's and Cy's computers each
   list House rules within 60 s, its 5.7 MB build with it.
2. **The question, answered by whoever brought it.** Ana's House rules
   first says "How does your house decide its rules?" with three choices
   and no default ticked. Pressed, it says "How your house decides" and "A
   rule holds once more than half of the people in your house when it was
   proposed agree, whoever proposed it among them.", within 5 s.
3. **A rule proposed** reads under "What’s proposed" on Ana's page as
   "Quiet hours are 10pm to 7am" and "You proposed it on 17 Oct.", within 5
   s; it reaches Bo's and Cy's computers within 30 s.
4. **The phone agrees through someone else's computer, said whose.** Dee's
   phone draws House rules within 120 s of pressing it, showing the
   proposal with "Agree" and "Object". Its agreement is written by the
   computer the phone reached, whose person is not Dee, and every page then
   says it as "Dee, through Ana’s computer" (or Bo's, or Cy's, as the
   record says; on that computer's own page "Dee, through your computer").
   Plant `via`: red, said as "Dee" alone.
5. **An objection asks why, and counts as the rule says.** "Object" asks
   "Why do you object?"; sent, Cy's page says "You object: “I work late on
   Fridays”", and the rule is still proposed: under "most of you agree"
   two of four have agreed, and one objection does not stop it.
6. **It holds, as the chosen rule says.** Bo's page, before his press,
   shows Dee's agreement said whose and Cy's objection. After it, three of
   four agree, and within 30 s every page — Ana's, Bo's, Cy's computers and
   Dee's phone — lists "Quiet hours are 10pm to 7am (Ana proposed it on 17
   Oct)" under "Your house’s rules" (on Ana's own, "you proposed it").
7. **One who agreed leaves, and the rule still holds as agreed.** Within
   30 s of Bo's press Ana's and Cy's computers hold him out. Ana's, Cy's and
   Dee's House rules still list the quiet hours under "Your house’s
   rules", say "Everyone in your house has a say:" naming Ana, Cy and Dee
   alone, and say Bo had a say until 17 Oct, when they left. Plant `now`:
   red, the quiet hours back under "What’s proposed".
8. **The house changes how it decides, as it decides anything.** Cy's
   change reads under "What’s proposed" as "Change how your house decides:
   A rule holds once everyone in your house when it was proposed agrees,
   and one objection stops it."; once Ana agrees (two of three), every
   page says that under "How your house decides".
9. **Under "everyone agrees", one leaves** and Ana's computer holds her out
   within 30 s; who has a say is Ana and Dee.
10. **The house still changes its rule.** The phone's proposal back to
    "most of you agree", agreed by Ana, holds: both of the two in the house
    agree. Within 30 s Ana's page and Dee's phone say "A rule holds once
    more than half of the people in your house when it was proposed agree,
    whoever proposed it among them." under "How your house decides". Plant
    `count`: red, the change still proposed, two of a count of three.
11. **No one runs a member's replacement of House rules' files until they
    say yes.** "Ana keeps the big room" is proposed, one of two agreeing.
    After Ana replaces `vendor/call.mjs`, within 60 s Dee's phone's Elm page
    says "Ana changed House rules’s files on 17 Oct. Open the new version?"
    with "Yes, use the new version of House rules"; Dee's House rules still
    shows "Ana keeps the big room" under "What’s proposed", and so does
    Ana's own, she having said no yes either. Pressed, the phone says "House
    rules opens at its new version for you now."; opened again, it shows
    "Ana keeps the big room" under "Your house’s rules" — the replacement
    running for Dee alone — while Ana's still shows it proposed. Plant
    `pin`: red, the replaced files running on Dee's phone and Ana's
    computer before any yes.
12. **Words.** No page read says scope, ratification, commitment, act,
    actor, canon, key or record (fence 6).
13. **Every page drawn**, none left saying "Reading your house’s rules…",
    and no page error in any browser.
14. **A house with real people, a second host, a phone on cellular**:
    never-ran, the operator's; every people bar: never-ran.
