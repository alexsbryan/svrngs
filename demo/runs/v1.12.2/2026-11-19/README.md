# V1.12.2, published, signed and read back

V1.12.2 is main at `c95c6509`, tagged `v1.12.2`. It keeps each lend to
the program it lent, and adds `svrngs share --pid`. RELEASES.md says what a
person sees.

## The whole check

`scripts/check.sh` ran on `c95c6509` itself, in a clean detached worktree
(`/Users/alexsbryan/dev/svrngs-v1.12.2`). Before it: `npm ci --prefix web`.
It ran under the gate lock, with Node 24 (`gate-c95c6509.txt`). It passed 45
of 45 steps in 1,707 s, exit 0. The worktree's `target/debug` was deleted
before the release build.

## The three steps

1. `publish.py` ran `deploy-page.sh --build` in that worktree, with
   CARGO_TARGET_DIR and RUSTFLAGS unset. `publish.json` says passed, exit
   0, from 21:07 to 21:13 UTC. The page built as version `dbe5e921907a`,
   replacing V1.12.1's `078d2a2042f9`. The Linux svrngs binary is a
   static-pie ELF.
2. The operator ran
   `scripts/sign-release.sh v1.12.2 c95c6509 <site>` and typed the
   passphrase. It signed the tag and pushed it. It also made GitHub release
   `v1.12.2`, titled V1.12.2, with 21 files: the eight downloads,
   `SHA256SUMS` and `install.sh`, a `.sig` for each, and
   `svrngs-release-key.asc`.
3. `deploy-page.sh --site <site>` checked `get/SHA256SUMS.sig` against
   the release key in a keyring of its own, then deployed to production
   (`deploy-site.log`: readyState READY, target production, aliased to
   group.svrnme.sh). The `.vercel/` and `.env.local` the Vercel CLI left
   in the site were deleted after.

## Readback

`verify.py` passed (`verification.json`, `verify.out`). It rejected its 13
planted faults first, the three signature plants among them. On the live
site at group.svrnme.sh:
- 91 public files equaled the built site;
- `SHA256SUMS` equaled the built one (`live-sums.txt`, `SHA256SUMS`);
- `get/SHA256SUMS.sig` equaled the signed site's, and verified against
  `svrngs-release-key.asc`, by `46B4D67C5E6621093BB1AB2B057BF9082AA03CFB`;
- `sw.js` named `dbe5e921907a` and was network-first.

The GitHub release was read back with `gh release download v1.12.2`, into
a keyring holding only `svrngs-release-key.asc`:
- all 10 signatures were valid by `46B4D67C…2AA03CFB`, none bad;
- `shasum -a 256 -c SHA256SUMS` passed for all 8 downloads;
- the release's `SHA256SUMS` equaled the built one;
- its `install.sh` equaled `web/install.sh` at `c95c6509`.

The resolver was not deployed, and the trust floor is unchanged:
`git diff v1.12.1 c95c6509 -- resolver scripts/trust_floor.txt` is empty.

## The Arch package

`scripts/arch-package.py 1.12.2` passed (`arch-package.json`,
`arch-package.log`):
- all 5 signatures it fetched verified by the release key alone;
- each tarball's sum equaled its line in `SHA256SUMS`;
- it wrote `packaging/arch/PKGBUILD` at pkgver 1.12.2;
- in an x86_64 Arch container, a copy with one sum changed was refused
  first, then `makepkg --verifysource` passed every sum and both
  signatures.

## Not judged

- No Linux run of the whole check.
- No phone was tried against the new page.
- No computer was updated from V1.12.1 to V1.12.2 through the live page.
