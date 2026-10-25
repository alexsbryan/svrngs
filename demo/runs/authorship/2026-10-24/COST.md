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

At 199417cb, the instrument run with two more folds beside the first
(`crates/svrngs-core/tests/cost.rs`): `holders`, the same 2,000 records
signed by each person's phone's holder, Money's pass on every line, so 8
distinct passes; and `cutover`, the computers' 2,000 records and the
keeper's `Signers` naming each key's newest, whose cover every fold walks
back through each chain. Each fold is asserted complete with no gap, and
the cutover asserted counted.

```
$ cargo test --release -p svrngs-core --test cost -- --ignored --nocapture --test-threads=1
{"fold":"computers","lines":2000,"median_ms":71.909542,"min_ms":71.022917}
{"fold":"holders","lines":2000,"median_ms":75.587125,"min_ms":73.36316599999999}
{"fold":"cutover","lines":2001,"median_ms":73.718166,"min_ms":72.209208}
{"fold":"computers","lines":2000,"median_ms":71.166375,"min_ms":70.164208}
{"fold":"holders","lines":2000,"median_ms":76.047167,"min_ms":73.40287500000001}
{"fold":"cutover","lines":2001,"median_ms":73.728875,"min_ms":72.245166}
```

Two runs, the same 12-core Mac at load 6.15 (`uptime`).

- `computers` is the before fold at today's code: 71.2 to 71.9 ms, within
  2% of before's 69.8 to 70.5 at a different load. Old acts cost what
  they did.
- `holders`: +4 ms, about 5%, for 8 passes over 2,000 lines. A pass is
  verified once however many lines carry it (`admit.rs`, `signed`), and
  `Grant::may` is a label compare per line; one extra verify per pass is
  about 35 µs, so most of the 4 ms is the per-line grant and the label
  derivation, not signatures.
- `cutover`: +2 ms, about 3%, for the ancestry walk over every chain.

Verification's cost per fold is still one Ed25519 verify per line, plus
one per distinct pass (DESIGN.md §15 beside this file), as the design
said.
