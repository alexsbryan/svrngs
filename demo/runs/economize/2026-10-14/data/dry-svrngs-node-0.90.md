# DRY report — `prod-43c0552`

Scope: `crates/svrngs-node` — 45 file(s) under `crates/svrngs-node/`, matched on whole path components.

1873 function/method symbols · 531 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**1** exact-clone groups · **7** near-clone clusters · ~**99** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **2 copies × 3 lines** (`451dec26b49c`)
  - `addr` — crates/svrngs-node/src/api.rs:212
  - `addr` — crates/svrngs-node/src/api.rs:228

## Near clones (similar, not identical)

- **2 symbols** · ~20 lines · cosine 0.924–0.924
  - pub `of` — crates/svrngs-node/src/door/sign_in.rs:336–355 (20 lines)
  - pub `forget` — crates/svrngs-node/src/door/sign_in.rs:353–368 (16 lines)
- **2 symbols** · ~18 lines · cosine 0.909–0.909
  - `kept_files` — crates/svrngs-node/src/synced.rs:336–353 (18 lines)
  - pub `kept_at` — crates/svrngs-node/src/synced.rs:350–363 (14 lines)
- **2 symbols** · ~14 lines · cosine 0.914–0.914
  - pub `router` — crates/svrngs-node/src/redeem.rs:124–135 (12 lines)
  - `ask_route` — crates/svrngs-node/src/redeem.rs:131–144 (14 lines)
- **2 symbols** · ~13 lines · cosine 0.910–0.910
  - `load` — crates/svrngs-node/src/door/choices.rs:79–91 (13 lines)
  - `read` — crates/svrngs-node/src/door/choices.rs:88–99 (12 lines)
- **2 symbols** · ~13 lines · cosine 0.908–0.908
  - pub `live_push` — crates/svrngs-node/src/rail.rs:617–629 (13 lines)
  - pub `live_drain` — crates/svrngs-node/src/rail.rs:626–637 (12 lines)
- **2 symbols** · ~9 lines · cosine 0.934–0.934
  - `admit` — crates/svrngs-node/src/ring_sync/tests.rs:85–93 (9 lines)
  - `ingest` — crates/svrngs-node/src/ring_sync/tests.rs:88–96 (9 lines)
- **2 symbols** · ~9 lines · cosine 0.912–0.912
  - pub `from_mbit` — crates/svrngs-node/src/lend.rs:153–161 (9 lines)
  - pub `bps` — crates/svrngs-node/src/lend.rs:157–165 (9 lines)


