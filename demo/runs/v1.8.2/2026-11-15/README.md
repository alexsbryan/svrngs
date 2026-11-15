# V1.8.2 release, 15 November 2026

Released at the operator's word, 2026-11-15: "Push and you can release
now right?"

## Scope and source evidence

The release before this one is V1.8.1 at 83f1cdab; its readback is
`../../v1.8.1/2026-11-14/README.md`. Before this run, `group.svrnme.sh`
was still serving V1.8.1's page: at 2026-11-15 23:56 UTC its `sw.js`
named version `7a84f5701243`, V1.8.1's own. Since `v1.8.1`, the shipped
files that changed are the home page's `acting.js`, the node's
`network.rs` and `room` (the `svrngs` program), and the page's
`authority.js`, `open.js`, `sign.js` and `lend.js`. All other changes are
tests, the gate image and records.

| Release note | Source or run |
|---|---|
| "What you lend" and "What acts as you" show beside groups made before V1.8 | 0129b0d9, `crates/svrngs-node/src/home/page/acting.js` (it no longer waits on a group an older svrngs made); `web/tests/acting.test.mjs`. c25282b1 moved `awaited` beforehand, keeping behaviour |
| A lend opened to the WiFi works with a VPN carrying all traffic; a café's WiFi still reads as the café's | 250aa65b, `crates/svrngs-node/src/network.rs` (the network read is the one beside this computer, not the VPN's route); `network_tests.rs`, `tests/room.rs`. b9093fe9 reads no network when no lend is on |
| The VPN line | 250aa65b, `crates/svrngs-node/src/room.rs` `Served::said_beside`, its wording verbatim; `room_tests.rs` |
| Opening an app works every time in Linux's WebKit browsers | 3da19985, `web/authority.js` `ed25519`, which imports seeds as JWK and not PKCS #8; `demo/runs/webkit-ed25519/2026-11-15/` (Linux's WebKit refused 200 of 200 seeds beginning with 0 as PKCS #8 and took 300 of 300 as JWK); `web/tests/authority.test.mjs` |
| Nothing else changes | The other commits since `v1.8.1` change tests, the gate image and records only: 9a992d5c, af7bd697, 530a1ead, ceb129b2, 953f5f42, bb46c17c, 7d817c69, 6829f24f, b7aece62 |

## Validation before packaging

`scripts/check.sh` ran whole at 3da19985 in the Linux gate image
(`scripts/gate/README.md`, Fedora host, 32 cores): exit 0, every step
passed, 1,217 s (`gate-3da19985-linux.txt`). This was the first whole
Linux run to pass. The causes of the earlier runs' failures are in the
bodies of bb46c17c, 7d817c69, 6829f24f, b7aece62 and 3da19985.

To run before publishing, on the Mac: the whole gate at the release
commit. 3da19985 makes the page import Ed25519 seeds as JWK, which the
Mac's WebKit has not yet run. Then the four targets
`scripts/build-release.sh` builds; this Linux machine has neither Apple
target.

## Publication

To do: `publish.py <clean worktree at v1.8.2> <this directory>`, as for
V1.8.1.

## Readback

To do: `verify.py`, which is V1.8.1's with `PREVIOUS_VERSION` set to
V1.8.1's `7a84f5701243`.
