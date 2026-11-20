#!/bin/bash
# navigations:: x10 at a commit, in svrngs-gate, from a plain clone of it.
set -u
src=/home/alexbryan/dev/svrngs-rebase; base=/home/alexbryan/dev/svrngs-rebase/target/nav-bisect
for c in "$@"; do
  d="$base/$c"
  [ -d "$d/.git" ] || git clone -q "$src" "$d"
  git -C "$d" fetch -q "$src" "$c" 2>/dev/null; git -C "$d" checkout -q --detach "$c" || exit 90
  echo "== $c $(git -C "$d" log -1 --format=%s)"
  cd "$d"
  flock /tmp/svrngs-gate.lock podman run --rm --init --userns=keep-id:uid=1000,gid=1000 \
    --security-opt label=disable --shm-size=2g --pids-limit=-1 \
    --security-opt unmask=ALL --security-opt seccomp=unconfined --device /dev/net/tun \
    --sysctl net.ipv4.ip_local_port_range="49152 65535" -v "$PWD:/work" \
    -v svrngs-gate-cargo-registry:/home/gate/.cargo/registry -v svrngs-gate-cargo-git:/home/gate/.cargo/git \
    -v svrngs-gate-npm:/home/gate/.npm -v svrngs-gate-containers:/home/gate/.local/share/containers svrngs-gate \
    bash -c 'npm ci --prefix web --no-audit --no-fund >/dev/null 2>&1; ulimit -Sn 10240; for i in 1 2 3 4 5 6 7 8 9 10; do cargo test --quiet --workspace --test browser -- navigations:: 2>&1 | grep -E "test result|_onTargetCreated|error(\[|:)" | head -3; done'
done
