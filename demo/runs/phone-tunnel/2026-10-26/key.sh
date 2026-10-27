#!/bin/sh
# The tunnel's key into the extension's build: the identity of a svrngs that
# redeemed a device link (../DESIGN.md §3), the lender's key and the lend's
# name, as ios/Extension/Key.generated.swift, which is never committed.
set -eu
[ $# -eq 3 ] || { echo "usage: key.sh <svrngs-dir> <lender-key> <lend-name>" >&2; exit 2; }
here=$(cd "$(dirname "$0")" && pwd -P)
seed=$(od -An -tx1 -v "$1/identity" | tr -d ' \n')
[ ${#seed} -eq 64 ] || { echo "key.sh: $1/identity is $((${#seed} / 2)) bytes, not 32" >&2; exit 1; }
# Each goes into a Swift string, so only a key's and a name's characters; the tunnel parses both again.
case $2 in *[!0-9a-z]* | '') echo "key.sh: the lender's key is not a key: $2" >&2; exit 1 ;; esac
case $3 in *[!A-Za-z0-9_-]* | '') echo "key.sh: not a lend's name: $3" >&2; exit 1 ;; esac
cat > "$here/ios/Extension/Key.generated.swift" <<SWIFT
// Written by key.sh from $1/identity; never committed.
enum Key {
    static let seedHex = "$seed"
    static let lender = "$2"
    static let lend = "$3"
}
SWIFT
echo "key.sh: wrote ios/Extension/Key.generated.swift for $3 at $2"
