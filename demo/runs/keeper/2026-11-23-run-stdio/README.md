# Run with a terminal, from the CLI and the page

## CLI: real logind in the scratch Fedora VM

Build `cargo build --locked -p svrngs -p svrngs-keeper` in the toolbox, then
run this on the host with a new scratch path each time:

```sh
W=/tmp/opencode/keeper-stdio-green BIN="$PWD/target/debug" \
  python3 -B demo/runs/keeper/2026-11-23-run-stdio/cli.py
```

The driver boots a disposable copy of the prepared VM and turns linger off
for its scratch `sam` account. It uses the existing SSH PTY driver, which
sends `y` only after it sees `Type y`. After Run it reads real logind again,
disables and removes `svrngs-keeper-test-stdio.service`, powers off the VM,
and discards the copied disk. `W/run/events.jsonl` holds the transcript.

**Red**, with `W=/tmp/opencode/keeper-stdio-red` and the old `output()` at
`a0672da8`: the PTY showed `PTY, linger=no` but no prompt in 12 s. The
driver killed SSH (exit `-9`, `answered: 0`) and the check exited 1:

```text
{"t": 1795422134.407, "event": "linger after Run", "value": "no"}
AssertionError: boot consent prompt was not visible
```

**Green**, with explicit stdio: the command exited 0 with `answered: 1`:

```text
PTY, linger=no
To start it before you log in, your account needs to run when nobody is logged in. Turn that on? Type y, then press Enter: y
svrngs-keeper: enabled lingering for sam — the programs it keeps now survives logout and starts at boot without a login
test-stdio is running here. To share it: `svrngs share 18191 --program <program> --name test-stdio --group <group>`.
{"t": 1795422393.936, "event": "linger after Run", "value": "yes"}
```

## Page: WebKit with the node on an unanswered PTY

```sh
bash demo/runs/keeper/2026-11-23-run-page/gate.sh --tty
```

The existing WebKit scene runs with real keeper and systemd user jobs in
the disposable gate. The node verifies that its stdin is a PTY. The driver
never writes to its master. The gate has no logind, so this half uses a
scratch `loginctl` that begins at `Linger=no` and records each invocation.
The scene must complete Run, Stop, Start and the separate Share, with no
consent prompt in the service log and only `show-user` calls recorded. The
gate stops `svrngs-keeper-test-browser.service` and
`svrngs-keeper-test-page-coder.service` and discards its scratch state.

**Red**, with the old `output()` at `a0672da8`:

```text
Page node stdin is a PTY
the page did not finish: TimeoutError: page.waitForFunction: Timeout 30000ms exceeded.
test explicit_run_then_share_in_webkit ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 49 filtered out; finished in 37.69s
```

**Green**, with explicit stdio:

```text
Page node stdin is a PTY
WebKit Run confirmation: test-page-coder is running here. To share it, find it under Already running here.
test explicit_run_then_share_in_webkit ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 20.19s
Page Run and controls: unanswered PTY, loginctl ['show-user', 'show-user', 'show-user', 'show-user', 'show-user', 'show-user', 'show-user', 'show-user', 'show-user']; linger=no
```
