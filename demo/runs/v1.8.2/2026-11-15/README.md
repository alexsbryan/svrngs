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

On the Mac, `scripts/check.sh` then ran whole at d2510734, the release
commit, under the gate lock, so the Mac's WebKit ran 3da19985's JWK
import too. It passed: exit 0, 45 steps, 1,268 s (`gate-d2510734.txt`).
The run before it failed `tests` alone (`gate-d2510734-podman-down.txt`).
give's one real-Podman test, `the_stand_in_keeps_its_state_across_a_restore`
(`apps/give/tests/check.rs`), said never-ran: the Mac had rebooted and
Podman's VM was down ("unable to connect to Podman socket"). After
`podman machine start`, `cargo test -p give --test check` passed 7 of 7,
and the whole gate was run again. Every other step of the failed run
passed.

`rustup target list --installed --toolchain 1.95.0` lists all four
targets `scripts/build-release.sh` builds.

## Publication

`publish.py` ran `scripts/deploy-page.sh` under the gate lock in a clean
detached worktree at d2510734 (`v1.8.2`), from 2026-11-16 00:46 to 00:53
UTC, exit 0 (`publish.json`, `publish.log`). Load was about 3.4, with 96
GiB free. The record was written outside the worktree and copied here
after. The build meets its groups at n0's relays (`SHIPPED`), so no relay
was asked. The page built as version e022fd11ba8f, and `svrngs` and `give`
built for all four platforms. Vercel deployment
`dpl_HkQngztQNcB1GiL9Qk5wr7MbGTL9` reached production as Ready, aliased to
`https://group.svrnme.sh` and `https://*.group.svrnme.sh` (`vercel inspect
svrngs-page-haxudhzsu-svrnmesh.vercel.app --scope svrnmesh`). The CLI
wrote `.vercel/`, `.env.local` and `.gitignore` into the upload
directory. All three answered 404 live and were deleted after.

## Readback

`verify.py` passed (`verification.json`). Its six planted faults were
rejected first. Then:

- 78 public files fetched from `group.svrnme.sh` each equaled the built
  site's bytes.
- The live `get/SHA256SUMS` equaled the built one (`SHA256SUMS`,
  `live-sums.txt`) and named exactly the eight downloads.
- The live page is version e022fd11ba8f, not V1.8.1's 7a84f5701243, and
  the live `sw.js` names it and sends a navigation to the network first.
- The live `install.sh` equals the built one.
- Both frame policies were as `vercel.json` sets them.
- `release-d2510734.group.svrnme.sh` served the same page.
- `.env.local`, `.vercel/project.json` and `vercel.json` answered 404.
- The downloaded darwin-aarch64 `svrngs` and `give` started.

No other platform's program was run here.
