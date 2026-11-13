#!/usr/bin/env bash
# launch.sh <checkout> <outdir> <steps...>   (on fedora)
co=$1; out=$2; shift 2
steps=$(printf '%s\n' "$@")
mkdir -p "$out"
podman run --rm --init --userns=keep-id:uid=1000,gid=1000 \
  --security-opt label=disable --shm-size=2g --pids-limit=-1 ${CPUS:+--cpus=$CPUS} \
  -e FLAKE_BUILD=${BUILD:-} -e FLAKE_STEPS="$steps" -e CARGO_BUILD_JOBS=${JOBS:-12} \
  -v "$co:/work" -v "${EUDOXIA:-$HOME/dev/eudoxia-flake}:/eudoxia:ro" -v svrngs-flake-rustup:/home/gate/.rustup -v "$out:/out" -v "$HOME/flake/round.sh:/round.sh:ro" \
  -v svrngs-gate-cargo-registry:/home/gate/.cargo/registry \
  -v svrngs-gate-cargo-git:/home/gate/.cargo/git \
  -v svrngs-gate-npm:/home/gate/.npm localhost/svrngs-gate:latest bash /round.sh /out
