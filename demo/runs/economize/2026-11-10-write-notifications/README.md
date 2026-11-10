# One notification for a journal change

Operator-selected ECONOMIZING_PLAN stage 1, row 2(a), on 2026-11-10.
Starting HEAD: `26889eb35f34`; the checkout also held unrelated HolyArrs and
permaculture work. No signed format, stored format or trust-floor change.

## Reproduced before fixing

`cargo test -p svrngs-node --lib node::tests:: -- --nocapture` ran six
tests: two existing tests passed; these four failed after their writes
succeeded (local log `target/write-notifications-red-node.log`):

| Test | Failing assertion |
|---|---|
| `minting_a_link_wakes_sync` | the Invite was written without waking sync |
| `changing_the_meeting_wakes_sync` | the Meets was written without waking sync |
| `redeeming_a_visit_wakes_sync` | the Visit was written without waking sync |
| `stopping_a_visit_wakes_sync` | the visit's Correct was written without waking sync |

The fixture uses real node startup wiring, then stops background rounds so
the test can inspect their stored wake permit. Visit redemption runs over
iroh with the guest's verified dialer; Stop traverses the home route's
existing origin and token checks. Two people are admitted so a visit is
within the core's bound: guests must be fewer than members.

`cargo test -p svrngs-runtime --lib tests::the_write_teller_reports_changes_not_successful_no_ops -- --exact --nocapture`
also failed before the fix: a successful empty commit called the teller,
giving a count of 2 instead of 1. Local log:
`target/write-notifications-red-runtime.log`.

## Retirement inventory, checked against the code

Nine write-related nudges were removed:

- `rail::carry`, after peer ingestion;
- `carried::yes`, **before** appending approval, rather than after it;
- `origin::acts`, after appending each act;
- `Folders::write`, after commit;
- the carried app response's successful append;
- `redeem_route`, after admission;
- removal and handover, after their commits;
- the record API's append wrapper, replaced by routing straight to
  `rail::append`; `RailApi` no longer carries its nudge.

The original plan's inventory needed correction. The home connection
button does not write: its explicit refresh stays. The folder's two nudges
do follow writes and are redundant once those writes notify centrally.
The arrival-specific relay wake and explicit `AskRound` requests stay.

Node startup installs the callback before cutovers and background work.
It captures the folder's watch sender and the round's Notify, not Folders
or Host, avoiding a rail → callback → host → rail ownership cycle.
Folder writers resume from node startup only after that callback and the
cutovers: resuming inside Folders::start could otherwise write on another
runtime thread before the constructor returned and the callback was installed.

## Validated instruments

The shared node regression checks folder notification and sync notification
on local writes and fresh peer lines, and neither on duplicate/empty
ingestion, an empty commit or a refused command. The runtime test also
checks invalid-namespace rejection and that a second teller cannot replace the first.

The two-node regression runs the actual sync loop over iroh, with a one-day
interval. It waits for both loops to settle, verifies that a successful
empty commit starts no round, then commits an act and requires the other
node to count it within five seconds. Both loops must settle again and
their digests must agree. No manual round is fired to deliver the act.

Plants, each removed before the passing run:

| Plant | Test and observed failure | Local log |
|---|---|---|
| Drop the callback's folder notification | shared node regression: `folders.has_changed()` false | `target/write-notifications-plant-folder.log` |
| Restore notifications on successful empty commits | two-node regression: no-op started a round, 4 instead of 3 | `target/write-notifications-plant-no-op.log` |
| Drop the callback's sync notification | two-node regression: write waited for the periodic timer, five-second timeout | `target/write-notifications-plant-sync.log` |

The no-op plant initially passed the convergence-only test: current relay
code guards redundant commits before calling commit. Adding an explicit
no-op/round-count assertion made the instrument detect the actual extra
round. No self-sustaining loop was reproduced.

## Fixtures that depended on missing wake-ups

The broad tests exposed six assumptions in four targets. The second gate's
`tests` step failed in 802 seconds; that run was stopped after canaries,
at 902 seconds (`target/write-notifications-full-check-fixtures.{log,json}`).

- `carried`: a deliberate pre-holder-cutover record was automatically
  upgraded before its refusal probe. Its keeper's background sync is now
  explicitly stopped before making that historical fixture.
- `door`: the running endpoint corrected a synthetic relay before the
  assertion that it was listed. Its owner's background sync is stopped
  while testing the synthetic record.
- `removal`: the cutover probe automatically received the keeper's cutover;
  two leave-race probes sent entries deliberately meant to remain unseen.
  Those fixtures now stop background sync, then drive the same real round
  explicitly after the leave/removal HTTP response. Their original cut,
  payload and no-more-entries assertions remain.
- `scale`: setup writes and peer ingestion ran extra rounds inside the
  measured round, moving the two acts before it and inflating fold counts.
  The instrument now stops background sync before filling the journals.
  Its exact hand-count assertion is unchanged.

`Node::stop_sync` is test support only. Ordinary node behavior still runs
with all write notifications. The existing automatic-removal delivery test
and new automatic two-node write test retain background sync.

After the fixture changes,
`cargo test -p svrngs-node --test carried --test door --test removal --test scale`
passed: 24 + 60 + 10 + 1 tests, with only their five existing ignored tests.

The first isolated gate also exposed two burst fixtures that sent 200
separate HTTP requests: under concurrent suite load they outlasted the
18-second refill, so the next request correctly returned 200 rather than
their expected 429. The burst fixtures now replicate their 199-line
prefix in one batch, carry the last allowed line, then probe the refused
line, void, duplicate and real-time refill exactly as before. Both pass
in isolation (0.70 and 19.90 seconds). The continuous-write tests still
exercise actual loops and replication. No limit or refill duration changed.
That gate's tests failed in 884 seconds; it was stopped at 1,013 seconds,
with output retained under suffix `-burst-fixtures` in `target/`.

## Verification

Passed after restoring all plants:

- `cargo check --workspace`
- `cargo test -p svrngs-node --lib node::tests:: -- --nocapture`: 8 passed
- the exact runtime test above: 1 passed
- `python3 scripts/lean_check.py`: non-JavaScript production code
  21,295 → 21,264; cap lowered to 21,264. Other counts unchanged.

Full locked `scripts/check.sh`: final repeat running; final result is owed.
At the operator's commit request, this isolated run had passed every cheap
check, clippy, the full Rust test suite (863 seconds), canaries and the
ceiling (0.0474 seconds of processor against 0.1). Browser/scene and later
steps were still running; the commit does not claim a full-gate pass.
Local log/status: `target/write-notifications-full-check.{log,json}`.
The first attempt was stopped after 313 seconds during tests to close the
startup ordering gap found in review; it had passed through clippy and
is not a full-gate verdict (`target/write-notifications-full-check-stopped.{log,json}`).
The third run finished in 1,948 seconds, exit 1: every stage passed except
the collaborative Doc scene. Alex's page did not hold all three expected
texts within 15 seconds; host load was 35.11, below the existing 36 bar,
so this was a failure, not could-not-judge. Its full output is retained in
`target/write-notifications-full-check-doc-scene.{log,json}`.
Without any source change,
`lockf -k /tmp/svrngs-gate.lock cargo test -p svrngs --test scenes doc::doc::chromium -- --exact --nocapture`
passed in 42.60 seconds. The full gate is repeated unchanged; the transient
failure's cause is not established.
The fourth attempt stopped at rustfmt in 25 seconds on the concurrently
added xenia film fixture; its owning session had already formatted those
lines by inspection, and `cargo fmt --all --check` then passed unchanged.
That output is retained with suffix `-concurrent-format` in `target/`.
The fifth attempt stopped at the JavaScript budget: concurrent work added
43 counted lines, taking the shared tree to 8,543 against 8,500. No budget
was raised and none of that work was edited. The final run uses a detached
validation checkout at `5db3d6cc`, with exactly the working diff of
`crates/svrngs-node`, `crates/svrngs-runtime` and `scripts/lean_check.py`
applied. Its counts are 3,510 core / 21,264 non-JavaScript / 8,500 JavaScript.
It shares installed dependencies and build output under the same gate lock.
The unrelated uncommitted HolyArrs and xenia additions are not in that
validation snapshot; all notification code and changed fixtures are.
After the burst-fixture adjustment, the source and validation checkout's
selected diffs both hash to SHA-256
`7e1d1acbca53bfc4e4ad5c4a433b7a7ce5630c27bd6f914d8e0b8dfa8617abbb`.
The run verifies the existing `.githooks` setting instead of rewriting Git
configuration. Its per-run SSH wrapper uses IPv4 to the same real Podman
VM, following the localhost self-connection reproduced in the HolyArrs
run earlier today; no machine configuration changes.
