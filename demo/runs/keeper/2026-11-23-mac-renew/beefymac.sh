#!/bin/bash
set -euo pipefail
demo=demo/runs/keeper/2026-11-23-mac-renew
host=(ssh -o BatchMode=yes -o ConnectTimeout=10 beefymac)

# A prior interrupted transfer may have left our archive here. The label
# checks in run.py are independent and run before acting on any service.
"${host[@]}" 'test -d "$HOME" && mkdir -p "$HOME/svrngs-keeper-scratch/repo"'
git archive --format=tar.gz HEAD Cargo.toml Cargo.lock crates apps scripts web \
    docs/guide/build.md docs/brand/favicon.svg "$demo" \
    | "${host[@]}" 'tar -xzf - -C "$HOME/svrngs-keeper-scratch/repo"'
"${host[@]}" /bin/bash -se <<'REMOTE'
cd "$HOME/svrngs-keeper-scratch/repo"
demo=demo/runs/keeper/2026-11-23-mac-renew
tar -cf ../control-originals.tar crates/svrngs/src/cli.rs crates/svrngs-node/src/job/mac.rs \
    crates/svrngs-node/src/door/bound.rs crates/svrngs-node/src/uid.rs
export SOURCE_DATE_EPOCH=1795392000 CARGO_INCREMENTAL=0
build() { cargo build --locked --offline --ignore-rust-version -p svrngs -p svrngs-keeper; }
/usr/bin/python3 "$demo/controls.py" plant
build
for case in changed holder taker held withdrawn; do
    code=0
    /usr/bin/python3 -u "$demo/run.py" "$case" "$case" || code=$?
    test "$code" -eq 5
done
tar -xf ../control-originals.tar
/usr/bin/python3 "$demo/controls.py" test
build
/usr/bin/python3 -u "$demo/run.py" all
tar -xf ../control-originals.tar
cargo check --locked --offline --ignore-rust-version -p svrngs -p svrngs-keeper
cd "$HOME"
rm -rf "$HOME/svrngs-keeper-scratch"
test ! -e "$HOME/svrngs-keeper-scratch"
printf '%s\n' 'Cleanup: scratch directory removed'
REMOTE
