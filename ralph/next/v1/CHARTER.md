# svrngs v1 — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and `docs/principles/ARCH_PRINCIPLES.md`
are the compass; where they and this charter disagree, they win and the
decision says so. The queue builds Demo C's units (`docs/DESIGN.md` §8,
"Demo C"; `demo/c.md`), removal first. The rules removal answers to are
already decided (DESIGN §6, "Removal has one decider", "A grow-only set,
linked like git"; §10): a row implements them and never re-decides them.

**Decide:** row order, splitting and re-scoping a row that cannot execute;
the shape of an act, a type or a fold that carries a rule §6 states; how a
fork is held so it voids nothing; what a request to remove carries and where
it lives; the words a person reads, by DESIGN §1's two rules; the smaller
reversible step.

**Bars by their spirit** (operator, 2026-10-08: "the bars are guidance to
align us at the start, the spirit of them are what guides how we
iterate"). Ask what a bar guards and judge by that. Where its letter and
its spirit part, rewrite the letter yourself, saying both in the commit,
and keep going; a finding that keeps the spirit is a follow-up row, not a
stop.

**Leave for the operator** (write `NEEDS_HUMAN.md` and stop): changing a rule
in DESIGN §6 or a decision in §10 — who decides a removal, whether a removed
key may act, whether a group can recover inside itself; raising the budget
past 15,350, or svrngs-core's past 3,000 (say the lines needed and why, with
the count) — but for v1-said-not-substituted's #124 and v1-keeper-office, which ends
#124's cycle, which may take svrngs-core to 3,050 for what no cut makes room
for (the operator, 2026-10-11, "Fix 123 + 124"; 2026-10-12, "Yes that sounds
good"), in its own commit with the reason; Demo C's bars (`demo/c.md`, the operator's to set); deploying the
page host (`scripts/deploy-page.sh`) or anything else outward-facing; docker,
OrbStack or any tool that is not open software (an open-source build tool
installed on this host for v1-onboarding's cross builds, e.g. zig and
cargo-zigbuild through Homebrew, is allowed, said in the commit); a new
crate, or a dependency on a commonwealth-ai or sovereign crate; publishing
what v1-onboarding builds (`scripts/deploy-page.sh`, the binaries with it); anything that needs a service or a
second computer this host cannot give (say what, and the command that would
run it); pushing, history rewrites, `--no-verify`; anything whose evidence
you could not reproduce.

**Always:** one decision, one commit, with the fork, the choice, the evidence
and what would falsify it in the body.
