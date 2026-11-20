# V1.13.0, the switch to svrngs's own relay

V1.13.0 is main at `198b8964`, the commit to tag `v1.13.0`. Its builds meet
their groups at svrngs's relay, relay.svrnme.sh, with n0's behind it
(`cli::SHIPPED`). RELEASES.md says what a person sees.

## Done before the release, on Linux

1. **The whole check.** `scripts/check.sh` on `198b8964` in svrngs-gate,
   from a plain clone, after `npm ci --prefix web`: 45 of 45 steps passed in
   1,411 s, exit 0 ([gate-198b8964.txt](gate-198b8964.txt)), the `internet`
   step against relay.svrnme.sh among them.
2. **The relay.** `scripts/deploy-relay.sh --host root@<box>` from a clean
   detached worktree at `198b8964` ([deploy-relay.log](deploy-relay.log)):
   the box answers `svrngs version` with `svrngs 1.13.0 (198b8964)`, and
   `https://relay.svrnme.sh/` answers. No AAAA: the box cannot reach out over
   IPv6 at regxa yet. DESIGN §7's order: the relay before the page.

## The release (the operator, on the Mac)

deploy-page.sh --build, sign-release.sh, deploy-page.sh --site, the
readback and the Arch package, as V1.12.2's (../../v1.12.2/2026-11-19/).
