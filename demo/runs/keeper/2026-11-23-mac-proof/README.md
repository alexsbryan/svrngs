# Mac Run and lifecycle controls on beefymac, 2026-11-23

[Run of show](../../../keeper-run-mac.md#run-and-lifecycle-controls).
Darwin arm64, cargo 1.94.1, Python 3.9.6 and launchd `gui/502`. A scratch
archive of `b3a2ce78` was built with `cargo build --locked --offline
--ignore-rust-version -p svrngs-keeper`. From `~/svrngs-keeper-scratch`,
each case used `/usr/bin/python3 proof.py repo/target/debug/svrngs-keeper
<case>`. The script sets scratch HOME, XDG_DATA_HOME and SVRNGS_DIR.

## Foreign answer after the busy checks

The test reserves a loopback socket without listening. It waits for the
registered agent's running PID before making the socket listen in the test
process. The agent only runs `/bin/sleep`; the test observes HTTP 200 and
checks the holder with `lsof`. Run's debug event records the ancestry
witness independently of the verdict.

**Red (exit 1).** On the scratch build only, replace `let owned =
matches!(witness, Some(Ok(true)));` in `native_run.rs` with `let owned =
ready && shown.as_ref().is_ok_and(|(_, pid)| pid.is_some());`.

```text
foreign HTTP 200: listener=92883 agent=92899 state=running verdict={"kind": "verified", "unit": "svrngs-keeper-test-mac-proof-lineage", "port": 53171}
AssertionError: {'kind': 'verified', 'unit': 'svrngs-keeper-test-mac-proof-lineage', 'port': 53171}
```

**Green (exit 0).** Restore the original verdict predicate. The active
agent and HTTP answer were both observed; the ancestry witness was false.

```text
port=53748 shown=Ok((Status { running: Since(None), starts: AtLogin, restarts: 0 }, Some(93139))) witness=Some(Ok(false)) ready=true owned=false
foreign HTTP 200: listener=93124 agent=93139 state=running verdict={"kind": "unverified", "state": "running", "ready": true, "port": 53748}
```

## Same label and plist path, no keeper card

Each verb starts with a separately bootstrapped job. Its path, State, PID
and run count come from `launchctl print`, not the keeper. The script
also checks that a refusal leaves the plist bytes unchanged and that no
parked plist or card appears.

**Red (`b3a2ce78`, exit 1).**

```text
stop: exit=0 test-mac-proof-ownership: stop requested
start: exit=0 test-mac-proof-ownership: start requested
restart: exit=0 test-mac-proof-ownership: restart requested
remove: exit=0 test-mac-proof-ownership: remove requested
AssertionError: unmanaged same-path registration accepted: ['stop', 'start', 'restart', 'remove']
```

**Green (exit 0).** All four returned exit 2, with the same refusal and an
unchanged launchd snapshot. The precise path was the same for every
registration shown below:

```text
stop: exit=2 svrngs-keeper: svrngs-keeper-test-mac-proof-ownership has no saved card here
  before={'path': '/Users/alexsbryan/svrngs-keeper-scratch/home/Library/LaunchAgents/svrngs-keeper-test-mac-proof-ownership.plist', 'state': 'running', 'runs': '1', 'pid': '93156'}
  after={'path': '/Users/alexsbryan/svrngs-keeper-scratch/home/Library/LaunchAgents/svrngs-keeper-test-mac-proof-ownership.plist', 'state': 'running', 'runs': '1', 'pid': '93156'}
```

| Command | Red `(state, runs, PID)` before → after | Green `(state, runs, PID)` before → after |
|---|---|---|
| Stop | `running, 1, 92703` → absent | `running, 1, 93156` → same |
| Start | `running, 1, 92716` → same | `running, 1, 93168` → same |
| Restart | `running, 1, 92729` → `running, 2, 92759` | `running, 1, 93180` → same |
| Remove | `running, 1, 92767` → absent | `running, 1, 93196` → same |

## Real keeper card and plist

In `managed`, a keeper-installed HTTP job verifies with its own listener.
The test temporarily replaces the plist with a symlink to the original
bytes. **Red on `b3a2ce78` (exit 1):**

```text
symlink start: exit=0; agent PID=92795
AssertionError
```

**Green (exit 0):** Start refused the symlink. Restoring the regular plist
allowed Stop to park it, Start to bootstrap it again, and Remove to unload
and delete it, leaving the card.

```text
managed HTTP: {"kind": "verified", "unit": "svrngs-keeper-test-mac-proof-managed", "port": 53767}
symlink start: exit=2; agent PID=93219
managed stop/start/remove: 93219 -> 93240; card retained
```

Mac `cargo clippy --locked --offline --ignore-rust-version -p svrngs-keeper
--lib --bins -- -D warnings` passed. Toolbox keeper tests (39), keeper
Clippy, rustfmt, and `python3 scripts/lean_check.py` passed (25,601 non-JS,
8,731 JS, unchanged).

Cleanup: `launchctl print` exited 113 (`Could not find service`) for
`svrngs-keeper-test-mac-proof-{lineage,ownership,managed}`; no live or
parked test plist remained. Removed `~/svrngs-keeper-scratch`, including
the binaries, cards, fixtures and logs. No containers were started.
