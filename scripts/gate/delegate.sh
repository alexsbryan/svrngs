#!/bin/sh
# The gate image's entrypoint (Containerfile), run as the container's root:
# gives `gate` the container's cgroup, then runs the command as `gate`.
# Podman inside (give's tests and scenes) makes a cgroup for each of its
# containers, and `pod pause` freezes them by it; the container's cgroup is
# its root's, so without this every pause failed, "cannot pause the
# container without a cgroup" (apps/give/tests/check.rs, 2026-11-15). Its
# processes move to a leaf first: cgroup v2 hands a cgroup's controllers to
# its children only while it holds no process itself. Where the cgroup
# can't be written (a run without README.md's flags), it says so and goes
# on, and give's tests fail naming the pause. A pid gone before its move
# (this shell's own `cat`) is let go.
cg=/sys/fs/cgroup
if mkdir "$cg/init" 2>/dev/null; then
  for p in $(cat "$cg/cgroup.procs"); do echo "$p" 2>/dev/null >"$cg/init/cgroup.procs"; done
  sed 's/[^ ]*/+&/g' "$cg/cgroup.controllers" >"$cg/cgroup.subtree_control"
  chown -R gate:gate "$cg"
else echo "note       the gate's cgroup can't be written, so podman in it pauses nothing" >&2; fi
exec env HOME=/home/gate USER=gate setpriv --reuid=gate --regid=gate --init-groups "$@"
