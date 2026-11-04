svrnmesh: bridged 1 legacy SOVEREIGN_* env var(s) to SVRNMESH_* (the SOVEREIGN_* prefix is deprecated — update your scripts)
dry_report: loaded 480 distinct code symbols (of 1554) in 0.3s
dry_report: exact tier found 4 clone group(s) from source (SCIP, un-deduped)
dry_report: near pass starting — O(n²) over 480 reps on 12 threads; minutes-scale, longer under CPU load
dry_report: near pass 48/480 rows (0s elapsed)
dry_report: near pass 96/480 rows (0s elapsed)
dry_report: near pass 144/480 rows (0s elapsed)
dry_report: near pass 192/480 rows (0s elapsed)
dry_report: near pass 240/480 rows (0s elapsed)
dry_report: near pass 288/480 rows (0s elapsed)
dry_report: near pass 336/480 rows (0s elapsed)
dry_report: near pass 384/480 rows (0s elapsed)
dry_report: near pass 432/480 rows (0s elapsed)
dry_report: near pass 480/480 rows (0s elapsed)
dry_report: near pass over 480 reps → 0 edges in 0.1s (12 threads)
# DRY report — `cap-econ-corpus`

Scope: `crates/svrngs-core` — 43 file(s) under `crates/svrngs-core/`, matched on whole path components.

1554 function/method symbols · 480 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.95

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

**4** exact-clone groups · **0** near-clone clusters · ~**12** redundant lines (lower bound).

## Exact clones (identical bodies, modulo formatting)

- **2 copies × 3 lines** (`7e4a51e8197d`)
  - `signed_by_link` — crates/svrngs-core/src/joining.rs:56
  - `signed_by_link` — crates/svrngs-core/src/joining.rs:148
- **2 copies × 3 lines** (`f6a912a9fe34`)
  - `lines` — crates/svrngs-core/src/command.rs:87
  - `lines` — crates/svrngs-core/src/command.rs:105
- **2 copies × 3 lines** (`a588a00c526a`)
  - `as_str` — crates/svrngs-core/src/link.rs:186
  - `as_str` — crates/svrngs-core/src/synced.rs:67
- **2 copies × 3 lines** (`7d516fc1b939`)
  - `is_empty` — crates/svrngs-core/src/act_set.rs:40
  - `is_empty` — crates/svrngs-core/src/command.rs:109


