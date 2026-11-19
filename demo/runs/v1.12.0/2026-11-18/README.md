# V1.12.0, published, signed and read back

V1.12.0 is main at `45ac6799`, tagged `v1.12.0`. It adds:
- updates only from a release the release key signed;
- `install.sh --archive` and `--uninstall`;
- `svrngs open`'s one-use code redeemed only from this account;
- each lend kept to the account its program runs as.

RELEASES.md says what a person sees. The four points of the plugin
marketplace's review are what it answers.

## The whole check

`scripts/check.sh` ran on `45ac6799` itself, in a clean detached worktree
(`/Users/alexsbryan/dev/svrngs-v1.12.0`). Before it: `npm ci --prefix web`.
It ran under the gate lock, with Node 24 (`gate-45ac6799.txt`).

The first run passed 32 of 45 steps in 1,553 s, exit 1. The scenes step
failed because the shared Chromium crashed mid-step:
- chrome-headless-shell, SIGSEGV KERN_INVALID_ADDRESS at 0x1b0 on
  CrBrowserMain, 23:13:57 -0800;
- the crash report is
  `~/Library/Logs/DiagnosticReports/chrome-headless-shell-2026-11-18-231359.ips`;
- every Chromium scene after it was refused a connection to the browser.

The scenes step alone, run as check.sh runs it, then passed: 18 scenes in
360 s. That rerun is in the same file, labeled with its reason.

`45ac6799` differs from `42fb774c` only by the version and docs, and
`42fb774c` had passed 45 of 45 earlier that night. The worktree's
`target/debug` (7.7 GB) was deleted before the release build.

## The three steps

1. `publish.py` ran `deploy-page.sh --build` in that worktree, with
   CARGO_TARGET_DIR and RUSTFLAGS unset. `publish.json` says passed, exit
   0, from 07:27 to 07:33 UTC. The page built as version `f72aa3cd476e`,
   replacing V1.11.2's `e3009ef31ec4`. The Linux svrngs and give binaries
   are ELF type 3 (DYN).
2. The operator ran
   `scripts/sign-release.sh v1.12.0 45ac6799 <site>` and typed the
   passphrase. It signed the tag and pushed it. It also made GitHub release
   `v1.12.0`, titled V1.12.0, with 21 files: the eight downloads,
   `SHA256SUMS` and `install.sh`, a `.sig` for each, and
   `svrngs-release-key.asc`.
3. `deploy-page.sh --site <site>` checked `get/SHA256SUMS.sig` against
   the release key in a keyring of its own, then deployed to production
   (`deploy-site.log`: readyState READY, target production, aliased to
   group.svrnme.sh). The `.vercel/` and `.env.local` the Vercel CLI left
   in the site were deleted after.

## Readback

`verify.py` passed (`verification.json`, `verify.out`). It rejected its
planted faults first. The signature plants were:
- sums changed after signing;
- sums signed by another key;
- no signature.

On the live site at group.svrnme.sh:
- 91 public files equaled the built site;
- `SHA256SUMS` equaled the built one (`live-sums.txt`, `SHA256SUMS`);
- `get/SHA256SUMS.sig` equaled the signed site's, and verified against
  `svrngs-release-key.asc`, by `46B4D67C5E6621093BB1AB2B057BF9082AA03CFB`;
- `sw.js` named `f72aa3cd476e` and was network-first.

The GitHub release was read back with `gh release download v1.12.0`, into
a keyring holding only `svrngs-release-key.asc`:
- all 10 signatures were valid by `46B4D67C…2AA03CFB`, none bad;
- `shasum -a 256 -c SHA256SUMS` passed for all 8 downloads;
- the release's `SHA256SUMS` equaled the built one;
- its `install.sh` equaled `web/install.sh` at `45ac6799`.

The resolver was not deployed, and the trust floor is unchanged:
`git diff v1.11.2 45ac6799 -- resolver scripts/trust_floor.txt` is empty.

## Not judged

- No Linux run of the whole check.
- No phone was tried against the new page.
- No computer was updated from V1.11.2 to V1.12.0 through the live page.
