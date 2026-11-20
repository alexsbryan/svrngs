# V1.13.1, three fixes from a phone test of V1.13.0

V1.13.1 is the branch `v1.13.1` at `ba2500da`, on main at `d3a09183`
(V1.13.0). RELEASES.md says what a person sees.

## Done before the release, on Linux

1. **The whole check.** `scripts/check.sh` on `ba2500da` in svrngs-gate,
   from a plain clone, after `npm ci --prefix web`: 45 of 45 steps passed in
   1,451 s, exit 0 ([gate-ba2500da.txt](gate-ba2500da.txt)), the `internet`
   step against relay.svrnme.sh among them. An earlier run was stopped at
   step 13 when the machine ran out of memory; nothing in it had failed.
2. **The relay.** Not yet: `scripts/deploy-relay.sh` from a clean worktree
   at `ba2500da` follows V1.13.0's release, since the node's fix changes the
   relay's binary.

## The release (the operator, on the Mac)

As V1.13.0's (../../v1.13.0/2026-11-20/).
