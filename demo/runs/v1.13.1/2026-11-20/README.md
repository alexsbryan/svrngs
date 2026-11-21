# V1.13.1, svrngs's own relay and each lend kept to what you chose

V1.13.1 is main at `bd8db5a0`, the commit to tag `v1.13.1`. It is the
first release since V1.12.2: V1.13.0 was never tagged or published, so
V1.13.1 carries V1.13.0's switch to svrngs's own relay, the three fixes a
phone test of it found (first gated on the branch `v1.13.1` at `ba2500da`,
[gate-ba2500da.txt](gate-ba2500da.txt)), and each lend kept to what its
person chose on its port. RELEASES.md says what a person sees.

## Done before the release, on Linux

1. **The whole check.** `scripts/check.sh` on `bd8db5a0` in svrngs-gate
   (scripts/gate/README.md's command), from the plain clone
   `~/dev/svrngs-gate-run` detached at `bd8db5a0`, after `npm ci --prefix
   web`: 45 of 45 steps passed in 1,319 s, exit 0
   ([gate-bd8db5a0.txt](gate-bd8db5a0.txt)), the `internet` step against
   relay.svrnme.sh among them.
2. **The relay.** `scripts/deploy-relay.sh --host root@<box>` from main at
   `bd8db5a0` on the Fedora box, since the node's fixes change the relay's
   binary. DESIGN §7's order: the relay before the page.

## The release (the operator, on the Mac)

As V1.12.2's (../../v1.12.2/2026-11-19/): `deploy-page.sh --build` in a
clean detached worktree at `bd8db5a0`, `sign-release.sh`, then
`deploy-page.sh --site`, the readback and the Arch package.

1. **The build.** `scripts/deploy-page.sh --build` in `~/dev/svrngs-v1.13.1`,
   detached at `bd8db5a0`: exit 0 ([publish.log](publish.log)).
2. **The release.** `scripts/sign-release.sh v1.13.1 bd8db5a0… <site>`: the
   tag `v1.13.1` is signed by the release key (`git tag -v v1.13.1`: "Good
   signature from svrngs releases") and is GitHub's latest release, 21 files.
3. **The Arch package.** `scripts/arch-package.py 1.13.1`: five signatures by
   the release key, a planted sum refused, makepkg `--verifysource` passed
   ([arch-package.json](arch-package.json), [arch-package.log](arch-package.log)).
4. **The page.** `scripts/deploy-page.sh --site <site>` checked
   `get/SHA256SUMS.sig` against the release key, then deployed to production
   ([deploy-site.log](deploy-site.log): readyState READY, target
   production). The `.vercel/`, `.env.local` and `.gitignore` the Vercel CLI
   left in the site were deleted after.

## Readback

`verify.py <site> . bd8db5a0…` passed ([verification.json](verification.json),
[verify.out](verify.out)). It is V1.12.2's, taking the commit as its third
argument in place of publish.json, with V1.12.2's page as the one before.
It rejected its 13 planted faults first. On the live site at
group.svrnme.sh:
- 91 public files equaled the built site;
- `SHA256SUMS` equaled the built one ([live-sums.txt](live-sums.txt),
  [SHA256SUMS](SHA256SUMS));
- `get/SHA256SUMS.sig` equaled the signed site's and verified against
  `svrngs-release-key.asc`;
- `sw.js` named `959b4ec5eb9b`, not V1.12.2's `dbe5e921907a`, and was
  network-first.

The GitHub release was read back with `gh release download v1.13.1`, into a
keyring holding only `svrngs-release-key.asc`: all 10 signatures valid by
`46B4D67C…2AA03CFB`, none bad; `shasum -a 256 -c SHA256SUMS` passed for all
8 downloads; its `SHA256SUMS` equaled the built one; its `install.sh`
equaled `web/install.sh` at `bd8db5a0`.
