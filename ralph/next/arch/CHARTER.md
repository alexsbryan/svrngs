# svrngs arch — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-15

- **The audit.** The operator asked for "an actually architectural audit
  looking for opportunities to consolidate and economize structurally and
  from a first principles level". The result is `AUDIT.md` here.
  - It read the code at 5bf9941 and ran nothing.
  - Its estimate is about 800 code lines, about 175 of them in svrngs-core.
  - Every line it cites is a claim to read again at HEAD before acting on
    it. Code may have moved since 5bf9941; v1ui-polish was editing door.rs,
    lend.rs and carried.rs as it read.
- **The bridge.** "Right after v1ui-polish". The bridge is arch-peer, AUDIT
  §2.1.
  - Each member is reached through a TCP listener on `127.0.0.1`, on a port
    derived from the member's key (`guest.rs:252`, `iroh.rs:363`).
  - That listener accepts anyone (`guest.rs:307-450`).
  - The friend's door carries a request that has no pass as the member
    (`door.rs:989-1003`).
  - So any program on the computer can reach the group's lends as the
    person, and possibly a web page can too. This was read in the code, not
    run. The row's first part runs it.
  - This queue runs after the v1ui queue closes and before groups.
- **The seal.** "Remove it". It is arch-seal, AUDIT §2.2, and it answers
  svrn-docs `rings/current/A_RECORD_FOR_YEARS.md` §6 question 2.
  - A seal any copy holds retires nothing.
  - An app can no longer append one.
  - The original stays in commonwealth-ai.
- **The other moves.** "The command asks the node, The core's traits, Stop
  signing the view, One version check": arch-command, arch-core, arch-view
  and arch-version (AUDIT §2.3, §2.4, §2.9 and §2.5).
  - Stopping the view answers A_RECORD_FOR_YEARS §6 question 1.
  - arch-command lands before groups-leave, so `svrngs leave` is written as
    an ask of the running node from its first line (groups-leave already
    says so).
- **The budget.** "Up to +60 inside a row". There is no raise.
  - Between its own commits, a row may sit up to 60 code lines over the
    count it started at, for a cut that has to build its replacement first.
  - Each row closes below the count it started at.
  - Each row lowers `scripts/lean_check.py`'s caps, the repository's and
    svrngs-core's, to where it closes, in its own commit with the counts
    before and after.

- **The helper, first** (the operator, 2026-10-15). The answer was "First row of
  arch, v1ui's 31 lines". After Priya lent Jellyfin beside its helper, her
  page offered the helper itself, on 8097, as something to lend
  (`demo/runs/v1ui/2026-10-15-helper/README.md` finding 2). That row is
  arch-helper.
  - It is the one row that may raise, within the 31 lines v1ui's +650 left
    unspent, up to 16,965.
  - The rest of the queue keeps the no-raise rule.
  - It also takes v1ui's closing audit's findings, #169–#173
    (REVIEW-audit-v1ui-auto-3): all five are v1ui's code, so they come out
    of v1ui's unspent lines too.

- **arch-peer's close, and the streaming test** (the operator, 2026-10-15).
  The answer was "Close it, arch-stream owns it".
  - arch-peer may close on a `scripts/check.sh` whose only red is
    `a_page_speaks_http_through_the_door_streaming_both_ways`, provided
    that test is red the same at the row's start (965b52b).
  - Its `done` names the test and both reds, and every other step must have
    passed. Never-ran is never accepted.
  - arch-stream, the next row, owns the test.
  - From arch-stream's close on, every row closes on check.sh at exit 0
    again.

## Why

The bar is AGENTS.md "Done": a raise is asked only after an economizing
pass, and a pass's cuts lower the cap. The audit is the economizing pass
the last three queues lacked.

The audit cuts by structure rather than clones:

- **One decider.** Today two makers of a group and of a link disagree on
  the relay (`cli.rs:1457`, `group.rs:315-319`), and they part when
  `SHIPPED` flips.
- **Layers kept for history.** The bridge was built for ggml's rpc-server
  (`guest.rs:471-480`).
- **Machinery your own drafts already call wrong.** The seal is
  A_RECORD_FOR_YEARS §2.
- **Bytes nothing reads.** The view is 25–39% of a journal's bytes on
  this host's runs.

The core's lines are worth the most. It sits at 3,149 of 3,150, and groups
asks +40.

## Decide

- Row order and splitting, within this order:
  1. arch-helper, then arch-peer.
  2. arch-command before the queue closes.
- The shape of the request helper on the stream: one module, or the server
  side's.
- Where a seal line already on disk is read, and how it is said.
- The version's form, and the words for "runs an older svrngs".
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going. A finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget:** a row closing above the count it started at, or more than
  +60 over it between its commits;
- **existing groups:** any change that alters an existing group's keys,
  makes an existing member someone new, needs a migration of what a
  computer holds on disk, or makes two computers in one group show
  different things. This includes a seal found in a journal on this host
  outside a test;
- **the oldest build:** any svrngs build that has left this host, which
  arch-version's survey lists;
- **a new kind of act** in the record;
- **people's own judgement:** cellular, a second computer, or a run with
  real people;
- **dependencies and tools:** a new crate, a dependency on a
  commonwealth-ai or sovereign crate, docker, OrbStack, or a tool that is
  not open software. Removing reqwest from the product is not one;
- **outward-facing steps:** publishing, `sudo`, a system service, or a port
  opened on this host's network;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.** The bridge's
  exposure included: if arch-peer (a) cannot show it, stop and say so.

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
