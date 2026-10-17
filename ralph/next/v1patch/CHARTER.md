# svrngs v1patch — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-16

- **Close the loopback allowance now.** The operator: "Let's just close the
  loopback allowance in the next row".
  - The published group page allows `ws://127.0.0.1:* http://127.0.0.1:*`
    in its `connect-src` because "the browser tests run one"
    (`web/index.html:25-27`, `:41`).
  - The page's own code dials no such thing, but the policy says it may.
  - This is a test convenience shipping in V1.
- **The invariant it serves** (svrn-docs `rings/current/WHAT_RUNS_AS_YOU.md`
  §1): what runs as you, and what can reach you, is what you or your group
  chose to bring. svrngs never widens it silently.

## Decide

- Where the tests add the allowance: when `web/tests/page.mjs` serves the
  page, or a test-only flag on `scripts/build-web.sh`. Either way, the
  published build, which `scripts/deploy-page.sh` ships, never holds it.
- The smaller reversible step.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:
- **publishing:** `scripts/deploy-page.sh` is the operator's;
- **any other widening or narrowing** of a page's policy than this one;
- **dependencies and tools:** a new crate or package;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
