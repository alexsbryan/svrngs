dry_report: loaded 15 distinct code symbols (of 285) in 0.1s
dry_report: exact tier found 0 clone group(s) from source (SCIP, un-deduped)
dry_report: near pass starting — O(n²) over 15 reps on 12 threads; minutes-scale, longer under CPU load
dry_report: near pass over 15 reps → 1 edges in 0.0s (12 threads)
# DRY report — `ps-e3fc4f62`

Scope: `web/v2` — 11 file(s) under `web/v2/`, matched on whole path components.

285 function/method symbols · 15 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.90

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

exact-clone groups **could-not-judge** · **1** near-clone clusters · ~**11** redundant lines (lower bound).

- exact tier **could-not-judge**: no SCIP function symbols in scope — the SCIP export for its language may have failed (`svrn project refresh` names a skipped one)

## Near clones (similar, not identical)

- **2 symbols** · ~11 lines · cosine 0.940–0.940
  - `stay` — web/v2/visit.js:42–49 (8 lines)
  - `visiting` — web/v2/visit.js:44–54 (11 lines)


