#!/bin/bash
# The gate flags from scripts/gate/README.md, with the worktree's git mount.
# The gate image carries ss and the private user manager's systemd marker.
set -euo pipefail
if command -v podman >/dev/null; then P=(podman); else P=(flatpak-spawn --host podman); fi
exec flock /tmp/svrngs-gate.lock "${P[@]}" run --rm --init --userns=keep-id:uid=1000,gid=1000 \
    --security-opt label=disable --shm-size=2g --pids-limit=-1 \
    --security-opt unmask=ALL --security-opt seccomp=unconfined --device /dev/net/tun \
    --sysctl net.ipv4.ip_local_port_range="49152 65535" \
    -v "$PWD:/work" \
    -v /home/alexbryan/dev/svrngs/.git:/home/alexbryan/dev/svrngs/.git \
    -v svrngs-gate-cargo-registry:/home/gate/.cargo/registry \
    -v svrngs-gate-cargo-git:/home/gate/.cargo/git \
    -v svrngs-gate-npm:/home/gate/.npm \
    -v svrngs-gate-containers:/home/gate/.local/share/containers \
    svrngs-gate python3 -B demo/runs/keeper/2026-11-23-run-page/run.py "$@"
