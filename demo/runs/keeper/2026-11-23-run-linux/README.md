# Run with svrngs, then share (Linux)

Replay [the original keeper scene](../2026-11-22/README.md) on a disposable
copy of the spike's Fedora VM. The scene uses the spike's Qwen2.5-0.5B-Instruct
Q4_K_M model and the two scratch accounts' homes and groups. The host's
installed svrngs and user services are not used. Build `svrngs` and
`svrngs-keeper` in the toolbox before starting the host driver:

```sh
W=/tmp/opencode/keeper-run-linux-01 python3 -B demo/runs/keeper/2026-11-23-run-linux/run.py
```

Use a new scratch path on each run. `W/run/` holds the transcript, summary,
and sampled door logs; the driver shuts down the VM and discards its disk.
Alex opens coder's
terminal copy and keeps its address. While that copy listens, Run must
refuse without installing a service or touching the share. Sam stops his
copy, withdraws the old share, then runs the same command with svrngs. The
first Share is made to fail by reserving the writer's partial-file name as
a directory. It must leave no share and report the separate save failure.
After the obstruction is removed, Share must succeed with the same PID.

The old address must answer health and chat after three distinct reboots
with no Sam login and no commands typed by Sam. Finally a SIGKILL of the
test service's main process must change its PID, increment its restart
count, and bring the same address back.
