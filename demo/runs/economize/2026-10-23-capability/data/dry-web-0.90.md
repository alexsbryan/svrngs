svrnmesh: bridged 1 legacy SOVEREIGN_* env var(s) to SVRNMESH_* (the SOVEREIGN_* prefix is deprecated — update your scripts)
dry_report: loaded 113 distinct code symbols (of 1554) in 0.1s
dry_report: exact tier found 0 clone group(s) from source (SCIP, un-deduped)
dry_report: near pass starting — O(n²) over 113 reps on 12 threads; minutes-scale, longer under CPU load
dry_report: near pass 11/113 rows (0s elapsed)
dry_report: near pass 22/113 rows (0s elapsed)
dry_report: near pass 33/113 rows (0s elapsed)
dry_report: near pass 44/113 rows (0s elapsed)
dry_report: near pass 55/113 rows (0s elapsed)
dry_report: near pass 66/113 rows (0s elapsed)
dry_report: near pass 77/113 rows (0s elapsed)
dry_report: near pass 88/113 rows (0s elapsed)
dry_report: near pass 99/113 rows (0s elapsed)
dry_report: near pass 110/113 rows (0s elapsed)
dry_report: near pass over 113 reps → 2 edges in 0.0s (12 threads)
# DRY report — `cap-econ-corpus`

Scope: `web` — 8 file(s) under `web/`, matched on whole path components.

1554 function/method symbols · 113 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

exact-clone groups **could-not-judge** · **2** near-clone clusters · ~**32** redundant lines (lower bound).

- exact tier **could-not-judge**: no SCIP function symbols in scope — the SCIP export for its language may have failed (`svrn project refresh` names a skipped one)

## Near clones (similar, not identical)

- **2 symbols** · ~23 lines · cosine 0.920–0.920
  - `uses` — web/app.js:324–346 (23 lines)
  - `choicesOf` — web/app.js:327–342 (16 lines)
- **2 symbols** · ~9 lines · cosine 0.900–0.900
  - `cookieFor` — web/lend.js:174–181 (8 lines)
  - `keepJar` — web/lend.js:177–185 (9 lines)


