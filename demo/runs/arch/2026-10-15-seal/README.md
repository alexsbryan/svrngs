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

# arch-seal (b) and (c): the reds, and the cut

- (b) an app's `{"op":"seal"}`: `crates/svrngs-node/src/rail/tests.rs`
  `an_apps_seal_is_refused_and_retires_nothing`, red at f903ea4 (200, the
  route sealed), green at bd3ac65 (422, `not_an_app_act`). In the commit
  body.
- (c) a seal held: `c-red.out` and `c-red-core.out`, the two new tests
  against bd3ac65, where copies honoured a seal (Ana's disk pruned to the
  seal alone; no hole below a seal); `c-green.out`, the same at 9db9dd2.

# arch-seal (c): a computer built at the row's start and one at its close

```
python3 mixed.py /tmp/arch-seal-bin/svrngs-start /tmp/arch-seal-bin/svrngs-close   # run/
```

`svrngs-start` is built at c9e4c12, the row's start, in a worktree;
`svrngs-close` at 9db9dd2. Bo runs the close's build and makes the group;
Ana runs the start's and joins by Bo's link. Each writes two records
through its record API, then Ana's app seals (the start's route still
took one) and Bo's tries to. Each writes one more record, and the run
waits for both copies to hold every act, then 75 s more, so a round each
way shows a prune or a resend. Links are redacted from `run/`. One host,
one run: not a rate.

| | Ana, the start's build | Bo, the close's build |
|---|---|---|
| an app's seal | taken: 200, retiring 3 of Ana's acts on her disk (her relay act and a1, a2; `retired` in `events.jsonl`) | refused: 422, `not_an_app_act` (WHO_KNOWS_WHOM bar 10, on the binary) |
| lines held | 9 | 12 |
| records counted | a3, b1, b2, b3 | a1, a2, a3, b1, b2, b3 |
| the log's `complete`, gaps | true, none | true, none |
| the seal, at debug | — | "a seal is held and retires nothing — an older build's act, read as one that does nothing", 3 times |
| exchanges stalled, warned | 3 | 0 |

**Agree on every act both hold: passed.** The 9 lines both hold are the
same line on each disk, and each counts the same 9 the same way
(`summary.json`: `both_hold_differently` and `counted_differently` empty).
The acts written after the seal, a3 and b3, reached both copies 0.66 s
after they were written.

**The 3 only Bo holds** are Ana's acts below her seal: the start's build
honoured its own seal and deleted them from Ana's disk, as every build
before this row did. Bo's copy, the close's build, retires nothing, so
Bo still counts a1 and a2. That difference is the operator's decision
meeting a build already built ("A seal any copy holds retires nothing",
CHARTER): this row cannot change what the start's build does. It needs a
seal, which the close's build can no longer write and no app in `apps/`
sends, and no svrngs group on this host holds one (the survey above).

**What the start's build pays for it.** Ana's copy claims Ana's run from
her floor (`from` 3); Bo's claims it from zero. The close's build keeps to
the run Ana claims, so it never sends her the 3 she pruned
(`an_older_copy_is_never_sent_below_the_floor_it_claims`). The start's
build, asked by Bo's digest, finds Ana's window folding differently and
offers 3 acts above her floor every round (`ana.log`, "ring sync: chunk
… offered=3"), which Bo already holds; its exchange then stops itself as
stalled and warns, as built (`ring_sync.rs` `exchange`). Bo's side
converges quietly. Three acts' bytes a round, on a group whose older
build sealed.
