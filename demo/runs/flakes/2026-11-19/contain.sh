#!/bin/bash
# The containment's runs, in svrngs-gate, from a plain clone of next-fixes
# at a commit, a diff applied or none (`-`) (nav-bisect.sh's
# podman invocation, under its lock).
#   contain.sh <commit> <diff|-> <navs N | step N | planted N | plantnav N | node N>
# navs: navigations:: N times. node: svrngs-node's tests. step: the browser step N times. planted: the
# browser step N times (each run's whole output kept in the clone's
# target/contain-<base|after>-<mode>-<run>.log), the node running web/tests/browsers.mjs killed
# (SIGKILL, as a crash ends it) $AT s into each, 20 unless given.
# plantnav: the browser step N times, navigate.mjs's WebKit driver killed.
set -u
src=/home/alexbryan/dev/svrngs-next-fixes; d=$src/target/gate-after/clone
c=$1 diff=$2 mode=$3 n=$4
[ -d "$d/.git" ] || git clone -q "$src" "$d"
git -C "$d" fetch -q "$src" next-fixes && git -C "$d" checkout -q -f --detach "$c" && git -C "$d" clean -qfd -e target || exit 90
[ "$diff" = - ] || git -C "$d" apply "$diff" || exit 91
echo "== $c $(git -C "$d" log -1 --format=%s) diff=$diff $mode x$n"
cd "$d"
flock /tmp/svrngs-gate.lock podman run --rm --init --userns=keep-id:uid=1000,gid=1000 \
  --security-opt label=disable --shm-size=2g --pids-limit=-1 \
  --security-opt unmask=ALL --security-opt seccomp=unconfined --device /dev/net/tun \
  --sysctl net.ipv4.ip_local_port_range="49152 65535" -v "$PWD:/work" -e MODE="$mode" -e N="$n" -e AT="${AT:-20}" -e TAG="$([ "$diff" = - ] && echo base || echo after)" \
  -v svrngs-gate-cargo-registry:/home/gate/.cargo/registry -v svrngs-gate-cargo-git:/home/gate/.cargo/git \
  -v svrngs-gate-npm:/home/gate/.npm -v svrngs-gate-containers:/home/gate/.local/share/containers svrngs-gate \
  bash -c '
    npm ci --prefix web --no-audit --no-fund >/dev/null 2>&1; ulimit -Sn 10240
    cargo test --quiet --workspace --test browser --no-run 2>&1 | grep -E "^error" | head -3
    kill_server() { sleep $AT; for p in /proc/[0-9]*; do a=$( (tr "\0" "\n" < $p/cmdline) 2>/dev/null | head -2 | tr "\n" " "); case "$a" in "node "*web/tests/browsers.mjs" ") kill -9 ${p#/proc/} && echo "planted: killed node browsers.mjs (${p#/proc/}) at $AT s";; esac; done; }
    # The crash planted where it happens: the node running navigate.mjs for
    # WebKit, which with its own browser is also its Playwright server,
    # killed 5 s after it starts, as the assertion ends it.
    kill_nav() { for t in $(seq 600); do for p in /proc/[0-9]*; do a=$( (tr "\0" " " < $p/cmdline) 2>/dev/null); case "$a" in "node "*web/tests/navigate.mjs*webkit*) sleep 5; kill -9 ${p#/proc/} && echo "planted: killed node navigate.mjs for webkit (${p#/proc/})"; return;; esac; done; sleep 0.5; done; echo "planted: navigate.mjs for webkit never ran"; }
    for i in $(seq "$N"); do
      echo "-- run $i"
      case $MODE in
        navs) cargo test --quiet --workspace --test browser -- navigations:: 2>&1 ;;
        step) cargo test --quiet --workspace --test browser -- --skip playback:: 2>&1 ;;
        plantnav) kill_nav & cargo test --quiet --workspace --test browser -- --skip playback:: 2>&1; wait ;;
        node) cargo test --quiet -p svrngs-node --no-fail-fast 2>&1 ;;
        planted) kill_server & cargo test --quiet --workspace --test browser -- --skip playback:: 2>&1; wait ;;
      esac > target/contain-$TAG-$MODE-$i.log 2>&1
      grep -aE "test result|_onTargetCreated|planted:|harness: web/tests|--- FAILED|^error" target/contain-$TAG-$MODE-$i.log
    done'
