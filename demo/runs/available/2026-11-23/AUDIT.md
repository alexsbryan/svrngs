# Availability release audit

`REVIEW-audit-available` at `a9db4471`, after the [packaged Fedora VM
proof](README.md).

## Port ledger — passed

Compared `python3 scripts/ports.py` with the tracked markers from
`git grep -InE 'ported from commonwealth-ai [^[:space:]]+@[0-9a-f]{7,40}' HEAD -- ':!docs' ':!ralph' ':!scripts/ports.py'`.
The comparison checks local paths and line numbers and upstream paths and
revisions. Omitted the launchd entry from the listing **in memory** first,
then compared the actual listing:

```text
planted omitted entry: FAIL: missing: [('apps/svrngs-keeper/src/launchd.rs:2', 'svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0')]; extra: []
unmodified ledger: PASS: 67 matching markers (path, line, upstream path and revision)
apps/svrngs-keeper/src/launchd.rs:2  <-  svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0  unchanged; here: changed in 2 commit(s)
apps/svrngs-keeper/src/systemd/tests.rs:1  <-  shared/crates/host-kit/src/service/tests.rs@8ed8d99b0  unchanged; here: changed in 1 commit(s)
apps/svrngs-keeper/src/systemd.rs:2  <-  shared/crates/host-kit/src/service.rs@8ed8d99b0  unchanged; here: changed in 5 commit(s)
```

## CSP and trust floor — passed

The keeper branch was rebased on `789707dc` (CHARTER, “The release”).
`git diff --exit-code 789707dc a9db4471 -- scripts/trust_floor.txt scripts/trust_check.py scripts/check.sh scripts/gate`
and the following CSP diff both exited 0 with no output:

```sh
git diff --exit-code 789707dc a9db4471 -G 'Content-Security-Policy|content-security-policy|[a-z]+-src|frame-ancestors|base-uri|form-action' -- apps crates web
```

There is one floor difference from the original `e5d9813b` base:
`9167f112`, already in `789707dc`, changes the relay listener's rule at
`scripts/trust_floor.txt:133` and gives its approved `Trust floor:` reason.
There are no keeper changes to that file. `python3 scripts/trust_check.py`
compared the served CSPs with the pins and exited 0:

```text
trust floor: 13 policies, 13 listeners, 17 gates
```

## Release envelope — passed

`python3 scripts/lean_check.py` exited 0:

```text
Lines of code, tests and apps/ not counted, and the most each may have:
  the core (svrngs-core)     3,471   most   3,539
  all code but JavaScript   25,719   most  25,719
  JavaScript                 8,727   most   8,731
  Swift                          0   most       0
```

The actual counts and the repo caps fit the CHARTER's final ceilings
(“The envelope now”): 25,727 non-JavaScript and 8,741 JavaScript, leaving
8 and 14 lines respectively from the actual counts.
