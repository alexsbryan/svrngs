svrnmesh: bridged 1 legacy SOVEREIGN_* env var(s) to SVRNMESH_* (the SOVEREIGN_* prefix is deprecated — update your scripts)
dry_report: loaded 658 distinct code symbols (of 1554) in 0.1s
dry_report: exact tier found 1 clone group(s) from source (SCIP, un-deduped)
dry_report: near pass starting — O(n²) over 658 reps on 12 threads; minutes-scale, longer under CPU load
dry_report: near pass 65/658 rows (0s elapsed)
dry_report: near pass 130/658 rows (0s elapsed)
dry_report: near pass 195/658 rows (0s elapsed)
dry_report: near pass 260/658 rows (0s elapsed)
dry_report: near pass 325/658 rows (0s elapsed)
dry_report: near pass 390/658 rows (0s elapsed)
dry_report: near pass 455/658 rows (0s elapsed)
dry_report: near pass 520/658 rows (0s elapsed)
dry_report: near pass 585/658 rows (0s elapsed)
dry_report: near pass 650/658 rows (0s elapsed)
dry_report: near pass over 658 reps → 8 edges in 0.2s (12 threads)
# DRY report — `cap-econ-corpus`

Scope: `crates/svrngs-node` — 59 file(s) under `crates/svrngs-node/`, matched on whole path components.

1554 function/method symbols · 658 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**1** exact-clone groups · **8** near-clone clusters · ~**157** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **2 copies × 3 lines** (`451dec26b49c`)
  - `addr` — crates/svrngs-node/src/api.rs:212
  - `addr` — crates/svrngs-node/src/api.rs:228

## Near clones (similar, not identical)

- **2 symbols** · ~42 lines · cosine 0.901–0.901
  - `lends` — crates/svrngs-node/src/door.rs:508–536 (29 lines)
  - `lends` — crates/svrngs-node/src/surface.rs:239–280 (42 lines)
- **2 symbols** · ~25 lines · cosine 0.941–0.941
  - `running` — crates/svrngs-node/src/home/home.js:185–209 (25 lines)
  - `look` — crates/svrngs-node/src/home/home.js:188–208 (21 lines)
- **2 symbols** · ~25 lines · cosine 0.914–0.914
  - `lent` — crates/svrngs-node/src/home.rs:227–249 (23 lines)
  - `change` — crates/svrngs-node/src/home.rs:246–270 (25 lines)
- **2 symbols** · ~18 lines · cosine 0.909–0.909
  - `kept_files` — crates/svrngs-node/src/synced.rs:334–351 (18 lines)
  - pub `kept_at` — crates/svrngs-node/src/synced.rs:348–361 (14 lines)
- **2 symbols** · ~13 lines · cosine 0.910–0.910
  - `load` — crates/svrngs-node/src/door/choices.rs:82–94 (13 lines)
  - `read` — crates/svrngs-node/src/door/choices.rs:91–102 (12 lines)
- **2 symbols** · ~13 lines · cosine 0.908–0.908
  - pub `live_push` — crates/svrngs-node/src/rail.rs:607–619 (13 lines)
  - pub `live_drain` — crates/svrngs-node/src/rail.rs:616–627 (12 lines)
- **2 symbols** · ~9 lines · cosine 0.934–0.934
  - `admit` — crates/svrngs-node/src/ring_sync/tests.rs:85–93 (9 lines)
  - `ingest` — crates/svrngs-node/src/ring_sync/tests.rs:88–96 (9 lines)
- **2 symbols** · ~9 lines · cosine 0.912–0.912
  - pub `from_mbit` — crates/svrngs-node/src/lend.rs:155–163 (9 lines)
  - pub `bps` — crates/svrngs-node/src/lend.rs:159–167 (9 lines)


