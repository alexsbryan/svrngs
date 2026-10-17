#!/usr/bin/env bash
# canon-floor (c): the floor's behavioural gates, read by scripts/trust_check.py,
# red when a named gate's test is gone or ignored, then the tree as it is,
# green. As 2026-10-16-b/plants.sh: each plant in a copy of this tree's
# crates/, web/ and scripts/, in a git repository of its own whose one commit
# gives the floor's reason. Run from the repository's root:
#   demo/runs/trust-floor/2026-10-16-c/plants.sh > demo/runs/trust-floor/2026-10-16-c/data/plants.txt
set -u
# No git of the copies' packs its objects behind the run, so each copy is whole.
export GIT_CONFIG_COUNT=2 GIT_CONFIG_KEY_0=gc.auto GIT_CONFIG_VALUE_0=0 GIT_CONFIG_KEY_1=maintenance.auto GIT_CONFIG_VALUE_1=false
work=$(mktemp -d /tmp/trust-floor-plants.XXXXXX)
base=$work/base
mkdir -p "$base"
git ls-files -co --exclude-standard -- crates web scripts | tar -cf - -T - | tar -xf - -C "$base"
at=$(git rev-parse --short HEAD)
fresh() {  # fresh <name>: a copy of the tree, its one commit giving the floor's reason
  case=$work/$1; cp -R "$base" "$case"; git -C "$case" init -q
  git -C "$case" add -A && git -C "$case" -c user.name=plant -c user.email=plant@invalid commit -q -m "base: the tree at $at, with the worktree's own changes

Trust floor: the floor as this run reads it."
}
run() {  # run <name> <what it plants>
  echo "== $1: $2"
  python3 "$case/scripts/trust_check.py"
  echo "exit $?"
  echo
}
edit() {  # edit <file> <python: s, the file's text, rewritten>
  python3 -I -c "import sys; p=sys.argv[1]; s=open(p).read(); t=$2; assert t != s, 'the plant changed nothing'; open(p,'w').write(t)" "$case/$1"
}

fresh renamed
edit crates/svrngs-node/tests/peer.rs "s.replace('fn no_loopback_port_reaches_a_friends_door(', 'fn no_loopback_port_reaches_a_friends_door_renamed(', 1)"
run renamed "peer.rs's test renamed (#175's gate)"

fresh ignored
edit crates/svrngs-node/tests/door/main.rs "s.replace('#[tokio::test]\nasync fn a_forged_identity_header_is_replaced_by_the_verified_one(', '#[tokio::test]\n#[ignore]\nasync fn a_forged_identity_header_is_replaced_by_the_verified_one(', 1)"
run ignored "the door's typed-identity test ignored"

fresh tree
run tree "the tree as it is, nothing planted"
rm -rf "$work"
