#!/usr/bin/env bash
# canon-floor (b): each check of scripts/trust_check.py red on a planted
# widening it names, then the tree as it is, green. Each plant runs in a copy
# of this tree's crates/, web/ and scripts/ (tracked or new, not ignored), in
# a git repository of its own whose one commit gives the floor's reason, so no
# plant touches the tree it reads. Run from the repository's root:
#   demo/runs/trust-floor/2026-10-16-b/plants.sh > demo/runs/trust-floor/2026-10-16-b/data/plants.txt
set -u
# No git of the copies' packs its objects behind the run, so each copy is whole.
export GIT_CONFIG_COUNT=2 GIT_CONFIG_KEY_0=gc.auto GIT_CONFIG_VALUE_0=0 GIT_CONFIG_KEY_1=maintenance.auto GIT_CONFIG_VALUE_1=false
repo=$(pwd)
work=$(mktemp -d /tmp/trust-floor-plants.XXXXXX)
base=$work/base
mkdir -p "$base"
git ls-files -co --exclude-standard -- crates web scripts | tar -cf - -T - | tar -xf - -C "$base"
commit() {  # commit <dir> <message>
  git -C "$1" add -A && git -C "$1" -c user.name=plant -c user.email=plant@invalid commit -q -m "$2"
}
run() {  # run <name> <what it plants>; the copy is $case, planted by the caller
  echo "== $1: $2"
  python3 "$case/scripts/trust_check.py"
  echo "exit $?"
  echo
}
at=$(git rev-parse --short HEAD)
fresh() {  # fresh <name>: a copy of the tree, its one commit giving the floor's reason
  case=$work/$1; cp -R "$base" "$case"; git -C "$case" init -q
  commit "$case" "base: the tree at $at, with the worktree's own changes

Trust floor: the floor as this run reads it."
}
edit() {  # edit <file> <python: s, the file's text, rewritten>
  python3 -I -c "import sys; p=sys.argv[1]; s=open(p).read(); t=$2; assert t != s, 'the plant changed nothing'; open(p,'w').write(t)" "$case/$1"
}
carried=crates/svrngs-node/src/carried.rs

fresh unsafe-eval
edit $carried "s.replace(\"script-src 'self';\", \"script-src 'self' 'unsafe-eval';\", 1)"
run unsafe-eval "'unsafe-eval' added to the carried policy's script-src"

fresh wasm
edit $carried "s.replace(\"script-src 'self';\", \"script-src 'self' 'wasm-unsafe-eval';\", 1)"
run wasm "'wasm-unsafe-eval' added to the carried policy's script-src, in the code alone"

fresh wasm-pinned
edit $carried "s.replace(\"script-src 'self';\", \"script-src 'self' 'wasm-unsafe-eval';\", 1)"
edit scripts/trust_floor.txt "s.replace(\"\n  script-src 'self'\n  style-src\", \"\n  script-src 'self' 'wasm-unsafe-eval' | canon's wasm\n  style-src\", 1)"
commit "$case" "plant: canon's wasm, pinned

Trust floor: a plant, carried pages given 'wasm-unsafe-eval' with a reason."
run wasm-pinned "'wasm-unsafe-eval' added to the carried policy and pinned in the floor with a reason, committed with one"

fresh host
edit crates/svrngs-node/src/home.rs "s.replace(\"\\\"default-src 'self'; frame-ancestors 'none'\\\"\", \"\\\"default-src 'self'; script-src 'self' https://cdn.example.com; frame-ancestors 'none'\\\"\", 1)"
run host "a host in the home page's script-src"

fresh drift
edit web/index.html "s.replace(\"connect-src 'self' wss: https:;\", \"connect-src 'self' wss: https: ws://127.0.0.1:*;\", 1)"
run drift "the published page's connect-src drifting from the floor: the tests' loopback allowance back (v1patch-loopback)"

fresh wide
edit crates/svrngs-node/src/room.rs "s.replace('tokio::net::TcpListener::bind(addr)', 'tokio::net::TcpListener::bind(\"0.0.0.0:0\")', 1)"
run wide "the room door's listener bound on 0.0.0.0"

fresh unreasoned
edit scripts/trust_floor.txt "s.replace('# Where svrngs listens.', '# Where svrngs listens, tidied.', 1)"
commit "$case" "tidy the floor"
run unreasoned "the floor changed in a commit whose body gives no reason"

fresh uncommitted
edit scripts/trust_floor.txt "s.replace('# Where svrngs listens.', '# Where svrngs listens, tidied.', 1)"
run uncommitted "the floor changed and not committed, so no reason can be read"

fresh no-why
sed -i '' -E "s/^(  connect-src 'self' wss: https:) \| .*/\1/" "$case/scripts/trust_floor.txt"
commit "$case" "plant: a widening's reason taken out

Trust floor: a plant, the group page's connect-src pinned with no reason."
run no-why "a widening pinned in the floor with no reason after it"

fresh tree
run tree "the tree as it is, nothing planted"
rm -rf "$work"
