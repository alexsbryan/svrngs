# guests-link needs a budget past "+150 repo, +60 core"

**The question.** guests-link (b) measures +118 repository lines, 7 of them in
svrngs-core. The operator's ceiling leaves 24 in the repository and none in
svrngs-core. Do you raise the guests queue's ceiling, to at most **16,863** for
the repository (+135 over 16,728) and **3,076** for svrngs-core (+7 over
3,069)? Or do you cut the row?

The charter leaves this to you ("the budget: past '+150 repo, +60 core'").

## Where it stands

- (a) is committed: 65c7fbb3. It is red at HEAD in both engines
  (`data/a-red.txt`). The test is
  `crates/svrngs-node/tests/browser/guests.rs` and its browser half is
  `web/tests/visit.mjs`.
- (b) is built, but not committed. It is kept whole as `b-draft.patch`,
  against 65c7fbb3. Applied, the test is green in Chromium and WebKit:
  - Kim taps **Have a guest** and keeps "48 hours".
  - Kim gets `<page>#visit=<token>`.
  - Sam is asked their name once and opens Kim's Jellyfin. The film plays.
  - Jellyfin is told `Sam (Kim's guest)` and Sam's visit key on every
    request.
  - Kim's register holds one visit, 48 hours from its act.
  - `cargo clippy -p svrngs-node -p svrngs-core --all-targets` is clean.

  `data/b-draft-green.txt` has the run and `lean_check`'s refusal:

      crates/svrngs-core     3075 / 3069 lines
      .                     16821 / 16704 lines

## What the draft spends

Counted by `scripts/lean_check.py`'s rule against 65c7fbb3:

| File | Lines | What |
|---|---:|---|
| crates/svrngs-core/src/register.rs | 7 | `impl Command for Visit`. `Append` is the apps' path (rail.rs `append_act`) and refuses a `Visit` (record.rs:33-35). Letting it through would let any app write visits. |
| crates/svrngs-node/src/invite.rs | 32 | A guest's link, kept on the host's computer and in no record (`visit-links/<link key>`), and its claim. The claim removes the file as it reads it, so a link lets one guest in, once, and lapses as a join link does. |
| crates/svrngs-node/src/redeem.rs | 54 | `/visit` on the redeem protocol. It judges the ask as `/ask` does and parses the name as a joiner's. It claims the link and writes the `Visit`, opening the host's declared lends, `until` = now + the length. It answers the host, until when and the lends, or refuses a visit the fold does not count. |
| crates/svrngs-node/src/home.rs | 8 | `/link` takes `visit` and answers `<page>#visit=<token>`. |
| crates/svrngs-node/src/home/home.js | 2 | **Have a guest**: 48, 24 and 6 hours, 48 chosen, then **Make a link for your guest**. |
| web/app.js | 15 | The guest's page. It asks the name once and makes a fresh key it keeps nowhere. It redeems the link, lists the lends and says until when. Passes last past the visit, so the door says when it has ended (door.rs `guest_of`). |
| **total** | **118** | |

## What remains of (b), estimated at +10 to +17

- The commit's refusals should be said as `serve_redeem` says them. The draft
  folds `NoStanding` and `Unsettled` into a refusal: a silent substitution, to
  fix. The survey's helper (`demo/runs/economize/2026-10-19-guests/SURVEY.md`
  §5) "pays then" with this third caller. Net about +1.
- Sam opening the link again mid-visit is told "This link lets nobody in". The
  draft keeps nothing of the visit, and only the lend's own address reopens.
  For the row's "says so plainly when it has ended", the page keeps the visit,
  never as a group, and on a reopened link says it again or says it has
  ended. About +8 to +15.
- The plants, the trust floor and vocab_check are checks, not counted.
  DESIGN.md is not counted either.

So the ask is 118 + up to 17 = **135** over the operator's 16,728, and **7**
over 3,069. Each raise is still one past where its commit lands.

## The economizing pass, and why the rest is not in the code

- guests-economize (93d17fe7) cut 5: `redeem.rs` `answer`'s two arms.
- Its survey found nothing else. Its one open candidate, the commit-refusal
  helper, pays only with this row's caller, and it is counted above.
- The 32 in invite.rs are state no code holds:
  - A join link is issued as an `Invite` in the record.
  - A guest's link cannot be one, since redeeming an `Invite` writes an
    `Admit` (a visit never becomes membership, GUESTS §3.8). Making the core
    tell the two apart costs svrngs-core more than the 7 here.
- The 54 in redeem.rs reuse the ask's judging (`parse_ask`), the name's rule
  (`Name::kept`) and the one `answer`. What is left is the new act and its
  answer.
- The page's 15 reuse `askName`, `reach`, `post` and `open`.

## Options

1. **Raise (recommended):** the repository to at most 16,863, svrngs-core to
   at most 3,076. The next session applies `b-draft.patch`, does what remains
   above, runs each plant red, and lands it in the row's commits.
2. **Raise less, drop the reopen:** at most 16,850 and 3,076. Sam gets back
   in through the lend's own address, never the link. The row's "says so
   plainly when it has ended" is then said only by the door, on the lend's
   page.
3. **Something else:** say which part to cut. Nothing in the draft is larger
   than what the row asks for.

## Reproduce

    git apply demo/runs/guests/2026-10-19-link/b-draft.patch
    CARGO_INCREMENTAL=0 cargo test -q -p svrngs-node --test browser guests::
    python3 scripts/lean_check.py
