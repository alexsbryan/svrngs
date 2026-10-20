# Vendored: canon, as plain JavaScript

House rules decides nothing itself: every answer is canon-core's. This is
canon-core at canon release 0.5.0, built to wasm and through binaryen's
`wasm2js` to one JavaScript module, so it runs under a carried page's
policy, which grants no `'wasm-unsafe-eval'` (svrn-docs
`rings/current/WHAT_RUNS_AS_YOU.md` §6). Nothing is fetched at run time.

| | |
|---|---|
| canon | `1b178c6840c68ee92d6c20c67261db8342decbbf`, release 0.5.0, tag `v0.5.0` (github.com/alexsbryan/canon) |
| built by | `apps/canon/build.sh --vendor`, from `apps/canon/build/` |
| with | rustc 1.95.0, `wasm32-unknown-unknown`; binaryen 133 (`wasm-opt`'s lowerings, then `wasm2js -O`) |
| proven | canon's 18 fixture replays, every step equal across canon-cli, the wasm and this file; canon's governance bar and canon-core's tests, its share rule's among them, all at that commit (`apps/canon/build.sh`; first proven at 0.3.0 in `demo/runs/canon/2026-10-16/`) |

- `canon.js` is the module `wasm2js` wrote, unchanged.
- `call.mjs` is the glue, `bind(exports)` to `call(request)`, copied from
  `apps/canon/build/call.mjs`.
- `LICENSE` is canon's, the GNU AGPL 3.0.

`canon.test.mjs` checks each file against its sha256 below, and fails on one
byte changed:

| file | sha256 |
|---|---|
| `canon.js` | `e66ce0ebd8729eb46a0d5bb1d53618df01bcfb843940a25bcc220abc8d6778d6` |
| `call.mjs` | `b869be78c4b4682151e640680c32ac8c95b1cab3ad46e2e72fb92c2a96cc7952` |
| `LICENSE` | `0d96a4ff68ad6d4b6f1f30f713b18d5184912ba8dd389f86aa7710db079abcb0` |

To build it again and check it is these bytes:

    apps/canon/build.sh --vendor
    git diff --exit-code apps/canon/vendor
