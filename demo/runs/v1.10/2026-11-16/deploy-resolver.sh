#!/usr/bin/env bash
# Deploy the resolver at the tag to join.svrnme.sh (Vercel project
# svrngs-join, resolver/README.md): exactly the files the deployment runs,
# api/join.mjs and what vercel.json bundles beside it, from `git archive`,
# so the upload is the tag's bytes and no README or test. The server-side
# token is the project's production env var already, never passed here.
# The CLI's .vercel/ and .env.local are deleted from the upload after.
#
#   deploy-resolver.sh <tag> <record dir>
set -euo pipefail
tag="${1:?tag}"; record="${2:?record dir}"
repo="$HOME/dev/svrngs"
work="$(mktemp -d)"
git -C "$repo" archive "$tag" resolver | tar -x -C "$work"
upload="$work/upload"
mkdir -p "$upload/api"
for f in handler.mjs store.mjs words.txt package.json vercel.json; do cp "$work/resolver/$f" "$upload/$f"; done
cp "$work/resolver/api/join.mjs" "$upload/api/join.mjs"
( cd "$upload" && find . -type f | sort | xargs shasum -a 256 ) > "$record/resolver-upload.txt"
echo "deploy-resolver: $(git -C "$repo" rev-parse --short "$tag^{commit}") to svrngs-join (svrnmesh) from $upload"
vercel link --yes --project svrngs-join --scope svrnmesh --cwd "$upload" >/dev/null
vercel deploy --prod --yes --scope svrnmesh --cwd "$upload"
rm -rf "$upload/.vercel" "$upload/.env.local"
echo "deploy-resolver: upload left at $upload, its CLI files deleted"
