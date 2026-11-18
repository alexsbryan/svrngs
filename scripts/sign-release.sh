#!/bin/sh
# sign-release.sh — sign a published release for packagers and keep it as a
# GitHub release: each download and SHA256SUMS gets a detached OpenPGP
# signature (`<file>.sig`, what a PKGBUILD's validpgpkeys checks), the tag
# is signed, and the release carries the signatures and the public key.
#
# The operator runs it and types the release key's passphrase in gpg's
# dialog; no agent signs (RELEASES.md, "Signed releases"). It signs what
# deploy-page.sh built and published, after verify.py read it back.
#
#   scripts/sign-release.sh v1.11.1 <commit> <site>
#
# <site> is the directory deploy-page.sh printed; its get/ holds the
# downloads and SHA256SUMS. The notes are RELEASES.md's section for the
# version, as the tagged commit has it.
set -eu
tag=$1 commit=$2 get=$3/get
version=V${tag#v}
key=$(gpg --list-secret-keys --with-colons "svrngs releases" | awk -F: '/^fpr/ { print $10; exit }')
if [ -z "$key" ]; then
  echo "sign-release: no 'svrngs releases' key here; make it once with:"
  echo "  gpg --quick-generate-key 'svrngs releases' ed25519 sign 2y"
  exit 1
fi
echo "sign-release: $tag ($commit) by $key from $get"

(cd "$get" && shasum -a 256 -c SHA256SUMS)
# The installer too, as the release's commit has it, so whatever installs
# svrngs for someone can check it before running it, where the line on the
# page trusts HTTPS alone.
git show "$commit:web/install.sh" >"$get/install.sh"
for file in "$get"/*.tar.gz "$get"/SHA256SUMS "$get"/install.sh; do
  gpg --yes --local-user "$key" --detach-sign --output "$file.sig" "$file"
  gpg --verify "$file.sig" "$file"
done
gpg --armor --export "$key" >"$get/svrngs-release-key.asc"

notes=$(mktemp)
git show "$commit:RELEASES.md" | awk -v head="## $version," 'index($0, head) == 1 { on = 1; next } /^## / { if (on) exit } on' >"$notes"
test -s "$notes" || { echo "sign-release: RELEASES.md at $commit has no $version section"; exit 1; }
printf '\nSigned by the svrngs release key, %s: each file has its `.sig`, and `svrngs-release-key.asc` is the key.\n' "$key" >>"$notes"

git tag --sign --local-user "$key" --message "svrngs $version" "$tag" "$commit"
git push origin "$tag"
gh release create "$tag" --verify-tag --latest --title "$version" --notes-file "$notes" \
  "$get"/*.tar.gz "$get"/*.tar.gz.sig "$get"/SHA256SUMS "$get"/SHA256SUMS.sig \
  "$get"/install.sh "$get"/install.sh.sig "$get"/svrngs-release-key.asc
echo "sign-release: $tag released; read it back with gh release download $tag"
