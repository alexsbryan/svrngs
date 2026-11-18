# V1.11.2, published and read back

V1.11.2 is main at `76366903`. It adds:

- packaging/arch, so svrngs installs with pacman from the signed release;
- the installer signed beside the downloads;
- Linux's svrngs and give as static-pie with full RELRO.

RELEASES.md says what a person sees.

## The whole check

`scripts/check.sh` ran on `76366903` itself, in a clean detached worktree
(`/Users/alexsbryan/dev/svrngs-v1.11.2`), after `npm ci --prefix web`,
under the gate lock, with Node 24. It exited 0, and all 45 of 45 steps
passed in 1,725 s (`gate-76366903.txt`). Its `target/debug` (7.6 GiB) was
deleted before the release build.

## Publication

`publish.py` ran `deploy-page.sh` in that same worktree, with
CARGO_TARGET_DIR and RUSTFLAGS unset. `publish.json` says passed, exit 0,
from 23:00 to 23:07 UTC. The page built as version `e3009ef31ec4`,
replacing V1.11.1's `783da285934f`. Both Linux `svrngs` binaries in the
built `get/` are ELF type 3 (DYN), as `build-release.sh` now requires.

## Readback

`verify.py` passed at 23:07 UTC (`verification.json`, `verify.out`). It
rejected its ten planted faults first. Then 79 public files at
`group.svrnme.sh` equaled the built site, the live `SHA256SUMS` equaled
the built one (`live-sums.txt`, `SHA256SUMS`), and the live `sw.js` named
`e3009ef31ec4` and was network-first.

The resolver was not deployed: `git diff v1.11.1 76366903 -- resolver`
is empty.

## The GitHub release

The operator ran `scripts/sign-release.sh v1.11.2 76366903 <site>`, with
the site that `publish.json` names. GitHub release `v1.11.2`, titled
V1.11.2, has 21 files: the eight downloads, `SHA256SUMS` and `install.sh`,
a `.sig` for each, and `svrngs-release-key.asc`.

Read back from `gh release download v1.11.2` into a keyring holding only
the released key `46B4D67C5E6621093BB1AB2B057BF9082AA03CFB`:

- all ten signatures said VALIDSIG, `install.sh.sig` among them;
- `shasum -a 256 -c SHA256SUMS` passed for all eight downloads, and
  `SHA256SUMS` equaled this run's;
- `install.sh` equaled `git show 76366903:web/install.sh`;
- planted: `SHA256SUMS` with one byte added said BADSIG;
- `git verify-tag v1.11.2` said VALIDSIG, and the tag is `76366903`.

packaging/arch's PKGBUILD moved to 1.11.2, pkgrel 1. Each sum was taken
from those downloaded assets, and each tarball's sum equals its
`SHA256SUMS` line. In an x86_64 Arch container, `makepkg --verifysource`
passed every source sum and both signatures. aarch64's sums were taken the
same way, but makepkg didn't check them.

Not run: a Linux gate, an installed package of 1.11.2, and a phone
against the published page.
