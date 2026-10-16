# DRY report — `prod-0a2e6b3a`

Scope: `crates/svrngs-node` — 52 file(s) under `crates/svrngs-node/`, matched on whole path components.

1927 function/method symbols · 588 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**3** exact-clone groups · **11** near-clone clusters · ~**173** redundant lines (lower bound).

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
- **2 symbols** · ~20 lines · cosine 0.924–0.924
  - pub `of` — crates/svrngs-node/src/door/sign_in.rs:336–355 (20 lines)
  - pub `forget` — crates/svrngs-node/src/door/sign_in.rs:353–368 (16 lines)
- **2 symbols** · ~18 lines · cosine 0.909–0.909
  - `kept_files` — crates/svrngs-node/src/synced.rs:334–351 (18 lines)
  - pub `kept_at` — crates/svrngs-node/src/synced.rs:348–361 (14 lines)
- **2 symbols** · ~15 lines · cosine 0.917–0.917
  - pub `door` — crates/svrngs-node/src/home/question.js:108–122 (15 lines)
  - `judge` — crates/svrngs-node/src/home/question.js:110–120 (11 lines)
- **2 symbols** · ~14 lines · cosine 0.914–0.914
  - pub `router` — crates/svrngs-node/src/redeem.rs:114–125 (12 lines)
  - `ask_route` — crates/svrngs-node/src/redeem.rs:121–134 (14 lines)
- **2 symbols** · ~13 lines · cosine 0.910–0.910
  - `load` — crates/svrngs-node/src/door/choices.rs:82–94 (13 lines)
  - `read` — crates/svrngs-node/src/door/choices.rs:91–102 (12 lines)
- **2 symbols** · ~13 lines · cosine 0.908–0.908
  - pub `live_push` — crates/svrngs-node/src/rail.rs:598–610 (13 lines)
  - pub `live_drain` — crates/svrngs-node/src/rail.rs:607–618 (12 lines)
- **2 symbols** · ~12 lines · cosine 0.928–0.928
  - pub `helper_of` — crates/svrngs-node/src/share.rs:180–189 (10 lines)
  - pub `signs_in` — crates/svrngs-node/src/share.rs:189–200 (12 lines)
- **2 symbols** · ~10 lines · cosine 1.000–1.000
  - `write` — crates/svrngs-node/src/guest/tests.rs:51–60 (10 lines)
  - `write` — crates/svrngs-node/src/relay_tests.rs:153–162 (10 lines)
- **2 symbols** · ~9 lines · cosine 0.934–0.934
  - `admit` — crates/svrngs-node/src/ring_sync/tests.rs:85–93 (9 lines)
  - `ingest` — crates/svrngs-node/src/ring_sync/tests.rs:88–96 (9 lines)
- **2 symbols** · ~9 lines · cosine 0.912–0.912
  - pub `from_mbit` — crates/svrngs-node/src/lend.rs:155–163 (9 lines)
  - pub `bps` — crates/svrngs-node/src/lend.rs:159–167 (9 lines)


