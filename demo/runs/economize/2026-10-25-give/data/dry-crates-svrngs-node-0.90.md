# DRY report — `give-econ-corpus`

Scope: `crates/svrngs-node` — 59 file(s) under `crates/svrngs-node/`, matched on whole path components.

1640 function/method symbols · 681 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**1** exact-clone groups · **8** near-clone clusters · ~**170** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **2 copies × 3 lines** (`451dec26b49c`)
  - `addr` — crates/svrngs-node/src/api.rs:212
  - `addr` — crates/svrngs-node/src/api.rs:228

## Near clones (similar, not identical)

- **2 symbols** · ~42 lines · cosine 0.901–0.901
  - `lends` — crates/svrngs-node/src/door.rs:513–541 (29 lines)
  - `lends` — crates/svrngs-node/src/surface.rs:242–283 (42 lines)
- **2 symbols** · ~25 lines · cosine 0.941–0.941
  - `running` — crates/svrngs-node/src/home/home.js:203–227 (25 lines)
  - `look` — crates/svrngs-node/src/home/home.js:206–226 (21 lines)
- **2 symbols** · ~25 lines · cosine 0.914–0.914
  - `lent` — crates/svrngs-node/src/home.rs:275–297 (23 lines)
  - `change` — crates/svrngs-node/src/home.rs:294–318 (25 lines)
- **3 symbols** · ~13 lines · cosine 0.903–0.905
  - `load` — crates/svrngs-node/src/door/choices.rs:89–101 (13 lines)
  - `file` — crates/svrngs-node/src/door/choices.rs:98–106 (9 lines)
  - `read` — crates/svrngs-node/src/door/choices.rs:103–114 (12 lines)
- **2 symbols** · ~18 lines · cosine 0.909–0.909
  - `kept_files` — crates/svrngs-node/src/synced.rs:334–351 (18 lines)
  - pub `kept_at` — crates/svrngs-node/src/synced.rs:348–361 (14 lines)
- **2 symbols** · ~13 lines · cosine 0.908–0.908
  - pub `live_push` — crates/svrngs-node/src/rail.rs:666–678 (13 lines)
  - pub `live_drain` — crates/svrngs-node/src/rail.rs:675–686 (12 lines)
- **2 symbols** · ~9 lines · cosine 0.934–0.934
  - `admit` — crates/svrngs-node/src/ring_sync/tests.rs:85–93 (9 lines)
  - `ingest` — crates/svrngs-node/src/ring_sync/tests.rs:88–96 (9 lines)
- **2 symbols** · ~9 lines · cosine 0.912–0.912
  - pub `from_mbit` — crates/svrngs-node/src/lend.rs:155–163 (9 lines)
  - pub `bps` — crates/svrngs-node/src/lend.rs:159–167 (9 lines)


