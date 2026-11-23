# Planner economizing pass, 2026-11-22

`avail-plan-economize` in `ralph/next/keeper/STATE.md`. Baseline at
`9bd6d566`: 24,945 counted non-JavaScript lines, 8,650 JavaScript, and
3,471 core (`python3 scripts/lean_check.py`). The planner had added 309
non-JavaScript lines (`befdaf97`, `Size:`).

| Commit | Preserving cut | Saved | Non-JS count and cap |
|---|---|---:|---:|
| `9576e8f4` | `plan.rs`: choose the verified return or its refusal in one match. Deployment refusals still come first, and the app still explains a tied unverified start. | 3 | 24,942 |
| `ff323e40` | `plan/words.rs`: one phrase table and sentence template for the verified returns, shared by the review and the publication report. | 2 | 24,940 |

The event and report types still distinguish observations from outcomes;
`Availability` is the person's choice, not the executor's `Method`. The
two recipe matches answer different questions: what can take over and
what the review shows even when it cannot take over. The existing types,
token inputs and ordered steps in `plan.rs` stay in place. The module
export in `lib.rs` and both pure files in `lean_check.py` stay in place
so the terminal and page can use the same planner and P4 still checks it.

## Proof

Toolbox `cargo test -p svrngs-node --test availability_plan -- --nocapture`
after each cut, with the test and its cases unchanged:

```text
test p1_every_prefix_replays_and_material_changes_invalidate_the_token ... ok
P2: 2400 host combinations, including the rehearsal hosts and the passphrase disk
test p2_host_product_matches_the_written_plans_and_words ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Toolbox `python3 scripts/lean_check.py` after the second ratchet, exit 0 (P4):

```text
  the core (svrngs-core)     3,471   most   3,539
  all code but JavaScript   24,940   most  24,940
  JavaScript                 8,650   most   8,650
  Swift                          0   most       0
```

Toolbox `cargo check --workspace`:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.57s
```

## Remaining release budget

The earlier [pass](../2026-11-22-available/SURVEY.md#close) sets the base
at 24,636 non-JS and 8,650 JS. The approved +435/+30 envelope therefore
ends at 25,071/8,680. The planner now costs 304 instead of 309, leaving
**131 non-JS and 30 JS** for the rest. The ratcheted cap is the current
count, not the envelope ceiling (keeper CHARTER, “The release”).

| Remaining work | Non-JS estimate | JS estimate | Basis: svrn-docs `KEEP_IT_RUNNING_RELEASE.md` at `4f77680`, §5 |
|---|---:|---:|---|
| Publish | 40 | 0 | Revision and a conditional write (R-publish) |
| Executor and terminal | 65 | 0 | 50 executor, 15 command (R1) |
| Packaging | 10 | 0 | Counted install changes (R1, §6) |
| Page | 30 | 30 | Two routes and the form (R4) |
| Mac renewal | 150 | 0 | Native and container witnesses (R7) |
| **Total** | **295** | **30** | **165 non-JS over; JS fits exactly** |

The row's 285-line subtotal excludes the 10 lines for packaging and is
itself 155 over. The spec's §6 options recover about 60 by dropping the
Mac container witness (containers there stay this run only) and about 10
by dropping Restart. Together they would still leave a 95-line gap
including packaging (85 against the row subtotal). Closing that gap
would require more preserving cuts, lower actual costs, or an operator
decision. These are forecasts; each row spends only what it lands, up to
the approved envelope.
