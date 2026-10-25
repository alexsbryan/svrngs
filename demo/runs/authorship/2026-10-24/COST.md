# What a fold costs, before and after authorship by signature

capability-signed's check: "verification's cost per fold measured before
and after" (DESIGN.md §15 beside this file). The instrument is
`crates/svrngs-core/tests/cost.rs`: 8 computers in one group, 250 records
each, 2,000 lines, folded 15 times; the median fold.

## Before, at 9b9201d9

```
$ cargo test --release -p svrngs-core --test cost -- --ignored --nocapture
{"lines":2000,"median_ms":69.79,"min_ms":69.199125}
{"lines":2000,"median_ms":70.464,"min_ms":69.179875}
```

Two runs, a 12-core Mac at load 13.16 (`uptime`). About 35 µs a line,
nearly all of it one Ed25519 verify per line.

## After

Measured when the holder's line, carrying its pass, lands: the same
group, its records signed by phones' holders, each line with its pass.
