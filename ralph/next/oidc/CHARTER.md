# svrngs oidc — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided

- **The design.** svrn-docs `rings/current/SIGN_IN_WITH_SVRNGS.md` (begun
  2026-10-20, at svrngs `a34edb62`), read whole. Its §5a read fourteen
  apps' clients, §8 resolved every open question, §7 is the trial's bars.
- **The trial.** "let's add the OIDC trial -- I actually want to start the
  tech design now" (2026-10-20); "OIDC trial" (2026-10-21), after the demo
  evening. Of `jellyfin-mw`: "If it's not something anyone in the room can
  run off the shelf I'm pretty turned off by idiosyncratic tweaks for
  specific apps"; and "if there's something that solves a general svrngs
  problem and the n+1 use case for it is now easier then we can do it".
- **The two apps, in this order: Jellyfin's SSO plugin, then Forgejo.**
  Jellyfin first retires the 1,571-line `apps/jellyfin/mw/`, which is the
  operator's reason. Forgejo is the general case and is served without an
  email; Open WebUI needs one (§8 4), and Immich and Audiobookshelf are
  served by neither channel (§5a). §7's bar 1 names Forgejo or Open WebUI
  first; §5a moved the trial's start to the Jellyfin plugin, and §8 4
  leaves Forgejo of that pair. Both apps' bars are §7's.
- **The design's §8 resolutions are the operator's**, and are not reopened:
  no consent screen and nothing typed (1); the issuer plain http on
  loopback (2); ES256 through `ring` (3); no `email` until mail is real
  (4); phones go the same way (5); a guest signs in as a guest (6);
  removal needs nothing new (7); a container uses the host's network (8);
  turned on from the home page, with a `share` flag (9); an app whose page
  navigates to the authorize address itself is not served (10).
- **Podman, never Docker** (2026-10-20: "We never use docker. Podman and
  open code only").

## What the operator decided, 2026-10-21, of the ceiling

The economizing pass (`oidc-economize`) found no cut and estimated the
provider's part at +800 repository and +80 svrngs-core
(`demo/runs/economize/2026-10-21-oidc/SURVEY.md`, 4239bc7c). Asked, the
operator chose the survey's number: at most **+800** repository lines and
**+80** svrngs-core over the caps the pass leaves (16,928 and 3,073, since
it cut nothing), so **17,728** and **3,153**. Each row raises only what it
needs, one past where its commit lands, in its own commit. Past these is
NEEDS_HUMAN.md again.

## What the operator decided, 2026-10-21, of more code

"The budget was already generous. I'm not likely to approve anymore
code." So there is no raise past 17,728 and 3,153. A row that cannot land
within them finds its lines by a cut first (AGENTS.md "Done": a pass
before a raise), and only when no cut exists does it write
`NEEDS_HUMAN.md`, saying what it would need and what it tried. The
remaining rows are expected to fit: the front channel and the switch are
the node's, the runs are `demo/runs/`, and the retirement cuts. The
retirement's cut lowers the caps to where it lands.

## What the operator decided, 2026-10-21, of the sign-in hook

Asked at the budget wall, the operator authorized the cut the survey
named: `crates/svrngs-node/src/door/sign_in.rs` and its wiring retire (the
survey's 420 counted lines, `demo/runs/economize/2026-10-21-oidc/SURVEY.md`,
4239bc7c), because sign-in with svrngs makes the hook userless and the
queue has no room beside it. It lands in `oidc-hook-cut`, its own commit
before `oidc-turn-on` adds a line; `apps/jellyfin/mw/` and the
`helper jellyfin` row stay for `oidc-retire`, which removes them after the
Jellyfin trial, so a failed trial is reverted with git, not re-bought.
DESIGN §3 "The room" call 6 is rewritten in place: the hook is gone, and
sign-in with svrngs is the door's own. A lend declaration holding the old
field is refused as unknown, never ignored.

## Why

The app already knows who the person is: the door sends `x-svrngs-key`,
`x-svrngs-user` and `x-svrngs-person` on every request. Sign-in only lets
an app that will not read headers hear them, in the standard it already
speaks. The alternative is a program per app, 1,571 lines of Jellyfin's
own words, and the next app another (`apps/jellyfin/mw/`).

The line it must not cross: only people the door would carry; nothing
leaves the lender's computer; no new listener; the issuer is the lender's
own computer, never a centre (NORTH_STAR 3).

## Decide

- Where the signature lives within RUST_ARCH and no new crate: svrngs-core
  is `no_std`, and `ring` is in `Cargo.lock` through rustls. The core makes
  the claims and the signing input; the node signs.
- The code store's home, lifetime and one-use rule.
- The declaration's field name, the client id and secret's exact shape, and
  the page's words.
- The routes' exact paths within the design's, and the token's exact form
  (the access token the signed token too, or an opaque one kept in memory).
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that. Where its letter and its spirit part, rewrite the letter
yourself, saying both in the commit, and keep going.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the budget:** past 17,728 for the repository or 3,153 for svrngs-core
  (2026-10-21), and the operator is not likely to raise: find the lines by
  a cut first, and only with no cut left write the ask;
- **the trust floor:** any widening of a page's policy or a listener;
- **a new crate** in the svrngs workspace;
- **an app-specific tweak:** serving an app whose stock client needs code
  in svrngs written in that app's words;
- **what a group holds:** a change to the record or to what an existing
  group means;
- **tools and the outward-facing:** a plugin or app that cannot be pinned
  and recorded, a system service, a port on this host's network,
  publishing;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the
evidence and what would falsify it in the body.
