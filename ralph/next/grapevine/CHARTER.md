# svrngs grapevine — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-15

- **The app.** The design is svrn-docs `rings/current/GRAPEVINE_DEMO.md`,
  read whole, its §12 and §13 included. It came from these answers:
  - "What would be the simplest demo app for the grapevine functionality?
    I don't necessarily want to build something production grade, but I
    want something that developers can extend for their cool stuff";
  - "put it under apps/ in svrngs, tweetnacl is fine";
  - "start queuing up this work (and doing the due diligence on tech design
    so we can implement as efficiently as possible)".

  It demonstrates `rings/current/GRAPEVINE.md` with no change to svrngs.
- **The V1 cut line.** "And then I think that's our V1 cut line". V1 is the
  arch queue, the groups queue and this app, on n0's relays ("arch, groups,
  grapevine; V1 on n0"). The own relay, #134 and #135 come after V1.
- **The order.** "arch → groups → grapevine". This queue runs last, so its
  recorded run uses the identity model V1 ships, after groups-keys.
- **Taps only.** "Taps only for V1". Every carry is the carrier's tap.
  There is no `/standing` route, no standing scope, and no GRAPEVINE bar 8.
  Stance 9 stays undecided.

- **The leave's last fixes come first** (the operator, 2026-10-16: "One
  row first in grapevine, +30"). gv-leave takes groups-run's #182 and #183
  and the groups closing audit's #184. It is the one row here that may
  change svrngs:
  - up to +30 repository lines over groups' close, so at most 16,358;
  - svrngs-core unchanged.

  Every other row keeps zero svrngs lines.

- **Which computer wrote it** (the operator, 2026-10-16, on
  `ralph/FINDING_ON_BEHALF_OF.md`). For its parts 1–2: "A grapevine row
  before gv-page". That row is gv-via, which is apps and tests only.
  - Every fold keeps `via`, and every page says "Bo, through Cy's
    computer".
  - For part 3: "Not for V1; showing it is V1's answer". `append` keeps
    accepting `on_behalf_of` from this computer.
  - A phone signing its own writes goes to DESIGN §9.

## Why

The grapevine is how an ask crosses rings through people
(GRAPEVINE.md §1). The app shows it running before the node gains notes
and routes (GRAPEVINE §8), so developers can build kinds on it now, and the
kind interface (GRAPEVINE_DEMO §7) is what they keep when it retires (§9).

NORTH_STAR §2's test applies to `kinds/`: could a developer read it all,
and did they have to? A kind is one file and a line, and it writes no
routing, scope, sealing or relaying.

## Decide

- File layout, row splits, and the page's shape, within GRAPEVINE_DEMO §2
  and §5.
- The kind interface's details, within §7: field types are a closed set.
- How the start line opens the person's page (§13.2).
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going. A finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **any svrngs line** outside gv-leave, in `crates/`, `web/`, `bridge/` or `scripts/`
  outside a new check's own test. This includes the per-group "me" of §13.3
  if no answer svrngs gives names this computer's key. `apps/`,
  `demo/runs/`, `demo/grapevine.md` and DESIGN's words are not svrngs
  lines;
- **dependencies:** anything beyond tweetnacl 1.0.3, vendored whole. That
  includes an npm install, a bundler or a framework;
- **standing carries**, counts across rings, the meet's group of two, or
  anything else GRAPEVINE §9 leaves to the operator;
- **people's own judgement:** GRAPEVINE §7's product bars and its kill bar
  are runs with real neighbours, and never-ran here; also a phone on
  cellular, or a second computer;
- **outward-facing steps:** publishing, `sudo`, a system service, or a port
  opened on this host's network. Fetching tweetnacl 1.0.3's published
  tarball once to vendor it is not one;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
