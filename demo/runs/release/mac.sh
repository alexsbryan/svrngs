#!/bin/bash
# Sent over SSH with the candidate archive, so staging and teardown have one owner.
set -euo pipefail
umask 077
candidate=$1 short=$2
export SOURCE_DATE_EPOCH=$3 SVRNGS_COMMIT=$2 TOKIO_WORKER_THREADS=4
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
root=$HOME/svrngs-keeper-scratch
export CARGO_TARGET_DIR=$root/repo/target
domain=gui/$(id -u)
tags=(avail-run-mac mac-proof-lineage mac-proof-ownership mac-proof-managed mac-renew mac-renew-door)
label() { printf '%s/svrngs-keeper-test-%s' "$domain" "$1"; }
result() { printf 'release-case %s %s %s\n' "$1" "$2" "$3"; }

# Refuse to inherit a previous run's scratch or take over any registration.
for tag in "${tags[@]}"; do
    code=0
    launchctl print "$(label "$tag")" >/dev/null 2>&1 || code=$?
    if [[ $code != 113 ]]; then
        printf 'Preflight: %s not available (launchctl print exit=%s)\n' "$(label "$tag")" "$code"
        exit 125
    fi
done
launchctl print "$domain" >/dev/null || exit 125
mkdir "$root" || exit 125
cleanup() {
    code=0
    for tag in "${tags[@]}"; do
        job=$(label "$tag")
        status=0
        registration=$(launchctl print "$job" 2>&1) || status=$?
        if [[ $status == 0 && $registration == *"path = $root/"* ]]; then
            launchctl bootout "$job" || code=1
            for ((i=0; i<40; i++)); do
                status=0
                launchctl print "$job" >/dev/null 2>&1 || status=$?
                [[ $status != 113 ]] || break
                sleep .25
            done
            printf 'Cleanup: unloaded test agent %s (launchctl print exit=%s)\n' "$job" "$status"
        fi
        if [[ $status != 113 ]]; then
            printf 'Cleanup: could not verify removal of %s (exit=%s)\n' "$job" "$status"
            code=1
        fi
    done
    cd "$HOME"
    if [[ $code == 0 ]] && rm -rf "$root" && [[ ! -e $root ]]; then
        printf 'Cleanup: all six test labels absent (exit 113); removed %s\n' "$root"
    else
        code=1
    fi
    result mac-cleanup "$code" launchd
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' HUP TERM
tar -xf - -C "$root"
actual=$(git get-tar-commit-id <"$root/source.tar")
printf 'release-candidate %s\n' "$actual"
[[ $actual == "$candidate" ]] || exit 1
mkdir "$root/repo" "$root/tests-home"
tar -xf "$root/source.tar" -C "$root/repo"
cd "$root/repo"
uname -sm
cargo --version
shasum -a 256 Cargo.lock "$root/qwen2.5-0.5b.gguf"

demo=demo/runs/keeper/2026-11-23
renew=$demo-mac-renew
python=/usr/bin/python3
pair=target/debug
build() {
    cargo build --locked --offline -p svrngs -p svrngs-keeper || return
    local version
    version=$("$pair/svrngs" version) || return
    printf 'Build version: %s\n' "$version"
    if [[ $version != *" ($short)" ]]; then
        printf 'Candidate mismatch: expected svrngs version (%s)\n' "$short"
        return 1
    fi
    shasum -a 256 "$pair/svrngs" "$pair/svrngs-keeper" || return
    build_id=$1:$(shasum -a 256 "$pair/svrngs" "$pair/svrngs-keeper" | shasum -a 256 | cut -d' ' -f1)
}

suite() {
    local log=$root/$1-test.log code=0
    env HOME="$root/tests-home" SVRNGS_DIR="$root/tests-home/node" XDG_DATA_HOME="$root/tests-home/data" \
        CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
        cargo test --locked --offline -p "svrngs-$1" >"$log" 2>&1 || code=$?
    cat "$log"
    if [[ $code == 0 ]] && ! grep -Eq '^test result: ok\. [1-9]' "$log"; then return 125; fi
    return "$code"
}

step() {
    local name=$1 build=$2 code=0
    shift 2
    printf 'Running %s (%s)\n' "$name" "$build"
    "$@" || code=$?
    result "$name" "$code" "$build"
}

step mac-keeper-tests darwin-arm64/cargo-test/svrngs-keeper suite keeper
step mac-node-tests darwin-arm64/cargo-test/svrngs-node suite node

build_id=none
if build native; then
    step mac-run "$build_id" "$python" -Bu "$demo-run-mac/run.py"
    for case in lineage ownership managed; do
        step "mac-proof-$case" "$build_id" "$python" -Bu "$demo-mac-proof/proof.py" "$pair/svrngs-keeper" "$case"
    done
else
    result mac-run 1 native-build
fi

# These two saved drivers need different fixture instrumentation; derive
# each from the same archive before applying its existing controls.
tar -cf "$root/originals.tar" crates/svrngs/src/cli.rs crates/svrngs-node/src/{job/mac,door/bound,uid}.rs
if "$python" -B "$renew/controls.py" test && build renewal-fixture; then
    step mac-renew "$build_id" "$python" -Bu "$renew/run.py" all
else
    result mac-renew 1 renewal-fixture-build
fi
tar -xf "$root/originals.tar"
if "$python" -B "$renew/controls.py" plist && build plist-fixture; then
    for case in formatted binary absent wrong-root malformed wrong-type; do
        step "mac-plist-$case" "$build_id" "$python" -Bu "$renew/run.py" "plist-$case"
    done
else
    result mac-plist-formatted 1 plist-fixture-build
fi
