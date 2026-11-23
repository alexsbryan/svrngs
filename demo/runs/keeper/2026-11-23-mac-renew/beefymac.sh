#!/bin/bash
set -euo pipefail
demo=demo/runs/keeper/2026-11-23-mac-renew
host=(ssh -o BatchMode=yes -o ConnectTimeout=10 beefymac)
mode=${1:-renew}
test "$mode" = renew || test "$mode" = plist

# Refuse an existing scratch root. run.py also checks both labels before use.
"${host[@]}" 'test -d "$HOME" && mkdir "$HOME/svrngs-keeper-scratch" && mkdir "$HOME/svrngs-keeper-scratch/repo"'
git archive --format=tar.gz HEAD Cargo.toml Cargo.lock crates apps scripts web \
    docs/guide/build.md docs/brand/favicon.svg "$demo" \
    | "${host[@]}" 'tar -xzf - -C "$HOME/svrngs-keeper-scratch/repo"'
if [[ "$mode" == plist && $# == 2 ]]; then
    git archive "$2" Cargo.lock crates/svrngs/Cargo.toml crates/svrngs/src/cli.rs \
        | "${host[@]}" 'dd of="$HOME/svrngs-keeper-scratch/plist-baseline.tar" status=none'
fi
"${host[@]}" /bin/bash -se -- "$mode" <<'REMOTE'
cd "$HOME/svrngs-keeper-scratch/repo"
cleanup() {
    cd "$HOME"
    rm -rf "$HOME/svrngs-keeper-scratch"
    test ! -e "$HOME/svrngs-keeper-scratch"
    printf '%s\n' 'Cleanup: scratch directory removed'
}
trap cleanup EXIT
demo=demo/runs/keeper/2026-11-23-mac-renew
tar -cf ../control-originals.tar Cargo.lock crates/svrngs/Cargo.toml \
    crates/svrngs/src/cli.rs crates/svrngs-node/src/job/mac.rs \
    crates/svrngs-node/src/door/bound.rs crates/svrngs-node/src/uid.rs
export SOURCE_DATE_EPOCH=1795392000 CARGO_INCREMENTAL=0
build() { cargo build --locked --offline --ignore-rust-version -p svrngs -p svrngs-keeper; }
if [[ "$1" == plist ]]; then
    if [[ -f ../plist-baseline.tar ]]; then
        tar -xf ../plist-baseline.tar
        /usr/bin/python3 "$demo/controls.py" plist
        build
        for case in formatted binary malformed; do
            code=0
            /usr/bin/python3 -u "$demo/run.py" "plist-$case" || code=$?
            test "$code" -eq 5
        done
        tar -xf ../control-originals.tar
    fi
    /usr/bin/python3 "$demo/controls.py" plist
    build
    for case in formatted binary absent wrong-root malformed wrong-type; do
        /usr/bin/python3 -u "$demo/run.py" "plist-$case"
    done
    exit 0
fi
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
REMOTE
