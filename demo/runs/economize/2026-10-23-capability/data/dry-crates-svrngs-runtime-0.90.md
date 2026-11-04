svrnmesh: bridged 1 legacy SOVEREIGN_* env var(s) to SVRNMESH_* (the SOVEREIGN_* prefix is deprecated — update your scripts)
dry_report: loaded 128 distinct code symbols (of 1554) in 0.1s
dry_report: exact tier found 1 clone group(s) from source (SCIP, un-deduped)
dry_report: near pass starting — O(n²) over 128 reps on 12 threads; minutes-scale, longer under CPU load
dry_report: near pass 12/128 rows (0s elapsed)
dry_report: near pass 24/128 rows (0s elapsed)
dry_report: near pass 36/128 rows (0s elapsed)
dry_report: near pass 48/128 rows (0s elapsed)
dry_report: near pass 60/128 rows (0s elapsed)
dry_report: near pass 72/128 rows (0s elapsed)
dry_report: near pass 84/128 rows (0s elapsed)
dry_report: near pass 96/128 rows (0s elapsed)
dry_report: near pass 108/128 rows (0s elapsed)
dry_report: near pass 120/128 rows (0s elapsed)
dry_report: near pass over 128 reps → 0 edges in 0.0s (12 threads)
# DRY report — `cap-econ-corpus`

Scope: `crates/svrngs-runtime` — 10 file(s) under `crates/svrngs-runtime/`, matched on whole path components.

1554 function/method symbols · 128 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**1** exact-clone groups · **0** near-clone clusters · ~**3** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **2 copies × 3 lines** (`67190d4545a0`)
  - `open` — crates/svrngs-runtime/src/tests.rs:19
  - `open` — crates/svrngs-runtime/src/tests_write_index.rs:12


