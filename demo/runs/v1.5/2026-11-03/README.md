# V1.5 release, 3 November 2026

Prepared after the JavaScript queue completed at 25056eec. Publication and
artifact verification are recorded below once they have run.

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
