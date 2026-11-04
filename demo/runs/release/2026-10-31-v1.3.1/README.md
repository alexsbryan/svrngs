# V1.3.1, published 31 October 2026

The operator: "Cut the release". Notes: `RELEASES.md`, V1.3.1 (086e216b),
each paragraph's commits in that commit's body.

- **The gate:** `scripts/check.sh` passed whole at `c5857105`
  (`target/browser/followup-check.log`, exit 0); after it only the notes
  changed (`git diff --stat c5857105 086e216b`: `RELEASES.md` alone).
- **The build:** a worktree detached at `086e216b`, `npm ci --prefix web`,
  then `CARGO_INCREMENTAL=0 scripts/deploy-page.sh`, exit 0: eight tarballs,
  svrngs and give for darwin-aarch64, darwin-x86_64, linux-x86_64 and
  linux-aarch64 (`SHA256SUMS` here); page version 51ec52480b4c; deployment
  `dpl_AMmYq21ffsa72tLSWhCfty9mtbt6`, production.
- **Read back:** the live `get/SHA256SUMS` equals the built one; the
  darwin-aarch64 tarballs of svrngs and give, fetched live, hash to their
  lines; `https://group.svrnme.sh/` and a name under it send
  `Content-Security-Policy: frame-ancestors 'none'`; the page serves
  version 51ec52480b4c. The upload directory's `.vercel` and `.env.local`
  removed.
- `git tag v1.3.1 086e216b`, local. Pushing main and the tag is the
  operator's.
