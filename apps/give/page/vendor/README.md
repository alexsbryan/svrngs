# Vendored: give's fold, as plain JavaScript

The page decides nothing about who keeps an app: every answer comes from the
fold in `apps/give/fold`, which the keeper program links too. This is that
fold, built to wasm and then through binaryen's `wasm2js` to one JavaScript
module, so it runs under a carried page's policy, which grants no
`'wasm-unsafe-eval'` (`scripts/trust_floor.txt`). Nothing is fetched at run
time.

| | |
|---|---|
| built by | `apps/give/build.sh --vendor` |
| with | rustc 1.95.0, `wasm32-unknown-unknown`, opt-level z, LTO; binaryen 133 (`wasm-opt`'s lowerings, then `wasm2js -O`) |
| proven | every case in `apps/give/fold/cases.json`, answered alike by the wasm and this file (`apps/give/fold.test.mjs`, which `build.sh` runs on each build) |

- `fold.js` is the module `wasm2js` wrote, unchanged.
- `call.mjs` is the glue, `bind(exports)` to `fold(ops)`.

`apps/give/fold.test.mjs` checks each file against its sha256 below, and the
fold's source against the sha256s it was built from, so a change to the
fold that is not built again fails the gate:

| file | sha256 |
|---|---|
| `fold.js` | `4959762fac5f25e9efdd513a77918d574fedab8d9551d611f6ca29e400cd3d66` |
| `call.mjs` | `33c2643acf58911ff16a03b760022165b4b4de984988451c3bcd229ff42de713` |
| `../../fold/src/lib.rs` | `3090c527b2f612f4b24fae5cbc275d64593880e75ad97766da24403c9663ebf0` |
| `../../fold/Cargo.toml` | `b2819db50e7a6207321f305d6e0472828a85cb457de9cea5359a893ddb065ccf` |

To build it again and check it is these bytes:

    apps/give/build.sh --vendor
    git diff --exit-code apps/give/page/vendor
