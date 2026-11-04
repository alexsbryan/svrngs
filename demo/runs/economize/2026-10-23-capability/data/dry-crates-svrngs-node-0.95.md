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
dry_report: near pass over 658 reps → 0 edges in 0.1s (12 threads)
# DRY report — `cap-econ-corpus`

Scope: `crates/svrngs-node` — 59 file(s) under `crates/svrngs-node/`, matched on whole path components.

1554 function/method symbols · 658 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.95

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**1** exact-clone groups · **0** near-clone clusters · ~**3** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **2 copies × 3 lines** (`451dec26b49c`)
  - `addr` — crates/svrngs-node/src/api.rs:212
  - `addr` — crates/svrngs-node/src/api.rs:228


