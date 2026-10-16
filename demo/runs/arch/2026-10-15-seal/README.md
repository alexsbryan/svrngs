# arch-seal (a): where seals are on this host (2026-10-15, at c9e4c12)

A survey before any change (arch-seal (a); CHARTER "existing groups": a seal
found in a journal on this host outside a test is NEEDS_HUMAN.md). Nothing in
the product changed.

## What was searched

Every journal file (`ring_oplog.jsonl`) under `$HOME` outside `~/Library`,
and under `/private/tmp`, where the runs in `demo/runs` left their data dirs:

    find ~ -name ring_oplog.jsonl -not -path "./Library/*"   # 28 files
    find /private/tmp -name ring_oplog.jsonl                 # 129 files

157 journals. A svrngs group's directory holds `seed.json` beside its
journal; 132 of the 157 do, among them `~/.svrngs` and `~/.svrngs-u4`, the
two svrngs data dirs on this host (the default is `$SVRNGS_DIR`, else
`~/.svrngs`: `crates/svrngs/src/cli.rs:659-669`). The other 25 are
commonwealth-ai's: `~/.svrnmesh` (10), `~/.commonwealth-rails` (9),
euphemia's commons (4), and two sovereign-desktop test profiles.

## What was found

    while read f; do n=$(grep -c '"op":"seal"' "$f"); [ "$n" != 0 ] && echo "SEAL $n $f"; done

    SEAL 1/1349 ~/.svrnmesh/rings/wikipedia-newsworthy-tracked/ring_oplog.jsonl
    SEAL 1/1372 ~/.svrnmesh/rings/corpus-engine/ring_oplog.jsonl
    SEAL 1/1    ~/.svrnmesh/rings/seal-smoke-4a/ring_oplog.jsonl
    SEAL 4/2946 ~/.svrnmesh/rings/work-atlas/ring_oplog.jsonl
    SEAL 1/306  ~/.commonwealth-rails/rings/activity-private/ring_oplog.jsonl

A looser search, any line holding `"seal"` in any form, found the same five
files and no others.

**No svrngs group on this host holds a seal.** All 132 svrngs journals,
`~/.svrngs` and `~/.svrngs-u4` among them, have none. The five that do are
commonwealth-ai's, in data dirs svrngs never opens; the seal stays there
(CHARTER, "The original stays in commonwealth-ai"). So fence 3 does not stop
the row: removing what a seal does changes no group's record on this host.

One host, one survey: it says what this host holds, not what another
computer holds. A seal on another computer is read as an act that does
nothing once arch-seal (c) lands, and said so.
