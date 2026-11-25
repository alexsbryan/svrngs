#!/usr/bin/env bash
# The vendor's CI, as the agent may call it: test a branch of the vendor's
# repo; if its tests pass, build it as a signed *candidate* and deliver it to
# the clinic, whose site tool trials it beside production by itself. A
# candidate can never be installed there. The result lands in the view's
# /trials.
#   ./ci.sh <branch> <version>
set -euo pipefail
cd "$(dirname "$0")"
B=$1 V=$2 R=out/vendor-repo W=out/ci-$1
rm -rf "$W"; git -C $R worktree prune; git -C $R worktree add -q -f "$PWD/$W" "$B"
echo "== tests on $B ($(git -C $W rev-parse --short HEAD))"
(cd $W && python3 -m unittest discover -s tests 2>&1 | tail -3)
node vendor.mjs bundle vendor/recipe.json keys "bundles/candidate-$V" --source "$W" --version "$V" --channel candidate --from "$B@$(git -C $W rev-parse --short HEAD)"
podman cp "bundles/candidate-$V" onprem-clinic:/media/delivery/
git -C $R worktree remove --force "$PWD/$W"
echo "delivered candidate $V; its trial result appears in ./view.sh /trials within a minute"
