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
