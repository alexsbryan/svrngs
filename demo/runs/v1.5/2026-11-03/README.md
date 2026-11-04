# V1.5 release, 3 November 2026

Published after the JavaScript queue completed at 25056eec, from source
**5a4cb567b1d504be0cdf80382c4f3fb8d809ce84**, tagged **v1.5** locally.
The page and all eight downloads are live at https://group.svrnme.sh/.

## Scope and source evidence

The preceding release is v1.4 at 6dafd336. This release includes:

| Release note | Source or run |
|---|---|
| Multiple group meeting points and tiered fallback | 105df0bf, 70cb543b; `demo/runs/relay-fallbacks/2026-11-02/README.md` |
| A join starts with the invitation's meeting points | a40ab3ea, f15002be |
| New two-meeting-point invitations require the new build | `crates/svrngs-core/src/link.rs`, `Format::Met` |
| Older builds retain their previous meeting point | `crates/svrngs-core/src/lib.rs`, `MEETS_LINE_VERSION` |
| A made or joined group stays visibly loading | e1ca7cb2, e1d3615a |
| A refused Stop appears beside the control | dd7a6b60 |
| Split WebSocket headers, two-byte lengths and DataView | cc24f9ae, 00f75721, adef516a |
| Readable, typed, structurally linted page source | `ralph/next/js/STATE.md`, 7/7 done; focused review 970a835c |

The relay recovery measurements are a one-host run, not a rate or a claim
about venue WiFi. The default production meeting points remain n0's
(`cli::SHIPPED`); this release does not switch the production relay service.

## Validation before packaging

`scripts/check.sh` exited 0 at 8e00b6e8 in 1,384 s. Runtime files, tests,
build/serve wiring and check configuration are identical to that tested
tree, verified again at the completed queue's HEAD. The later executable
delta is the budget jubilee, whose +1 JavaScript plant failed before its
removal restored a pass. See
`demo/runs/economize/2026-11-03-js-read/CLOSE.md`.

The full gate's doc scene was could-not-judge under load. Its separate
locked rerun on main passed 1 of 1 in 32.56 s. At the focused review,
106 page-law tests, formatting, lint, types and typed lint passed. Deferred
semantic-clone and cross-repository noun checks are not called passed.

The release notes and this record change no product code. Packaging must
build svrngs and Give for macOS ARM/Intel and static Linux x86_64/aarch64,
then publish them beside the same page build and SHA256SUMS.

## Publication

`publish.py` ran the existing `scripts/deploy-page.sh` under the gate lock
from a clean detached worktree at 5a4cb567. All four target builds passed
for both svrngs and Give. `file` identifies the two Mac pairs as Mach-O
ARM64/x86_64 and both Linux pairs as statically linked ELF executables.
The Linux builds warn that musl drops give-fold's `cdylib`; the Rust
library and both shipped programs build successfully.

The final controlled invocation exited 0 in 43 seconds, reusing the
completed release builds. Its actual source, exit, times and upload path
are in `publish.json`, with the complete output in `publish.log`.
It finished at 2026-11-04 00:11:46 UTC (3 November, Pacific time).

- Deployment: **dpl_E58G4keystRG3ubFZMZFcQp5M2zz**.
- Immutable address: https://svrngs-page-p5lzijv91-svrnmesh.vercel.app/.
- Production aliases: `group.svrnme.sh` and `*.group.svrnme.sh`.
- Vercel inspection: production, Ready, created 3 November 16:11:42 PST.
- Web content version: **6a3a8c04fa55**.

The initial `launchctl submit` job unexpectedly relaunched after successful
returns (`LastExitStatus = 0`, a new PID, and repeated Ready deployments).
That submission was removed and no publishing process remained. Its
interrupted repeat is retained as `restarted-run.log` and
`restarted-run.json`; it is not called a failed build. Because the helper
opens its log anew, the first cold-build log was replaced by that repeat.
The complete final foreground run above is the publication receipt. Do not
use the legacy submission form as a one-shot publisher.

## Readback and startup checks

```text
$ python3 demo/runs/v1.5/2026-11-03/verify.py <published-site> <this-directory>
Instrument plants rejected: corrupt bytes, wrong digest, widened policy, exposed hidden file.
Release verification: passed; 78 public files checked.
exit 0
```

`verification.json` records the actual checks:

- Live `get/SHA256SUMS` equals the built file byte for byte, preserved here
  as `SHA256SUMS` and `live-sums.txt`.
- All **eight archives** are present and their fetched SHA-256 hashes match
  the manifest and built bytes.
- Every non-hidden public file in the upload, **78 total**, matches its
  build, including the page, all JavaScript modules, workers and wasm.
- Root policy is `frame-ancestors 'none'`; `sign.html` is
  `frame-ancestors https:`, as the source configuration specifies.
- `release-5a4cb567.group.svrnme.sh` serves the built index and root policy.
- `.env.local`, `.vercel/project.json` and `vercel.json` each return 404.
- The fetched Mac ARM svrngs archive starts with `--help`, exit 0, and
  includes the new repeated-relay guidance. The fetched Give archive starts
  and prints its usage with the expected no-command exit 2. Both run with a
  temporary HOME and data directory, not the operator's groups.

The comparator's four planted bad inputs were rejected before readback.
These checks do not claim runtime tests of the other CPU/OS binaries or a
new physical-phone run. Their builds and the prior runtime gate remain
separate evidence.

The Git tag is local. No Git push or new GitHub release was performed;
this follows the existing page/download publication procedure.

After successful readback, the clean detached worktree and the known
initial, interrupted-repeat and final upload directories were removed.
The source tag, manifest, verification results and logs remain here.
