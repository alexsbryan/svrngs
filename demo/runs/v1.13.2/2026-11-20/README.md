# V1.13.2, friends joining a container's game together

V1.13.2 is main at `9be0fcc7`, the commit to tag `v1.13.2`. It is V1.13.1
(`bd8db5a0`) with one change in what a person sees: a UDP socket read at
an address and port 0, as a server's is while it connects, is read as
connecting (`4a692a95`), so friends joining a rootless Podman game at once
keep its lend open. RELEASES.md says what a person sees.

## Done before the release, on Linux

1. **The whole check.** `scripts/check.sh` on `9be0fcc7` in svrngs-gate
   (scripts/gate/README.md's command), from the plain clone
   `~/dev/svrngs-gate-run` detached at `9be0fcc7`, after `npm ci --prefix
   web`.
   - The first run failed one test of 45 steps' `tests`
     ([gate-9be0fcc7-run1.txt](gate-9be0fcc7-run1.txt)):
     `relay_tiers::a_dial_reaches_a_member_at_a_relay_this_computer_does_not_hold`,
     which docs/seams.tsv calls fragile. Its dial through a real relay on
     127.0.0.1 did not finish its handshake in 10 s (`Err(Elapsed(()))`,
     relay_tiers.rs:84), and Ada's accept saw the handshake aborted
     (relay_tiers.rs:77). `4a692a95` changes no relay or dial code.
   - The second run, unchanged: 45 of 45 steps passed in 1,129 s, exit 0
     ([gate-9be0fcc7.txt](gate-9be0fcc7.txt)).
2. **The relay.** Not deployed again: relay.svrnme.sh runs `bd8db5a0`'s
   build (../../v1.13.1/2026-11-20/deploy-relay.log). `4a692a95` changes
   only how a lender reads the sockets on its own lend's port (uid.rs
   `peer`), which a relay never runs.
