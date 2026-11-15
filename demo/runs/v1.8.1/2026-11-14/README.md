# V1.8.1 release, 14 November 2026

Published from the commit tagged **v1.8.1**, on the operator's choice of
2026-11-14, "A: release V1.8.1".

## Scope and source evidence

The preceding release is V1.8 at 3c54f1ec, whose readback is
`../../v1.8/2026-11-13/README.md`. Before this run `group.svrnme.sh` still
served V1.8's page: its `sw.js` named version `ba66ef18e374`, V1.8's own,
read at 2026-11-15 03:32 UTC. `git diff --stat v1.8 0002248d` touches only
`web/` among shipped files (`web/sw.js`, `web/store.js`, `web/app.js`,
`web/lend.js`), so the programs do what V1.8's do.

| Release note | Source or run |
|---|---|
| The newest page opens every time, invitations included; offline, the kept copy | 0002248d; `web/sw.js` `answered` (a navigation to the network first, the cache when the fetch rejects); `web/tests/worker.test.mjs`; `browser::updated::a_new_build_runs_at_an_invitation_s_address_with_no_reload_by_hand` (`web/tests/updated.mjs`), red on 8a5e1689 in both engines, green after |
| "This browser could not open the page." until a hard reload | 0002248d's body: V1.7's page ran at a V1.8 invitation in a browser that had opened `group.svrnme.sh` before; fresh profiles opened it |
| A page left open loads again on a new version | 0002248d; `web/store.js` `reloadsOnUpdate`, registered before the worker in `web/app.js` |
| A browser holding V1.8's copy may show it once more | 0002248d's body: the worker already in people's browsers cannot be changed; V1.8's `answered` is cache-first, so its first open after V1.8.1 is V1.8's page, and the new worker answers the next |
| Nothing else changes | c7386ee1, `web/lend.js` reads and writes a frame's lengths by `DataView`, behavior kept (its body: 0 differences over every opcode and 20,000 random buffers) |

## Validation before packaging

`scripts/check.sh` ran whole at 0002248d under the gate lock, exit 0, 45
steps passed, 1,538 s in all (`gate-0002248d.txt`). The release commit
changes only `RELEASES.md` and this directory, so the gate was not run
again.

`rustup target list --installed --toolchain 1.95.0` lists
`aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`
and `aarch64-unknown-linux-musl`, every target `scripts/build-release.sh`
builds, so no platform is missing.

## Publication

`publish.py` ran `scripts/deploy-page.sh` under the gate lock in a clean
detached worktree at 83f1cdab (`v1.8.1`), 2026-11-15 03:35 to 03:47 UTC,
exit 0 (`publish.json`, `publish.log`), at a load of about 6 with 60 GiB
free. Its record was written outside the worktree, which `publish.py` holds
clean, and copied here after. The build meets its groups at n0's relays
(`SHIPPED`), so no relay was asked. The page built as version
7a84f5701243, `svrngs` and `give` built for all four platforms, and Vercel
deployment `dpl_JDRawCkzKnvKXJ2MjReenofiLW6q` reached production, Ready,
aliased to `https://group.svrnme.sh` and `https://*.group.svrnme.sh`
(`vercel inspect svrngs-page-d4n3aknks-svrnmesh.vercel.app --scope
svrnmesh`; this CLI's deploy printed no JSON summary into the log). The
`.vercel/` and `.env.local` the CLI wrote into the upload directory were
deleted after.

## Readback

`verify.py` passed (`verification.json`). It is V1.8's, plus one check:
the live `sw.js` equals the built one, and its `answered` sends a
navigation to the network before the cache. Its six planted faults were
rejected first, a cache-first `answered` among them, and V1.8's live
`sw.js`, read before the run, was rejected by the same check. Then 78
public files fetched from `group.svrnme.sh` each equaled the built site's
bytes; the live `get/SHA256SUMS` equaled the built one (`SHA256SUMS`,
`live-sums.txt`) and named exactly the eight downloads, so the install
line fetches this build; the live page is version 7a84f5701243, not V1.8's
ba66ef18e374, and is the one the live `sw.js` names; the live `install.sh`
equals the built one; both frame policies were as `vercel.json` sets them;
the wildcard name `release-83f1cdab.group.svrnme.sh` served the same page;
`.env.local`, `.vercel/project.json` and `vercel.json` each answered 404;
and the downloaded darwin-aarch64 `svrngs --help` (exit 0) and `give`
(exit 2, its usage) started. No other platform's program was run here.
