# V1.13.2, friends joining a container's game together

V1.13.2 is main at `9be0fcc7`, the commit to tag `v1.13.2`. It is V1.13.1
(`bd8db5a0`) with one change in what a person sees: a UDP socket read at
an address and port 0, as a server's is while it connects, is read as
connecting (`4a692a95`), so friends joining a rootless Podman game at once
keep its lend open. RELEASES.md says what a person sees.

## Done before the release, on Linux

1. **The whole check.** `scripts/check.sh` on `9be0fcc7` in svrngs-gate
   (scripts/gate/README.md's command), from the plain clone
   `~/dev/svrngs-gate-run` detached at `9be0fcc7`, after `npm ci --prefix
   web`.
   - The first run failed one test of 45 steps' `tests`
     ([gate-9be0fcc7-run1.txt](gate-9be0fcc7-run1.txt)):
     `relay_tiers::a_dial_reaches_a_member_at_a_relay_this_computer_does_not_hold`,
     which docs/seams.tsv calls fragile. Its dial through a real relay on
     127.0.0.1 did not finish its handshake in 10 s (`Err(Elapsed(()))`,
     relay_tiers.rs:84), and Ada's accept saw the handshake aborted
     (relay_tiers.rs:77). `4a692a95` changes no relay or dial code.
   - The second run, unchanged: 45 of 45 steps passed in 1,129 s, exit 0
     ([gate-9be0fcc7.txt](gate-9be0fcc7.txt)).
2. **The relay.** Not deployed again: relay.svrnme.sh runs `bd8db5a0`'s
   build (../../v1.13.1/2026-11-20/deploy-relay.log). `4a692a95` changes
   only how a lender reads the sockets on its own lend's port (uid.rs
   `peer`), which a relay never runs.

## The release

As V1.13.1's (../../v1.13.1/2026-11-20/):

1. **The build.** `scripts/deploy-page.sh --build` in `~/dev/svrngs-v1.13.2`,
   detached at `9be0fcc7`: exit 0 ([publish.log](publish.log)).
2. **The release.** `scripts/sign-release.sh v1.13.2 9be0fcc7… <site>`, the
   operator typing the passphrase: the tag `v1.13.2` is signed by the
   release key (`git tag -v v1.13.2`: "Good signature from svrngs
   releases") and is GitHub's latest release, 21 files.
3. **The Arch package.** `scripts/arch-package.py 1.13.2`: five signatures by
   the release key, a planted sum refused, makepkg `--verifysource` passed
   ([arch-package.json](arch-package.json), [arch-package.log](arch-package.log)).
   svrngs-omarchy `ffec02b` copies it (`packaging/arch/bump 1.13.2
   ~/dev/svrngs 72246837`); in an x86_64 Arch container, with only the key in
   keys/pgp imported, makepkg `--verifysource` passed every sum and both
   signatures.
4. **The page.** `scripts/deploy-page.sh --site <site>` checked
   `get/SHA256SUMS.sig` against the release key, then deployed to production
   ([deploy-site.log](deploy-site.log): readyState READY, target
   production). The `.vercel/`, `.env.local` and `.gitignore` the Vercel CLI
   left in the site were deleted after.

## Readback

`verify.py <site> . 9be0fcc7…` passed ([verification.json](verification.json),
[verify.out](verify.out)). It is V1.13.1's with V1.13.1's page,
`959b4ec5eb9b`, as the one before. It rejected its 13 planted faults first.
On the live site at group.svrnme.sh:
- 91 public files equaled the built site;
- `SHA256SUMS` equaled the built one ([live-sums.txt](live-sums.txt),
  [SHA256SUMS](SHA256SUMS));
- `get/SHA256SUMS.sig` equaled the signed site's and verified against
  `svrngs-release-key.asc`;
- `sw.js` named `c2e267153f70`, not V1.13.1's `959b4ec5eb9b`, and was
  network-first.

The GitHub release was read back with `gh release download v1.13.2`, into a
keyring holding only `svrngs-release-key.asc`: all 10 signatures valid by
`46B4D67C…2AA03CFB`, none bad; `shasum -a 256 -c SHA256SUMS` passed for all
8 downloads; its `SHA256SUMS` equaled the built one; its `install.sh`
equaled `web/install.sh` at `9be0fcc7`.
