# Relay-meet demo candidate — focused verification

The operator selected this worktree for an urgent demo. It starts from the
completed onboarding/welcome/portal/OIDC-guidance/grouped-home baseline at
`e7ef3c45`, excluding the unfinished scoped-lending WIP on main. The released
relay-meet patch adds keeper-only `link --relay` using existing `MeetsAt`.
The run-of-show is `demo/mesh-house.md`.

Observed verification on 2026-11-05 UTC:

- `cargo test --locked -p svrngs --test meet --test flags --test new
  --test join_by_link -- --nocapture`: 3 meet, 2 flags, 15 new and 14 join
  tests passed (34 total), including keeper refusal and relay propagation.
- Named plant: replace `link_to`'s `Host::meet` call with `let _ = relays`.
  The keeper test failed at `meet.rs:101`, "the new link carries the new
  relay" (exit 101). Restore the call: all three meet tests passed, exit 0.
- `cargo check --locked --workspace --all-targets`: exit 0.
- `cargo fmt --all --check` and source whitespace check: exit 0.
- Lean: core 3,486/3,494, non-JS 19,832/19,833, JS 8,187/8,188,
  Swift 0/1; exit 0. No cap raise.

This is focused demo-candidate verification, not a newly passing full
check.sh or a recorded physical modem-cut scene. `build.py` builds only
the Mac ARM64 and Fedora x86_64 native downloads plus the canonical page
from one recorded committed source, with independent cloned build caches.
It records build exits and checksums; no artifacts are publicly deployed.
The installer retains its checksum verification. Per-ring lending, native
home development preview and Jellyfin SSO are outside this demo cut.
