# #211: a leave that outruns the leaver's last act (survey, 2026-10-21)

This survey covers `ralph/REVIEW_FINDINGS.md` #211, which
`demo/runs/guests/2026-10-20-run/README.md` found under "What it found",
item 1. It fixes nothing. The operator chooses the design, and a loop row
builds it. Line numbers are at db1ab12e. Code has moved since the run's
commit, so the finding's own cites sit a few lines earlier (86a3bd08
added lines to membership.rs).

## Findings

1. **It reproduces deterministically, with no sleep that decides the
   outcome.** Run `demo/runs/leave/2026-10-21/repro.sh`. The script copies
   two test files into their crates' `tests/`, runs them, and removes the
   copies, so the tree is left as it was. Its output is in `repro.out`.
   - **The fold** (`leave_race_core.rs`, svrngs-core alone). Four cases
     pass, because each asserts HEAD's divergence (`repro.out:2-17`).
     - Kim's copy holds every act. It lists nobody visiting, and applies
       `kim-paid-40`.
     - Ana's copy lacks only Lee's visit. It lists "Max (ana's guest)" and
       applies `bo-typo`, which Kim's Correct had voided. It applies no
       `kim-paid-40`. It shows five `removed` gaps and one `unheld_cut`
       (`repro.out:7-8`).
     - When Lee's visit reaches Ana's copy, the two folds are equal
       (`repro.out:9`, and the test's `assert_eq!` on `membership()`).
   - **Three computers** (`leave_race_node.rs`, svrngs-node). Each
     computer's round runs a day apart (`Node::start_every`), so only a
     wake or a round the test asks for moves anything.
     - Kim writes `kim-last` with no round, then leaves. Every computer
       then rounds three times.
     - Kim's copy applies `["kim-paid", "kim-last"]`. Ana's and Bo's apply
       `[]`, and their `/v1/rail/log` says `complete false`
       (`repro.out:30-33`).
     - The test asserts that the copies agree, so it is red: "Ana's copy
       agrees with Kim's, left: [] right: ["kim-paid", "kim-last"]"
       (`repro.out:35-38`).
     - It was red in five runs of five, each time with the same output.
2. **The cause: the leave travels alone, and nothing carries the leaver's
   acts after it.**
   - A leave is told as its `Remove` lines and nothing else: once from
     `cut` (`crates/svrngs-node/src/removal.rs:216-222`), then each round
     by `leave_told` (`ring_sync.rs:432-439`, `RingRail::own_removes`).
     The request's `ops` are those lines alone (`removal.rs:249-253`).
   - Once a copy holds the cut, Kim can no longer reach it:
     - No round dials Kim. A group is offered only to keys standing in
       it (`ring_sync.rs:363-374`).
     - Kim's own round only tells the leave (`ring_sync.rs:347-351`).
     - The sync route takes none of a cut asker's acts. It answers that
       asker only its cut (`ring_routes.rs:129-158`, `:174-179`).
     - Kim's computer serves nobody. For a group a computer has left,
       `Standing::member` answers no one (`crates/svrngs-runtime/src/register.rs:108-111`,
       `:252-254`). Its acceptor refuses a dialer who stands in no other
       group held there (`register.rs:316-326`;
       `crates/svrngs-node/src/node.rs:948-954`).
   - This breaks the rule DESIGN states: "a copy holds an act only with
     its parent, so copies converge by union" (`docs/DESIGN.md:944-945`).
     The leave's parent is Lee's visit (`crates/svrngs-core/src/register.rs:241-250`:
     `through` is the leaver's newest act).
   - The fold then reads the cut as covering only itself. `covered` walks
     `prev` from the leave and stops at the act it lacks
     (`membership.rs:543-554`). Any act of a cut key that the cut does not
     cover counts nowhere (`membership.rs:595-599`).
3. **Every kind of act is affected.**
   - The fold skips an uncovered act before it looks at what kind of act
     it is (`membership.rs:595-599`).
   - Admission then leaves that act out of everything built from the
     counted acts: what an app folds, folder changes, and `Runs`
     (`crates/svrngs-core/src/admit.rs:614-620`, `:727`;
     `synced.rs:176-186`).
   - These kinds were run:
     - a money entry and a Correct: the void was undone, so Bo's typo
       came back (core case 1);
     - Visits and a Stop (core case 1);
     - an `Admit`: Dee, whom Kim admitted, is an `unknown_signer` on
       Ana's copy and stands nowhere there (core case 2, `repro.out:10-12`).
       That copy's register then refuses Dee's sync and lends, because the
       round, the sync route and the door all ask that one register
       (`docs/DESIGN.md` §6, "The group decides who is in it").
   - These kinds follow from the same lines but were not run: folder
     changes (`Synced`), a yes to a carried version (`Runs`), `Invite`,
     `Relay`, and House rules' acts, which are app Records.
4. **A leave diverges for good. A keeper's removal heals.**
   - **A leave.** It heals only where a standing copy holds the leaver's
     last act. Then the next round between the two carries it, because
     sync does not filter by author (`sync.rs:44-52`). Node case 2 shows
     this: Ana heals at her round with Bo (`repro.out:25-29`).
   - Where only the leaver's computer holds that act, nothing ever brings
     it. Node case 1 shows this (`repro.out:30-33`). Leaving is final from
     that computer (`docs/DESIGN.md:1027`).
   - The run read its end 1.96 s after Kim's leave (`data/events.jsonl:44`,
     `:48`). So in the two takes where Bo held Lee's visit, the run could
     not have seen Ana's copy heal.
   - **A keeper's removal.** Its `through` names the removed key's newest
     acts *as the keeper holds them* (`register.rs:241-250`). The keeper
     stands, and the keeper's round carries the cut and what it names
     together.
     - Node case 3: every copy applies `["kim-seen"]`
       (`repro.out:21-24`).
     - A copy can still hold the cut before what it names for a moment.
       There the removed key's earlier acts count nowhere, exactly as on a
       leave, until the act arrives (core case 3, `repro.out:3-5`).
     - An act the keeper never held counts nowhere on any copy, including
       the removed computer's own (`kim-unseen`, `repro.out:24`). That is
       by design (`docs/DESIGN.md:973-978`).
5. **What each copy shows.**
   - **Home page.** In every take, Ana's and Bo's home pages listed "Max,
     Ana's guest" (the run's README, bar 9; `summary.json`, `leave_9`).
     Kim's copy lists nobody (the README, "What it found" 1; core case 1
     here). The page reads the fold's visits and gives
     no sign that the answer is a subset (`crates/svrngs-node/src/rail.rs:187-199`).
     Only `/v1/rail/log` carries `complete` (`rail.rs:374`).
   - **`/v1/rail/log`.** Ana's and Bo's say:
     - "an op from 2fba72572228… has not reached this node yet (#4)";
     - four times, "an act by 2fba72572228… after they were removed from
       the group does not count";
     - "a removal names an act (13e1911ffb7e…) this node does not hold
       yet".

     Kim's says `complete false` because of Max's `not_a_member` gap, which
     is #213 (`summary.json`, `end_record`).
6. **History and intent.** The case was not anticipated for a leave.
   - U7 (c) brought cuts by hash and `UnheldCut`, an absence (f915bc2c).
     Its test is named for a gap that lasts "until it arrives"
     (`removal_tests.rs:347-366`). That holds for a keeper's cut: the acts
     a cut names are the keeper's to hold, and the keeper stands.
     `Cut::unheld` says the same: acts this copy "cannot *yet* cover"
     (`membership.rs:266-268`).
   - groups-leave (a) made a person's own `Remove` the cut, covering
     itself (1b19d80f). Part (b) then reused the removal's telling: "the
     cut told once, the cut alone — … a leave's to the group's other
     computers" (1676df59).
   - On a leave, the acts the cut names belong to the leaver. After the
     leave, only the leaver holds them, and the leaver never syncs again.
     So "until it arrives" never comes.
   - #190 (1716275e) made the leaver's round tell the leave until each
     computer answers. Its body says "Nothing else is sent, as before
     ("Record only")".
   - The operator's "Record only" (svrn-docs `rings/current/LEAVING.md:170-171`,
     §3.5 `:99-102`) means "Ana left" is read from the record, with no
     other message. LEAVING.md's bars (§5) and the groups-leave row
     (`ralph/next/groups/STATE.md:16`) have no case of a last act still
     unsent.
   - #179 (3d3ad2d6) already sends a cut with its signer's acts before it,
     but only to a computer that was off when the cut landed
     (`ring_routes.rs:146-149`). The path that tells at once does not
     (`removal.rs:249-253`).
7. **The inventory (principle 11).** commonwealth-ai has no answer to fork,
   so the fix is ours to write.
   - Its fold cuts by place in the order. It sorts by `ts_unix` and drops
     the key at its `Remove`, ignoring `through_seq`
     (`~/dev/commonwealth-ai/shared/crates/commonwealth-rail-core/src/membership.rs:123`,
     `:136-155`). Its own spec names this defect "D1"
     (`docs/RING_SPEC.md:314-316`).
   - It has no writer of a `Remove` and no leave (`RING_SPEC.md:317-320`,
     "D2").
   - Its sync is author-blind and filters no removed key, so a late act
     counts once it arrives, if its timestamp sorts before the cut. That
     result is order-free but can be dodged by backdating, which is the
     hole svrngs closed by cutting by hash (f915bc2c).
   - Its `view` (a signed per-author digest) is read by nothing but the
     signature (`RING_SPEC.md:103-105`). That could be adapted for option
     D below, and for nothing else.
   - Not applicable as a fork. These files are byte-identical between
     e1a43bcd2 and its HEAD, ad6b3a5f6.

## Options

"Structural" means every copy that holds the cut also holds every act it
covers. Then the copies agree because the fold is a function of the set
(`admit.rs:22-28`), not because of timing. Counted lines are by
`scripts/lean_check.py`'s rule: not blank, not a comment, not a test. The
repo is at 16,906 of 16,907 and svrngs-core at 3,075 of 3,076, so every
option here needs a raise or an economizing pass first.

| | What changes, where | Counted lines (estimate) | Existing records; older builds | Agreement | Failure modes |
|---|---|---|---|---|---|
| **A** The cut is told with what it covers | node: `removal::tell` first asks each computer's digest (one sync request with no `ops`, whose answer's acts it does not keep), then sends the cut with the signer's acts before it that the computer lacks and the hand-ons it lacks. That is the selection `told_cut` already makes (`ring_routes.rs:146-149`), moved to one shared function | repo +16 to +25 (`tell` about +13, the shared selection +4, `told_cut` −1, a warn when the selection cannot be read +2); core 0 | The record format does not change. A leaver on an older build still tells the cut alone. A copy that already holds a lone leave is not healed: it takes none of a cut asker's acts (`ring_routes.rs:129-139`) | **Structural.** The tell carries the leave's chain, and every other path sends a key's acts in order of seq from the peer's mark (`sync.rs:277-279`, `:302-306`), parents first even when truncated. So no path leaves a copy holding the leave without its parent (`told_cut`'s hand-ons go alone, but only to a cut computer) | Computer unreachable, or the second request fails: not answered, so `leave_told` asks again next round. An unsent chain over the 4 MB budget is sent as a prefix and marked answered, so the row must repeat until the computer's digest covers the leave. Never fall back silently to the cut alone (principle 6) |
| **A+** A, and the sync route also takes from a cut asker what its cut covers | node: `told_cut` takes the asker's offered acts that this copy's cut of it names as unheld, and what they descend from. It answers its own digest, so the asker knows what to send | A, plus repo +15 to +25 | Heals copies that already diverged, once the leaver's computer runs the new build and tells again (it tells once more after a restart: `ring_sync.rs:411-414`) | Structural, as A | It loosens #179's "none of its own taken" to "only what its cut covers". It needs the leaver's computer on after the upgrade |
| **B** Sync before writing the leave | node: `leave` (`removal.rs:150-153`) runs a round, then commits | repo +3 to +6 | No change | **Rarer.** An unreachable group, or an act written between that round and the commit (the redeem route writes Visits on its own), still diverges. Refusing to leave while nobody is reachable would fix that, at the cost of "needs nobody's yes" and the page for the afraid | A computer off during that round hears a lone leave later. It heals only from a standing copy that holds the act |
| **C** Copies keep fetching a cut key's acts up to its cut | node: the round dials a cut key while an `UnheldCut` names it. The left computer's acceptor (`node.rs:948-954`) and sync route serve those acts, its own up to the leave and nothing after. core: the relay of a cut key (`Membership::relay` answers only a standing key, `membership.rs:381-385`) | repo +30 to +50; core +1 to +3 | Heals existing copies while the leaver's computer stays on | **Rarer.** It depends on the leaver's computer staying on and reachable after leaving, which is when people switch off or uninstall | Leaving no longer stops the computer serving the group. "No round, no sync" (LEAVING §3.4) bends |
| **D** The fold counts a cut key's held acts below the leave, by seq | core: `covered` (`membership.rs:520-559`). A leave covers every held act of its key with seq below its own. A keeper's cut names ids only, so D would need `through` to carry seqs, a new field at line version 12 | core +4 to +8 for leaves alone (+10 to +20 with core and node to also cover a keeper's cut) | Changes what existing records mean. On a copy that lacks the chain, the leaver's earlier acts count again (the run's Max goes). On every copy, a leaver's fork below its leave now counts, where by hash it does not (f915bc2c). Older and newer builds disagree on such a copy | **Rarer and smaller, not structural.** The missing act never arrives (`unheld_cut`, `sequence_hole`, `complete false` forever), and whatever turns on it still diverges: core case 4, Pat (`repro.out:13-15`), where a visit's bound is judged without Lee's | Equivocation below the leave diverges between copies holding different branches |
| **E** #212: each write others wait on wakes the round | node: the Visit and Stop writers (`redeem.rs` `visit_route`, :170; `home.rs` `stop`, :437) | per #212's row | No change | **Rarer.** The window shrinks to the push's milliseconds. A computer that is off, a failed push, or a leave inside that window still diverges | — |

## Recommendation

**Choose A: tell the cut with what it covers.** It is node-only, about 20
counted lines, and it changes no line format. It restores DESIGN §6's
invariant (`docs/DESIGN.md:944-945`) on the one path that breaks it.
`told_cut`'s selection becomes the single place that decides "what a cut
is told with" (principle 8), and both tells use it. The line count is an
estimate from a sketch. No code was changed or run for it.

- **The red test** is `a_leave_reaches_every_copy_with_the_acts_it_covers`
  (`leave_race_node.rs`). It is red at HEAD (`repro.out:35-38`) and should
  be committed with the fix into `crates/svrngs-node/tests/removal.rs`.
  `a_leave_heals_where_a_standing_copy_holds_its_last_act` and
  `a_keepers_removal_reaches_every_copy_with_what_it_names` pass at HEAD
  and must stay green.
- **Plants**, each to be watched failing:
  1. The tell's `ops` set back to the lines alone (`removal.rs:249-253` as
     at HEAD). This should fail at "Ana's copy agrees with Kim's".
  2. The shared selection keeping only the `Remove`s (no signer's acts
     before them). This should fail the same test.
  3. A new case: Bo writes an entry with no round, then Kim leaves. Kim's
     copy must not hold Bo's entry. Plant: the tell keeping the first
     request's answer. This guards that the leaver's computer takes
     nothing more of the group ("No round, no sync", LEAVING §3.4).
- **Also close in the row:**
  - repeat the second request until the computer's digest covers the
    leave, so a chain over the 4 MB budget is not marked told;
  - name the highest `Remove` of the signer as the seq to select up to.
    `told_cut` takes the first `Remove` naming the asker
    (`ring_routes.rs:146-147`), so it never sends a person's other
    computer the leave of the signing key, which `Leave` writes last
    (`crates/svrngs-core/src/register.rs:231`). That was read in the code, not run.
- **Docs in the same commit:**
  - DESIGN §6, `docs/DESIGN.md:994-997` and `:1015-1017` ("the cut
    alone", "the leave alone is sent");
  - `removal.rs:16-24`;
  - `leave_told`'s doc (`ring_sync.rs:416-419`).

## Adjacent, not #211

- **The home page shows a subset as the whole.** It lists visits from a
  fold the record calls incomplete, and says nothing of it
  (`rail.rs:187-199`, against `:374`). This is principle 6, and it is the
  operator's call whether it becomes a finding.
- **#212 makes #211 likelier.** A Visit wakes no round, so Lee's visit
  stayed unsent for the 2.2 to 4.0 s before the leave.
- **#213 is why Kim's copy, though whole, says `complete false`.**

## For the operator

1. **The budget.** A needs about +20 repo lines and no core lines; A+
   needs another +15 to +25. The repo is at its cap (16,906 of 16,907).
   Is that an economizing pass first, or a raise (AGENTS.md, "Done")?
2. **"Record only."** May the leaver's own acts from before the leave,
   and no one else's, travel with the leave? A needs a yes. DESIGN
   (`docs/DESIGN.md:1017`) says "the leave alone is sent". LEAVING.md
   §6.3 says "Nothing else is sent", which this survey reads as "no
   message beyond the record".
3. **One rule for both tells?** Should the at-once tell to a removed
   computer also carry the keeper's acts before the cut, and the
   hand-ons, as `told_cut` already does for one that was off
   (`ring_routes.rs:129-139`)? Or should A apply to leaves only? One rule
   is the same code. Leaves only sends nothing new to a removed person
   (`docs/DESIGN.md:1001-1002`).
4. **Existing records.** Has anyone left a real group (canon's house, or
   any other) while an act of theirs was held by their computer alone?
   Such a copy's `/v1/rail/log` shows an `unheld_cut` whose `by` is a
   leave. If so, A+ is needed to heal it, with the leaver's computer
   upgraded and on. If not, A alone is enough.
5. **D changes what existing records mean** (a leaver's fork below the
   leave would count). It is listed for completeness, not recommended. It
   would need a core raise.

## Files

- `repro.sh` runs both test files: `core`, `node`, or both when given no
  argument. It uses `CARGO_INCREMENTAL=0` and `CARGO_BUILD_JOBS=4`.
- `leave_race_core.rs` has the fold's four cases. They pass, asserting
  HEAD's divergence.
- `leave_race_node.rs` has the three computers' three cases. The first is
  red at HEAD.
- `repro.out` is one run of `repro.sh`, at db1ab12e, with cargo's
  progress lines dropped. It exits 101, from the node red test.
