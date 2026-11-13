# n0 out of the scenes, and n0's one contained test (2026-11-13)

The operator: "hardening the tests by identifying fragile seams and keeping
95% of the tests on pretty bulletproof codepaths, then save the fragile crap
for all one tests". The gate at 119dd235 failed on the events scene alone,
Sam's and Bo's shipped svrngs losing n0's usw1-1 mid-scene
(../2026-11-12/mac/events-n0-119dd235.log). The decision: the scenes meet at a
relay on 127.0.0.1, and n0 gets one contained test.

Commits: 56d21cee (build-web's stale deploy steps cut, the economizing pass
for the next), f7de7fe6 (the scenes' svrngs as `main` at relays on
127.0.0.1), aa90326d (tests/n0.rs and check.sh's `n0` step). Each body has
its evidence; what follows is the raw record.

## The scenes, sampled

`run.sh` runs check.sh's own scenes and n0 steps (check.sh with
`SVRNGS_STEPS` set and each step's log kept, made by the sed in
../2026-11-12/round.sh, adapted: `RUN_STEPS`, `RUN_OUT`) in a clean detached
worktree, under the gate lock, while `sockets.py` samples `lsof -i` over the
run's whole process tree every 2 s: bdd3bd51's probe, every descendant. Both
runs are of these three commits on 4b99d7f9, before ad25eab2 landed, with
Eudoxia 5067f12, a clean clone.

- `scenes-plain-relay/` (893e8cb8, unpublished): the scenes' build relay
  as_main.rs's own, plain HTTP. 14 scenes passed, 6 failed, each where a
  phone joins: "A TLS error caused the secure connection to fail", then "The
  computer that sent this link did not answer" (creator.rs:155, doc.rs:169,
  events.rs:134, life.rs:269, night.rs:208, holyarrs.rs:210). A link carries
  its issuer's home relay first (svrngs-core relay.rs `carried`); a phone's
  page over TLS reaches a relay over TLS alone, and the home had been n0's,
  https. So the scenes' phones had reached their computers through n0 too.
- `scenes-fronted-relay/` (f9cf7242, unpublished, the trees committed): the
  build relay the browser harness's, behind its TLS front (scenes/main.rs
  `svrngs`). scenes passed in 357 s, all 20; n0 passed in 17 s.
  `sockets.txt`: every svrngs a scene ran (`svrngs_here`) 421 sockets to
  loopback and 13 to this host's LAN address (night's room, a browser
  connecting to it), none outside; 0 of 47 person logs name an n0 host
  (`grep -l 'iroh.link\|n0.computer\|iroh-canary'`). Outside this host only
  Jellyfin's own (20) and Node's browser driver, browsers.mjs, to
  142.251.x:443 (4), neither svrngs; and the n0 step's shipped binary to n0's
  relays (10), as it should. Before: 51 of 51 person logs of the scenes run
  in the main checkout at 00:16 the same day, the shipped binary, name
  iroh.link.
- `sockets-n0-control.tsv`: the sampler over the n0 test alone, the shipped
  binary: 12 sockets outside, so the sampler sees them.

## The n0 test's plants

- `plant-registration.patch`, `plant-registration.log`: cli::reach's N0 arm
  given `https://relay.n0.invalid/`: failed, n0 answering iroh alone, "Sam's
  computer was met at no home relay before its link".
- `blocked.sb`, `plant-n0-blocked.log`: the test binary under `sandbox-exec`,
  every outbound connection but localhost denied: could-not-judge, why
  written to target/could-not-judge/n0.step, the test ok.
- `step-n0.sh`, `step-n0-blocked-then-open.log`: check.sh's own `step` (its
  definitions cut from check.sh) running the n0 step blocked, then open:
  "could-not-judge n0 (0 s): n0 did not answer before any svrngs ran ...",
  failed=0; then "passed n0 (17 s)".

## The full gate

`gate-aa90326d.txt`: `gate.sh`, scripts/check.sh under the gate lock in a
clean detached worktree at aa90326d (main, on ad25eab2), Eudoxia fff11f7 (a
clean clone of that commit, which ad25eab2 builds with), CARGO_INCREMENTAL=0,
2,035 s, **exit 0**: every step passed, nothing could-not-judge, every scene
passed (scenes 423 s), playback 172 s, n0 14 s; WebKit's seeks 3 of 3 within
5 s through a lend.

## The recount

`recount.py` (119dd235's, the n0 target added) over the gate worktree's
binaries, `recount-aa90326d.out`: 956 gate Rust tests, 516 (54.0%) crossing
no seam, 440 at least one: P 432, Q 420, R 206, B 158, C 79, D 10, M 7. Since
c87af561's 954: n0's contained test (R, P, Q) and one deterministic test of
ad25eab2's. The isolated Node laws: 234, all deterministic. The scenes still
cross R, a relay process on 127.0.0.1, so the share does not move; what
moved is that no gate test but n0's own depends on n0.

## Open

- browsers.mjs's connections to 142.251.x:443 (Google) during the scenes:
  not svrngs, not traced to which request here.
- Jellyfin reaches its own servers in films, night, movie_night and
  holyarrs (seam D, a third-party app).
- lean_check's "." cap: 21,185 of 21,187 after these commits; the cap is
  lean_check.py's to lower.

## Harness 4, and the gate at 5cfa2609

Commits: 9787472e (the economizing pass, -24 counted lines), e3e831ac
(the design of harnesses 3 and 4, docs/CHECKS.md), bde74682 (`iroh::Fetch`,
+8; the non-JS cap 21,188 -> 21,170), 34cad974 and 206d5733 (ring_sync's
eight exchange tests over `InProcess`, then their QUIC copies retired),
6eda83b5 (rail's three), 5cfa2609 (record_api's loopback refusal). Each
body names the plants it was watched red on.

`gate-5cfa2609.txt`: `gate.sh`, scripts/check.sh under the gate lock in a
clean detached worktree at 5cfa2609, Eudoxia fff11f7 (a clean clone),
CARGO_INCREMENTAL=0, npm ci --prefix web: 1,720 s, **exit 0**, every step
passed, nothing could-not-judge, scenes 391 s, playback 175 s, n0 14 s.
It ran after another session's gate released the lock.

`recount-5cfa2609.out`: `recount.py` over that worktree's binaries: 956
gate Rust tests, 528 (55.2%) crossing no seam, 428 at least one: P 424,
Q 408, R 206, B 158, C 79, D 10, M 7. Since aa90326d: the eight ring_sync
tests moved (eight retired, eight new with no seam), rail's three and
record_api's one deterministic, 12 in all.

## Harness 3, units 4 and 6 on the wire, and the gate at 1c33d4ff

Commits: 6f9f7239 (iroh.rs `Net`, a node's network half; the non-JS cap
21,241 -> 21,326 within the operator's +90 for harness 3, 2026-11-13,
after the pass 9787472e), 4c288b80 (the in-process `Wire` and folders),
a8c038e2 (carried), f066e8d1 (record_api), 467aa382 (removal), f3868b53
(surface), a5860b42 (the node's and blob's unit tests), 1c33d4ff
(group_not_mesh, canaries). Each body names the plants it was watched
red on.

`gate-1c33d4ff.txt`: `gate.sh` in a clean detached worktree at 1c33d4ff
(main), Eudoxia fff11f7 (a clean clone), CARGO_INCREMENTAL=0: 1,859 s,
**exit 0**, every step passed, nothing could-not-judge.

`recount-1c33d4ff.out` (`recount.py`, unchanged): 957 gate Rust tests,
602 (62.9%) crossing no seam, 355 at least one: P 341, Q 320, R 206,
B 158, C 79, D 10, M 7. At 5cfa2609, 528 (55.2%).

## After seam B's pass

`recount-630967b0.out` (`recount.py`, unchanged, over 630967b0's
binaries): 899 gate Rust tests, 617 (68.6%) crossing no seam, 282 at
least one: P 267, Q 234, R 140, B 90, C 75, D 10, M 7. Browser 43
fragile, pages 3, home 13, home_group 4. The isolated Node laws: 309.
