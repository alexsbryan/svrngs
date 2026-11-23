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
