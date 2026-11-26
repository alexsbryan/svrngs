# Native Run on beefymac, 2026-11-23

[Run of show](../../../keeper-run-mac.md). Darwin arm64; Rust 1.94.1;
llama-server 10180 (`11b068d06`); Qwen2.5-0.5B-Instruct Q4_K_M copied from
`/tmp/keeper-demo/qwen2.5-0.5b.gguf`. Scratch checkout from `git archive`
plus the row's patch; `cargo build --locked --ignore-rust-version` for
`svrngs` and `svrngs-keeper` (`SOURCE_DATE_EPOCH=1795392000` for svrngs).
`/usr/bin/python3 run.py` invokes the scratch binaries, with scratch HOME
and SVRNGS_DIR, inside the current GUI launchd domain. No group was created.

## Red

On the unchanged fork `0ae488d7` (before wiring it in), with a live IPv4
wildcard listener, the required busy sentence was missing:

```text
busy 0.0.0.0: exit=1 svrngs: svrngs couldn’t verify this. Check it from your terminal, then look again.
AssertionError
```

With the first launchd implementation, a bare `llama-server` installed as
`Program` returned `launchctl bootstrap: Bootstrap failed: 5: Input/output
error`. After resolving that command from `PATH`, a stop followed immediately
by restart once failed `that name is already registered from a different
setup`. Launchctl had returned before dropping the registration; the final
keeper waits up to 10 seconds before moving or removing the plist.

Removing the expected PID comparison in `job/mac.rs` failed
`only_the_installed_agent_and_the_asking_daemon_match` at
`assertion failed: !registered(&good, path, Some(38))`. Substituting `bootout`
for `print` (static check only; never executed) failed `scripts/lean_check.py`:

```text
BROKEN  crates/svrngs-node/src/job/mac.rs: an unlisted process call; the node may only read app jobs (KEEP_IT_RUNNING_RELEASE O3)
```

## Green

```text
busy 0.0.0.0: exit=1 svrngs: Port 60983 is in use by Python, the copy you started: stop it, then press Run.
busy 127.0.0.1: exit=1 svrngs: Port 60983 is in use by Python, the copy you started: stop it, then press Run.
busy ::1: exit=1 svrngs: Port 60983 is in use by Python, the copy you started: stop it, then press Run.
run: exit=0 test-avail-run-mac is running here. To share it: `svrngs share 60983 --program <program> --name test-avail-run-mac --group <group>`.
prefill: qwen2.5-0.5b.gguf resolved from relative path
Metal: offloaded 25/25 layers to GPU, HTTP /health 200, pid 48110
SIGKILL: 48110 -> 48258, /health 200 after 10.8s
test-avail-run-mac: stop requested
stop: still stopped after 12s and restart request; no login plist or port
start then restart: 48381 -> 48539, HTTP 200
test-avail-run-mac: remove requested
unrun: agent and plists removed; card and model retained
```

`cargo clippy --locked --ignore-rust-version -p svrngs-keeper --lib --bins --
-D warnings` passed on beefymac. In the toolbox: `cargo test -p svrngs-keeper
--locked` (39), `cargo test -p svrngs-node --test availability_plan --locked`
(4), the registration matcher (1), keeper Clippy, and `python3
scripts/lean_check.py` (25,601 non-JS; 8,731 JS) passed. Run and lifecycle
passed; sign-in and reboot were never run here (HUMAN-release-hosts).

Cleanup: `launchctl print gui/502/svrngs-keeper-test-avail-run-mac` exited 113
(`Could not find service`). Neither the live nor the parked test plist
remained. Removed `~/svrngs-keeper-scratch`, including the copied model,
scratch binaries, card and logs. No containers were started.
