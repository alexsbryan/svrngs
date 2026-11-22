# V1.13.3, your apps use a friend's lend with no key, and svrngs's token goes nowhere

V1.13.3 is main at `7effd03a`, the commit to tag `v1.13.3`. RELEASES.md
says what a person sees:
- Play at a lend's address, for this account's programs, with no key;
- updates only forward;
- svrngs proves itself before anything of its own asks it, and a program
  asks svrngs with a code made for its own connection, never with its
  token (`svrngs ask`, apps/client/svrngs.mjs).

## Done before the release, on Linux

1. **The whole check.** `scripts/check.sh` on `7effd03a` in svrngs-gate
   (scripts/gate/README.md's command), from the plain clone
   `~/dev/svrngs-gate-run` on the Fedora box. The clone was given the
   unpushed commits by a git bundle, then detached at `7effd03a`, after
   `npm ci --prefix web`.
   - **At `02e393a1`, the commit before, it failed.** The second run
     ([gate-02e393a1-run2.txt](gate-02e393a1-run2.txt)) passed 24 of 27
     steps, exit 1.
     - The failures were clippy, scenes and playback, all from one compile
       error, E0716 at crates/svrngs/tests/scenes/people.rs:392: a token
       freed before the future that borrowed it was awaited.
     - The first run also failed `relay_tiers`' fragile seek once ("waited
       8090 ms behind the lookahead"), which passed on the second.
   - **`7effd03a` binds the token first** and changes nothing else.
   - **At `7effd03a`:** every step passed, 45 of 45, 1,163 s, exit 0
     ([gate-7effd03a.txt](gate-7effd03a.txt)).
2. **The unsigned candidate.** svrngs `02e393a1` and svrngs-fam `c06c84d`
   were built as `scripts/build-release.sh` builds a release, and ran
   through the release test on Omarchy guests.
   - **The result:** 39 cases passed, none failed. Two waited for the
     release itself: the plugin's Get svrngs, and the marketplace's checks.
   - **A new case:** a program sending svrngs's token is refused, whether
     at the API as Bearer, or at a lend's address as Bearer or `X-Api-Key`,
     with Play on and off. The lent service never sees it, and `svrngs ask`
     answers.
     - It failed 8 of 10 on the signed 1.13.2 pair, which took the token.
     - It passed 10 of 10 on the candidate.
   - **The product code is unchanged at `7effd03a`:** only the test file
     above differs from `02e393a1`.

## The release

1. **The build.** `scripts/release.sh build 7effd03a`, in the release
   worktree `~/dev/svrngs-release`: exit 0 ([publish.log](publish.log)).
2. **The signing.** The operator ran `scripts/release.sh sign` and typed the
   passphrase.
   - The tag `v1.13.3` names `7effd03a` and is signed by the release key
     (`git tag -v v1.13.3`: "Good signature from svrngs releases").
   - It is GitHub's latest release, with 21 files.
3. **The page.** `scripts/release.sh page`: deployed to production and
   aliased to group.svrnme.sh ([deploy-site.log](deploy-site.log),
   readyState READY).
4. **The Arch package.** `scripts/arch-package.py 1.13.3` checked five
   signatures by the release key, refused a planted sum, and passed
   makepkg `--verifysource` ([arch-package.json](arch-package.json),
   [arch-package.log](arch-package.log)).

## Readback

`verify.py <site> . 7effd03a…` passed ([verification.json](verification.json),
[verify.out](verify.out)). It is V1.13.2's, with V1.13.2's page,
`c2e267153f70`, as the one before, and it rejected its 13 planted faults
first. On the live site at group.svrnme.sh:
- 91 public files equaled the built site;
- `SHA256SUMS` equaled the built one ([live-sums.txt](live-sums.txt),
  [SHA256SUMS](SHA256SUMS));
- `get/SHA256SUMS.sig` equaled the signed site's, and verified against
  `svrngs-release-key.asc`;
- `sw.js` named `f8386cead4a6`, not V1.13.2's `c2e267153f70`, and was
  network-first.

The GitHub release was read back with `gh release download v1.13.3`, into
a keyring holding only `svrngs-release-key.asc`:
- all 10 signatures were valid, by `46B4D67C…2AA03CFB`, none bad;
- `shasum -a 256 -c SHA256SUMS` passed for all 8 downloads;
- its `SHA256SUMS` equaled the built one;
- its `install.sh` equaled `web/install.sh` at `7effd03a`.
