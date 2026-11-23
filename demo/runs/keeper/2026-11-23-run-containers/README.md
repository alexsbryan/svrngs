# Can plain rootless Podman keep a share through reboot?

Proof 1 of `avail-run-containers`. Reuse the native scene's prepared Fedora
45 disk, scratch accounts, local relay, and two svrngs nodes. Use a single
Python HTTP container from the spike's pinned image instead of Actual: the
HTTP body is a random marker written inside its named volume before reboot.
The hand-written test unit mirrors the detached-pod launch in
`apps/svrngs-keeper/src/runtime.rs:172–212` without building an adapter.
The only mount is the named volume. This is a field run in the disposable
VM, not a Podman test on the host. The driver discards the VM disk and its
test unit, pod and volume when it finishes.

```sh
W=/tmp/opencode/keeper-container-plain python3 -B demo/runs/keeper/2026-11-23-run-containers/run.py
```

If the plain run breaks on job attribution, repeat with a new `W` and
`--split` (`Delegate=yes` on the unit; `--cgroups=split` on the workload).

## Predictions before the run

- The planted wrong subtree fails the socket membership check. To count as
  a positive read, `ss` must show the TCP listener **and** the server end of
  a fresh, byte-free connection in the running unit's subtree.
- Alex reads the marker through the saved lend address, then reads it again
  at that exact address after a different boot ID and with only Sam's
  linger manager present. Sam makes no second approval. Direct HTTP after
  reboot distinguishes lost data or startup failure from refusal at the door.
- The real Share must save a keep grant for the test unit, and the door
  must log successful renewal and persist a fresh approval after reboot.
  A listening port, or even an unchanged volume, cannot prove renewal alone.

This is one run per configuration. An orderly reboot exercises neither
recovery from a dead foreground adapter nor same-name foreign-pod ownership
(Proof 2).

### Plain Podman result

`W=/tmp/opencode/keeper-container-plain python3 -B …/run.py` exited 0 (45 s).
Fedora 45, kernel 7.2.11, Podman 6.1.3, systemd 262, `Delegate=no`.
Selected output from `avail-run-containers-1.await.log:3–12` and the
driver's `run/after-door.log:3–4`:

```text
planted subtree …/svrngs-keeper-test-recipes.service/wrong-job: accepted=false
before: LISTEN and established, pasta pid=1608, both …/svrngs-keeper-test-recipes.service; in_unit=true
Alex reads the marker before reboot: passed (28264fac2cb645b8beb8dd01860df1dc, HTTP 200)
after: LISTEN and established, pasta pid=952, both …/svrngs-keeper-test-recipes.service; in_unit=true
boot_before=68ed43bf-7084-41d0-b096-2263d6e35ca6 boot_after=86a1c5f1-01f2-4fcc-801b-ff7c5dff0fd9
approved: boot=7560773838930461136 cgroups=[7591] → boot=9701252711243206604 cgroups=[5112]
TCP read: taker=Cgroup(5112) checked=Err(Held(Another)) renewed=Ok(())
same marker at same address after reboot: passed (direct and Alex both HTTP 200; only Sam's linger manager)
```

No `--split` run was needed. The driver powered the VM off and removed
the scratch disk, including its test unit, pod and volume.

## Proof 2: recovery without the foreground process

`recover.py` uses the same scratch VM and a disposable shell recipe, with
no `ExecStop`. A pod label is set at creation; cleanup checks that label
and addresses the pod by the inspected ID. Plant: `--plant` skips the
label check, so a running same-name pod with another label must fail the
preservation check. In the unplanted run, killing only the unit's main
process must produce a new pod and container with the same volume marker,
once before and once after the container's healthcheck passes. Finally,
stopping the container itself must restart the unit; a failing or
unreadable healthcheck is not an exit.

```sh
W=/tmp/opencode/keeper-container-recover-red2 python3 -B demo/runs/keeper/2026-11-23-run-containers/recover.py --plant --control-group
W=/tmp/opencode/keeper-container-recover-trace python3 -B demo/runs/keeper/2026-11-23-run-containers/recover.py --control-group
W=/tmp/opencode/keeper-container-recover-process2 python3 -B demo/runs/keeper/2026-11-23-run-containers/recover.py
```

Selected output from the three `run/events.jsonl` captures (IDs shortened):

```text
RED --plant: cleaning pod d94c074298e5 (another-job)
same-name foreign pod is untouched: failed; expected=d94c074298e5 after=89f9983ceceb state=Running
AssertionError: the same-name foreign pod was touched
RED --control-group: foreground death recovers: failed; pod=d1dd0ecd8bf8 Degraded; manager=deactivating restarts=0
GREEN KillMode=process: refused foreign pod 41f17a28c0be (another-job)
same-name foreign pod is untouched: passed; expected=41f17a28c0be after=41f17a28c0be state=Running
kill before readiness: passed; health=starting restarts=1 old=47b56c993310 new=8ebb8deea24e body=5c6cb4760c8843a38ebd81a4a659b4e6 HTTP 200
nonready and unreadable are not exits: passed; unhealthy_exit=1 unreadable_exit=125 main_pid=2512
kill after readiness: passed; health=healthy restarts=2 old=8ebb8deea24e new=dec46fb6bece body=5c6cb4760c8843a38ebd81a4a659b4e6 HTTP 200
actual container exit restarts the unit: passed; restarts=3
```

Under the default kill mode, systemd's group teardown took down the infra
container but did not finish deactivating in the 25 s observation. With
`KillMode=process`, systemd observes the foreground exit immediately and
leaves the detached Podman workload for the next invocation to reclaim.
The green run exited 0. Its journal shows both `status=9/KILL` events and
the labelled pod IDs being stopped and removed by the next invocation.
All three VM disks, including test units, pods and volumes, were discarded.
