# svrngs onair — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so.

## What the operator decided, 2026-10-13

- On ON_AIR, while the radio idea was explored (2026-10-12): "what's the
  minimal primitives we would want to have in svrngs to allow someone else
  to realize a vision like this? This is largely just a test of the
  infra". Then "what's the minimal scope PoC we could build to prove the
  primitives?", which ON_AIR §7 answers.
- 2026-10-13: "Add the on air spike too". It is queued after the start
  queue (`ralph/next/start/`).
- **The budget is raised for this queue**: the repository's cap from 15,950
  to 16,250, and svrngs-core's from 3,130 to 3,190. Each raise goes in its
  own commit, with the operator's words and the counts, made by the first
  row that needs it.
- **Which key signs for an app: the computer's own**, the one that signs its
  acts (ON_AIR §4.2). Claude proposed this to the operator on 2026-10-13 and
  they did not object. Saying to a stranger that a key is a person's is not
  offered.

The design is svrn-docs `rings/current/ON_AIR.md`: §4 the three primitives,
§7 the proof of concept and its table of bars and plants. DESIGN stays
normative, and each primitive rewrites it in place as it lands.

## Why

- **The test of the infra.** Could someone else build a radio show on
  svrngs: an evening on air to their rings, then to the public, then off?
  ON_AIR's answer is three primitives and no public door. Signed,
  hash-named bytes need no trusted carrier, so svrngs runs none (ON_AIR §1,
  §2; NORTH_STAR 3).
- **What exists** (ON_AIR §3):
  - group folders move bytes by hash, verified, from whichever member holds
    them, but only for a synced directory, and with no lifetime
    (`crates/svrngs-node/src/folder.rs:1-13`);
  - a lend's label and a pass each sign under a domain of their own
    (`crates/svrngs-core/src/pass.rs:32`, `:36`);
  - the live lane's receiving route holds the sender's verified key, but
    buffers the payload alone (`rail.rs`, `ring_routes.rs`, `ring_live`).
    DESIGN §9 keeps that as a row: "a live payload naming its sender", its
    trigger "an app needs to know who offered a live payload"
    (`docs/DESIGN.md:2191`). This queue's app is that trigger.
- **Beyond radio**, MOONSHOTS names who else needs these. P1, bytes by
  hash: witness, making things together, the group's own model. P2, an
  app's signature: instrument, firm. P3, the live sender: play, when the
  centre fails.

## Decide

- Row order, and splitting or re-scoping a row that cannot be done as
  written.
- The routes' exact shapes, starting from ON_AIR §4's sketches.
- How a lapse is kept, and what "lapsed" says apart from "never held".
- How the provider's rule (a hash served only to a dialer standing in the
  group) learns a hash an app placed, with one decider shared with
  folder.rs.
- The app's own nouns (`apps/on-air/nouns.txt`).
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for any of these:

- the budget past 16,250, or svrngs-core's past 3,190;
- a real CDN, a bucket, an account, or a public address; 10,000 listeners
  (ON_AIR §5's arithmetic);
- a second computer, a phone, Safari or HLS (ON_AIR §7, "Left out on
  purpose");
- the sound, heard once by a person (not a bar);
- music anyone holds rights to: the station plays audio generated here or
  under CC0, said where it came from;
- a new crate, or a dependency on a commonwealth-ai or sovereign crate. The
  page's `@noble/curves` and `@noble/hashes` may be vendored into
  `apps/on-air/page/`, pinned, with their licences;
- `sudo`, a system service installed on this host, or opening a port on
  this host's network;
- docker, OrbStack or any tool that is not open software;
- pushing, history rewrites, `--no-verify`;
- anything whose evidence you could not reproduce.

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
