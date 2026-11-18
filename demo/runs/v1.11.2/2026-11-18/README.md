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

It waits for the operator's signature. The command is
`scripts/sign-release.sh v1.11.2 76366903 <site>`, where the site is the
one `publish.json` names. It now also signs `install.sh` as the commit has
it.

Not run: a Linux gate, an installed package of 1.11.2, and a phone
against the published page.
