dry_report: loaded 73 distinct code symbols (of 285) in 0.1s
dry_report: exact tier found 0 clone group(s) from source (SCIP, un-deduped)
dry_report: near pass starting — O(n²) over 73 reps on 12 threads; minutes-scale, longer under CPU load
dry_report: near pass over 73 reps → 2 edges in 0.1s (12 threads)
# DRY report — `ps-e3fc4f62`

Scope: `crates/svrngs-node/src/home` — 31 file(s) under `crates/svrngs-node/src/home/`, matched on whole path components.

285 function/method symbols · 73 distinct locations considered (≥8 lines) · near-clone threshold cosine ≥ 0.95

First-party production code only — path segments excluded: `vendor`, `node_modules`, `.cargo-container`, `research`, `external`, `target`, `.claude`, `tests`, `benches`, `examples`, `build.rs`.

exact-clone groups **could-not-judge** · **2** near-clone clusters · ~**19** redundant lines (lower bound).

- exact tier **could-not-judge**: no SCIP function symbols in scope — the SCIP export for its language may have failed (`svrn project refresh` names a skipped one)

## Near clones (similar, not identical)

- **2 symbols** · ~11 lines · cosine 0.951–0.951
  - `Found` — crates/svrngs-node/src/home/v2/running.js:33–43 (11 lines)
  - `Running` — crates/svrngs-node/src/home/v2/running.js:39–47 (9 lines)
- **2 symbols** · ~8 lines · cosine 0.962–0.962
  - pub `Group` — crates/svrngs-node/src/home/v2/group.js:39–46 (8 lines)
  - `Read` — crates/svrngs-node/src/home/v2/group.js:41–48 (8 lines)


