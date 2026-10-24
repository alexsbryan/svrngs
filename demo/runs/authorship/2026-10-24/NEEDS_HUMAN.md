# capability-authorship: the act format, for the operator's yes

At 78231455. The row stops here by its own terms. CHARTER.md leaves "a
change to what an act is" to the operator, and capability-signed waits for
"the operator's yes to capability-authorship's design". The design is
`DESIGN.md` beside this file. Nothing in product code changed. This row
added only `crates/svrngs-core/tests/golden.rs` and its two fixtures, and
this directory.

## The yes asked

**Q0. Authorship by signature as `DESIGN.md` §2-§5 and §9-§10 write it?**

It has four parts:
- **Each device signs its own lines.** A phone signs with its own key on
  the group page for a yes to a version, and with its pass's holder for
  an app's acts. The pass rides on each of the holder's lines.
- **Computers carry lines unchanged.** A new carry route stores a line
  as the round stores a peer's.
- **svrngs-core's `Grant`** decides Append by signer and Decide by the
  person's own devices alone.
- **`on_behalf_of` is written by nobody new.** The record API refuses it
  for anyone but this computer's own person. An old act naming someone
  else keeps its meaning only where the keeper's new cutover act,
  `Signers { through }`, covers it by hash, as a cut does. One version,
  12, marks a holder's line, the cutover, and a cut naming a holder's
  acts.

What it moves:
- **Every phone person's version yes,** which went through another
  computer. Their page asks once, "1 app changed. See what's new" (§10).
- **A version yes naming bytes no folder act placed** stops counting
  (shortfall 8).

The golden tests turn red at exactly those two moves, and at the keys
row's renaming (§12).

## The three forks

**F1. A lent app on a port that writes as the person asking (§6).**
Events, lent from Sam's computer, writes each guest's "coming" through
Sam's record API (`apps/events/server.mjs:122-129`). The Events scene
plays it in every gate. Under Q0 it is "carried for another" and counts
nowhere.
- **(a) Recommended:** the app's answer names the act in
  `x-svrngs-append`. The person's own device signs it on the way back,
  within the lend's declared `--kind`, and carries it to the lender
  before the page sees the answer. The open says so: "It writes what you
  say there into The House's record, as you." Any app on a port keeps
  writing as the person asking. Cost: +25 to +45 lines.
- **(b)** Only code the person chose writes as them: the note's table,
  where Append needs pinned code. A lent app writes only as its lender,
  and the Events scene moves to the carried Events. Cost: 0 lines. It
  narrows what a port app can do.

**F2. A phone's chained pass (H) and a computer on an older svrngs
(§11).**
- **(a) Recommended:** an open that reaches any older door gets today's
  raw-bytes pass. The open's writes wait on the phone until a computer on
  the new svrngs takes them, and the page says so. This keeps opening a
  friend's lend on a computer not yet updated.
- **(b)** Refuse the open, in words. The copyable key closes everywhere
  at once, and that path breaks until the friend's computer updates.
  Saves about 10 lines.

**F3. The ceiling for capability-signed and capability-keys (§14).**
- Measured now: 17,789 of 17,790 for the repository, and 3,149 of 3,150
  for svrngs-core (`python3 scripts/lean_check.py` at 289b9531).
- Estimated from the design's sites: +278 to +534 repository and +101 to
  +178 svrngs-core.
- **The ask:** +535 and +180 over today's caps, at most 18,325 and 3,330.
  Each row raises only what it lands, to one past where it lands. A row
  past its range stops for you.

Why it is past the economizing pass's +135 to +255 / +60 to +120:
- (H) moved into capability-signed after the pass;
- the keeper's cutover;
- F1;
- the phone's tips and unsent lines.

F1 (b) and F2 (b) would lower it by the amounts above.

## Evidence

```
$ cargo test -p svrngs-core --test golden
test result: ok. 3 passed; 0 failed; 1 ignored
```

Plants, each reverted (78231455's body):
- `synced.rs:183` reading the writer's person, never `on_behalf_of`:
  both fold tests red.
- `MEMBERSHIP_LINE_VERSION` 2 → 12: the version test red (left: 12,
  right: 2).

The frozen fold (`tests/fixtures/acts_2026-10-24.fold.json`) records
today's shortfalls:
- `"money as cy"` is the unplaced hash `70c0d0fa…`;
- alex's computer's `Runs` for bo counts in `said` as bo's;
- cy's correction voids alex's dinner (`"voided": true`).

## Reproduce

```
cargo test -p svrngs-core --test golden
cargo test -p svrngs-core --test golden -- --ignored --nocapture emit_golden_journal
```
