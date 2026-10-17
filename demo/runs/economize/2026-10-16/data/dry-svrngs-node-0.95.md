# DRY report — `prod-0a2e6b3a`

Scope: `crates/svrngs-node` — 52 file(s) under `crates/svrngs-node/`, matched on whole path components.

1927 function/method symbols · 588 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.95

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**3** exact-clone groups · **2** near-clone clusters · ~**50** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **3 copies × 3 lines** (`e9e991b4866d`)
  - `flush` — crates/svrngs-node/src/blob_tests.rs:161
  - `flush` — crates/svrngs-node/src/guest/tests.rs:58
  - `flush` — crates/svrngs-node/src/relay_tests.rs:160
- **2 copies × 4 lines** (`ab90a91d6176`)
  - `write` — crates/svrngs-node/src/guest/tests.rs:54
  - `write` — crates/svrngs-node/src/relay_tests.rs:156
- **2 copies × 3 lines** (`451dec26b49c`)
  - `addr` — crates/svrngs-node/src/api.rs:212
  - `addr` — crates/svrngs-node/src/api.rs:228

## Near clones (similar, not identical)

- **2 symbols** · ~27 lines · cosine 0.962–0.962
  - `running` — crates/svrngs-node/src/home/home.js:142–168 (27 lines)
  - `look` — crates/svrngs-node/src/home/home.js:145–167 (23 lines)
- **2 symbols** · ~10 lines · cosine 1.000–1.000
  - `write` — crates/svrngs-node/src/guest/tests.rs:51–60 (10 lines)
  - `write` — crates/svrngs-node/src/relay_tests.rs:153–162 (10 lines)


