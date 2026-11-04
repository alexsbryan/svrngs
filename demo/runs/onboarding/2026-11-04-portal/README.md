# Bare-command portal reminder

Unit: `onboarding-portal-reminder`. No deployment or publication.
The pre-existing V1.6 `publish.json` and `publish.log` were inspected and
left untouched, as instructed.

## Real-process red, then green

The check extends the existing `crates/svrngs/tests/address.rs` process
harness. A first bare invocation starts a real node. A second bare command
must succeed, name the same address that the first process printed, trace
the reminder, and launch no browser (a fake executable records any launch).
The first process stays alive and answers HTTP afterward. `open` still
invokes that executable; `new` still refuses the held lock. Removing the
saved address, replacing it with malformed text, or making it a directory
must refuse with no invented reminder. The output must contain no API token.

Red on the old implementation, before changing `cli.rs`:

```text
$ cargo test -p svrngs --test address a_second_bare_command_reminds_without_starting_or_opening -- --nocapture
running 1 test
thread 'a_second_bare_command_reminds_without_starting_or_opening' (3240299) panicked at crates/svrngs/tests/address.rs:122:5:
svrngs: svrngs is already running on this computer (/var/folders/qf/4ntssn_10d598hw_5rjhx3740000gp/T/.tmpxUm1cw/lock is held); stop it, then run this again
test a_second_bare_command_reminds_without_starting_or_opening ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.32s
error: test failed, to rerun pass `-p svrngs --test address`
```

Green after reusing `api::kept_address` and `home::address` in the locked
`Verb::Run` branch:

```text
$ cargo fmt --all && cargo test -p svrngs --test address --test flags --lib
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/address.rs
running 3 tests
test another_program_holding_the_address_is_said_and_svrngs_stops ... ok
test a_second_bare_command_reminds_without_starting_or_opening ... ok
test the_home_page_and_a_lend_answer_at_one_address_across_a_restart ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.62s
Running tests/flags.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19789 / 19833 lines
js                     8066 / 8090 lines
swift                     0 / 1 lines
```

## Initial closing gate attempt

`lockf -k /tmp/svrngs-gate.lock scripts/check.sh` ran in the foreground
with the required 600000 ms tool timeout. It reached that timeout before
completion. Principles, lean, trust floor, advisories, app nouns, lint,
format, page laws, rustfmt, clippy, canaries, ceiling canary, web clippy,
browser, types, typed lint, joining and pages passed. The tests step failed
in an unrelated Podman startup:

```text
failed     tests (89 s)
the_stand_in_keeps_its_state_across_a_restore --- FAILED
apps/give/tests/check.rs:72:44:
Podman(Failed { command: "run --detach --pod svrngs-test-keep-99868 ...", status: 126, stderr: "Error: starting some containers: internal libpod error" })
test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 54.41s
```

The unchanged single test passed immediately on rerun:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p give --test check the_stand_in_keeps_its_state_across_a_restore -- --nocapture
running 1 test
snapshot: [("data", Reflink)], paused 935.78825ms
test the_stand_in_keeps_its_state_across_a_restore ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 24.99s
```

The full locked gate must complete with exit 0 before this row is done.
Use `ralph-result await` to give it the necessary budget and preserve its log.

## Resumed closing gate

The loop-managed full gate completed with exit 1 in 788 seconds. Its log is
`ralph/next/onboarding/ctl/log/sessions/onboarding-portal-reminder-1.await.log`.
Every stage except `tests` passed, including all browser, page, scene,
conformance and app-law stages. This time Give's
`a_snapshot_of_another_card_version_is_refused` failed at
`apps/give/tests/check.rs:140` on Podman's container startup, status 126:
`Error: starting some containers: internal libpod error`.

The same test passed unchanged in isolation on resumption:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -p give --test check a_snapshot_of_another_card_version_is_refused -- --exact --nocapture
running 1 test
test a_snapshot_of_another_card_version_is_refused ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 12.79s
```

This establishes a passing isolated rerun, not a passing full gate or a
diagnosis of Podman's internal error. The next full gate runs under the
loop's await budget because the completed run exceeded the foreground
tool's ten-minute timeout. No product or test changes were made for this retry.

## Second resumed closing gate

The next loop-managed gate completed with exit 1 in 1,187 seconds. Its log is
`ralph/next/onboarding/ctl/log/sessions/onboarding-portal-reminder-2.await.log`.
Every stage except the ceiling canary passed, including `tests` (520 seconds),
all browser/page/scene stages, conformance and app laws. The ceiling reported
a median of 0.3883 seconds against its unchanged 0.1-second CPU limit, from
rounds `[0.7030590000000032, 0.3883379999999974, 0.05959900000000218]`.

The unchanged canary passed in isolation on resumption:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test --release --quiet -p svrngs-node --test scale -- --ignored --exact an_idle_round_is_under_the_ceiling --nocapture
running 1 test
CEILING idle round at 25 x 10,000: 0.0318 s of processor, the median of [0.21965099999999893, 0.03178200000000331, 0.029434999999999434], against 0.1 s; the last round's account {"acts_moved":0,"asleep":0,"cpu_s":0.029434999999999434,"dialled":24,"folds":{"digest":{"kept":96,"rebuilt":0},"offer":{"kept":48,"rebuilt":0},"register":{"kept":27,"rebuilt":0},"write_index":{"kept":126,"rebuilt":0}},"reached":24,"refused":0,"wall_s":0.004558375}
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 3.41s
```

This is an isolated pass, not a diagnosis of the earlier CPU reading and not
a passing full gate. No product code, test, threshold or budget was changed.
The full locked gate remains owed and runs again under the loop's await budget.

## Final resumed closing gate — passed

The loop reports exit **0** for the unchanged locked full gate at HEAD
`86802901`. Its complete log is
`ralph/next/onboarding/ctl/log/sessions/onboarding-portal-reminder-3.await.log`.
The following output is pasted from that run (the scene and conformance
detail is retained in the complete log):

```text
$ lockf -k /tmp/svrngs-gate.lock scripts/check.sh
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (0 s)
passed     trust floor (0 s)
passed     advisories (2 s)
passed     app nouns gate (0 s)
passed     app nouns (0 s)
passed     lint (0 s)
passed     format (0 s)
passed     page laws (16 s)
passed     rustfmt (1 s)
passed     clippy (1 s)
passed     tests (525 s)
passed     canaries (1 s)
passed     ceiling canary (4 s)
passed     web rustfmt and clippy (8 s)
passed     browser (156 s)
passed     types (2 s)
passed     typed lint (0 s)
passed     joining in a browser (41 s)
passed     pages (61 s)
passed     scenes (235 s)
passed     conformance canary (95 s)
passed     app laws (41 s)
note       took 1189 s in all
```

The ceiling's median was 0.0301 seconds against 0.1 seconds. The conformance
log also retains its rate observations: two direct-to-canary WebKit seeks
missed five seconds; all four through-lend seeks passed. Its stage verdict
was passed. No implementation, test, threshold or budget changed during
these retries. This completed full gate closes the obligation recorded by
the earlier attempts; the real-process red/green and affected CLI proofs
above establish the row's behavior.
