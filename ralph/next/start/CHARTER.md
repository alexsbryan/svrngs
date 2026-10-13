# svrngs start — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so.

## What the operator decided, 2026-10-13

- "Let's sort out what's next in the queue as we approach our v1 … I'm
  thinking I want to nail down the svrn / llama-server on svrngs story and
  make it part of an early demo document for anyone getting started."
- Asked, the operator chose:
  - **the story is llama-server alone**: a friend's llama-server, lent;
    its chat page on any member's computer or phone, and an
    OpenAI-compatible address for their tools that stays put. Open WebUI
    in front and svrn's recipes are later chapters (svrn-docs
    `rings/current/SVRN_IN_SVRNGS.md` §4, §7-§8);
  - **the document is for newcomers, in svrngs**: a run-of-show in `demo/`
    with its bars and its recorded run (`demo/README.md`), written from
    what the run did, with the README's "Run it" pointing to it;
  - **the budget is raised for this queue**: the repository's cap from
    15,800 to 15,950; svrngs-core's stays at 3,130. The raise goes in its
    own commit, with the operator's words and the counts, made by the first
    row that needs it.
- "Also maybe want to focus on making sure our localhost dashboard really
  makes the system easy to navigate for someone who is a bit afraid
  technically". The dashboard is the home page, `http://svrngs.localhost:<port>/`
  (`crates/svrngs-node/src/home.rs`, `home/`).
- "And sure we can queue #126–#129 and #131" (`ralph/REVIEW_FINDINGS.md`).
- After start-measure (`demo/runs/start/2026-10-13/README.md` §5), asked,
  the operator chose:
  - **+50, to 16,000**, so the home page can make a link to invite someone
    and open the group's folder (the ranked list's rows 10 and 12);
  - **leaving a group is designed after this queue**, not built in it. No
    command for it exists, and who may remove whom is the core's
    (DESIGN §6). Until then the home page says plainly that leaving is
    not possible yet.
- After start-home, which found that a computer set up by the one line
  cannot make a group (`svrngs new` is refused while svrngs runs from
  login, and the page had no way to do it;
  `demo/runs/start/2026-10-13-home/NEEDS_HUMAN.md`), asked, the operator
  chose **"+40 now, before start-run"**: the cap to 16,040, for making a
  group on the home page (start-group). start-run had found the founder's
  half too: with no link there is no one line (a136f60), so start-group
  carries that as well, within the same cap if it fits.
- "I think we need to schedule an 'economizing' row. We keep just raising
  the bar without considering how we could lower the floor". Asked, the
  operator chose **"Back under 15,800, then ratchet"**: cut at least what
  2026-10-13's raises added, then lower the cap to where the cuts land, so
  the room freed is never spent silently. And **"Every raise needs a pass
  first"**, a standing rule: a raise is asked only after an economizing
  pass, its commit naming what the pass cut (start-economize writes it
  into AGENTS.md and DESIGN §5).

## Why

- **One run proves the lend, on one host** (`demo/runs/llama/2026-10-11/`).
  A stock llama-server was lent with
  `svrngs share 18089 --name llama --kind llama --limit 4 --refuse /slots --refuse /lora-adapters`.
  Another member's browser opened its chat page, and its API answered,
  whole and streamed, 3 ms slower than straight. The run left unmeasured
  two homes, a phone, and several people at once.
- **The address moves.** The record API, and with it the home page and
  every lend's address on a member's computer, listens on "any free port"
  (`crates/svrngs/src/cli.rs:214-218`, `API`, port 0). So it changes each
  time svrngs starts: a bookmark breaks, and so does a tool's base URL.
  Whether programs other than browsers resolve `*.localhost` at all is
  unmeasured.
- **The lender types a lot.** `well_known.txt` does not name
  llama-server. Its default port, 8080, is Open WebUI's and many dev
  servers' too, so svrngs cannot know it by port alone.
- **The home page lists; it does not lead.** In 220 lines
  (`home/index.html`, `home.js`, `home.css`, `question.js`) it shows
  groups, people, what each lends, the group's folder, this computer's
  lends with their limits, and what apps use. Every other act a newcomer
  has (make a group, a link to invite someone, a phone or another
  computer, share something, remove someone) is a terminal command. And
  its own address moves.

## Decide

- Row order, and splitting or re-scoping a row that cannot be done as
  written.
- How an address stays put. The choices: picked once and kept in the data
  dir, or a port svrngs names. Either way, what happens when the port is
  taken is said, never silently moved.
- How svrngs knows a service is llama-server. The choices: one word from
  the lender, with what the kind keeps back held in one place; or svrngs
  asking the service what it is.
- Which acts move from the terminal to the home page, by start-measure's
  walk, and which stay and why.
- The document's name.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for any of these:

- the budget past 16,040, or svrngs-core's past 3,130, and a raise asked
  before an economizing pass;
- a cut that would change what a person sees or a test asserts. A home-page change
  that does not fit is written there with its line estimate, and the rows
  that fit go on;
- a second computer or network (two homes), or a real phone on cellular
  (say what is needed, and the command that would run it);
- a run with real people, for any bar a person has to judge (AGENTS.md 7);
- publishing, `deploy-page.sh` or `deploy-relay.sh`, and flipping the
  relay switch;
- `sudo`, a system service installed on this host, or opening a port on
  this host's network;
- a new crate, or a dependency on a commonwealth-ai or sovereign crate;
- changing llama.cpp, or lending any app other than llama-server;
- docker, OrbStack or any tool that is not open software;
- pushing, history rewrites, `--no-verify`;
- anything whose evidence you could not reproduce.

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
