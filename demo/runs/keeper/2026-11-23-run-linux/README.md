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
refuse without installing a service or touching the share. The JSONL
records the listener PID and start tick, localhost `/health` body SHA-256,
unit file and keeper card directory presence, systemd `LoadState`, and
the existing declaration's SHA-256 and revision on both sides of Run.
Sam stops his copy, withdraws the old share, then runs the same command
with svrngs. The first Share is made to fail by reserving the writer's
partial-file name as a directory. It must leave no share and report the
separate save failure.
After the obstruction is removed, Share must succeed with the same PID.

The old address must answer health and chat after three distinct reboots
with no Sam login and no commands typed by Sam. Finally a SIGKILL of the
test service's main process must change its PID, increment its restart
count, and bring the same address back.

## Observed on 2026-11-23

At `fabda979`, `cargo build -p svrngs -p svrngs-keeper` in the toolbox
followed by the command above exited 0 in 1m35s. All nine cases passed
(`avail-run-linux-6.await.log:4–32`; scratch `run/events.jsonl` and
`run/summary.json`). Fedora's `llama-cpp-b9840-2.fc45` served the model.

| Case | Verdict | Observation |
|---|---|---|
| Alex opens the terminal copy | passed | Health and chat returned 200 through the shared address. |
| Run while the port is busy | passed | Instrumented replay below: exit 1, same listener and response, no unit or keeper record, and the same share and address. |
| Run after Sam stops the copy and withdraws its share | passed | Exit 0. Sam consented to unattended startup; PID 2073 started and no share was written. |
| Share with the partial-file directory planted | passed | Exit 1. The declaration was absent and PID 2073 stayed up. |
| Share after removing the obstruction | passed | Exit 0. The declaration named `svrngs-keeper-coder.service`; PID 2073 and Alex's original address were unchanged. Health and chat returned 200. |
| No-login boot 1 (`8bf9204f`) | passed | Health 200 at 8.35 s uptime; chat 200 at 9.74 s. |
| No-login boot 2 (`c9d58ab4`) | passed | Health 200 at 8.26 s uptime; chat 200 at 9.65 s. |
| No-login boot 3 (`62ee7940`) | passed | Health 200 at 8.17 s uptime; chat 200 at 9.57 s. |
| SIGKILL of the test service's main process | passed | PID 874 → 1461, `NRestarts=1`; health and chat returned 200 at the same address. |

The refusals and retry, from the terminal transcript (other lines omitted):

```text
svrngs: Port 8101 is in use by llama-server, the copy you started: stop it, then press Run.
coder is running here. Share it separately with `svrngs share`.
svrngs: Running here; not shared. The share couldn’t be saved: /home/sam/.svrngs/lends/.coder.json.partial cannot be written: Is a directory (os error 21).
Shared with house. house can open it again after you sign in.
```

At Alex's first request on each boot, logind showed only a `manager`
session for Sam. Sam typed nothing after the successful Share. Each boot
and the crash went from svrngs's 502 (“not answering now”) through
llama-server's 503 (“Loading model”) to 200 at the original address. The
sampled door logs show `renewed=Err(Held(Another))` while no taker could be
read, then `kept=Some(Ok(true)) renewed=Ok(())` at 7.838, 7.746, and
7.653 s uptime on the three boots, and at 14.898 s after the crash on
boot 3's clock.

The focused red/green runs are in `24bfe771` (busy listeners, readiness,
and reset of a failed unit) and `d6069b2b` (foreign connection taker and
the blocked-save retry). Packaged hardware startup was not run here.

### Instrumented busy-port replay

The red scratch replay used `W=/tmp/opencode/keeper-busy-witness-red` and
temporarily wrote `compose.yaml` in the keeper's job directory immediately
after the busy refusal. It exited 1: `run/events.jsonl:8` records `job_record`
changing from `absent` to `present` even though the other witnesses agreed.

With the plant removed, the focused green replay through successful Share
exited 0 (`W=/tmp/opencode/keeper-busy-witness-green`, `run/events.jsonl:7–16`).
Run refused with exit 1. Before and after, `ss` named PID 1420, started at
tick 1408, and `/health` hashed to `a29ee2b15c49…`. The unit file and card
directory were absent and systemd reported `LoadState=not-found` both
times. The declaration hashed to `79c44220f73b…` at revision 1 both times,
and Alex kept the same address. After Sam stopped his copy, Run verified
the new managed listener (PID 2498) and the retry shared it at that address.
