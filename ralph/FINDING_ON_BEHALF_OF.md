# Finding: an app can't see which computer wrote an act

> From a canon session, 2026-10-16, by reading svrngs at `542a155a`. Not
> run against a live group. For the agent running the svrngs work: turn it
> into a row (or fold it into `gv-fold`/`gv-page`) as the charter allows;
> part 3 is the operator's.

## The finding

Any member's computer can write an act in any other member's name, and
every app's fold credits it to that person with nothing to tell them apart.

1. `POST /v1/rail/append` takes `on_behalf_of` from any caller on loopback
   and checks only that it names a person in the group
   (`crates/svrngs-node/src/rail.rs:186-199`, `:229-236`). Loopback is the
   only gate ("Localhost is the trust boundary", `docs/DESIGN.md:2249`).
2. The act is signed by that computer, so the log keeps the truth: `person`
   is whose computer signed, `on_behalf_of` whom it speaks for.
3. Every fold throws the first away:
   - `apps/expenses/expenses.js:132`
   - `apps/events/events.js:123`
   - `apps/grapevine/grapevine.js:202` (landed today, `gv-fold (b)`)

   each `by: o.on_behalf_of ?? o.person`.

**Scene.** A script, or an agent, on Cy's laptop posts
`{op: <expense: Bo paid $400>, on_behalf_of: "bo"}` to Cy's loopback API.
Every copy's Money shows "Bo paid $400". Nobody's screen says Cy's computer
wrote it.

**Why now.** V1 makes this the ordinary path, not the edge: Bo's phone
writes to Money through Cy's computer (`svrn-docs/rings/current/V1_UI_BUILD.md`,
DESIGN:563-565 "a phone's own writes … through an app naming the person in
`on_behalf_of`"). A vouched write and a forged one look the same. It is
also the case `COMMONS_COORDINATION_DESIGN.md` warns of: "a signed carrier
does not authenticate every author assertion in its payload", and "an agent
must not inherit the operator's authority merely by running on the same
machine."

## The fix, smallest first

1. **Keep `via` in every fold** (apps only, nothing in the core): each
   `actsOf`-style map adds `via: o.on_behalf_of && o.on_behalf_of !== o.person
   ? o.person : null`. One line per app, three apps. `by` is unchanged, so
   every existing law holds.
2. **Say it where a person reads the act.** Money's entries, Events' replies,
   grapevine's asks: "Bo, through Cy's computer" when `via` is set. Words
   per DESIGN §1: names and computers, never key, act or member. Own-device
   writes (`via` null) read as today.
3. **The operator's, not this row's:** whether `append` should refuse an
   `on_behalf_of` the door didn't resolve in this request. That narrows
   step 1 of the scene to a page someone actually opened, but other copies
   can't check it after the fact, and it touches DESIGN §8's boundary. Write
   it to `NEEDS_HUMAN.md` with this file's scene; don't decide it.

## Check (red first, per `ralph/PROMPT.base.md`)

- **Plant:** a log op `{person: "cy", on_behalf_of: "bo", payload: <expense>}`
  in each app's fold test (`expenses.test.mjs`, `events.test.mjs`, the
  grapevine laws). Red: the fold returns no `via`. Green: `via: "cy"`, `by:
  "bo"`, balances unchanged.
- **Plant:** the same op with `person === on_behalf_of`. `via` null.
- **Page:** a browser test (`crates/svrngs-node/tests/browser/`, Chromium
  and WebKit) that Money shows "through Cy's computer" for the planted
  entry, red first with the label out.
- `scripts/lean_check.py` before and after; expected well under 40 lines
  across `apps/`, zero in `crates/`.

## Not in scope

- Restricting which computers may write for whom (part 3, the operator's).
- Per-process identity on a computer (upstream's tie, DESIGN §9).
- canon. It needs this before it can say "Jo, through Cy's computer"
  (`svrn-docs/rings/current/CANON_IN_SVRNGS.md` §4), but that's canon's, later.
