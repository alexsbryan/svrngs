<!-- APPEND ONLY. One section per REVIEW-audit- row; a finding is cited by row
     and number, so entries are never renumbered or reordered — save once:
     syp-3's merge of main into `folders` renumbered that branch's #108-#116
     as #112-#120, after main's #108-#111, and syp-5's merge renumbered
     them as #114-#122, after main's #112-#113, each table row saying its
     number on `folders`. -->

# svrngs — review findings

One row per finding: principle, path:line (both sites for a duplicate),
fixed-in hash or why not fixed.

## REVIEW-audit-e0-auto-1 (2026-10-03, range ad29ffb..5d01962)

No earlier `REVIEW-audit-` row exists, so the range starts at ad29ffb, the
commit that created this queue. It covers units e0-2, e0-1, e0-3 and e0-4.

### (1) Net lines per unit, product code

`git log --numstat --format='@@%s' ad29ffb..5d01962`, summed by the unit id in
the subject. Product code is `crates/`, `bridge/` and `apps/`. Tests are
`*/tests/*`, `*.test.mjs`, and `crates/*/src/tests.rs` (the `#[cfg(test)]`
modules at svrngs-core/src/lib.rs:27 and svrngs-engine/src/lib.rs:69).
Lockfiles are left out (Cargo.lock +1117, bridge/package-lock.json +312).

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| e0-1 | 610 | 49 | 561 | 627 | 0 | 627 |
| e0-2 | 1491 | 0 | 1491 | 344 | 0 | 344 |
| e0-3 | 1246 | 27 | 1219 | 780 | 1 | 779 |
| e0-4 | 56 | 16 | 40 | 169 | 4 | 165 |
| total | 3403 | 92 | 3311 | 1920 | 5 | 1915 |

Size budget (`python3 scripts/lean_check.py`, rc 0): crates/svrngs-core is at
708 of 2500 lines, the whole repository at 2476 of 12000.

### (2) Clones (`code dry-report`), with each tier checked against a plant first

The binary is `~/dev/commonwealth-ai/target/debug/sovereign-cli`. The svrngs
corpus did not exist, so it was built for this audit
(`code index ~/dev/svrngs --corpus-id svrngs`). Two problems came up:

- **The near tier could not judge.** The embedder at :9741 answered 503
  (`Embed decode failed: Decode Error -3`), both to `code index` and to a
  direct POST to `/v1/embeddings`. The index was therefore built `--fts-only`
  (`model: none (fts-only)`), and the near tier's "0 edges" means nothing.
  The tool printed "No duplication found" all the same (see I2).
- **The exact tier was checked against a plant.** `project refresh` filled
  `scip_graph.db`, which held 0 symbols until then. The Rust export had 1611
  symbols. The TypeScript export failed, so the exact tier also **could not
  judge** `bridge/` or `apps/`: neither has any SCIP symbols.
  - Plant: a verbatim copy of `admit.rs:75-81` `fn app_of`, put in guests.rs.
  - First result was still **0** groups: the line above the plant differed
    from the line above the original (I1).
  - With a blank line above the plant, the tool reported **1 group**:
    `app_of — admit.rs:74 == guests.rs:116`.
  - With the plant removed, it reported **0**.
  - So the tier works only when the line above each copy normalizes the same.
    Nearly every function here sits under a `///` line, so a clean 0 from
    this tier is partial evidence.

| scope | exact tier | near tier |
|---|---|---|
| crates/svrngs-core | 0 groups (partial, I1) | could-not-judge |
| crates/svrngs-engine | 0 groups (partial, I1) | could-not-judge |
| crates/svrngs (prefix match, so also covers -core and -engine: 161 symbols) | 0 groups (partial, I1) | could-not-judge |
| bridge | could-not-judge (no SCIP) | could-not-judge |
| apps/expenses | could-not-judge (no SCIP) | could-not-judge |

**Substitute check, named as a substitute.** This is an exact-clone pass over
`.rs`, `.js` and `.mjs` files in the same five places. It hashes every run of
5 consecutive normalized lines (whitespace collapsed; blank, comment-only and
brace-only lines dropped), and overlapping hits merge into one span. It caught
the `app_of` plant (red); with the plant removed, the plant span was gone
(green). All of `crates/`, `bridge/` and `apps/` was added in this range, so
every side of every hit is new code. Every hit:

| hit | disposition |
|---|---|
| crates/svrngs-core/src/tests.rs:32-37 == :103-108 | **fixed in 9bb7d22** (#1). This hit also shows at a 6-line window, which is the window the commit body names. |
| crates/svrngs-core/src/chain.rs:343-347 == :356-360, and :357-361 == :376-380 == :453-457 | **recorded** (#2) |
| crates/svrngs/src/journal.rs:122-126 == :131-135 | **no change** (#3) |

### (3) Duplicate type names (`code converge noun`)

71 types were added in the range. 63 are `struct`/`enum`/`trait`
declarations taken from the added lines of `git diff ad29ffb 5d01962 --
crates`. The other 8 are newtypes the `hex_bytes!` and `text!` macros declare
(chain.rs:64-168: ActId GroupId Nonce Sig Name AppId Speaker Interaction).

**All 71 have exactly one definition.** The check was tested against a plant:
`pub struct Token(pub String);` added to svrngs-engine/src/lib.rs gave
`Token defs=2` (bridge.rs:44 and svrngs-engine/src/lib.rs:72). With the plant
removed, it gave `defs=1`. `converge shape` found 6 cross-crate candidate
pairs and 0 past its gates.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 1 | 8 | crates/svrngs-core/src/tests.rs:30-38, crates/svrngs-core/src/tests.rs:100-108 | 9bb7d22 | `group()` and `an_admit_its_key_did_not_cosign_is_refused` built the same Bridge admit and differed only in the co-signing role. `bridge_admit(person, bridge, cosigned_as)` now builds it once. The refusal test still fails when its argument is flipped to `Role::Bridge`. |
| 2 | 8 | crates/svrngs-core/src/chain.rs:342 (`Line`), crates/svrngs-core/src/chain.rs:355 (`VerifiedAct`), crates/svrngs-core/src/chain.rs:452 (`Draft`); the same fields are also the parameters of `check` at chain.rs:374 and `derive_id` at chain.rs:308 | not fixed | The act header (`actor, seq, prev, ts`) is written out five times. The compiler does not keep these in step: `VerifiedAct::parse` (chain.rs:369-371) copies each field of `Line` by name, so a field added to `Line` alone still compiles and is dropped at parse time. Not small, because it reshapes `Line`, the DESIGN C1 wire type with `deny_unknown_fields`, and `VerifiedAct`, the core's central type. That is a refactor for its own commit, with the wire round-trip test (`tests.rs` `a_line_round_trips_and_its_id_derives_from_it`) as its witness. |
| 3 | 8 | crates/svrngs/src/journal.rs:121, crates/svrngs/src/journal.rs:130 | no change | `JournalActor`'s fields and `spawn`'s parameters list the same five names, because `spawn` is the one place the struct is built. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

They belong to commonwealth-ai (`sovereign/crates/sovereign-code/src/dry_report.rs@b4c23c4d6`).
svrngs never links to it.

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | dry_report.rs:589-590 | `let s = (start.max(1) - 1)` treats SCIP `line_start` as 1-based, but it is 0-based: the svrngs row for `fn app_of` says 74, and the function is on line 75 of admit.rs. So each body read starts one line above the function and stops before its closing brace. Two identical functions with different lines above them hash differently. Evidence: the plant runs in (2), with 0 groups when the line above differed and 1 group when it was blank. |
| I2 | 5, 6 | dry_report.rs:210 | The check that skips symbols without vectors did not fire on an `--fts-only` index: 72 symbols were loaded, and the near pass over them reported 0 edges and "No duplication found". It should have reported could-not-judge. |
| I3 | 6 | dry_report.rs:203 | `--scope` matches a path prefix, so `crates/svrngs` also takes in `crates/svrngs-core` and `crates/svrngs-engine` (161 symbols), and nothing in the output says so. |
| I4 | 6 | `project refresh` | The TypeScript SCIP export failed (`typescript: skipped (export failed)`), and dry-report then printed "0 exact-clone groups" for `bridge/` and `apps/` instead of saying it had no symbols there. |

## REVIEW-audit-u1-auto-1 (2026-10-03, range a984fd1..0030f09)

This queue has no earlier `REVIEW-audit-` row, so the range starts at a984fd1,
the commit that created it (`git log -- ralph/next/u1/STATE.md | tail -1`).
It covers units u1-1, u1-2, u1-3 and u1-4. The range touched one crate,
crates/svrngs-node (`git diff --stat a984fd1 0030f09 -- crates bridge apps`).

### (1) Net lines per unit, product code

`git log --numstat --format='@@%h %s' a984fd1..0030f09`, summed by the
`u1-N` in each subject. Product code is `crates/`, `bridge/` and `apps/`.
Tests are `*/tests/*` and `*/tests.rs`. Cargo.lock is left out (u1-1
+4163 −320, u1-2 +3). crates/svrngs-node/Cargo.toml counts as src. The
docs-only commits (DESIGN.md, NORTH_STAR.md, STATE.md, ARCHITECTURE.md)
touch no product code.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u1-1 | 1469 | 1 | 1468 | 202 | 0 | 202 |
| u1-2 | 280 | 136 | 144 | 147 | 0 | 147 |
| u1-3 | 1093 | 37 | 1056 | 701 | 37 | 664 |
| u1-4 | 340 | 187 | 153 | 151 | 0 | 151 |
| total | 3182 | 361 | 2821 | 1201 | 37 | 1164 |

u1-1 and u1-3 are forks, the files as upstream had them, so most of their
lines are upstream's. The size budget (`python3 scripts/lean_check.py`,
rc 0) puts crates/svrngs-core at 2297 of 2500 and the whole repository at
5902 of 12000. This range did not touch svrngs-core. The audit's own fixes
are test-only: 40 lines added and 67 deleted
(`git diff --shortstat 0030f09 HEAD -- crates`).

### (2) Clones (`code dry-report --scope crates/svrngs-node`), each tier checked against a plant first

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, at
commonwealth-ai bb7771863. The embedder at :9741 answered this time, so the
corpus was rebuilt with vectors (`code index ~/dev/svrngs --corpus-id svrngs
--full`: 838 chunks, qwen-embedding-0.6b, 1024-d). The SCIP graph came from
`svrn refresh` (rust: 3377 symbols). On HEAD's code, the unplanted run
reported **0 exact groups and 0 near clusters** over 213 symbols. Then the
plants:

- **Exact tier, plant A:** a byte-identical copy of `guest.rs:206`
  `redact_userinfo` in iroh.rs, with the same doc line above it. Result:
  **1 group** (`guest.rs:205 == iroh.rs:780`). Red, so the tier fires.
- **Exact tier, plant B:** the same copy with a different line above it.
  Result: **0 groups**. I1 is still there: the body read starts one line
  above the function, so a clean 0 from this tier is partial evidence.
- **Near tier, plant B:** the near tier missed it as well. Two
  byte-identical bodies gave **0 edges** at the default 0.95 and none at
  0.85 or 0.8. They first share a cluster at 0.75 (cluster cosine
  0.777–0.798). Plant C, the same body with renamed identifiers
  (`hide_credentials`), never pairs with `redact_userinfo` above 0.7, but at
  0.8 it pairs with its line neighbour `key_of` (node.rs:197–206 and
  205–215 overlap). Each symbol is embedded with its neighbours' lines, so
  where a function sits counts for more than what it says (I5). So the near
  tier **could not judge**, and its 0 means nothing.
- With the plants removed: 0 and 0 again.

| scope | exact tier | near tier |
|---|---|---|
| crates/svrngs-node | 0 groups (partial, I1) | could-not-judge (I5) |

**Substitute check, named as a substitute.** This is the same pass the e0
audit used. It hashes every run of 5 consecutive normalized lines over
`crates/svrngs-node/**/*.rs`, test files included, and merges overlapping
spans. It caught plant B
(`guest.rs:206-211 == iroh.rs:781-786`, red). With the plant removed, that
span was gone (green). Each hit with a side in a line the range added
(`git diff -U0 a984fd1 0030f09`), and what became of it:

| hit at 0030f09 | disposition |
|---|---|
| tests/accept.rs:31-42 == tests/node_sync.rs:48-59 (`loopback_sockets`) | **fixed in 1cc2e21** (#4) |
| ring_routes/tests.rs:39-43 == ring_sync/tests.rs:110-114 (`serve` / `serve_at`) | **fixed in 0b29f85** (#5) |
| guest/tests.rs:105-110 == iroh.rs:483-488 (and == accept.rs:33-38, node_sync.rs:50-55) | **fixed in 42fa132** (#6) |
| iroh.rs:469-492 == tests/accept.rs:19-42, iroh.rs:481-492 == tests/node_sync.rs:48-59 | **recorded** (#7) |
| guest/tests.rs:120-124 and :126-130 == iroh.rs:567-571 and :573-577 | **recorded** (#8) |
| ring_sync/tests.rs:175-179 == :358-362; :191-195 == :221-225; :251-256 == :301-306 == :426-431 | **no change** (#9) |
| ring_sync/tests.rs:411-416 == tests.rs:676-681 | **no change** (#10) |

After the fixes, the pass leaves only #7–#10 among new-side hits
(`python3 clones.py 5 crates/svrngs-node`, diffed against the run before).
None is in non-test code: every clone this range added to src is inside a
`#[cfg(test)]` module.

### (3) Duplicate type names (`code converge noun`)

27 type declarations were added in the range (`git diff a984fd1 0030f09 --
crates`, `^+ (pub..)? (struct|enum|trait|type) Name`). `Err` is an
associated type, so 26 nouns remain. RingSyncHost (a trait) and
RingSyncJournal were both added and deleted inside the range (82e90cc,
9cf49c7), so neither exists at HEAD.

**25 have exactly one definition.** They are Admit, CachedBridge, Contact,
DialKey, Dialer, Discovery, ExchangeOutcome, ExchangeStop, GuestTunnel,
Host, HttpBridge, IrohAcceptor, IrohTransport, LiveEnvelope, Node,
NodeError, OnlinePeer, PumpSide, RelayConfig, RingInbound, RingSyncHandle,
RingSyncRequest, RingSyncResponse, RoundMembers and RoundOutcome. The tool
reports `OnStream` as defs=0, because it sits in iroh.rs's test module
(iroh.rs:427) and the tool's scope leaves test code out. By grep it has one
definition.

The check was tested against a plant: `pub struct Contact;` added to node.rs
gave `definitions (2)` (iroh.rs:82, node.rs:206). With the plant removed it
gave `(1)`. `converge shape` found 8 cross-crate candidate pairs, with 0
past its gates.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 4 | 8 | crates/svrngs-node/tests/accept.rs:31, crates/svrngs-node/tests/node_sync.rs:48 | 1cc2e21 | The two integration tests each wrote out `loopback_sockets`. It is now in tests/common/mod.rs:10, and both files call it. |
| 5 | 8 | crates/svrngs-node/src/ring_routes/tests.rs:37, crates/svrngs-node/src/ring_sync/tests.rs:108 | 0b29f85 | ring_routes' test `serve` wrote out `serve_at` for one router. Upstream had the same pair at e1a43bcd2 (ring_routes/tests.rs:79-83 == ring_sync/tests.rs:109-113). `serve` now calls `serve_at`. |
| 6 | 8 | crates/svrngs-node/src/guest/tests.rs:95-113, crates/svrngs-node/src/iroh.rs:469 and :481 | 42fa132 | `spawn_lender` inlined `hermetic_endpoint` and `loopback_sockets`, a copy upstream already had (guest_tests.rs:103-108 == iroh.rs:630-635). It now calls both helpers. |
| 7 | 8 | crates/svrngs-node/src/iroh.rs:469 (`hermetic_endpoint`) and :481 (`loopback_sockets`), crates/svrngs-node/tests/accept.rs:21 and crates/svrngs-node/tests/common/mod.rs:10 | not fixed | An integration test cannot reach a `#[cfg(test)]` item, so the crate's unit tests and its integration tests each hold a copy. One copy would need a public test-support surface (a feature-gated module, say). That adds API to svrngs-node for tests, so it is not small. It is u1-5's to weigh, because group_not_mesh.rs will need the same two helpers. |
| 8 | 8 | crates/svrngs-node/src/iroh.rs:565 (`spawn_splicing_acceptor`), crates/svrngs-node/src/guest/tests.rs:88 (`spawn_lender`'s accept loop, :108-125) | not fixed | u1-2 wrote the splicing acceptor in the shape of `spawn_lender`, and its doc says so (iroh.rs:559-564). `spawn_lender` could call it, but the pump's trace label would change from "guest" to "peer". That changes a test's traces, so it is not behaviour-preserving, and it stays here for the next unit that touches guest/tests.rs. |
| 9 | 8 | crates/svrngs-node/src/ring_sync/tests.rs:175 and :358; :191 and :221; :251, :301 and :426 | no change | Each repeat is a test's own setup or assertion lines (admit then assert complete; the seal fixture; one budgeted `ops_missing_from_within`), and each is upstream's at e1a43bcd2 (ring_sync/tests.rs:175/420, 193/283, 313/362/490). Each test states what it checks, and nothing is decided twice. |
| 10 | 8 | crates/svrngs-node/src/ring_sync/tests.rs:411, crates/svrngs-node/src/tests.rs:675 | no change | Both cut the journal file to a seq by hand. tests.rs:672-674 says `truncate_by_hand` exists "ONLY" for its negative control, as "a second implementation … shown to be wrong". Calling it from ring_sync's seal test would make that doc false, so the two stay apart. |
| 11 | 12 | crates/svrngs-node/src/peer_wire.rs:89 (`LiveEnvelope`) | not fixed | Nothing in svrngs uses it (`grep -rn LiveEnvelope crates`). u1-1 forked peer_wire whole, and its only user is the live route, which DESIGN §8 gives to U3 (DESIGN.md:302, :337). Deleting it now would undo the row, and U3 would fork it back. U3 decides, or deletes it if U3 does not fork the live lane. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | dry_report.rs (see the e0 entry) | Still present at bb7771863. Plant B (identical body, different line above) gave 0 exact groups, and plant A (same line above) gave 1, reported at `guest.rs:205` for a function at :206. |
| I5 | 5, 7 | `code dry-report` near tier | Two byte-identical 8-line bodies scored below 0.8 cosine and gave 0 edges at the default 0.95. The near tier's symbol spans overlap their neighbours (guest.rs `redact_userinfo` 202–215 for a function at 206–213, and node.rs `key_of` 197–206 next to `hide_credentials` 205–215), so where a function sits counts for more than what it says. The tier then prints "No duplication found" when it should report could-not-judge. |
| I6 | 6 | `code converge shape` | The header printed `graph: svrngs @ 9bb7d22` (the e0 audit's commit) after `svrn refresh` had rewritten scip_graph.db at HEAD 42fa132's code. The noun plant shows the data was current, so the stamp is what is stale, and a reader would think the data was too. |

## REVIEW-audit-u2-auto-1 (2026-10-03, range ffc0856..6697f9b)

This queue has no earlier `REVIEW-audit-` row, so the range starts at
ffc0856, the commit that created it (`git log --format=%h --
ralph/next/u2/STATE.md | tail -1`). It ends at 6697f9b, the commit that
added this row. It covers u2-1, u2-2, u2-3 and u2-4, plus two operator
commits made between rows: eaffcbf (delete svrngs-engine) and 0261ac5
(docs). dfc7148 (u2-4b's row, DESIGN.md and STATE.md) landed after the row
and touches no product code. The range touched crates/svrngs-core,
crates/svrngs-node, crates/svrngs-engine (deleted) and one line of
apps/expenses (`git diff --stat ffc0856 6697f9b -- crates bridge apps`).

### (1) Net lines per unit, product code

`git log --reverse --numstat --format='@@%h %s' ffc0856..6697f9b`, summed by
unit. The subjects of work commits carry no unit id, so each commit counts
toward the row whose `started` (or `in progress`) commit comes before it, and
the two operator commits count on their own. Product code is `crates/`,
`bridge/` and `apps/`. Tests are lean_check's test files (its regex,
scripts/lean_check.py:33), plus `*/tests/*` and `*/fixtures/*`. Cargo.lock is
left out (u2-2 +2, eaffcbf −37). The counts are raw diff lines, comments
included.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| operator eaffcbf | 0 | 391 | -391 | 1 | 172 | -171 |
| u2-1 | 14 | 1092 | -1078 | 1091 | 0 | 1091 |
| u2-2 | 446 | 237 | 209 | 675 | 315 | 360 |
| u2-3 | 25 | 1 | 24 | 10 | 16 | -6 |
| u2-4 | 307 | 23 | 284 | 192 | 8 | 184 |
| total | 792 | 1744 | -952 | 1969 | 511 | 1458 |

u2-1 is a move: its −1078 src and +1091 test are the same blocks, now in
sibling `*_tests.rs` files. `python3 scripts/lean_check.py` (rc 0) counts:

| at | svrngs-core | repository |
|---|---:|---:|
| ffc0856, range start (`lean_check.py <worktree>`) | 2293 / 2500 | 6059 / 12000 |
| 6697f9b, range end | 1723 / 2500 | 5314 / 12000 |
| 3a172f2, after this audit's fixes | 1734 / 2500 | 5317 / 12000 |

The growth that is not a move is u2-2's id (+209 src) and u2-4's link
(+284 src). The audit's fixes add 11 core lines (`moves_membership`) and
remove 8 node lines (register's list).

### (2) Clones (`code dry-report`), each tier checked against a plant first

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, built
2026-10-03 20:53. The SCIP graph came from `sovereign-cli refresh` (rust:
3544 symbols; the TypeScript export failed again, as I4). The corpus came
from `code index ~/dev/svrngs --corpus-id svrngs --full` (848 chunks,
qwen-embedding-0.6b, 1024-d); the embedder answered. On the range-end code,
the unplanted run at the default threshold reported **0 exact groups and 0
near clusters** for crates/svrngs-core (265 symbols), crates/svrngs-node
(217) and apps/expenses (24). Then the plants, with both refreshed each time:

- **Plant A** (exact tier): `sig.rs` `redemption_message`, a function u2-4
  added, copied byte for byte into attest.rs with the same last doc line
  above it. Result: **1 group**, `attest.rs:145 == sig.rs:154`. Each site is
  one line above the `fn`, so I1 is still present.
- **Plant B** (near tier): the same body as `consent_message` in hash.rs,
  under a different doc line. Exact tier: 0. Near tier: **0 edges at 0.95,
  none at 0.9**. It first clusters at 0.85 (`attest.rs`, `hash.rs`, `sig.rs`;
  cosine 0.851–0.863), so I5 is still present.
- With the plants removed: 0 and 0 again at 0.95.

So the default near tier **could not judge**. The near tier was rerun at
0.85, the threshold where plant B showed: svrngs-core gave 33 clusters,
svrngs-node 18. A side counts as **added** when its `fn` name is not defined
anywhere in that crate at ffc0856 (33 names in core, 7 in node). This keeps
u2-1's moved tests from counting as new. Every 0.85 cluster with an added
side:

| cluster (dry-report, 0.85) | disposition |
|---|---|
| `invite` membership_invite_tests.rs:8–19 ~ `redeem_as` :16–29 (0.906) | no change (#17): two test builders on neighbouring lines, each building a different act |
| `a_redemption_by_someone_who_did_not_sign_the_invite_is_refused` :107–118 ~ `a_redemption_co_signed_by_another_key_is_refused` :117–128 (0.929) | no change (#17): two tests, each stating its own case through the shared builders |
| `from_hash` op.rs:80–88 ~ `as_bytes` :87–95 (0.882) | no change (#17): a constructor and an accessor on neighbouring lines |
| `an_op_a_correction_names_survives_a_seal_above_it` svrngs-node tests.rs:836–884 ~ `compaction_keeps_a_cited_op_whatever_id_its_line_carries` :967–1008 (0.858) | no change (#17): both are about compaction and a correction; the setup and the assertion differ |

**Substitute check, named as a substitute.** This is the pass the e0 and u1
audits used, with one change in how a hit counts as new. It hashes every run
of 5 normalized lines over `crates/svrngs-core` and `crates/svrngs-node`
`**/*.rs`, test files included. A hit is **new** when a line in either span
is blamed inside the range by `git blame -w -M -C -C ffc0856..HEAD`, so
u2-1's moves blame to the base. It caught plant B (`NEW hash.rs:132-143 ==
sig.rs:156-167`, red). With the plant removed, its output was byte-identical
to the unplanted run (green). Every new-side hit at 6697f9b:

| hit | disposition |
|---|---|
| core tests_sealing.rs:153-161 == node tests.rs:787-795 == node tests.rs:1019-1025 (a forged line built by hand) | **fixed in 474c61e** (#13) |
| core tests.rs:119-126 == :281-288 (a target and its `Correct`) | no change (#17) |

The exact tier missed one product-code duplicate that both clone passes
skip, because its body is a single line: `hex::encode(key.sign(..).to_bytes())`
at sig.rs:172, :179 and attest.rs:102 (#12). Reading the range's diff found
it.

### (3) Duplicate type names (`code converge noun`)

`git diff ffc0856 6697f9b -- crates`, added lines matching
`(pub..)? (struct|enum|trait|type) Name`: **Redemption** (lib.rs:383),
**InviteFault** (membership.rs:59), **Newest** (journal.rs:58), and
**OpId**, whose definition was rewritten (op.rs:80). The removed types
(`Engine`, `Segment`, `Breach`, `EngineError`, `Limits`, `QuickJs`,
`Budgeted`) went with svrngs-engine.

**All four have exactly one definition** (`definitions (1)` each). The check
was tested against a plant: `pub struct Newest;` added to svrngs-core's
hash.rs gave `definitions (2)` (hash.rs:129, journal.rs:58). With the plant
removed, it gave `(1)`.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 12 | 8 | crates/svrngs-core/src/sig.rs:172 (`sign_message`, added by u2-4), crates/svrngs-core/src/sig.rs:179 (`sign_ring_op`), crates/svrngs-core/src/attest.rs:102 (`GuestAttestation::sign`) | 80b0e3c | u2-4 added a third spelling of "hex Ed25519 signature over these bytes". The other two now call `sign_message`, the counterpart of `verify_hex`. The ignored `emit_journal_fixture` re-emits rail_ops_before_on_behalf_of.jsonl byte for byte. |
| 13 | 8 | crates/svrngs-core/src/tests_sealing.rs:138, crates/svrngs-node/src/tests.rs:778 and :1010 (`forged_as_alex`) | 474c61e | u2-2 built "cy's signature over a line claiming alex's" by hand three times (act_id + sign_ring_op + `Op {..}`), which tests_support's header rules out. `tests_support::forged` (tests_support.rs:135) builds it through `sign_act`. The core site still fails when its signer is alex's own key. |
| 14 | 8, 10 | crates/svrngs-node/src/register.rs:62 (its `matches!`, now gone), crates/svrngs-core/src/membership.rs:276 | 3a172f2 | Which act kinds membership reads was decided twice, in two crates. u2-4 had to remember to add `Invite` to the node's list, and only its plant 6 would have caught a miss. `RailAct::moves_membership` is now the one list, beside the fold, with no wildcard: a planted variant fails to compile at membership.rs:277. Correction to 3a172f2's body: it calls register.rs a fork, but register.rs is svrngs's own and has no `ported from` line (register.rs:1-5). Only membership.rs is a fork. |
| 15 | 3; U2's words (the queue prompt, DESIGN §1) | crates/svrngs-core/src/membership.rs:79 ("this node does not hold its link") and :82 ("the joining key did not sign it"), crates/svrngs-core/src/admit.rs:248 (`RefusedRedemption`'s sentence) | not fixed | u2-4's new gap sentences use *node* and *key*. `RailGap`'s Display says it is "one sentence a person can act on" (admit.rs:199-206), and upstream's gap sentences say *node* and *op* as well. Nothing in svrngs renders a gap to a person yet, because the command arrives with u2-5. Changing the text is a change in output, so it is not this audit's to make. u2-6's check, that nothing `new`, `join` or `link` prints contains these words, is where it lands. Whoever first renders a gap rewrites these in DESIGN §1's words. admit.rs:201-203 also names three places that show a gap (`svrn ring log`, an app page, the door's refusal), and svrngs has none of them yet. |
| 16 | 6, 10 | crates/svrngs-node/src/journal.rs:76-89 (`Held::catch_up` folds every line's seq, verified or not), crates/svrngs-node/src/journal.rs:297-310 (the door follows the newest seq, and names an unverified line when none at that seq verifies) | fixed, u7-removal (d): `Held::catch_up` (now crates/svrngs-runtime/src/journal.rs) moves an actor's newest seq only on a line whose signature verifies; `a_forged_line_above_a_members_newest_never_moves_their_next_seq`, red at 888a3ee (seq 6), green | One forged line claiming a signer's key, above everything that signer wrote, moves the door's next seq past it, which leaves holes that never fill. Reproduced with a scratch test, not committed: alex appends `one` (seq 0), the journal ingests `forged_as_alex(5, &one)`, and alex appends `two`, which the door signs at **seq 6** naming the forgery. Admission then reports BadSignature and **SequenceHole 1, 2, 3, 4 and 5**, so the fold is never complete; `one` and `two` still apply. Upstream's `next_seq` counted every line the same way, so this is inherited. u2-2 kept it and wrote it down ("when none verifies … names the first"). The fix (seq from verified lines only, as the door already does for `prev`) changes which seq the door signs. That is a behaviour change, for the operator or U7 (forks) to place. |
| 17 | 8 | membership_invite_tests.rs:12/:20, :111/:122; op.rs:80/:87; svrngs-node tests.rs:839/:967; core tests.rs:119/:281 | no change | The near clusters at 0.85 and the one test-setup repeat above. Each is two neighbouring builders or accessors, or two tests that each state their own case through shared helpers. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code dry-report` exact tier; also `code converge noun` | Still present. Plant A was reported at attest.rs:145 for a `fn` at :146. `converge noun` has the same off-by-one: `Redemption` at lib.rs:382 (the struct is at :383) and the planted `Newest` at hash.rs:129 (it is at :130). |
| I5 | 5, 7 | `code dry-report` near tier | Still present. A byte-identical 14-line body under another name scored 0.851–0.863 and gave 0 edges at the default 0.95 and at 0.9, yet the report still says "No duplication found". |
| I7 | 6 | `code converge noun` owner candidates | `Redemption` and `OpId` are defined in svrngs-core and used by svrngs-node, which depends on it (crates/svrngs-node/Cargo.toml:36). For both the tool printed "none — the users share no dependency. This noun needs a new home crate". The defining crate is not counted as a candidate for its own users, so a noun that already has its right home reads as homeless. |
| I8 | 6 | `code dry-report` scope | The header says "First-party production code only". Its exclusion is a `tests` path segment, so the `*_tests.rs` siblings u2-1 created (and `tests.rs`, `tests_sealing.rs`) are counted as production: most of the 0.85 clusters above are test functions. |

## REVIEW-audit-u2-auto-2 (2026-10-04, range afbacc4..e3e2f8c)

The previous `REVIEW-audit-` row's hash is afbacc4 (STATE.md, the
REVIEW-audit-u2-auto-1 row). The range ends at e3e2f8c, the commit that
added this row. It covers u2-4b, u2-5, u2-6 and u2-7. The operator's
907a345 (docs), 35de30a, fa99f64, 0b2c9a3, 56b8fd1 and 9f2f5af (rows) touch
no product code. The range touched crates/svrngs-core, crates/svrngs-node
and crates/svrngs, which u2-5 created (`git diff --stat afbacc4 e3e2f8c --
crates bridge apps`).

### (1) Net lines per unit, product code

`git log --reverse --numstat --format='@@%h %s' afbacc4..e3e2f8c`, summed by
unit. Each commit counts toward the row whose `started` commit comes before
it. Product code is `crates/`, `bridge/` and `apps/`. Tests are files whose
path has a `tests/` or `fixtures/` segment or whose name lean_check counts as
a test (scripts/lean_check.py:33). Cargo.lock is left out. The counts are raw
diff lines, comments and Cargo.toml included. Inline `#[cfg(test)]` modules
count as src here, as lean_check counted them (#21).

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u2-4b-close | 43 | 11 | 32 | 61 | 4 | 57 |
| u2-5-command | 1685 | 301 | 1384 | 235 | 0 | 235 |
| u2-6-join | 773 | 111 | 662 | 373 | 8 | 365 |
| u2-7-origin-kept | 127 | 30 | 97 | 107 | 8 | 99 |
| total | 2628 | 453 | 2175 | 776 | 20 | 756 |

u2-5's +1384 is the new `svrngs` crate: cli.rs 425, link.rs 379 forked then
rewritten (+225 −246), group.rs 265, identity.rs 155 forked then +121 −43.
It includes 151 test code lines inside identity.rs and link.rs (#21). u2-6's
+662 is mostly redeem.rs (+351) and the join verb in cli.rs (+177).
`python3 scripts/lean_check.py <tree>` (rc 0):

| at | svrngs-core | repository |
|---|---:|---:|
| afbacc4, range start (`git archive` into a temp dir) | 1734 / 2500 | 5317 / 12000 |
| e3e2f8c, range end | 1750 / 2500 | 6852 / 12000 |
| b556e0b, after this audit's fixes | 1753 / 2500 | 6680 / 12000 |

The fixes remove 172 repository lines (lean_check at each fix commit): 149
moved test lines (#21); 10 from #18 (svrngs-node −13, svrngs-core +3 that
make the helpers `pub`); 2 from #19; 5 from #20; 6 from #22.

### (2) Clones (`code dry-report`), each tier checked against a plant first

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, built
2026-10-04 07:11. The SCIP graph came from `sovereign-cli refresh` (rust:
4443 symbols; TypeScript export failed again, as I4). The corpus came from
`code index ~/dev/svrngs --corpus-id svrngs --full` (1013 chunks). On the
range-end code, the unplanted run at the default threshold reported **0
exact groups and 0 near clusters** for crates/svrngs-core (273 locations),
crates/svrngs-node (231) and crates/svrngs (62). Then the plants, with both
refreshed:

- **Plant A** (exact tier): redeem.rs's `judge`, which u2-6 added, copied byte
  for byte with its doc lines into alpn.rs. Result: **1 group**,
  `judge — alpn.rs:46` and `judge — redeem.rs:280` (the `fn`s are at :47 and
  :281, so I1 is still present).
- **Plant B** (near tier): `judge`'s body as `weigh` in register.rs under
  another doc line, and group.rs's `groups` as `held_groups` in cli.rs.
  At 0.95: neither is reported. At 0.9: `held_groups ~ groups` (0.904),
  but not `weigh`. At 0.85: `weigh` joins the judge cluster (0.863–0.973).
  So I5 is still present.
- With the plants removed: 0 and 0 again at 0.95 in all three.

So the default near tier **could not judge**. The near tier was rerun at
0.85, where both plants showed: svrngs-core gave 35 clusters, svrngs-node
21, svrngs 2. A side counts as **added** when its `fn` name is not defined
anywhere in that crate at afbacc4 (`git grep fn afbacc4 -- <crate>`). Every
0.85 cluster with an added side:

| cluster (dry-report, 0.85) | disposition |
|---|---|
| `read_seed` group.rs:197–225 ~ `read_origin` :232–259 (0.867) | **fixed in d652b85** (#20), pinned first by bd53a47 |
| `standing_in` register.rs:45–65 ~ `membership_in` :64–93 (0.887) | no change (#24): `standing_in` calls `membership_in` and maps its answer |
| `append` journal.rs:203–219 ~ `append_all` :235–253 ~ `append_if` :256–274 (0.865–0.898) | no change (#24): three doors, each one call into `write` |
| `write` journal.rs:272–353 ~ `write` :354–440 (0.879) | no change: one function, two chunks of it (I9) |
| `invite_for` membership_invite_tests.rs:14–25 ~ `redeem_as` :22–35 (0.915) | no change (#24): two neighbouring test builders, each building a different act |
| `three_redemptions_under_one_invite_…` :67–85 ~ `a_redemption_timed_before_the_newer_link_…` :222–235 (0.888) | no change (#24): two tests, each stating its own case |
| link.rs's three tests :318–373 (0.901–0.923) | no change (#24): three tests through one builder; now in link_tests.rs (#21) |

**Substitute check, named as a substitute.** This is the same pass as the
u2-auto-1 audit. It hashes every run of 5 normalized lines over
crates/svrngs-core, crates/svrngs-node and crates/svrngs `**/*.rs`, test
files included. A hit is **new** when a line in either span is blamed inside
the range by `git blame -w -M -C -C`, or is uncommitted. Plant B
(`held_groups`) gave `NEW cli.rs:615-646 == group.rs:263-294` (red). With the
plant removed, the output was byte-identical to the unplanted run (`cmp`,
green). Every new-side hit at e3e2f8c:

| hit | disposition |
|---|---|
| group.rs:206-212 == :244-250 (`read_seed`'s and `read_origin`'s file read) | **fixed in d652b85** (#20) |
| svrngs tests join_by_link.rs:134-139 == new.rs:60-65 (`cli::new_group(&node, name("house"), name("Sam"), …)`) | no change (#24): one call with its arguments |

Reading the range's diff found five repeats that neither clone pass can
see, because each body is a single line or a closure: #18's three, #19 and
#22. A count of repeated lines across the new files (`uniq -c`, three or
more) found only signatures and match arms besides.

### (3) Duplicate type names (`code converge noun`)

`git diff afbacc4 e3e2f8c -- crates`, added lines matching
`(pub..)? (struct|enum|trait|type) Name`: **Alpn**, **RedeemRequest**,
**Redeemed**, **Refused**, **RedeemError** (svrngs-node); **Command**,
**Refusal**, **Said**, **Name**, **Seed**, **SeedFile**, **SeedError**,
**KeyError**, **Origin**, **Dial**, **Link** (svrngs); **Computer**
(join_by_link.rs, a test). No type was removed.

**All sixteen production nouns have exactly one definition**
(`definitions (1)` each). `Computer` gives `definitions (0)`, because the
tool skips `tests` paths. The check was tested against a plant:
`pub struct Seed;` appended to svrngs-node's alpn.rs gave `definitions (2)`
(group.rs:70, alpn.rs:44). With the plant removed, it gave `(1)`.

The tool matches names. Read for meaning, one pair is the same noun under
two names, with two rules: svrngs's `Name` (group.rs:41-56) and the core's
`Person` (crates/svrngs-core/src/lib.rs:159-177). That is #23.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 18 | 8 | crates/svrngs-node/src/redeem.rs:100, :108-111, :123-133 (at e3e2f8c); crates/svrngs-core/src/sig.rs:73 (`field`), :175 (`sign_message`), :243 (`verify_hex`) | 5380ed4, pinned first by 51a9837 | u2-6's redeem request signed, framed and verified its bytes by hand: a fourth spelling of the signature #12 gave one home, and a second hex-key parser beside `verify_hex`. The three helpers are `pub` and redeem.rs calls them; svrngs-node's `ed25519-dalek` dependency goes with it. Nothing tested `signed_by_link`: planted `true`, every svrngs-node and svrngs test passed, because Fay's forged link is refused earlier at the `Invite`-key check. Gus (join_by_link.rs) presents an open link's key, which every copy of the record shows, on a request another key signed. He fails on the plant at :250. |
| 19 | 8 | crates/svrngs/src/identity.rs:36-39 (`node_pubkey`, at e3e2f8c); crates/svrngs-core/src/sig.rs:67-69 (`actor_of`) | 9712bf9 | The fork made upstream's typed `node_pubkey` answer hex, so it became `actor_of`'s body. Its two traces call `actor_of` now. |
| 20 | 6, 8 | crates/svrngs/src/group.rs:201-216 (`read_seed`) and :236-250 (`read_origin`); :186-198 and :227-233 (the writers), at e3e2f8c | d652b85, pinned first by bd53a47 | u2-7 copied u2-5's seed file handling for the origin, so "only no file is an absence" was decided twice. `keep`/`kept` (group.rs:235, :244) decide it now. No test held the rule: each reader, planted to read every failure as `None`, passed every test. new.rs's `an_unreadable_seed_or_origin_refuses_naming_its_file` fails on each plant. |
| 21 | 5 (budget), 10 | crates/svrngs/src/identity.rs:149-235 and crates/svrngs/src/link.rs:265-379, at e3e2f8c; scripts/lean_check.py:33 | caa377b | Forked with their test modules inline, which lean_check counts as product code (151 lines). This is the case DESIGN.md:433-436 had u2-1 move out of the core. They are now `identity_tests.rs` and `link_tests.rs`: 10 tests before and after, and each block equal to its sibling but for whitespace. |
| 22 | 8 | crates/svrngs/src/group.rs:102-115 (`Seed::id`, at e3e2f8c); crates/svrngs-core/src/sig.rs:73 (`field`) | b556e0b | The group id's length-prefixed framing was a third copy of `field`. The ids are unchanged: a scratch comparison with the old body agrees on 15 seeds and fails when the founder is planted unframed. |
| 23 | 8, 10 (parse at the edge), 6 | crates/svrngs-node/src/redeem.rs:230 (`person: req.person.clone()` into the `Admit`), crates/svrngs/src/group.rs:41-56 (`Name::parse`: trimmed, 1..=64 bytes, no control characters); crates/svrngs-core/src/lib.rs:159-177 (`Person`, any string) | d725031 (u2-9) | A person's name has one rule, and only the joiner's command applies it (cli.rs:412 makes a `Person` from a parsed `Name`). The issuer's node writes whatever `person` a request carries into a signed `Admit`. An empty name, 10 KB, or terminal escape codes go into every copy of the record, and `svrngs` later prints people's names. Refusing them at the redeem route, or making `Person` parse, changes what is admitted, so it is not this audit's to make. The natural home is the core, where `Person` lives, so the node and the command share one parser. u2-8 rewrites redemption next. |
| 24 | 8 | the "no change" rows in (2) | no change | Each is a function and its caller, a family of doors over one `write`, neighbouring test builders, tests stating their own cases, or one call written with its arguments. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code dry-report` exact tier; `code converge noun` | Still present: plant A at alpn.rs:46 / redeem.rs:280 for `fn`s at :47 / :281; `Alpn` at alpn.rs:11 (it is at :12); `Seed` at group.rs:70 (:71). |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". svrngs has no TypeScript for it to export. |
| I5 | 5, 7 | `code dry-report` near tier | Still present: a renamed byte-identical body is missed at the default 0.95 (`held_groups`, first at 0.9; `weigh`, first at 0.85), while the report says "No duplication found". |
| I7 | 6 | `code converge noun` owner candidates | Still present, in another form: `Alpn` is defined and used only in svrngs-node, and the tool names svrngs-core, "covers 1/1", as its canonical owner. |
| I8 | 6 | `code dry-report` scope | Still present: inline test modules and `*_tests.rs` siblings count as "first-party production" (link.rs's three tests above). |
| I9 | 6, 7 | `code dry-report` near tier | New: one function split into two embedding chunks is reported as a near-clone of itself, `write` journal.rs:272–353 ~ `write` :354–440 (0.879). It is one `fn write` (journal.rs:272). |
| I10 | 6 | `code dry-report` header | New: "654 function/method symbols" in all three scopes. The count is the corpus's, not the `--scope`'s; the next figure (273, 231, 62 locations) is the scope's. |

## REVIEW-audit-u3-auto-1 (2026-10-04, range 7278e08..e2ef4ab)

The previous `REVIEW-audit-` row is REVIEW-audit-u2-auto-2, hash 7278e08
(ralph/next/u2/STATE.md:13), made after this queue's first commit 371d63d;
so the range starts there (the row's correction, c3e24be) and ends at
e2ef4ab, the commit that added this row. It covers u2-8 and u2-9 (the u2
queue's last two rows, which no audit covered), u3-1 to u3-4, and the
operator's 4a5fb20 (redeem.rs's routes as doors). c05fca5, 707c1b8 and the
`ralph:` commits touch no product code. The range touched crates/svrngs-core,
crates/svrngs-node, crates/svrngs, apps/expenses and the deleted bridge/
(`git diff --stat 7278e08 e2ef4ab -- crates bridge apps`, 52 files).

### (1) Net lines per unit, product code

`git log --reverse --numstat --format='@@%h %s' 7278e08..e2ef4ab`, summed by
unit. A commit counts toward the row whose `started` commit comes before
it; the operator's 4a5fb20, made inside u3-4's window, has its own line.
Product code is `crates/`, `bridge/` and `apps/`; Cargo.lock is left out.
Tests are files with a `tests/` or `fixtures/` segment or a name lean_check
counts as a test (scripts/lean_check.py:33). Raw diff lines, comments
included; inline `#[cfg(test)]` modules count as src, as lean_check counts
them (#29).

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u2-8-link-short | 597 | 382 | 215 | 463 | 273 | 190 |
| u2-9-person-parsed | 141 | 76 | 65 | 213 | 27 | 186 |
| u3-1-carve | 5 | 785 | −780 | 3 | 257 | −254 |
| u3-2-api | 817 | 167 | 650 | 701 | 71 | 630 |
| u3-3-live | 604 | 94 | 510 | 541 | 62 | 479 |
| u3-4-door | 1042 | 23 | 1019 | 931 | 0 | 931 |
| operator 4a5fb20 | 93 | 51 | 42 | 132 | 0 | 132 |
| total | 3299 | 1578 | 1721 | 2984 | 690 | 2294 |

u2-8's +215 is mostly redeem.rs's ask (+150) and cli.rs (+54). u3-1's −780
includes 352 lines of bridge/ JSON (package-lock.json 312, package.json 15,
words.json 6, and setup's) that lean_check never counted. u3-2's +650 is
rail.rs (+438) and api.rs (+139); u3-3's +510 is rail.rs (+218), live.rs
(+197) and ring_routes.rs (+61); u3-4's +1019 is door.rs (+444), lend.rs
(+280) and declared.rs (+276), the last two with 193 lines of inline tests
(#29). `python3 scripts/lean_check.py <tree>` (rc 0), trees from `git
archive`:

| at | svrngs-core | repository |
|---|---:|---:|
| 7278e08, range start | 1753 / 2500 | 6680 / 12000 |
| e2ef4ab, range end | 1796 / 2500 | 8087 / 12000 |
| 85eb5c0 (#25) | 1796 | 8079 |
| ad5a006 (#26) | 1796 | 8074 |
| 4606b07 (#27) | 1796 | 8074 |
| f6037ed (#28) | 1796 | 8065 |
| 3375e11 (#29) | 1796 | 7923 |
| 4726651 (#30), after this audit's fixes | 1796 / 2500 | 7923 / 12000 |

The fixes remove 164 repository lines: 142 moved test lines (#29), 9 from
#28, 8 from #25, 5 from #26. The bodies of 85eb5c0 and ad5a006 give 8083
and 8076, figures written before measuring; this table, measured at each
commit, replaces them.

### (2) Clones (`code dry-report`), each tier checked against a plant first

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, built
2026-10-04 08:02. The corpus came from `code index ~/dev/svrngs
--corpus-id svrngs --full` (1266 chunks); the SCIP graph from `refresh`
(rust: 5952 symbols; TypeScript export failed again, I4). On the
range-end code, unplanted, at the default 0.95: crates/svrngs-core 0 exact
and 0 near (280 locations); crates/svrngs-node 0 exact and 1 near (361);
crates/svrngs/ 0 and 0 (67); apps/expenses 0 and 0 (19). The one cluster:

- `harden_file` declared.rs:123–132 ~ `harden_file` :140–148 (0.966), both
  sides added in the range (4fb6c70). No change (#32): the `cfg(unix)` and
  `cfg(not(unix))` bodies of one function, and each 10-line chunk spans its
  neighbour too (I9).

Then the plants, re-indexed `--full` with `refresh`:

- **Plant A** (exact tier): door.rs's `person_of`, which u3-4 added, copied
  byte for byte with its doc lines to the end of live.rs. Result: **1
  group**, `person_of — door.rs:130` and `live.rs:200` (the `fn` is at
  :131, so I1 is still present).
- **Plant B1** (near tier): lend.rs's `comparable` as `normalized` at the
  end of api.rs under another doc line. **Never reported**, at any
  threshold, scoped or not (I13).
- **Plant B2** (near tier, across crates): `person_of`'s body as
  `who_stands` at the end of crates/svrngs/src/cli.rs. No per-crate run
  reports it at any threshold (I12). Unscoped: missed at 0.95, first at
  0.9, `person_of ~ who_stands` (0.919) (I5).
- With the plants removed and `--full` again: the unplanted counts above,
  exactly.

So the default near tier **could not judge**, and the scoped runs could not
judge a copy across crates. The near tier was rerun unscoped at 0.85,
where B2 showed (B1 never did): 75 clusters. A side counts as **added**
when its `fn` name is not defined in its crate at 7278e08 (`git grep -hoE
'fn [A-Za-z0-9_]+' 7278e08 -- <crate>`). Every 0.85 cluster with an added
side (expenses.js's `settleKey ~ splitShares` came up as added only because
the regex reads `fn`, not `function`; both exist at 7278e08):

| cluster (dry-report, 0.85, unscoped) | disposition |
|---|---|
| `ring_sync` ring_routes.rs:120–176 ~ added `ring_live` :177–223 (0.867) | no change (#32): both already ask one `standing_refusal`; **its doc was stale, fixed in 4726651** (#30) |
| added `standing_of` rail.rs:122–174 ~ `standing` register.rs:97–140 (0.875) | no change (#32): `standing_of` asks `standing` and maps each answer to a response |
| added `drain_answer` rail.rs:610–633 ~ `live_push` :630–646 ~ `live_drain` :643–655 (0.884–0.891) | no change (#32): a door and its two routes, overlapping chunks (I9) |
| `answer` redeem.rs:270–295 ~ added `serve_ask` :294–322 (0.857) | no change (#32): the wire's answer and one decision it answers |
| `router` redeem.rs:243–254 ~ added `ask_route` :250–263 (0.918) | no change (#32): neighbouring chunks (I9) |
| `harden_file`, `harden_dir`, `write_owner_only` declared.rs:123–153, five chunks (0.868–0.966) | no change (#32), as above |
| added `bind` api.rs:132–147 ~ `serve` :144–158 (0.876) | no change (#32): neighbours (I9) |
| added `issuer_addr` link.rs:90–102 ~ `encode` :99–117 (0.875) | no change (#32): neighbours (I9) |
| added `lends_dir` lend.rs:172–180 ~ `secret_dir` :178–186 (0.887) | no change (#32): two paths, one line each |
| added `parse` svrngs-core lib.rs:227–235 ~ `as_str` :231–239 (0.874) | no change (#32): neighbours (I9) |
| tests: `a_seal_renders_its_prune` rail/tests.rs:217–243 ~ two in tests.rs (0.858–0.862); ring_routes/tests.rs :169–209, :209–247 (0.871, 0.873); declared.rs's two owner-only tests :230–265 (0.870); door/tests.rs `map ~ identity` :3–31 (0.866); link_tests.rs `issuer ~ a_token` :1–18 (0.869); redeem/tests.rs `hal ~ request` :30–42 (0.857) | no change (#32): tests stating their own cases, or neighbouring builders; inline ones counted as production (I8) |

**Substitute check, named as a substitute.** The same pass as the u2
audits: every run of 5 normalized lines over crates/**/*.rs, test files
included, hashed; a hit is **new** when a line of either span is blamed
inside the range by `git blame -w -M -C -C`, or is uncommitted. Planted (A,
B1, B2 above): `NEW api.rs:163-168 == lend.rs:96-101`, `NEW api.rs:185-192
== lend.rs:118-125`, `NEW door.rs:129-154 == live.rs:199-224`, `NEW
door.rs:132-154 == cli.rs:685-707`, `NEW live.rs:202-224 == cli.rs:685-707`
(red: all three plants, B1 included). Unplanted, the output equals the
planted output less those five lines (green). Every new-side hit at
e2ef4ab:

| hit | disposition |
|---|---|
| svrngs cli.rs:486-490 == :551-555, read out to join_group :475-490 and new_group :543-555 (keep a group: journal, seed, origin, roster) | **fixed in f6037ed** (#28). Its body cites :476-491 and :545-557; these lines, from `git show e2ef4ab:…`, are the sites |
| rail.rs:431-439 == :638-642 (`append`'s and `live_push`'s `namespace_of` match) | no change (#32): each door's parse step, one call |
| svrngs-node tests group_not_mesh.rs:37-41 == record_api.rs:108-112; :59-72 == :55-68 | **recorded, #31** |
| record_api.rs:169-173 == :208-212, :238-243 == :353-358; ring_routes/tests.rs:98-102 == :288-292; join_by_link.rs:203-207 == :486-490, :261-265 == :494-498 | no change (#32): a request or an assertion written with its arguments in each test |

Reading the range's diff found two repeats neither pass shows, because one
is a loop with different traces and the other a one-expression predicate:
#25 and #26.

### (3) Duplicate type names (`code converge noun`)

`git diff 7278e08 e2ef4ab -- crates apps`, added lines matching
`(struct|enum|trait|type) Name`: **Format**, **Token** (link.rs);
**NotAName**, **Name**, **Person** (svrngs-core lib.rs; `Name` moved there
from svrngs group.rs and `Person` rewritten, u2-9); **AskRequest**,
**Asked**, **Issuer**, **SeedText** (redeem.rs); **Refusal**, **Bound**,
**Serving** (api.rs); **RailApi**, **RailQuery**, **LiveBuffer**,
**LiveBufferInner** (rail.rs); **LivePeer**, **PeerDelivery** (live.rs);
**Door** (door.rs); **Lend**, **LendFile**, **LendName**, **RefusedPath**,
**LendError** (lend.rs); **Printed** (cli.rs); **Pair**, **Reply**
(tests/door/main.rs). Removed: `Dial` (link.rs, u2-8).

Twenty-four production nouns have exactly one definition each. `Pair`
and `Reply` give `definitions (0)`, since the tool skips `tests` paths.
**`Refusal` gives `definitions (2)`**: crates/svrngs/src/cli.rs:180 and
crates/svrngs-node/src/api.rs:26 (the `enum`s are at :181 and :27, I1).
That real hit is the check's red; after #27 and a `refresh`, `Refusal`
gives `(1)`, cli.rs:180, and `ApiError` `(1)`, api.rs:27 (green). 4606b07's
body claims the second before it was run: the tool then answered `(0)`,
because the graph had not been refreshed. The run above is the evidence.

Read for meaning, `LendName` and a declared header's name were one rule in
two bodies (#26), and the door's `person_of` was the acceptor's question
(#25). The rest name one thing each.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 25 | 8 | crates/svrngs-node/src/node.rs:312-333 (`admit_dialer`'s loop) and crates/svrngs-node/src/door.rs:129-157 (`person_of`), at e2ef4ab | 85eb5c0 | Two deciders of "the first group held that this key stands in": same order, same rule for an unreadable register, different traces. A change to one, such as how a seedless group counts, would admit a member at the protocol and refuse them at the door, or the reverse. `RingRail::standing_anywhere` (register.rs) answers both. Both callers' tests hold it: planted `None`, door/main.rs fails 9 of 9, group_not_mesh.rs 2 of 2 and record_api.rs 5 of 9. |
| 26 | 8 | crates/svrngs-node/src/lend.rs:32-37 (`LendName::parse`) and crates/svrngs-node/src/declared.rs:30-35 (`valid_header_name`), at e2ef4ab | ad5a006 | One predicate (1..=64 bytes, ASCII letters, digits, `-`, `_`) in two bodies, for two names that each become a file under the data dir and a wire token. `LendName::parse` asks `valid_header_name`. u3-5 writes both. |
| 27 | 8 ("one name") | crates/svrngs-node/src/api.rs:27 and crates/svrngs/src/cli.rs:181, at e2ef4ab | 4606b07 | `converge noun Refusal`: two `Refusal` types, the command's and the API bind guard's. The node crate's is `ApiError` now, in line with NodeError, LendError, RedeemError and OplogError. |
| 28 | 8 ("collapse the drive") | crates/svrngs/src/cli.rs:475-490 (`join_group`) and :543-555 (`new_group`), at e2ef4ab | f6037ed | The same group-keeping sequence was written twice: open the journal, write the seed, write the origin when there is one, set the roster. It is now `keep_group`. When planted to keep no origin, join_by_link.rs fails 2 of 6 and new.rs fails 1 of 10. |
| 29 | 5 (budget), 10 | crates/svrngs-node/src/declared.rs:153-276 and crates/svrngs-node/src/lend.rs:212-280, at e2ef4ab; scripts/lean_check.py:33 | 3375e11 | #21's case again. u3-4 landed both files with their test modules inline, so 142 lean_check lines of tests counted as product code. They now live in declared/tests.rs and lend/tests.rs, as door/, rail/ and redeem/ keep theirs. The blocks are equal but for whitespace, with 9 and 4 tests before and after. This is the second time. The structural fix is a lean_check rule that counts a `mod tests {` body as test, or refuses one. That is a gate change, so it belongs to the operator. |
| 30 | 3 | crates/svrngs-node/src/ring_routes.rs:176-180 (`ring_live`'s doc), at e2ef4ab | 4726651 | The doc said a namespace with no live registration was refused. bfb6300 replaced that check with `standing_refusal`, and the route reads no registration. |
| 31 | 8 (tolerable code, not deciding) | crates/svrngs-node/tests/group_not_mesh.rs:23-72 and crates/svrngs-node/tests/record_api.rs:26-68, :108-112; crates/svrngs-node/tests/door/main.rs:35-42; crates/svrngs-node/tests/accept.rs:31-33; crates/svrngs-node/tests/common/mod.rs | not fixed | u3-2 to u3-4 copied the test fixtures `signing` (3 copies), `no_seed` (3), `no_relay` (2), `key` (3: accept.rs's builds it through iroh's `SecretKey`, which gives the same key), `start` (2), `hold` (2) and `founded_by` (2, with the founder named "founder" in one and "ada" in the other) across the node's test binaries. tests/common/mod.rs exists for this case (REVIEW-audit-u1-auto-1). Not moved here: the unshared ones would trip `dead_code` under `-D warnings` in each binary that does not use them, which needs an allow or a split of common/. Moving them is a test-only change and decides nothing. It is the next one to make when another binary needs a node. |
| 32 | 8 | the "no change" rows in (2) | no change | Each is one of: a door and the decision it asks; a `cfg` pair; neighbouring chunks of one file; a parse step written as one call; tests stating their own cases. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code dry-report` exact tier; `code converge noun` | Still present. Plant A was reported at door.rs:130 / live.rs:200 for `fn`s at :131 / :201. `Refusal` was reported at cli.rs:180 / api.rs:26 (:181 / :27). `ApiError` was reported at api.rs:27 (:28). |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". |
| I5 | 5, 7 | `code dry-report` near tier | Still present. A renamed byte-identical body (`who_stands`) is missed at the default 0.95 and first appears at 0.9 (0.919), while the report says "No duplication found". |
| I8 | 6 | `code dry-report` scope | Still present. declared.rs's inline tests appear as "first-party production". |
| I9 | 6, 7 | `code dry-report` near tier | Still present. Chunks overlap their neighbours (`harden_file` 123–132 covers `harden_dir` 129–132), so neighbours are reported as near clones: `bind ~ serve`, `router ~ ask_route`, `drain_answer ~ live_push`. |
| I10 | 6 | `code dry-report` header | Still present. Every scope's header says "764 function/method symbols". |
| I11 | 6 | `code dry-report --scope` | New. The scope is a string prefix. `--scope crates/svrngs` considered 708 locations (280 svrngs-core + 361 svrngs-node + 67 svrngs) and reported declared.rs's cluster as the svrngs crate's. Only `--scope crates/svrngs/` scopes the crate. |
| I12 | 5, 6 | `code dry-report --scope` | New. A scoped run compares only within its scope, so a copy in another crate is never seen. Plant B2 (svrngs `who_stands`, a renamed copy of svrngs-node's `person_of`) is in no per-crate report at any threshold, and the report still says "No duplication found". The row's per-crate-dir runs therefore cannot judge a clone across crates. |
| I13 | 6, 7 | `code dry-report` symbol set | New. Plant B1 was `normalized`, appended at api.rs:162: 31 lines, a renamed copy of lend.rs's `comparable`. It is absent from the report even at `--threshold 0.0 --min-lines 1` (every symbol the tool considers is listed there), after `code index --full` (1268 chunks) and `refresh`. `comparable` itself is present. The report gives no sign that a function was left out. The cause was not found. |
| I14 | 6 | `code index` incremental | New. A plant was edited after an earlier incremental run. The next run printed "3 changed file(s) since commit 32afc065" and then "0 file(s) changed — 0 chunk(s) embedded", so the edit was not embedded. Only `--full` picked it up. Every count in this section comes from a `--full` index. |

Erratum: daa0b71's body says "twenty-six new nouns". The list in (3) has
twenty-seven: twenty-five production nouns and two test nouns. Of the
production nouns, twenty-four have one definition and `Refusal` had two.

## u3-8-demo-dry (2026-10-04): a door test that fails under load

| # | principle | site | fixed in | finding |
|---|---|---|---|---|
| 33 | 2, 7 | crates/svrngs-node/tests/door/main.rs:574 (`a_member_asks_what_is_lent_and_how_many_use_it_now`), crates/svrngs-node/src/door/limit.rs:54-57 (`guess`), crates/svrngs-node/src/door.rs:503, at 4048cbb | not fixed: u3-8 changes no product code | `scripts/check.sh` failed its tests step in 2 of 3 runs on 2026-10-04 with no product change, each time `bo is using it while it streams`, `left: 0`, `right: 1`. Alone it passed 6 of 6, and the door suite alone 4 of 4. Four door binaries run at once (`for i in 1 2 3 4; do target/debug/deps/door-<hash> -q & done; wait`) failed 1 of 4 the same way. The likely cause is unconfirmed, since the door tests install no tracing subscriber: the in-process endpoints also learn this host's LAN address (192.168.1.4 in the dry run's node log), and when iroh selects that path, `guess` takes the viewer to be home and the door does not count them. The next step is to trace the selected path in the test, and then either pin the test's endpoints to loopback or count a guessed-home person in the people-using figure the list reports. Which one is the operator's call or the next door row's. |

## REVIEW-audit-u3-auto-2 (2026-10-04, range 2049cb1..7a18c02)

The previous `REVIEW-audit-` row is REVIEW-audit-u3-auto-1, hash 2049cb1
(ralph/next/u3/STATE.md:8). The range ends at 7a18c02, the commit that
gave this row its final words. It covers u3-5 to u3-8 and the operator's
commits no unit audited: 17a829b (core is no_std), commonwealth-ai-7f's
df42e63, 6a56575, 6af5ae8, dfde31b, af4a997 and de4e636 (the write side, the
lint bans and the svrngs-runtime crate), 0c2952e (process managers) and
75cbf14 (Projection). 07b541d, 364b83a, ac37105 and u3-8's commits touch no
product code. The range touched crates/svrngs-core, crates/svrngs-node,
crates/svrngs-runtime (new), crates/svrngs and apps/expenses (`git diff
--stat 2049cb1 7a18c02 -- crates apps`).

### (1) Net lines per unit, product code

`git log --reverse --numstat --format='@@%h %s' 2049cb1..7a18c02`, summed
the way REVIEW-audit-u3-auto-1 summed it: a commit counts toward the row
whose `started` commit comes before it, the operator's commits each have
their own line, product code is `crates/` and `apps/` less Cargo.lock,
tests are files with a `tests/` or `fixtures/` segment or a name
lean_check counts as a test (scripts/lean_check.py:33), raw diff lines,
renames followed. Cargo.toml lines count as src.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| operator 17a829b | 52 | 19 | 33 | 5 | 1 | 4 |
| operator df42e63 | 151 | 66 | 85 | 287 | 184 | 103 |
| operator 6a56575 | 426 | 140 | 286 | 158 | 40 | 118 |
| operator 6af5ae8 | 272 | 306 | −34 | 263 | 87 | 176 |
| operator dfde31b | 40 | 0 | 40 | 6 | 0 | 6 |
| operator af4a997 | 252 | 294 | −42 | 54 | 159 | −105 |
| operator de4e636 | 210 | 204 | 6 | 210 | 193 | 17 |
| u3-5-share | 988 | 72 | 916 | 820 | 26 | 794 |
| u3-6-limit | 478 | 57 | 421 | 471 | 12 | 459 |
| operator 0c2952e | 225 | 26 | 199 | 92 | 0 | 92 |
| operator 75cbf14 | 130 | 39 | 91 | 58 | 22 | 36 |
| u3-7-expenses | 142 | 7 | 135 | 161 | 0 | 161 |
| total | 3366 | 1230 | 2136 | 2585 | 724 | 1861 |

u3-5's +916 is cli.rs (+298), lend.rs (+180), door.rs (+158), node.rs
(+141) and share.rs (+126); u3-6's +421 is door/limit.rs (+168), door.rs
(+136), iroh.rs (+49) and lend.rs (+45); u3-7's +135 is canary.mjs (+117)
and expenses.js (+18). Of the operator's, 6a56575's +286 is core's
command.rs (+218) and redeem.rs (+82) less node's redeem.rs (−60); 0c2952e's
+199 is process.rs (+177); 75cbf14's +91 is project.rs (+114); af4a997 and
de4e636 move the journal into svrngs-runtime (lib.rs +188 and +112, node's
lib.rs −182). `python3 scripts/lean_check.py <tree>` (rc 0), trees from
`git archive`:

| at | svrngs-core | repository |
|---|---:|---:|
| 2049cb1, range start | 1796 / 2500 | 7923 / 12000 |
| 7a18c02, range end | 2359 / 2500 | 9596 / 12000 |
| 683719f (#33, the knob) | 2359 | 9613 |
| ea7363c, a16aeb8 (#33, #37: tests) | 2359 | 9613 |
| a18e52e (#34) | 2359 | 9591 |
| c0e063f (#35) | 2359 | 9601 |
| 5188193 (#36), after this audit's fixes | 2359 / 2500 | 9601 / 12000 |

c0e063f's +10 is rustfmt wrapping lines the longer name lengthened
(redeem.rs 456 → 466 lines; `git diff --word-diff` shows the rename alone).
svrngs-core went from 72% to 94% of its budget in the range (#38).

### (2) Clones (`code dry-report`), each tier checked against a plant first

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, built
2026-10-04 08:02. The corpus came from `code index ~/dev/svrngs
--corpus-id svrngs --full` (1605 chunks) and the SCIP graph from `refresh`
(rust: 7233 symbols; TypeScript export failed, I4), on the range-end code
plus 683719f..a16aeb8 (the knob and the door tests). At the default 0.95,
scoped with a trailing slash (I11): crates/svrngs-core/ 0 exact and 0 near
(350 locations); crates/svrngs-node/ 1 exact and 1 near (304);
crates/svrngs-runtime/ 0 and 0 (116); crates/svrngs/ 0 and 0 (84);
apps/expenses/ 0 and 0 (25). The two:

- exact: `try_from` lend.rs:38 == :96, 3 lines, both added in u3-5 (#39).
- near: `harden_file` declared.rs:123–132 ~ :140–148 (0.966), neither side
  added in the range; REVIEW-audit-u3-auto-1 (2) gave its disposition.

Then the plants, re-indexed `--full` (1607 chunks) with `refresh`:

- **Plant A** (exact tier): door/limit.rs's `guess` (u3-6) copied byte for
  byte with its doc lines to the end of door.rs. Result: **1 group**,
  `guess — door.rs:713` and `door/limit.rs:53` (the `fn`s are at :714 and
  :54, so I1 is still present).
- **Plant B** (near tier): process.rs's `Compact::react` (0c2952e) as a free
  `retire_of` in the same file, its locals renamed. Missed at 0.95; first at
  **0.9**, `react ~ retire_of` (0.920) (I5).
- Plants removed, re-indexed `--full` and refreshed: the unscoped 0.9
  report equals the planted one less those three clusters and the two
  symbols (`diff`: 929 → 927 symbols, 2 → 1 exact groups, 21 → 19 near).

So the near tier was run unscoped at 0.9, where B shows; unscoped also
compares across crates (I12). A side counts as **added** when `git blame -w
-M -C -C` puts any of its lines in 2049cb1..HEAD. Every 0.9 cluster with an
added side:

| cluster (dry-report, 0.9, unscoped) | disposition |
|---|---|
| `try_from` lend.rs:38 == :96 (exact) | no change (#39) |
| `append` svrngs-runtime oplog.rs:153–189 ~ `append_all` :187–224 (0.913) | **fixed in a18e52e** (#34) |
| `decide` record.rs:57–65 ~ register.rs:89–100 ~ tests_support.rs:262–270 (0.938–0.944) | no change (#39) |
| `len` act_set.rs:39–47 ~ `is_empty` view.rs:136–144 (0.906) | no change (#39) |
| tests: tests_introduce.rs :96–136 ~ :136–166 (0.913); register_tests.rs :69–85 ~ :82–106 (0.901), :103–120 ~ :117–128 (0.927); ring_sync/tests.rs `admit ~ ingest` :84–95 (0.934) | no change (#39) |

**Substitute check, named as a substitute.** The u3-auto-1 pass again
(windows of 5 normalized lines over crates/**/*.rs and apps/**/*.{js,mjs},
blank and comment lines dropped, hashed; a hit is **new** when a line of
either span is blamed inside the range or is uncommitted). With plants A
and B in: one more line than without, `NEW door.rs:713-722 ==
door/limit.rs:54-63` (A). B, its locals renamed, is not seen by this pass;
the near tier sees it. 40 new hits unplanted:

| hit | disposition |
|---|---|
| door/limit.rs:43-51 == door/main.rs:65-73 (a node started for the door tests, twice in one binary, after ea7363c) | **fixed in a16aeb8** (#37) |
| svrngs-runtime journal.rs:205-210 == :370-375 (an `Author` literal in the test-only `append` and in `seal`; lib.rs:195 is a third) | no change (#39) |
| svrngs-core admit.rs:61-66 == sync.rs:66-71 (imports); svrngs-node rail.rs:407-412 == :419-424 (`append`'s and `log`'s parse step, as #32 found for :431/:638); svrngs cli.rs:404-408 == :416-420 (`Said::Shared` printed, running or not) | no change (#39) |
| 26 in tests_introduce.rs, record_tests.rs:8-12 == register_tests.rs:43-47, rail/tests.rs:125-129 == :198-202, ring_sync/tests.rs:228-232 == :413-417, door/limit.rs:182-186 == :208-212 and :58-63 == door/main.rs:77-82, runtime tests.rs:166-171 == :631-636 | no change (#39) |
| canaries.rs:64-70 == record_api.rs:76-82; group_not_mesh.rs:58-67 == record_api.rs:56-65; svrngs join_by_link.rs:33-48 == new.rs:16-31, :47-58 == share.rs:33-43 | #31's case: fixtures copied across test binaries; not moved, for #31's reason |

Reading the range for decisions neither pass would show: the lend list's
"using" (`Counts::people`, door/limit.rs:158) and the door's limit
(`Counts::ask`, :115) both ask `Viewer::counted` (:93); the command writes,
withdraws and lists lends only through `svrngs_node::lend` (cli.rs:659,
:671, :674). Nothing found.

### (3) Duplicate type names (`code converge noun`)

`git diff 2049cb1 7a18c02 -- crates apps`, added lines matching
`(struct|enum|trait|type) Name`, less associated types (`type Error`,
`Refusal`, `Effect`) and the three moved by af4a997 (`RingJournal`,
`RingRail`, `struct Sealed`, also run: one definition each): ActSet,
Answer, Append, AppendRefusal, Author, Authored, Batch, Command, Compact,
Counts, Due, Fixture, Invited, Invites, Kind, Lends, Lent, Memory, Mine,
MintInvite, NotACommand, Observed, Offered, ProcessManager, Projection,
Reads, Recorded, Redeem, RedeemRefusal, Refused, Retire, Seal, Sealed
(trait), Share, ShareRefusal, Store, Tip, Uncaused, Upload, Using, Verdict,
Via, Whereabouts, Wire. Forty have one definition; `Mine` and
`NotACommand` give `(0)`, being test nouns. More than one:

- **`Command` (2)**: crates/svrngs/src/cli.rs:120 (`enum`, what was typed,
  at 2049cb1 too) and crates/svrngs-core/src/command.rs:37 (`trait`, the
  decider, 6a56575). **Fixed in 5188193** (#36); after `refresh`,
  `Command` (1) command.rs:37 and `Verb` (1) cli.rs:120.
- **`Refused` (2)**: crates/svrngs-core/src/command.rs:66 and
  crates/svrngs-node/src/redeem.rs:210 (the `enum` at :211, I1). **Fixed in
  c0e063f** (#35); after `refresh`, `Refused` (1) and `NotAdmitted` (1).
- **`Sealed` (3)**: crates/svrngs-core/src/process.rs:16 and
  project.rs:15, the private sealed-trait modules of `ProcessManager` and
  `Projection`, and crates/svrngs-runtime/src/lib.rs:271, the seal's
  result. No change (#39).

### #33, reproduced and settled

At 7a18c02, with a tracing subscriber added for the run and not
committed (tracing-subscriber as a dev-dependency, `door=info`), six door
binaries ran at once, three rounds: **9 of 18 failed**. limit.rs:205 failed
six times, limit.rs:179 three times and main.rs:574 (the finding's test)
once. So a second test had the same cause. The door logged "door: guessed in this computer's
home" 39 times, each `addr=Some(192.168.1.4)`. Both nodes bind iroh's
wildcard, and iroh moves the connection dialled on loopback onto this
computer's private address, which `guess` (door/limit.rs:54) takes to be
home, as DESIGN §8 "The local network is guessed" says it should. The fault
was in the tests' world, not in the door. The fix: `RelayConfig::on_loopback_only`
(683719f; planted, its test sees `[0.0.0.0:…, [::]:…]`), used by every
node of the door binary (ea7363c, a16aeb8). Same load and instrument
after: **18 of 18 passed**, with no "guessed in" line.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 33 | 2, 7 | crates/svrngs-node/tests/door/main.rs:574 and door/limit.rs:179, :205, at 7a18c02; crates/svrngs-node/src/guest.rs `relayed_endpoint_builder` | 683719f, ea7363c | Reproduced above. Cause: iroh carried the tests' members on this computer's private address, so the door guessed them home. The door tests now listen on 127.0.0.1 alone. |
| 34 | 8 ("collapse the drive") | crates/svrngs-runtime/src/oplog.rs:153-189 (`append`) and :187-224 (`append_all`), at 7a18c02 | a18e52e | de4e636 made `append` test-only, so production writes only through `append_all`, while tests wrote through a second body of the single-write-then-fsync rule. `append` now calls `append_all` with one op. Planted, with `append_all` writing no newline: 20+ tests fail, `a_logical_line_including_its_newline_is_written_once` among them. |
| 35 | 8 ("one name") | crates/svrngs-core/src/command.rs:66 and crates/svrngs-node/src/redeem.rs:211, at 7a18c02 | c0e063f | Two `Refused` types, and redeem.rs imported core's as `CommitRefused` to tell them apart (:57). The node's private one is `NotAdmitted` now. |
| 36 | 8 ("one name") | crates/svrngs/src/cli.rs:121 and crates/svrngs-core/src/command.rs:37, at 7a18c02 | 5188193 | The command's `enum Command` and core's `trait Command`. The command's is `Verb`, the word its doc already used. |
| 37 | 8 | crates/svrngs-node/tests/door/limit.rs:43-51 and door/main.rs:65-73, at ea7363c | a16aeb8 | Two node starters in one test binary, each spelling the RelayConfig #33 depends on. Now there is one, `start` in door/main.rs. |
| 38 | 5 (budget) | scripts/lean_check.py:25 (`crates/svrngs-core: 2500`); crates/svrngs-core/src/command.rs, process.rs, project.rs | raised to 3,000 by the operator, 2026-10-05 (the commit that changes `scripts/lean_check.py`'s `BUDGET`) | svrngs-core grew from 1796 to 2359 of 2500 lines in this range. 563 lines are the operator's commits (6a56575 command.rs +218, 0c2952e process.rs +177, 75cbf14 project.rs +114). 141 lines are left, so the next decider or projection added to the core may hit the ceiling. Recorded so that is not a surprise. Nothing is duplicated there: the core is 0 exact and 0 near at 0.95. |
| 39 | 8 | the "no change" rows in (2) and (3) | no change | Each is one of: serde glue that asks one `parse` (`try_from`); one `Command` impl's one-line decision each (`decide`); a struct literal around the one clock read (`Author`, `now_unix`, journal.rs:608); a door's parse step; imports; trivial accessors; a sealed-trait module per sealed trait, each with its own compiler diagnostic; tests stating their own cases. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code dry-report` exact tier; `code converge noun` | Still present: `guess` at door.rs:713 for the `fn` at :714; `NotAdmitted` at redeem.rs:210 for :211; `Command` at cli.rs:120 for :121. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". |
| I5 | 5, 7 | `code dry-report` near tier | Still present. A renamed copy (`retire_of`, 0.920) is missed at the default 0.95 while the report says "No duplication found". |
| I10 | 6 | `code dry-report` header | Still present: "927 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Still present; every scoped run here used a trailing slash. |

Erratum: a18e52e's body says "oplog.rs: -34 +11 lines". `git show --stat
a18e52e` gives 11 insertions and 33 deletions.

The row's check: `scripts/check.sh` at 374ed25, three runs in a row, each
exit 0 with every step `passed` (principles pinned, lean and RUST_ARCH, app
nouns gate, app nouns, rustfmt, clippy, tests, canaries, app laws), the
first in 2:35. On 2026-10-04, before this audit, it had failed 2 runs of 3
(#33).

## REVIEW-audit-u4-auto-1 (2026-10-05, range 3e6100c..2b6d626)

There is no earlier `REVIEW-audit-` row in this queue, so the range starts
at 3e6100c, the commit that wrote ralph/next/u4/STATE.md. It ends at
2b6d626, the commit that made this row. It covers u4-1, u4-2 and u4-3.
319dac0 and 357d07c (the operator's raises of the size budget) come before
it. 0c9ffa7, 02a3ef6 and f2250ac change only ralph/. The range touched
crates/svrngs-core, crates/svrngs-node, crates/svrngs-runtime,
crates/svrngs and web/, a new crate with its own workspace (`git diff
--stat 3e6100c 2b6d626 -- crates apps web scripts`). This audit's fixes are
3c06b23 (#40) and 7dc91ac (#41). The row names `target/debug/sovereign-cli`,
which svrngs does not build. The binary used here is
~/dev/commonwealth-ai/target/debug/sovereign-cli (built 2026-10-04 23:34),
as in every earlier audit.

The planted runs under (2) were against a working tree holding five
planted copies on top of 7dc91ac. This session's permission layer refused
to remove them, so the unplanted results were first *derived*: the planted
report less the clusters whose sides are plants. On 2026-10-05 the seat
stashed the five paths (`stash@{0}`, never to be applied). The clean
re-index, `refresh`, the unscoped 0.9 report and the noun runs were then
measured at 0127f6e with `git status --short` empty, and they confirm the
derived results; the one correction is in the table under (2).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 3e6100c..2b6d626`,
summed as REVIEW-audit-u3-auto-2 summed it: a commit counts toward the row
whose `started` commit comes before it. Product code is `crates/`, `apps/`
and `web/`, less Cargo.lock and package-lock.json. Tests are files with a
`tests/` or `fixtures/` segment, or a name lean_check counts as a test
(scripts/lean_check.py:22), or `*.test.mjs`. The counts are raw diff lines,
with renames followed. Cargo.toml and package.json count as src.
scripts/build-web.sh (+127) and scripts/check.sh (+7) are scripts, so they
are not counted here.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u4-1-browser-key | 228 | 64 | 164 | 419 | 11 | 408 |
| u4-2-link-core | 250 | 175 | 75 | 46 | 30 | 16 |
| u4-3-web-dial | 577 | 171 | 406 | 287 | 0 | 287 |
| total | 1055 | 410 | 645 | 752 | 41 | 711 |

- u4-1's +164 is mostly svrngs-core lib.rs (+80, `Device` and the
  redemption's field) and membership.rs (+37), plus svrngs-runtime
  register.rs (+16) and node.rs (+12).
- u4-2's +75 is a move: svrngs-core joining.rs +138 and svrngs issuer.rs
  +58, against svrngs-node redeem.rs −112 and the renamed link.rs −23.
- u4-3's +406 is web/src/lib.rs (+183), app.js (+90), index.html (+47),
  sw.js (+37) and Cargo.toml (+35). Its 171 removed lines are the edits
  made to the ported files after the port commit 811b137.

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive`:

| at | svrngs-core | repository |
|---|---:|---:|
| 3e6100c, range start | 2359 / 3000 | 9601 / 14000 |
| 2b6d626, range end | 2663 / 3000 | 10006 / 14000 |
| 7dc91ac, after #40 and #41 | 2663 / 3000 | 10011 / 14000 |

svrngs-core rose by 304 lines, to 89% of its ceiling. u4-1 added 117 of
them and u4-2's move added the rest. Lines leaving svrngs-node for the core
count against the core's ceiling, though not against the repository's
(+5 there).

### (2) Clones (`code dry-report`), each tier checked against a plant first

The corpus came from `code index ~/dev/svrngs --corpus-id svrngs --full`
(1661 chunks, 2m1s). The SCIP graph came from `refresh` (rust: 7450
symbols, python: 109; "typescript: skipped (export failed)", I4). Both were
run on the planted tree. The plants are the uncommitted diff the previous
session of this row left, and they are still in the tree:

- **A** (exact tier): crates/svrngs/src/issuer.rs's `addr` (u4-2), copied
  byte for byte into a `pub mod plant` at the end of the file.
- **B** (near tier): `request_bytes`, a free fn at the end of
  crates/svrngs-core/src/joining.rs. It is `RedeemRequest::message`
  (u4-2) with `self` renamed to `r` and `msg` renamed to `bytes`.
- **C** (web/, Rust): web/src/lib.rs's `dial` (u4-3), copied as
  `dial_again`, with an identical body.
- **D** (web/, JS): web/app.js's `heldKey`, copied into web/sw.js as
  `keyHeld`, with an identical body.
- **N** (noun): `pub struct Origin(pub String)` at the end of
  crates/svrngs-node/src/lend.rs. svrngs-core's link.rs:169 has the
  original.

Scoped runs at the default 0.95, each scope with a trailing slash (I11):

| scope | locations | exact | near | plants seen |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 389 | 5 | 0 | B missed |
| crates/svrngs-node/ | 300 | 4 | 1 | — |
| crates/svrngs-runtime/ | 116 | 1 | 0 | — |
| crates/svrngs/ | 70 | 1 | 1 | **A: exact `addr` issuer.rs:48 == :65, and near 0.968** |
| web/ | 6 | **could-not-judge** | 0 | **C and D missed** (I14) |

- B is missed at 0.95, 0.9, 0.8 and 0.7. At 0.7 it clusters with
  `deserialize` (view.rs:167) and `parse` (lib.rs:199), never with
  `message`. B is still in the symbol set: it is listed at `--threshold 0.0
  --min-lines 1`, joining.rs:138–151, so this is not I13. This is I5 again,
  and worse than the 0.920 of REVIEW-audit-u3-auto-2.
- In web/, the exact tier says could-not-judge ("no SCIP function symbols
  in scope"). The near tier says 0 at 0.95. At `--threshold 0.0
  --min-lines 1` it scores C, two identical 134-line bodies, at 0.402, with
  the spans `dial` 50–186 and `dial_again` 183–316, which overlap. It
  scores D at no more than 0.856.
- N: `converge noun Origin` gives **definitions (2)**, link.rs:168 and
  lend.rs:437. The definitions are at :169 and :438, so I1 is still
  present.

The near tier was then run unscoped at 0.9, as REVIEW-audit-u3-auto-2 ran
it (12 exact groups and 18 near clusters, with plant A in both). Plants
removed, re-indexed `--full` (1654 chunks) and refreshed (rust: 7441
symbols): **11 exact groups and 17 near clusters**, the planted report less
A's two, over 950 function/method symbols (954 planted). A side
counts as **added** when `git blame -w -M -C -C HEAD` puts any of its lines
in 3e6100c..HEAD. Every cluster except A's was blamed, and the 28 clean ones again at
0127f6e. Two have an added side:

| cluster (dry-report, 0.9, unscoped) | added | disposition |
|---|---|---|
| exact `signed_by_link` joining.rs:61 == :135 | 5 and 3 lines, u4-2 | no change (#43): one line on each of the two request types, `verify_hex(&self.link, &self.message(), &self.sig)`, each asking its own `message` |
| near register_tests.rs:104–120 ~ :117–141 (0.901) | 2 lines, :104–105, the closing brace and blank line of u4-1's test above it (3314033) | no change (#43): tests stating their own cases |

Correction from the clean run: the derived table also listed
register_tests.rs:138–155 ~ :152–163 (0.927) as added, by u4-1's `Device`
in the fixtures. `git blame -w -M -C -C -L 104,163 HEAD` gives 55 lines
6a56575, 3 lines 6af5ae8 and 2 lines 3314033 (:104–105), and no line of
:104–163 names `Device`. So that cluster is wholly before the range. The
other 26 clusters are blamed wholly before the range. Among them are
the trivial `as_str`, `len`, `is_empty`, `try_from` and `fmt` groups, and
the declared.rs `harden_*` group, which REVIEW-audit-u3-auto-1 (2)
disposed of. The u4-2 move kept its blame (`-C -C` traces joining.rs and
the core's link.rs back to redeem.rs and the command's link.rs).

**Substitute check, named as a substitute.** This is the earlier audits'
pass, here at /tmp/u4audit/windows.py and extended to web/. It takes
windows of 5 normalized lines over crates/**/*.rs, web/src/*.rs, web/*.js,
web/tests/*.mjs and apps/**/*.{js,mjs}. Blank lines, comment lines,
attribute lines and lone closers are dropped. The windows are hashed. A
hit is **new** when a line of either span is blamed in 3e6100c..HEAD or is
uncommitted. On the planted tree it gives 93 hits, 4 of them new:

- `NEW crates/svrngs/src/issuer.rs:48-52 == :65-69` (A)
- `NEW web/src/lib.rs:54-182 == :187-315` (C)
- `NEW web/app.js:34-41 == web/sw.js:40-47` (D)
- `NEW crates/svrngs-node/tests/browser/main.rs:85-90 == tests/door/main.rs:55-60`:
  a node started on 127.0.0.1 with `on_loopback_only`, in two test
  binaries. This is #31's case: fixtures copied across test binaries, not
  moved, for #31's reason.

B, with its names changed, is seen by neither this pass nor the near tier.
The substitute is the only instrument here that sees a copy in web/.
This pass was not rerun on the clean tree: /tmp/u4audit/ was gone by
2026-10-05, so its one unplanted new hit (browser/main.rs ~ door/main.rs)
rests on the planted run.

Reading the range for decisions that neither pass would show:

- **#40**: a group's dialled keys were decided in node.rs and in rail.rs.
  Fixed in 3c06b23.
- **#41**: the ask's and the redemption's three refusals were spelled in
  both of redeem.rs's parse steps. Fixed in 7dc91ac. Both commits name
  their plants.
- **#42**: how a token becomes the address a dial uses is decided twice.
  The command's `key` and `addr` are at crates/svrngs/src/issuer.rs:40-54.
  The page's is at web/src/lib.rs:82-110: its own key conversion, with a
  refusal where issuer.rs `expect`s, and the relay-or-lookup choice.
  Recorded, not fixed. The page cannot link the `svrngs` command (tokio
  and svrngs-node, in a separate wasm workspace), and the core has no
  iroh. So one home is a change of shape, not a small move.

The page's protocol name asks svrngs-core (web/src/lib.rs:28), and its
token is parsed by svrngs-core's `Token::decode` (:78). Both are one
decider.

### (3) Duplicate type names (`code converge noun`)

`git diff 3e6100c 7dc91ac -- crates apps web`, added lines matching
`(struct|enum|trait|type) Name`, less associated types, gives
`AskRequest`, `RedeemRequest`, `Device` and `Member`. The types u4-2 moved
by rename were also run: `Alpn`, `Format`, `Token`, `Link`, `Origin`.
web/src/lib.rs declares none. Results:

- Eight have one definition each: AskRequest (joining.rs:29), RedeemRequest
  (:68), Device (lib.rs:512), Member (svrngs-runtime register.rs:40, with
  kin `Membership`), Alpn, Format, Token and Link.
- `Origin` has 2 only because of plant N, which is listed under (2). On
  the clean tree `converge noun Origin` gives definitions (1), link.rs:168
  (the struct is at :169, I1), as do the other eight again.

No noun added in the range has more than one definition.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 40 | 8 ("collapse the drive") | crates/svrngs-node/src/node.rs:367-379 and rail.rs:571-582, at 2b6d626 | 3c06b23 | The round and the live fan-out each decided which standing keys this node dials. u4-1 added the browser test to both. `node::dialled` decides it now. Planted: one disabled branch failed both callers' tests. |
| 41 | 8 | crates/svrngs-node/src/redeem.rs:218-227 and :279-286, at 2b6d626 | 7dc91ac | `parse_ask` and `parse_redeem` each spelled the same three refusals. `presented` decides them now. Planted: the signature check disabled failed both routes' tests. |
| 42 | 8 | crates/svrngs/src/issuer.rs:40-54 and web/src/lib.rs:82-110, at 7dc91ac | not fixed | A token's key and relay become a dial address twice, the command's and the page's. They differ in kind: the page refuses where the command `expect`s, and the page falls back to iroh's lookup. One home needs a crate both workspaces can link that names iroh. That is a change of shape for a unit, not an audit's small move. Revisit when u4-4 or u4-5 touches the page's dial. |
| 43 | 8 | the "no change" rows in (2) and (3) | no change | Each is one of: a one-line verify on each of two request types; tests stating their own cases; a node starter copied across test binaries (#31); a same-family name (`Member` and `Membership`). Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: `Origin` at link.rs:168 for :169, `AskRequest` at joining.rs:29 for :30. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". |
| I5 | 5, 7 | `code dry-report` near tier | Still present and worse. A free-fn copy of a method with two names changed (plant B) is missed down to 0.7. |
| I10 | 6 | `code dry-report` header | Still present: "954 function/method symbols" in every scope (950 on the clean tree). |
| I11 | 6 | `code dry-report --scope` | Still present; every scoped run here used a trailing slash. |
| I14 | 6, 7 | `code dry-report --scope web/` near tier | New. web/ is a separate cargo workspace with JS beside it, so it has no SCIP symbols. The exact tier says could-not-judge, as it should. The near tier says "0 near-clone clusters" at 0.95 with two identical 134-line Rust bodies (plant C, cosine 0.402) and two identical JS bodies (plant D, ≤0.856) in scope. Its spans are misaligned (`dial` 50–186 overlaps `dial_again` 183–316). A 0 from a tier that cannot see what it scores is a silent pass. For web/, only the substitute pass judges. |

The row's check (LINT): `scripts/check.sh` at 0127f6e on the clean tree,
exit 0 in 4:32, every step `passed` (principles pinned, lean and RUST_ARCH,
app nouns gate, app nouns, rustfmt, clippy, tests, canaries, web rustfmt
and clippy, browser, app laws). 3c06b23 and 7dc91ac each name the tests
their plants failed.

## u4-7-webkit (2026-10-05): what WebKit needs

Every browser test now runs in Playwright 1.59.1's WebKit 26.4 as an
iPhone 15, as well as in Chromium (977e589). What the row named passed in
WebKit with no change to the page: a service worker's streamed 206 to
`<video>` (the H.264 film's first frame in 262 ms, seeks 262/261/283 ms,
a second computer's range 206 and sha-exact); IndexedDB in the worker (it
unseals the key from the store when it starts again); WebCrypto's AES-GCM
(the joining test's seal non-extractable, the key's bytes in no storage).

| # | principle | site | fixed in | finding |
|---|---|---|---|---|
| 44 | 2, 7 | web/app.js:203-218 at 977e589 (the opening at the module's top) | e0d44d1 | WebKit wedged the tab for good when a person left the page while its module awaited the wasm's fetch at its top, then opened it again: the next navigation never reached the origin. Bisected on copies of the page (e0d44d1's body has the runs): a module awaiting any `fetch` body at its top wedges within 2-4 transitions; the same from a function it does not await, 0 of 48. Fixed small in the page. Unverified on a phone: whether iOS Safari wedges the same way is for u4-8's package to record (leave the page while it first loads, then reopen the link in the same tab). |
| 45 | 6, 7 | web/store.js:56-62 (`sealingKey`, the seal put in IndexedDB) | not fixed: not the page's | On this host's first WebKit run, two tests at once failed `DataCloneError: The object can not be cloned.` WebKit wraps a CryptoKey it stores with a master key from the macOS keychain ("Playwright WebCrypto Master Key", account `com.apple.WebKit.WebCrypto.master+org.webkit.Playwright`, made 2026-10-05 15:46:52Z during that run). Reproduced: with that item deleted, three WebKits storing a key at once, 1 of 3 failed `DataCloneError`; with it kept, 0 of 3. So `scripts/check.sh` can fail once on a host whose user never ran Playwright's WebKit. A phone's Safari makes its master key once, so two first stores racing is unlikely there, and if one fails the page says "This browser could not open the page." and the next opening works. |
| 46 | 6, 7 | web/tests/runtime.mjs (the worker stopped mid-play), crates/svrngs-node/tests/browser/main.rs (`stopped_and_ran_again`) | not fixed: Playwright's limit | Playwright reaches no WebKit service worker: no DevTools (`CDP session is only available in Chromium`) and no worker console (the WebKit run's log lines are all the page's). So WebKit's runtime run closes the browser mid-play and opens it again on its profile, and the film playing again at the lend's address is the evidence the worker reopened its endpoint. Chromium alone shows the worker stopped while its page plays, and counts the endpoint opened twice. Measured for call 3: with no page open, WebKit stopped an idle worker within 12-20 s, a new run each time (a probe worker answering an id made when it starts, 20 of 20); Chromium kept its worker across 20 s idle (7 of 7). Whether iOS stops a worker while its page plays is for u4-8's phone. |

Erratum to #46 and ef8fa76's body: Chromium kept its worker across 20 s
idle 3 of 3 times (7 transitions in all, every one the same run), not
"7 of 7" or "0 of 7". WebKit's 20 of 20 are the minimal probe workers'
(a classic and a module worker, 2 passes each, 5 idle waits of 12 s a
pass); the page's own worker restarted at every idle wait of 12 s or more
too, until a run wedged (#44).

## REVIEW-audit-u4-auto-2 (2026-10-05, range a7497f9..61d44e4)

The range starts at a7497f9, the hash REVIEW-audit-u4-auto-1's row carries
(ralph/next/u4/STATE.md). It ends at 61d44e4, the commit that made this
row. It covers u4-4, u4-5, u4-6 and u4-7. 8ff154c and 1397c39 change only
docs. d4c390b and the later 401d185 change only ralph/next/u5/, from the
other loop. The range touched crates/svrngs-core, crates/svrngs-node,
crates/svrngs and web/ (`git diff --stat a7497f9 61d44e4 -- crates apps
web scripts`). It did not touch crates/svrngs-runtime. This audit's fix
is 9b8fe7d (#47), with its test in a4e1326 and 689de62. The row cites
`docs/ARCH_PRINCIPLES.md`, which is at docs/principles/ARCH_PRINCIPLES.md.
It names `target/debug/sovereign-cli`, which svrngs does not build. The
binary used is ~/dev/commonwealth-ai/target/debug/sovereign-cli (built
2026-10-05 07:09).

This time the plants never touched this tree. They went into a clone,
/tmp/u4audit2/svrngs-plant (`git clone` of 7a3a499 plus one commit holding
the plants), indexed as its own corpus, `svrngs-plant`. The clean runs are
on this tree at 7a3a499 with `git status --short` empty, corpus `svrngs`.
7a3a499's code is 61d44e4's.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' a7497f9..61d44e4`,
summed by /tmp/u4audit2/ledger.py with REVIEW-audit-u4-auto-1's rules. A
commit counts toward the row whose `started` commit comes before it.
Product code is `crates/`, `apps/` and `web/`, less Cargo.lock. Tests are
files with a `tests/` or `fixtures/` segment, or a lean_check test name,
or `*.test.mjs`. So web/tests/*.mjs and crates/svrngs/tests/join_page.rs
count as tests. scripts/check.sh (+6 −3) and scripts/build-web.sh (+1 −1)
are scripts, so they are not counted.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u4-4-web-http | 319 | 66 | 253 | 281 | 16 | 265 |
| u4-5-join-page | 825 | 301 | 524 | 580 | 107 | 473 |
| u4-6-runtime | 267 | 75 | 192 | 401 | 48 | 353 |
| u4-7-webkit | 18 | 11 | 7 | 272 | 128 | 144 |
| total | 1429 | 453 | 976 | 1534 | 299 | 1235 |

- u4-4's +253 is web/src/http.rs (+232, hyper's HTTP/1 client over a
  bi-stream), with web/Cargo.toml +11 and lib.rs +9.
- u4-5's +524: svrngs-core seed.rs +147 is a move (d79391a) that pairs
  with svrngs group.rs −122. The rest is web/app.js +125, node.rs +122
  (the people route), web/src/join.rs +107, web/store.js +83, lib.rs +26,
  joining.rs +15, door.rs +11 (9e4b368's `offered`) and redeem.rs −13.
- u4-6's +192 is web/sw.js +108 (the runtime), lib.rs +72 (d675630's
  endpoint and dial split) and store.js +10.
- u4-7's +7 is web/app.js (e0d44d1). Its tests are WebKit runs of every
  browser test (977e589).

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive`:

| at | svrngs-core | repository |
|---|---:|---:|
| a7497f9, range start | 2663 / 3000 | 10011 / 14000 |
| 61d44e4, range end | 2766 / 3000 | 10624 / 14000 |
| 9b8fe7d, after #47 | 2766 / 3000 | 10623 / 14000 |

svrngs-core rose by 103 lines, to 92% of its ceiling. All of it is u4-5's
move of the seed and the issuer's answer out of the `svrngs` command
(d79391a).

### (2) Clones (`code dry-report`), each tier checked against a plant first

**Clean corpus.** `code index ~/dev/svrngs --corpus-id svrngs --full`:
1772 chunks in 152 s. `refresh`: rust 7689 symbols and python 109;
"typescript: skipped (export failed)" (I4).

**Planted corpus.** The same at /tmp/u4audit2/svrngs-plant, `--corpus-id
svrngs-plant`: 1780 chunks, rust 7733 symbols. The plants are each a copy
of a symbol the range added:

- **A** (exact tier): node.rs `people_lends` (u4-5, :541-586), copied
  whole into a `pub mod plant` at the end of node.rs.
- **B** (near tier): node.rs `Host::lends_in` (:382-422, 18 of its 41
  lines blamed to u4-5's ecbdbf6), as
  a free fn `gather_lends` in lend.rs. `self` became `host`, `peers`
  became `others`, and `asks` became `pending`.
- **C** (web/, Rust): web/src/http.rs `request` (u4-4, :125-167), copied
  into web/src/join.rs as `request_again`, its body identical.
- **D** (web/, JS): web/sw.js `endpointFor` (u4-6, :96-106), copied into
  web/app.js as `endpointOf`, its body identical.
- **N** (noun, main workspace): `pub struct Lending` at the end of
  svrngs-runtime register.rs. node.rs:514 has the original.
- **N2** (noun, web/): `struct Asking` at the end of web/src/join.rs.
  node.rs:524 has the original.

Scoped runs at the default 0.95, each scope with a trailing slash (I11):

| scope | clean: locations / exact / near | planted: exact / near | plants seen |
|---|---|---|---|
| crates/svrngs-core/ | 398 / 5 / 0 | 5 / 0 | — |
| crates/svrngs-node/ | 305 / 4 / 1 | 5 / 1 | **A: exact `people_lends` node.rs:541 == :617**; B missed at 0.95 |
| crates/svrngs-runtime/ | (not in range) | 1 / 0 | — |
| crates/svrngs/ | 59 / 0 / 0 | 0 / 0 | — |
| web/ | 49 / **could-not-judge** / 0 | could-not-judge / 0 | C and D missed at 0.95 (I14) |

- **B** is found at 0.9: `gather_lends` lend.rs:434–477 ~ `lends_in`
  node.rs:379–425, cosine 0.937. It is missed at 0.95. At 0.8 it joins
  `ask_lends` and both `people_lends`. Listing at `--threshold 0.0
  --min-lines 1` shows `gather_lends` in the symbol set. I5 is better than
  REVIEW-audit-u4-auto-1's miss down to 0.7, and still misses at the
  default.
- **C** is found at 0.9: `request` http.rs:122–167 ~ `request_again`
  join.rs:106–151, cosine 0.934 for two identical 46-line bodies. It is
  missed at the default 0.95.
- **D** is never paired. At 0.8 `endpointOf` clusters at 0.826 with
  something else. At 0.7 it shares a 12-symbol cluster with `endpointFor`.
  The web/ near tier still cannot see an identical JS body (I14).
- **N**: `converge noun Lending` gives **definitions (2)**, node.rs:513
  and register.rs:277. **N2** is missed: `converge noun Asking` gives
  definitions (1), node.rs:523. The noun tier has no sight of web/ (I15).

The near tier on the clean corpus, unscoped at 0.9 as the last two
audits ran it: **11 exact groups and 17 near clusters** over 1003
function/method symbols (950 at the last audit). /tmp/u4audit2/blame.py
blames every side of every cluster with `git blame -w -M -C -C HEAD` and
counts the lines in a7497f9..61d44e4. **No side of any cluster has a line
added in the range.** The counts are the ones REVIEW-audit-u4-auto-1
measured on its clean tree, and both clusters its table disposed of are
here. `signed_by_link` is joining.rs:63 == :150 now, for :61 == :135,
because u4-5's joining.rs edits (+16 −1) sit between and above them.

**Substitute check, named as a substitute.** /tmp/u4audit2/windows.py
redoes the earlier audits' pass: windows of 5 normalized lines over
`git ls-files crates web apps` matching `*.rs`, `*.js` or `*.mjs`. Blank
lines, comment lines, attributes and lone closers are dropped, and each
window is hashed. A hit is **new** when either span has a line blamed in
the range.

- On the planted clone (range a7497f9..HEAD there): 178 hits, 10 new. A
  (node.rs:541-585 == :617-661), B (lend.rs:455-466 == node.rs:400-411
  and :471-476 == :416-421), C (http.rs:126-163 == join.rs:110-147) and
  D (app.js:228-235 == sw.js:97-104) are each found. The other five new
  hits are the clean ones below.
- On this tree at 7a3a499: 173 hits, 5 new, all of them in tests:
  - crates/svrngs-node/tests/browser/main.rs:176-182 == :274-280: two
    tests' `drive(engine, "drive.mjs", json!({ site, key, token, …`.
  - web/tests/drive.mjs:122-127 == join.mjs:87-92, and drive.mjs:158-163
    == join.mjs:135-140 == runtime.mjs:131-136: each Playwright driver's
    `launch`, its console and pageerror listeners, and its
    `catch`/`finally` that prints the logs and closes. page.mjs already
    shares the launch, the serve and the store (3aa92be, ee8b306). What
    is left is each driver's own frame, and runtime.mjs's `finally` also
    removes its profile. No change (#50).

Reading the range for decisions that neither pass would show:

- **#47**: the 500 for unreadable declarations was built by both routes
  that list lends: door.rs `lends` :283-288 and node.rs `people_lends`
  :560-565 at 7a3a499. No test reached either. Fixed in 9b8fe7d, after
  a4e1326's test of both routes, which went red on a plant at each site.
- **#42, still open, and grown**: a token becomes the address a dial
  uses at crates/svrngs/src/issuer.rs:40-54 and web/src/lib.rs:136-241
  (`bind` and `reach`, after d675630's split). u4-5 added a second copy
  *inside* svrngs-web (#48).
- **#48**: svrngs-web decides "these bytes are the person's key" and
  "this fragment is a link's token" twice, with the same words. join.rs
  `person` :29-33 and `link` :35-37 do it, and so does lib.rs `bind`
  :140-152. They refuse with different trace prefixes (`JOIN: REFUSED —`
  and `LAYER key/parse: REFUSED —`), and the key comes out as a different
  type (svrngs-core's `SigningKey` or iroh's `SecretKey`). Giving it one
  home changes the trace lines, so it is not a pure move. Recorded.
- **#49**: a lend's address under the worker's scope, `l/<lender>/<lend>/`,
  is built by the page at web/app.js:150 (`open`) and parsed by the worker
  at web/sw.js:66 (`LENDS`) and :119 (the regex). That is two processes
  hand-deriving one path (principle 8, "one accessor per path"). The
  browser tests catch a divergence end to end, because the runtime and
  joining tests navigate by the list. One home is a module both import.
  store.js is the key's store, not this, and a new module is a shell
  file in four places: sw.js `SHELL`, build-web.sh:113-115 (`cp` and the
  version stamp), and the two imports. Recorded for the next unit that
  touches the runtime.
- The page's `PEOPLE_LENDS` (app.js) spells node.rs `PEOPLE_LENDS_ROUTE`,
  and sw.js's `/lend/<lend>` spells door.rs `LEND_ROUTE`. Each is a
  protocol constant across the Rust/JS line, cited at the JS site and
  exercised by the browser tests. No change.

### (3) Duplicate type names (`code converge noun`)

`git diff a7497f9 61d44e4 -- crates apps web`, added lines matching
`(struct|enum|trait|type) Name`. New: `Lending`, `Asking` (node.rs),
`Engine` (tests/browser/harness.rs), `Refused`, `BodyError`, `Duplex`,
`RequestBody`, `PageBody` (web/src/http.rs), `Group` (web/src/join.rs),
`Endpoint`, `Bound`, `Dialled` (web/src/lib.rs). Moved by d79391a:
`Asked`, `Seed`, `SeedFile`. The JS `class Said` (app.js) has no SCIP
symbol. Each name was run, and then `git grep -nwE '(struct|enum|trait|type)
<Name>' -- crates apps web` as the substitute for web/ (I15):

- One definition by both: Asked (joining.rs:72), Seed (seed.rs:30, with
  kin SeedError, SeedFile, SeedText), SeedFile (:125), Lending
  (node.rs:514), Asking (:524). converge cites each a line early (I1).
- `Engine` is test-only, and converge excludes tests: definitions (0).
- web/'s nouns: converge gives definitions (0), "none — check the
  spelling", for BodyError, Duplex, RequestBody, PageBody, Group,
  Endpoint and Dialled. git grep finds one definition each.
- **Two definitions, by git grep only.** converge saw one side of each:
  - `Refused`: svrngs-core command.rs:67, `Refused<R>` (why a commit
    wrote nothing), and web/src/http.rs:41 (why a request got no
    response).
  - `Bound`: svrngs-node api.rs:131 (a loopback listener bound, not yet
    served) and web/src/lib.rs:121 (a browser endpoint bound with the
    person's key).

  Each pair is two concepts that share a short name, in two cargo
  workspaces, and neither decides what the other does. No change (#50).
- `Lending` (node.rs:514: person, key, offered) and `Lent` (:79: the
  same plus `address`) are the people route's wire answer and
  `Node::lends`'s answer. A browser has no loopback bridge, so it gets no
  address. Kin, not one noun twice. No change (#50).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 47 | 8 | crates/svrngs-node/src/door.rs:283-288 and node.rs:560-565, at 7a3a499 | 9b8fe7d | Both routes that list lends built the 500 for unreadable declarations from `offered`'s `None`. `offered` returns the refusal now. a4e1326's test reaches both routes. Each site's plant failed it, and so did the plant at the one home. |
| 48 | 8 | web/src/join.rs:29-37 and web/src/lib.rs:140-152, at 61d44e4 | not fixed | svrngs-web reads the person's 32 bytes and the link's token twice, with the same refusal words under different trace prefixes, into different key types. One home changes the trace lines. Revisit with #42. |
| 42 | 8 | crates/svrngs/src/issuer.rs:40-54 and web/src/lib.rs:136-241, at 61d44e4 | not fixed | As recorded at REVIEW-audit-u4-auto-1. u4-4 to u4-7 touched the page's dial (d675630 split `bind` and `reach`) and did not take it up. It is still a change of shape: a crate both workspaces can link that names iroh. |
| 49 | 8 ("one accessor per path") | web/app.js:150 and web/sw.js:66, :119, at 61d44e4 | not fixed | The page builds a lend's address under the worker's scope and the worker parses it, each by hand. The browser tests would go red on a divergence. One home is a shared module, which is a shell file in sw.js `SHELL` and build-web.sh:113-115 too. Recorded for the next runtime unit. |
| 50 | 8 | the "no change" items in (2) and (3) | no change | Each is one of: a Playwright driver's own frame (launch, listeners, catch/finally) beside page.mjs's shared parts; two tests' `drive` calls; one short name for two concepts in two workspaces (`Refused`, `Bound`); kin types (`Lending`/`Lent`); a protocol constant across the Rust/JS line, exercised end to end. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: Asked at joining.rs:71 for :72, Seed at seed.rs:29 for :30, Lending at node.rs:513 for :514, Bound at api.rs:130 for :131. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". |
| I5 | 5, 7 | `code dry-report` near tier | Still present, and better. Plant B, a method made a free fn with four names changed, scores 0.937. It is found at 0.9 and missed at the default 0.95. |
| I10 | 6 | `code dry-report` header | Still present: "1003 function/method symbols" in every scope (1007 planted). |
| I11 | 6 | `code dry-report --scope` | Still present; every scoped run here used a trailing slash. |
| I14 | 6, 7 | `code dry-report --scope web/` near tier | Still present, and partly better. Two identical 46-line Rust bodies (C) score 0.934, so they are missed at the default 0.95 and found at 0.9. Two identical JS bodies (D) are never paired above 0.7. The exact tier still says could-not-judge, which is right. |
| I15 | 6, 7 | `code converge noun` over web/ | New. web/ is a separate cargo workspace, so its types have no SCIP symbols. converge reports "definitions (0) … none — check the spelling" for a type that exists (`Dialled`, web/src/lib.rs:246). For a name both sides define, it counts only the main workspace's side: `Bound` (1) and `Refused` (1), each with a second definition in web/, and plant N2's `Asking` (1). That is a silent pass, not a could-not-judge. For web/, only `git grep` judges. |

The row's check (LINT): `scripts/check.sh` at b9e2bf3 on a clean tree
exited 0 in 9:35. Every step `passed`: principles pinned, lean and
RUST_ARCH, app nouns gate, app nouns, rustfmt, clippy, tests, canaries,
web rustfmt and clippy, browser, joining in a browser, app laws. That is
up from 4:32 at the last audit, because every browser test now runs in
WebKit too (977e589). It is within 25 s of a ten-minute session's
foreground limit.

## u4-8-phone-dry (2026-10-05): what a phone will pay

U4's dry run (`demo/runs/u4/2026-10-05/`, its README has the table):
Playwright 1.59.1's Chromium 147 and WebKit 26.4 as an iPhone 15, three cold
runs of each through a relay on this computer and through n0's.

| # | principle | site | fixed in | finding |
|---|---|---|---|---|
| 51 | 7 | web/sw.js:28-34 (`SHELL`, the wasm with `?v=`), wasm-bindgen's glue fetching `svrngs_web_bg.wasm` without it | 0cb236c (u4-9): one URL with the version, one cache entry the page fills and the worker reads (web/store.js `runtime`); a cold join fetched it once, 5,335,290 bytes from the origin (`demo/runs/u4/2026-10-05-reach/`) | A cold join took 15,350,659 bytes from the origin, when the page is 5,103,571: the wasm (5,027,239) fetched by the page, by the worker's install as `…wasm?v=<version>`, and by the worker's `init()`, which no cache answers (`data/runs-*.jsonl` `origin_bytes`; the origin's log, three wasm lines a Chromium run, two a WebKit run whose worker never opened a lend). The test origin sends no cache headers; an origin marking the wasm immutable should save the third copy, unmeasured. The module is not `wasm-opt`ed on this host (no binaryen); upstream's was 4,295,444 bytes. |
| 52 | 6, 7 | iroh-1.0.2/src/defaults.rs:27-33, :48 (n0's relays spelled `use1-1.relay.n0.iroh.link.`), reached from web/src/lib.rs's endpoint | ea651e0 (u4-9; the operator chose to fix first): every relay URL dotless where it enters (svrngs_core::relay, crates/svrngs-node/src/relay.rs, which the page compiles); WebKit joined through n0 in 3 of 3 runs (`demo/runs/u4/2026-10-05-reach/`) | WebKit refuses TLS to a host with a trailing dot, and every n0 relay is spelled with one. All three WebKit runs through n0 ended "The computer that sent this link did not answer" after 60 s, the page's log holding `https://<each of n0's four relays>./ping: The server did not accept the certificate.` Probed in both engines from an https page: `https://use1-1.relay.n0.iroh.link/ping` loads and `…link./ping` fails in WebKit, both load in Chromium; `https://example.com./` fails in WebKit too ("A TLS error caused the secure connection to fail."). So an iPhone, whose every browser is WebKit, should not reach a group on n0's relays. The browser tests never could show it: they run a relay on 127.0.0.1, dotless. |
| 53 | 7 | web/sw.js `lent` (each answer's body streamed as the door gives it), a `<video>`'s open-ended range | b6ea0ab, 3c4873b (u4-9): a range that runs on, open or a `<video>`'s longer than 2 MiB, asked of the door as a 2 MiB part (web/sw.js `PART`); every WebKit seek played on within 3.4 s through n0 (`demo/runs/u4/2026-10-05-reach/`). Corrected by u4-9: WebKit's first range was closed, `bytes=66394-45245597` (data/lend-local.log:25), not open as written in this row | In WebKit an early seek far ahead waited for the open range to be read: the first range, `bytes=66394-`, ran to the film's end (45,179,204 bytes in 26.2 s and 23.3 s, runs 1 and 3, `data/lend-local.log`), and the seek to 60 s asked 1.1 s after play played on only after it: past 30 s, and 29.0 s; in run 2 the range was cut at 23.9 MB after 3.9 s and the seek played on in 3.4 s. Chromium cut every open range within 0.9 s and asked anew; all its 30 seeks played on within 2.0 s. Whether WebKit holds the next range back or the worker's stream ignores WebKit's cancel is not shown: Playwright reaches no WebKit worker (#46). On a film of gigabytes such a seek waits for the bytes between. |

## REVIEW-audit-u4-auto-3 (2026-10-05, range 584406f..b56d42b)

The range starts at 584406f, the hash REVIEW-audit-u4-auto-2's row carries
(ralph/next/u4/STATE.md). It ends at b56d42b, the commit that made this
row. It covers u4-8, u4-9 and u4-10, and the scale-harness branch merged
at 8923106 (cd751ab, 384e007, c1e07ac, 6493a8c; a clean merge, `git show
--cc 8923106` shows no hunk of its own). The range touched
crates/svrngs-core, crates/svrngs-node, crates/svrngs and web/ (`git diff
--stat 584406f b56d42b -- crates apps web scripts`). This audit's fix is
ac3ade0 (#54). The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md, and names `target/debug/sovereign-cli`,
which svrngs does not build: the binary is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-05 07:09,
the one REVIEW-audit-u4-auto-2 planted against (sha256 6501033e…b92ae).

**Not planted again.** The binary is the one whose tiers the last audit
measured on plants A-D, N and N2. Those results (A exact; B and C only at
0.9; D never; N by converge; N2 missed, I15) are this run's limits too,
so each clean run below is read with them, and web/ is judged by the
window pass and `git grep` as well.

Every run is on this tree at dd92406 (b56d42b plus this row's `started`
line), `git status --short` empty.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 584406f..b56d42b`,
summed by /tmp/u4audit3/ledger.py with the last audit's rules (a commit
counts toward the row whose `started` commit comes before it; product
code is `crates/`, `apps/`, `web/` less Cargo.lock; tests are a `tests/`
or `fixtures/` segment, a lean_check test name, or `*.test.mjs`). One
rule is new: a commit off the first-parent line counts toward the branch
its merge names, so scale-harness's commits, which `git log` lists inside
u4-9, are a row of their own. scripts/build-web.sh (+13 −7) is a script,
not counted.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u4-8-phone-dry | 0 | 0 | 0 | 91 | 0 | 91 |
| u4-9-webkit-reach | 351 | 41 | 310 | 341 | 18 | 323 |
| scale-harness (merged 8923106) | 20 | 1 | 19 | 342 | 0 | 342 |
| u4-10-safari-worker-key | 170 | 21 | 149 | 254 | 33 | 221 |
| total | 541 | 63 | 478 | 1028 | 51 | 977 |

- u4-8's code is its test, crates/svrngs/tests/phone_dry.rs (+91); its
  dry run is demo/, not counted.
- u4-9's +310: crates/svrngs-node/src/relay.rs +109 and svrngs-core
  relay.rs +46 (#52's dotless relay, one body: the node's file wraps
  core's `dotless`, and web/src/lib.rs compiles the node's file by
  `#[path]`), web/sw.js +58 (#53's `PART`), web/store.js +46 (#51's
  `runtime`), web/src/lib.rs +15, guest.rs +13, link.rs +5, cli.rs +5,
  web/Cargo.toml +5.
- scale-harness's +19 is node.rs +16 (`Node::start_every`) and
  svrngs-node Cargo.toml +3; its test is tests/scale.rs +342.
- u4-10's +149: web/sw.js +64 (the trace, the handed key, the hand-over
  page), web/store.js +37, web/hand.js +37 (new), web/app.js +11. Its
  tests: web/tests/handover.mjs +119, browser/main.rs +98, and
  ecd5f72's move of `play` from runtime.mjs (−29) into page.mjs (+33).

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive`
in /tmp/u4audit3:

| at | svrngs-core | repository |
|---|---:|---:|
| 584406f, range start | 2766 / 3000 | 10623 / 14000 |
| b56d42b, range end | 2789 / 3000 | 11296 / 14000 |

svrngs-core rose 23 lines, to 93% of its ceiling: relay.rs and link.rs's
dotless read (u4-9). The repository rose 673, to 81%.

### (2) Clones (`code dry-report`)

`code index ~/dev/svrngs --corpus-id svrngs --full`: 1852 chunks in 155 s.
`refresh`: rust 7943 symbols, python 157, "typescript: skipped (export
failed)" (I4). Scoped runs at the default 0.95, each with a trailing
slash (I11), then /tmp/u4audit3/blame.py blaming every side of every
cluster (`git blame -w -M -C -C HEAD`) for lines in 584406f..b56d42b:

| scope | locations | exact | near | a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 401 | 5 | 0 | none |
| crates/svrngs-node/ | 319 | 4 | 1 | none |
| crates/svrngs/ | 59 | 0 | 0 | — |
| web/ | 59 | **could-not-judge** | 0 | — (I14) |

Unscoped at 0.9, as the last three audits ran it: 11 exact groups and 18
near clusters over 1046 function/method symbols (1003 at the last audit;
11 and 17 then). The one new cluster is the only one with a side added in
the range, and web/ at 0.9 finds it too:

```
- **2 symbols** · ~14 lines · cosine 0.917–0.917
  - `keepTrace` — web/sw.js:88–100 (13 lines)      added=12/13 ['0549de8']
  - `traceView` — web/sw.js:96–109 (14 lines)      added=13/14 ['0549de8', 'b24759b']
```

The spans are misaligned again (I14): keepTrace was :91-97 and traceView
:99-106 at b56d42b, so each span takes in three lines of the other. What
the two share is the plain-text content type, spelled at both and a third
time in `say` (:208). **#54, fixed in ac3ade0**: one const, `PLAIN`.

**Substitute check, named as a substitute.** /tmp/u4audit3/windows.py
(the last audit's windows of 5 normalized lines over Rust and JS, a hit
new when either span has a line blamed in the range): 187 hits, 5 new,
all in tests:

- crates/svrngs-core/src/link_tests.rs:72-85 against
  crates/svrngs-core/src/relay_tests.rs:36-58 and
  crates/svrngs-node/src/relay_tests.rs:103-111: given/then rows of the
  dotless relay ("a relay of one's own, dotted", "a loopback relay"), one
  table at each place a relay enters: core's `dotless`, a token's
  `decode`, the node's `parse`. Each tests its own entry point. No change
  (#55).
- web/tests/handover.mjs:21-26 == web/tests/runtime.mjs:23-28: two
  drivers' imports and their `args` line. No change (#55).

Reading the range for what neither pass would show:

- **#49, still open, and grown.** u4-10 parses a lend's address a third
  time: web/hand.js:21 (`/^l\/([0-9a-f]{64})\//`, the lender), beside
  the page's build at web/app.js:155 and the worker's parse at
  web/sw.js:74 (`LENDS`) and :234. hand.js is a new shell file
  (sw.js `SHELL`, build-web.sh's `PAGE`) and imports store.js, as the
  page and the worker do; one home for the address is still a module all
  three import. Recorded.
- **#42, still open.** A token becomes the address a dial uses at
  crates/svrngs/src/issuer.rs:41-54 (`key`, `addr`) and web/src/lib.rs
  `bind` :146 and `reach` :223. u4-9 touched `bind` (n0's relays and the
  lookup, dotless) and did not take it up. It did show a shape short of a
  new crate: web/src/lib.rs compiles crates/svrngs-node/src/relay.rs by
  `#[path]`, so the page and the node share one body that names iroh,
  each workspace with its own pins. #42 could land the same way.
- One relay normaliser, three entries: svrngs-core relay.rs `dotless`
  decides; the node's relay.rs `dotless`/`parse`/`n0_relays`/
  `DotlessLookup` wrap it for iroh's types; link.rs:140 calls core's for
  a token. Nothing decided twice.
- The page's sealing key: only `keep` makes one (store.js `sealingKey(true)`
  at :87; `unseal` passes `false` at :120). One decider for "who makes the seal".
- `Node::start` now forwards to `Node::start_every` with
  `DEFAULT_RING_SYNC_INTERVAL` (node.rs:148): one body.

### (3) Duplicate type names (`code converge noun`)

`git diff 584406f b56d42b -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: `DotlessLookup`, `Dotless`
(crates/svrngs-node/src/relay.rs:57, :70) and the JS `class Keyless`
(web/sw.js:131). Each was run, then `git grep -nwE
'(struct|enum|trait|type|class) <Name>' -- crates apps web`:

- `DotlessLookup`: definitions (1), crates/svrngs-node/src/relay.rs:56
  (a line early, I1). git grep: one.
- `Dotless`: definitions (1), relay.rs:69; morphological kin
  `DotlessLookup`. The two are iroh's builder/built pair
  (`AddressLookupBuilder` and `AddressLookup`), each wrapping its own
  side. Kin, not one noun twice. No change (#55).
- `Keyless`: definitions (0), "none — check the spelling" (I15; JS has no
  SCIP symbol). git grep: one, web/sw.js:131.

The relay.rs that web/src/lib.rs compiles by `#[path]` is one file, so
its two nouns have one definition in both workspaces.

No noun has more than one definition.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 54 | 8 | web/sw.js:95, :101, :208, at b56d42b | ac3ade0 | The worker spelled its plain-text content type three times: keepTrace and traceView (both 0549de8) and `say`. dry-report paired the first two at 0.917. One const, `PLAIN`; the browser tests read the trace through it. |
| 49 | 8 ("one accessor per path") | web/hand.js:21, web/app.js:155, web/sw.js:74 and :234, at ac3ade0 | not fixed | As recorded at REVIEW-audit-u4-auto-2, and grown: u4-10's hand-over page parses the lend's address too, so three files hand-derive it. One home is a module all three import, a shell file in sw.js `SHELL` and build-web.sh's `PAGE`. |
| 42 | 8 | crates/svrngs/src/issuer.rs:41-54 and web/src/lib.rs:146-256, at ac3ade0 | not fixed | As recorded. u4-9's `#[path]` share of crates/svrngs-node/src/relay.rs is a shape that would give it one body without a new crate. |
| 48 | 8 | web/src/join.rs:29-37 and web/src/lib.rs `bind`, at ac3ade0 | not fixed | As recorded; the range did not touch join.rs. |
| 55 | 8 | the "no change" items in (2) and (3) | no change | Given/then rows of the dotless relay at each place it enters (core, a token, the node); two drivers' imports; `Dotless`/`DotlessLookup`, iroh's builder and built pair. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". |
| I10 | 6 | `code dry-report` header | Still present: "1046 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Still present; every scoped run here used a trailing slash. |
| I14 | 6, 7 | `code dry-report --scope web/` near tier | Still present. The range's one real duplicate, in JS, scored 0.917: missed at the default 0.95 (web/ says "0 near-clone clusters" there), found at 0.9, its spans each three lines off into the other function. |
| I15 | 6, 7 | `code converge noun` over web/ | Still present: `Keyless` gives definitions (0), "check the spelling", for a class that exists. |

The row's check (LINT): `scripts/check.sh` at b225d09 on a clean tree
exited 0 in 6:34 (zsh `time`). Every step `passed`: principles pinned,
lean and RUST_ARCH, app nouns gate, app nouns, rustfmt, clippy, tests,
canaries, web rustfmt and clippy, browser, joining in a browser, app laws
(log /tmp/u4audit3/check.log).

## REVIEW-audit-u5-auto-1 (2026-10-05, range 7ed7b3e..ade62aa)

No earlier `REVIEW-audit-` row exists in ralph/next/u5/STATE.md. The queue
started at 02a3ef6, but every product commit from there to 7ed7b3e is
u4's, and REVIEW-audit-u4-auto-1 to -3 audited it. So the range starts at
7ed7b3e, the hash REVIEW-audit-u4-auto-3's row carries
(ralph/next/u4/STATE.md:16). It ends at ade62aa, the commit that made this
row. It covers u5-0, u5-1, u5-2 and u5-3. 8e830a5 and 28f736d, from the u6
loop, touch only ralph/next/u6/ and demo/runs/u6/. u5-3's instrument
(4b23bb4) and data (f42f25f) are under demo/, so they are not counted. The
range touched crates/svrngs-core, crates/svrngs-node, crates/svrngs, web/
and apps/ (`git diff --stat 7ed7b3e ade62aa -- crates apps web scripts`).
This audit's fix is 30652ed (#56).

The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md. It names `target/debug/sovereign-cli`,
which svrngs does not build. The binary used is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-05 16:42,
sha256 2a1d2c43…e79372. That is **not** the binary the last two audits
planted against (6501033e…b92ae), so its tiers were planted again.

The clean runs are on this tree at 4ecebe2 (ade62aa plus this row's
`started` line), `git status --short` empty, corpus `svrngs`. The plants
went into a clone of 4ecebe2, /tmp/u5audit1/svrngs-plant, with one commit
holding them (fad0907 there), indexed as corpus `svrngs-plant`.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 7ed7b3e..ade62aa`,
summed by /tmp/u5audit1/ledger.py with the last audit's rules. A commit
counts toward the row whose `started` commit comes before it. Product
code is `crates/`, `apps/` and `web/`, less Cargo.lock. Tests are a
`tests/` or `fixtures/` segment, a lean_check test name, or `*.test.mjs`.
scripts/check.sh (+7) and scripts/build-web.sh (+1 −1) are scripts, not
counted.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u5-0-inventory | 16 | 16 | 0 | 0 | 0 | 0 |
| u5-1-app-surface | 304 | 54 | 250 | 281 | 9 | 272 |
| u5-2-lend-address | 2443 | 575 | 1868 | 1756 | 141 | 1615 |
| u5-3-jellyfin-measure | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 2763 | 645 | 2118 | 2037 | 150 | 1887 |

- u5-0's 0 is 330fb6f: apps/films/nouns.txt became apps/jellyfin/nouns.txt.
- u5-1's +250: surface.rs +169 (new: `/v1/groups`, `/v1/lends`), node.rs
  +40 (461d26c's move of the people's lends into `Asking::lent`), api.rs
  +26, lend.rs +6, lib.rs +4, rail.rs +3, door.rs +1, cli.rs +1.
- u5-2's +1868: web/lend.js +392 (new; the runtime moved out of sw.js,
  −227, and web/hand.js, −37), door.rs +307, svrngs-core pass.rs +229
  (new), cli.rs +195, origin.rs +158 (new), apps/web-conformance
  serve.mjs +152 and checks.js +99, web/src/http.rs +114, web/shim.js
  +100 (new), lib.rs +83, open.js +62 and open.html +16 (new), api.rs +50,
  app.js +50, group.rs +35, link.rs +27, the rest under +20 each.
- u5-3 wrote no product code: its instrument is demo/runs/u5/.

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive`
in /tmp/u5audit1:

| at | svrngs-core | repository |
|---|---:|---:|
| 7ed7b3e, range start | 2789 / 3000 | 11297 / 14000 |
| ade62aa, range end | 2951 / 3000 | 13240 / 14000 |

svrngs-core rose 162 lines, to **98%** of its ceiling: pass.rs and
link.rs's lends origin (u5-2). The repository rose 1943, to **95%**. 760
lines are left for u5-4, u5-catalogue-dry and u5-9, and 49 in
svrngs-core. A raise is the operator's; this audit names it and does not
make it.

### (2) Clones (`code dry-report`), each tier checked against a plant first

**Clean corpus.** `code index ~/dev/svrngs --corpus-id svrngs --full`:
2093 chunks in 301 s. `refresh`: rust 8882 symbols, python 243,
"typescript: skipped (export failed)" (I4).

**Planted corpus**, each plant a copy of a symbol the range added:

- **A** (exact): surface.rs `name_of` (u5-1), verbatim, in a `mod plant`
  at the end of surface.rs.
- **B** (near): surface.rs `lends` (u5-1, u5-2) as `lends_again`, with
  `q`, `standing`, `own`, `lent` and `unanswered` renamed.
- **D** (JS): web/lend.js `setCookie` (u5-2) as `keepCookie` at the end
  of web/open.js, its body identical.
- **N** (noun): `pub struct Label(String)` in the same `mod plant`.
  svrngs-core pass.rs:85 has the original.

| scope | clean: locations / exact / near | plants seen |
|---|---|---|
| crates/svrngs-core/ | 421 / 7 / 0 | — |
| crates/svrngs-node/ | 350 / 5 / 1 | **A exact** at 0.95, 0.9 and 0.8: `name_of` surface.rs:108 == :189. B missed at 0.95; at 0.9, `lends` :134 ~ `lends_again` :215, cosine 0.942 |
| crates/svrngs/ | 67 / 0 / 0 | — |
| web/ | 79 / **could-not-judge** / 0 | D missed at 0.95; at 0.9, `setCookie` lend.js:118–139 ~ `keepCookie` open.js:62–82, cosine 0.938 |
| apps/ | 31 / **could-not-judge** / 0 | (not planted; JS, as web/) |

N: `converge noun Label` on the plant gives **definitions (2)**, pass.rs:84
and surface.rs:263 (each a line early, I1). The window pass (below) finds
A, B and D on the plant: 239 hits, 17 new, the four new ones beyond the
clean tree's 13 being surface.rs:108-131 == :189-212, :147-159 == :228-240,
:168-175 == :249-256, and lend.js:122-138 == open.js:66-82.

So the exact tier sees an identical Rust body; the near tier sees a
renamed Rust copy and an identical JS copy only at 0.9; and converge sees
a second Rust definition. D at 0.938 is better than the last plant's
"never paired above 0.7" (I14).

**Clean runs.** Scoped at 0.95 (table above), each scope with a trailing
slash (I11), and unscoped at 0.9 as the last audits ran it: **13 exact
groups and 20 near clusters** over 1165 function/method symbols (11 and 18
over 1046 at the last audit). /tmp/u5audit1/blame.py blames every side of
every cluster (`git blame -w -M -C -C HEAD`) for lines in
7ed7b3e..ade62aa. Every cluster with a side added in the range:

```
ADDED - **6 copies × 3 lines** (`a588a00c526a`)
      as_str crates/svrngs-core/src/pass.rs:113-115 added=3/3 ['becc0c7']
      (beside lib.rs:231, link.rs:189, lend.rs:62, :90, :129, none added)
ADDED - **2 symbols** · ~47 lines · cosine 0.900–0.900
      people_lends crates/svrngs-node/src/node.rs:591-623 added=8/33 ['461d26c']
      lends crates/svrngs-node/src/surface.rs:134-180 added=43/47 ['1c61b37', 'bf6276d']
ADDED - **2 symbols** · ~9 lines · cosine 0.910–0.910
      write_lends crates/svrngs/src/group.rs:104-112 added=9/9 ['c1f3ae7']
      read_lends crates/svrngs/src/group.rs:109-117 added=9/9 ['c1f3ae7']
ADDED - **2 symbols** · ~9 lines · cosine 0.900–0.900
      cookieFor web/lend.js:135-142 added=8/8 ['e502de9']
      keepJar web/lend.js:138-146 added=9/9 ['e502de9']
```

The unscoped exact tier also groups pass.rs:132 `fmt` with lib.rs:237
(2 × 3 lines, blamed outside the range by -C).

- `as_str` and `fmt`: a newtype's `&self.0` and a `Display` that writes
  it. Each is its own type's accessor. No change (#59).
- `people_lends` ~ `lends`: the two routes that list lends already share
  their body, `Asking::lent` (node.rs, 461d26c). What is left is each
  route's own asker and refusal: an iroh dialer and 403 against a
  loopback query and 409. No change (#59). One thing the pair shows is
  #57.
- `write_lends`/`read_lends` and `cookieFor`/`keepJar`: the spans are
  misaligned (I14), each taking in lines of the function after it. Each
  pair already calls one body: group.rs `write_in`/`read_in`, and
  lend.js `jarOf`. No change (#59).

**Substitute check, named as a substitute.** /tmp/u5audit1/windows.py
(windows of 5 normalized lines over Rust and JS; a hit is new when either
span has a line blamed in the range): 235 hits, 13 new, all in tests:

- crates/svrngs-node/tests/conformance.rs, lend_origin.rs, record_api.rs,
  surface.rs, door/main.rs: their imports, the admit-Sam
  `tests_support::Fixture` (lend_origin.rs:150-156 == surface.rs:87-93),
  and a `reqwest` GET read as JSON (conformance.rs:115-119 ==
  lend_origin.rs:176-180 == record_api.rs:231-235). Each is its own
  test binary's setup. tests/common holds what they already share. No
  change (#59).
- tests/browser/main.rs:598-613 == :768-781: two tests' `drive` calls.
- web/tests/handover.mjs:69-79 == runtime.mjs:59-69: two drivers'
  join-and-open steps. page.mjs holds the shared parts. No change (#59).

Reading the range for what neither pass would show:

- **#56, fixed in 30652ed.** web/src/http.rs `request` (u4-4) and
  `socket` (e502de9) each read the page's `[name, value]` pairs, refused
  with the same `Refused` variants, and asked `Host: svrngs` when none
  was named, at ade62aa :152 and :275. 30652ed's body says :150 and :266;
  those are wrong, and these are right. One `headers_of`, one
  `UNNAMED_HOST`. Each request's bytes are unchanged: `socket` still
  writes each header as named and in order.
- **#57.** Absences read as nothing, with no trace (principles 1, 6):
  - crates/svrngs-node/src/surface.rs:169-171: `/v1/lends` drops a lend
    whose key does not give an origin (`Host::origin`, node.rs:367, `None`
    on a key that is not 64 hex), or that does not serialize, by
    `filter_map`'s `?`. The app is answered a list without that lend, and
    nothing says why.
  - web/lend.js:112 `jarOf` and :259: a lend's cookies, and the app's
    worker, that fail to read from this origin's store become none
    (`.catch(() => null)`). keepJar at :142 traces its failure.
  - web/src/http.rs:244: the `Set-Cookie` values become `""` if they do
    not serialize (`unwrap_or_default`).

  Each fix adds a trace line or a refusal, so none is behaviour-preserving
  and none is fixed here. Recorded for the unit that next touches each
  file.
- **#58.** A lender's 64-hex key becomes the 32 bytes `Label::of` takes in
  two workspaces: node.rs:368 `Host::origin` (`None`) and
  web/src/lib.rs:337 `label` (`not a computer's key`). lib.rs:171 `reach`
  reads the same hex for a dial. `Label::of` is the one formula, so the
  label is not decided twice; the hex read is. A home is svrngs-core
  beside `Label` (pass.rs), which both workspaces link, and svrngs-core
  has 49 lines left. Recorded.
- **#49, still open, and moved.** u5-2 removed web/hand.js and gave each
  lend an origin of its own, so the address the page builds and the lend's
  origin parses is now `l/<label>/` under the lends origin, or
  `<label>.<host>`. It is hand-derived in three files: web/app.js:169 and
  :171 (`lendAt`), web/open.js:44 (the check that a pass opened at its
  own lend), and web/lend.js:38-39 (`UNDER`, `OWN`) and :67-68
  (`baseOf`). store.js is imported by all three and is the key's store,
  not this. The Rust side has one home: origin.rs:44 and :49 parse and
  build `<label>.localhost`.
- **#48, still open, and grown.** svrngs-web reads "these 32 bytes are a
  key" a third time: web/src/lib.rs:347-351, `pass`'s `key` closure
  ("a key is 32 bytes"), beside join.rs:29 `person` and lib.rs:102
  `bind`. Each uses its own words and key type, and `pass`'s refusal
  emits no trace.
- **#42, still open.** crates/svrngs/src/issuer.rs is untouched in the
  range. web/src/lib.rs `bind` (:98) and `reach` (:165) are where they
  were.
- One decider each, checked: a pass is signed only by
  svrngs-core `Pass::sign` (called from web/src/lib.rs:357) and checked
  only by `Pass::check` (pass.rs:196; door.rs `holder_of`); a label is
  made only by `Label::of` (door.rs:631, :678; node.rs:369;
  web/src/lib.rs:339) and read only by `Label::parse` (door.rs:627,
  origin.rs:44, lib.rs:354). The canary's route names are in
  apps/web-conformance/checks.js, web/tests/conformance.mjs:78 and
  crates/svrngs-node/tests/conformance.rs:208. A name missing from the
  driver reads as `failed` in `verdicts` (conformance.rs:229), not as a
  pass. No change (#59).

### (3) Duplicate type names (`code converge noun`)

`git diff 7ed7b3e ade62aa -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: `Apps`, `Pages` (crates/svrngs
cli.rs), `PassFault`, `Pass`, `Label` (svrngs-core pass.rs), `Pending`,
`Who`, `Asker` (door.rs), `Held`, `Surface` (surface.rs), the JS
`class Socket` (web/shim.js), and test types `Way`, `Canary`, `Group`
(tests/conformance.rs), `House`, `Listed` (tests/surface.rs). `Asking`,
`Endpoint` and the `type Error` (an associated type) moved and are
not new. Each name was run (/tmp/u5audit1/nouns.txt), then `git grep -nwE
'(struct|enum|trait|type|class) <Name>' -- crates apps web`:

- One definition by both: Apps (cli.rs:512), Pages (:523), PassFault
  (pass.rs:156), Label (:85), Pass (:141, kin PassFault), Pending
  (door.rs:602), Who (:581), Asker (:594), Surface (surface.rs:46).
- `Socket`: definitions (0), "check the spelling" (I15; JS). git grep: one,
  web/shim.js:47.
- Test types: converge leaves tests out, definitions (0). git grep: one
  each for Way, Canary, House and Listed.
- **`Held`, definitions (2)**: crates/svrngs-node/src/surface.rs:61, a
  group as `GET /v1/groups` answers it, and crates/svrngs-runtime/src/
  journal.rs:55, the folded state of a journal's tail. Both are private
  to their modules, and neither decides what the other does. One short
  name, two concepts. No change (#59).
- **`Group`, by git grep only**: tests/conformance.rs:126 (a test's
  group of nodes) and web/src/join.rs:50 (the group a link joins). One is
  a test type, and the two are in two workspaces. No change (#59).

No production noun the range added is defined twice for one concept.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 56 | 8 | web/src/http.rs:131-153 (`request`) and :261-276 (`socket`), at ade62aa | 30652ed | A request and a socket each read the page's headers and asked the default `Host: svrngs`, spelled at :152 and :275. One `headers_of` and one `UNNAMED_HOST` now; the bytes sent are unchanged. The conformance canary passed every route in every way on the rebuilt wasm. 30652ed's body cites :150 and :266; this row's lines are right. |
| 57 | 1, 6 | crates/svrngs-node/src/surface.rs:169-171; web/lend.js:112 and :259; web/src/http.rs:244, at 30652ed | not fixed | Absences become nothing, with no trace: `/v1/lends` drops a lend with no origin or one that will not serialize; a lend's cookies, or the app's worker, that cannot be read from the store become none; `Set-Cookie` values that do not serialize become `""`. Each fix adds a trace or a refusal, so it is a change of behaviour. Recorded for the next unit in each file. |
| 58 | 8 | crates/svrngs-node/src/node.rs:368 and web/src/lib.rs:337 (and :171), at 30652ed | not fixed | A lender's 64-hex key is read into `Label::of`'s 32 bytes in both workspaces. The label has one formula; the hex read does not. A home beside `Label` in svrngs-core would serve both, and svrngs-core has 49 lines left. |
| 49 | 8 ("one accessor per path") | web/app.js:169, :171; web/open.js:44; web/lend.js:38-39, :67-68, at 30652ed | not fixed | As recorded, and moved by u5-2: the lend's address on a phone, `l/<label>/` or `<label>.<host>`, is built by the page, checked by open.js and parsed by the lend's worker, each by hand. |
| 48 | 8 | web/src/join.rs:29, web/src/lib.rs:102 and :347-351, at 30652ed | not fixed | As recorded, and grown: `pass` reads the 32 bytes a third time, in words of its own and with no trace. |
| 42 | 8 | crates/svrngs/src/issuer.rs:41-54 and web/src/lib.rs:98-190, at 30652ed | not fixed | As recorded; the range did not take it up. |
| 59 | 8 | the "no change" items in (2) and (3) | no change | A newtype's `as_str` and `Display`; the two lend routes, already one body in `Asking::lent`; misaligned spans over functions that already call one body; each test binary's own setup; two drivers' steps; `Held` and `Group`, one short name for two concepts; the canary's route names, a missing one failing. Nothing is decided twice. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: Label at pass.rs:84 for :85, the plant at surface.rs:263 for :264. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)", so the exact tier is could-not-judge for web/ and apps/. |
| I5 | 5, 7 | `code dry-report` near tier | Still present on the new binary. Plant B, a 50-line copy with five names changed, scores 0.942: found at 0.9, missed at the default 0.95. |
| I10 | 6 | `code dry-report` header | Still present: "1165 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Still present; every scoped run here used a trailing slash. |
| I14 | 6, 7 | `code dry-report --scope web/` near tier | Still present, and better. Plant D, an identical 18-line JS body, scores 0.938: found at 0.9, missed at 0.95, where the last binary never paired one above 0.7. Its spans are still off by three lines (setCookie :118–139 for :121–138). |
| I15 | 6, 7 | `code converge noun` over web/ | Still present: `Socket` gives definitions (0), "check the spelling", for a class that exists. |

The row's check (LINT): `scripts/check.sh` at e4fcd11 on a clean tree
exited 0 in 7:33 (zsh `time`). Every step `passed`: principles pinned,
lean and RUST_ARCH, app nouns gate, app nouns, rustfmt, clippy, tests,
canaries, web rustfmt and clippy, browser, joining in a browser,
conformance canary (every route passed on a computer and in the wildcard
mode; on the floor the root and local storage `needs the wildcard`, the
rest passed), app laws (log /tmp/u5audit1/check.log).

## REVIEW-audit-u5-auto-2 (2026-10-06, range e2cb20b..5212420)

The range starts at e2cb20b, the hash REVIEW-audit-u5-auto-1's row
carries (ralph/next/u5/STATE.md:6), and ends at 5212420, the commit that
made this row. It covers u5-4, u5-catalogue-dry, u5-9 and u5-10, and three
commits between rows (65794de, 4060a38, 180443e: the page's deploy, its
name in the help, two new rows). The range touched crates/svrngs-node,
crates/svrngs, web/ and apps/; not svrngs-core (`git diff --stat e2cb20b
5212420 -- crates apps web scripts`). This audit's fixes are 51bfacb,
83d156e and 983547e (#60).

The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli`,
which svrngs does not build. The binary used is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-05 22:09,
sha256 f37d4d53…0625. That is **not** the binary the last audit planted
against (2a1d2c43…e79372), so its tiers were planted again.

The clean runs are on this tree at fed9963 (5212420 plus this row's
`started` line), `git status --short` empty, corpus `svrngs` (2157
chunks; `refresh`: rust 8964 symbols, python 352, "typescript: skipped
(export failed)"). The plants went into a clone of fed9963,
/tmp/u5audit2/svrngs-plant, one commit holding them (ea76c36 there),
corpus `svrngs-plant`.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' e2cb20b..5212420`,
summed by /tmp/u5audit2/ledger.py with the last audit's rules and two
more: a commit after a row's `done` and before the next `started` counts
as "between rows", not toward the row before; and a `check/` segment is
a test, since apps/<app>/check/ is that app's instrument (9cb64e4's body),
not code that serves a person. Product code is `crates/`, `apps/` and
`web/`, less Cargo.lock. scripts/ (deploy-page.sh +26, build-web.sh
+1 −1, check.sh) is not counted.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u5-4-jellyfin-in | 75 | 17 | 58 | 995 | 57 | 938 |
| u5-catalogue-dry | 148 | 24 | 124 | 9 | 7 | 2 |
| u5-9-demo-a-dry | 0 | 0 | 0 | 647 | 286 | 361 |
| (between rows) | 22 | 1 | 21 | 0 | 0 | 0 |
| u5-10-page-host | 61 | 30 | 31 | 91 | 10 | 81 |
| total | 306 | 72 | 234 | 1742 | 360 | 1382 |

- u5-4's +58: share.rs +29 (`rows`, `accounts_of`), cli.rs +23 (what
  `share` says of accounts), well_known.txt +6. Its +938 of tests is
  apps/jellyfin/check/: u5-3's instrument copied from demo/runs/u5/
  (9cb64e4, +633), then judge.py +87, lender.mjs +81, check.sh; and
  crates/svrngs/tests/share.rs +60, share/tests.rs +17.
- u5-catalogue-dry's +124: web/lend.js +30, door.rs +29 (`own_origin`),
  apps/web-conformance checks.js +25 and serve.mjs +16 (four routes),
  web/shim.js +24.
- u5-9 wrote only apps/jellyfin/check/: person.mjs moved out of
  drive.mjs (024ed19: +279, −265), trio.mjs +172, trio.py +116.
- Between rows: web/vercel.json +21 (65794de), cli.rs's help +1 −1.
- u5-10's +31: cli.rs +26 (`pages` asks for the page, not DNS),
  Cargo.toml +5 (reqwest 0.13, rustls, the platform verifier, which
  iroh's tree already builds).

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive`
in /tmp/u5audit2:

| at | svrngs-core | repository |
|---|---:|---:|
| e2cb20b, range start | 2951 / 3000 | 13247 / 14000 |
| 5212420, range end | 2951 / 3000 | 13823 / 14000 |
| 983547e, this audit's fixes | 2951 / 3000 | 13818 / 14000 |

The repository rose 576 lines, to **98.7%** of its ceiling. 182 lines
are left for u5-11 and whatever follows; svrngs-core has 49. A raise is
the operator's; this audit names it and does not make it.

### (2) Clones (`code dry-report`), each tier checked against a plant first

**Plants**, each a copy of a symbol the range added:

- **A** (exact): door.rs `own_origin` (u5-catalogue-dry), verbatim, in a
  `mod plant` at the end of door.rs.
- **B** (near): share.rs `rows` (u5-4) as `rows_again`, its eight locals
  renamed.
- **D** (JS, exact body): apps/jellyfin/check/person.mjs `judge` (u5-9's
  move) as `judgeAgain` at the end of trio.mjs.
- **N** (noun): `pub enum Apps` in the same `mod plant`; cli.rs:525 has
  the original.

| scope | clean: locations / exact / near | plants seen |
|---|---|---|
| crates/svrngs-node/ | 351 / 5 / 1 | **A exact** at 0.95: `own_origin` door.rs:544 == :1063 |
| crates/svrngs/ | 70 / 0 / 0 | **B missed** at 0.95, 0.9 and 0.8. At `--scope crates/svrngs/src/share.rs --min-lines 4 --threshold 0.7`, `rows` :40–57 ~ `rows_again` :161–175 at cosine **0.786**, below an unrelated pair, `resolve_target` ~ `well_known_kind`, at 0.791 |
| web/ | 81 / **could-not-judge** / 0 | (not planted; the exact tier cannot judge JS, I4) |
| apps/ | 53 / **0** / 0 | **D missed** by the exact tier, which said 0 (I16). Near: missed at 0.95 and 0.9; at 0.8, `judge` person.mjs:41–64 ~ `judgeAgain` trio.mjs:171–191 at 0.823–0.827, in a cluster with trio.mjs `over` |

N: `converge noun Apps` on the plant gives **definitions (2)**, cli.rs:524
and door.rs:1078 (each a line early, I1).

So the exact tier sees an identical Rust body; it does not see an
identical JS body, and says 0 where it should say it could not judge; the
near tier does not see a renamed 12-line Rust copy above 0.786, nor an
identical 18-line JS body above 0.827; converge sees a second Rust
definition. The near tier is weaker on this binary than on the last (B
was 0.942, D 0.938).

**Clean runs.** Scoped at 0.95 (table above), each scope with a trailing
slash (I11), and unscoped at 0.9: **14 exact groups and 29 near
clusters** over 1223 function/method symbols (13 and 20 over 1165 at the
last audit). /tmp/u5audit2/blame.py blames every side of every cluster
(`git blame -w -M -C -C HEAD`) for lines in e2cb20b..5212420, and now
also counts a side whose file did not exist at e2cb20b as added (blame's
-C gives a copied file's lines to the commit it was copied from). Scoped:
no cluster has a side added in the range. Unscoped at 0.9, every one:

```
ADDED - **2 copies × 25 lines** (`125054c9cd24`)
      line [file new in range] apps/jellyfin/check/summary.py:11-35 added=25/25 []
      line demo/runs/u5/2026-10-05-jellyfin/summary.py:11-35 added=0/25 []
ADDED - **2 symbols** · ~38 lines · cosine 0.977–0.977
      record [file new in range] apps/jellyfin/check/person.mjs:103-140 added=38/38 ['024ed19']
      record demo/runs/u5/2026-10-05-jellyfin/drive.mjs:122-159 added=0/38 []
ADDED - **2 symbols** · ~32 lines · cosine 0.990–0.990
      seek [file new in range] apps/jellyfin/check/person.mjs:62-93 added=32/32 ['024ed19']
      seek demo/runs/u5/2026-10-05-jellyfin/drive.mjs:81-112 added=0/32 []
ADDED - **2 symbols** · ~26 lines · cosine 0.997–0.997
      watch [file new in range] apps/jellyfin/check/person.mjs:18-43 added=26/26 ['024ed19']
      watch demo/runs/u5/2026-10-05-jellyfin/drive.mjs:37-62 added=0/26 []
ADDED - **2 symbols** · ~24 lines · cosine 0.998–0.998
      judge [file new in range] apps/jellyfin/check/person.mjs:41-64 added=24/24 ['024ed19']
      judge demo/runs/u5/2026-10-05-jellyfin/drive.mjs:60-83 added=0/24 []
ADDED - **2 symbols** · ~18 lines · cosine 0.981–0.981
      socket [file new in range] apps/jellyfin/check/person.mjs:265-279 added=15/15 ['024ed19', '509ca15']
      socket demo/runs/u5/2026-10-05-jellyfin/drive.mjs:301-318 added=0/18 []
ADDED - **2 symbols** · ~16 lines · cosine 0.992–0.992
      timed [file new in range] apps/jellyfin/check/person.mjs:90-105 added=16/16 ['024ed19']
      timed demo/runs/u5/2026-10-05-jellyfin/drive.mjs:109-124 added=0/16 []
ADDED - **2 symbols** · ~16 lines · cosine 0.903–0.903
      phase [file new in range] apps/jellyfin/check/drive.mjs:39-54 added=16/16 ['9cb64e4']
      phase demo/runs/u5/2026-10-05-jellyfin/drive.mjs:280-295 added=0/16 []
ADDED - **2 symbols** · ~13 lines · cosine 0.996–0.996
      bind [file new in range] apps/jellyfin/check/person.mjs:20-32 added=13/13 ['024ed19']
      bind demo/runs/u5/2026-10-05-jellyfin/drive.mjs:39-51 added=0/13 []
ADDED - **2 symbols** · ~13 lines · cosine 0.965–0.965
      lender [file new in range] apps/jellyfin/check/person.mjs:258-270 added=13/13 ['024ed19', '509ca15', '9cb64e4']
      jf demo/runs/u5/2026-10-05-jellyfin/drive.mjs:294-306 added=0/13 []
```

Every one pairs apps/jellyfin/check/ with demo/runs/u5/2026-10-05-jellyfin/,
the scripts of u5-3's run, which 9cb64e4 copied into apps/ and left in
demo/ as "the record of u5-3's run". A run's record is what ran, kept so
its data can be read against it (principle 7); it decides nothing now, and
nothing runs it but someone rerunning that run. No change (#61). The
near-tier pairs among the clusters above with no side in the range
(declared.rs `harden_file` ×2 and the like) were there at the last audit.

**Substitute check, named as a substitute.** /tmp/u5audit2/windows.py
(windows of 5 normalized lines; a hit is new when either span has a line
blamed in the range), widened to .py and .sh: 236 hits, 2 new:

- crates/svrngs/src/cli.rs:468-473 == :481-486: `share` printed
  `Said::Shared` twice, differing only in `running`, and u5-4 added
  `accounts` to both. **Fixed, 83d156e** (#60).
- crates/svrngs/tests/new.rs:471-475 == :508-512: two tests read the
  page's refusal by the same six lines (u5-10). **Fixed, 83d156e** (#60).

Reading the range for what neither pass would show:

- **#60, fixed.** 51bfacb: door.rs `own_origin` (20868ee) spelled the
  service's own address, `http://localhost:{port}`, at :554, beside
  `own_address`'s `localhost:{port}` at :532; it now builds on
  `own_address`. 983547e: web/shim.js parsed a cookie line at :25-30 and
  again in `takeIn` (c950d87), and spelled `x-svrngs-page-cookies` at
  its fetch and its XMLHttpRequest; one `pairs`, one `PAGE_COOKIES`. And
  web/lend.js `lent` asked whether a request posts twice; once now.
- **#62.** Demo A's seek bar, 5 s, has three homes in apps/jellyfin/check:
  drive.mjs:49 (`within_5s`, `<= 5000`), judge.py:50 (`> 5000`) and
  trio.py:20 (`BAR_MS`); and "never still 5 s", two: person.mjs:59
  (`still < 5000`) and trio.mjs:44 (`run >= 5000`). drive.mjs and
  judge.py judge one run (u5-4's check) and agree, `<= 5000` passing in
  both. Each is an instrument's copy of demo/a.md's bar across two
  languages; one home would be a value each script reads, which changes
  how each runs. Recorded.
- **#63.** `x-svrngs-page-cookies` is named in web/lend.js:166 and
  web/shim.js:53. The worker sets it and the page's shim reads it, and the
  two share no module. The worker already names the shim's lend and
  cookies on its address (lend.js `shim`, `l=` and `c=`); the header's
  name could ride the same way, which changes the shim's address.
  Recorded.
- **#64.** crates/svrngs/src/share.rs:43 `rows` gives a row's kind as a
  `&str`, and the two readers decide it two ways: `well_known_kind`
  (:99) parses it (`Kind::parse`), `accounts_of` (:111) compares the raw
  text with `kind.as_str()` (principle 10, parse, don't validate). They
  agree for every row well_known.txt holds today. `rows` giving a `Kind`
  would drop a row whose kind does not parse from `accounts_of` too,
  which is a change of behaviour. Recorded.
- **#49, still open, and grown.** u5-catalogue-dry's door.rs `own_origin`
  reads the lend's own address, `<label>.<host>`, from a request's
  `Origin` by its first DNS label (door.rs:548-550). The address is now
  read in Rust as well as hand-derived in web/app.js, web/open.js and
  web/lend.js; origin.rs:39 `label_of` reads only `<label>.localhost`.
- **#57, still open.** surface.rs:169, web/lend.js:112 and :289 (was
  :259; u5-catalogue-dry moved it), web/src/http.rs:244: as recorded.
- **#58, #48, #42, still open.** node.rs, web/src/lib.rs, web/src/join.rs
  and crates/svrngs/src/issuer.rs are untouched in the range.
- One decider each, checked: what `share` tells a lender of accounts has
  one home, well_known.txt's third column, read only through share.rs
  `accounts_of` (cli.rs:456); whether a page's host gives each app an
  address of its own is decided only by cli.rs `pages`; the canary's new
  routes are named in apps/web-conformance/checks.js and listed once by
  crates/svrngs-node/tests/conformance.rs:207-218. No change (#61).

### (3) Duplicate type names (`code converge noun`)

`git diff e2cb20b 5212420 -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: one, `enum Elsewhere`
(crates/svrngs/tests/new.rs:445, a test's stand-in host). `converge noun
Elsewhere`: definitions (0), "defined only in test code". `git grep -nwE
'(struct|enum|trait|type|class) Elsewhere' -- crates apps web`: one,
new.rs:445. The range added no production noun; no noun is defined twice.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 60 | 8 | crates/svrngs-node/src/door.rs:532 and :554; crates/svrngs/src/cli.rs:465-490; crates/svrngs/tests/new.rs:471-475 and :508-512; web/shim.js:25-30 and takeIn, :59 and :63; web/lend.js:203 and :214, at 5212420 | 51bfacb, 83d156e, 983547e | The service's own address spelled in `own_origin` beside `own_address`; `share`'s `Said::Shared` printed twice; one refusal read by six lines in two tests; a cookie line parsed twice and one header named twice in the shim; whether a request posts asked twice. One home each now. scripts/check.sh at 983547e passed every step, the canary's routes as the last audit left them. |
| 61 | 8 | the "no change" items in (2) and (3) | no change | apps/jellyfin/check/ beside demo/runs/u5/2026-10-05-jellyfin/, the record of the run it was copied from; `Elsewhere`, a test's type; `accounts_of` and `pages`, each the one decider of its question. |
| 62 | 8 | apps/jellyfin/check/drive.mjs:49, judge.py:50, trio.py:20 (5 s to play again); person.mjs:59, trio.mjs:44 (5 s still), at 983547e | not fixed | Demo A's bars are spelled in each instrument that judges them, in two languages. They agree today. One home would be a value every script reads. |
| 63 | 8 | web/lend.js:166 and web/shim.js:53, at 983547e | not fixed | The header that carries the cookies a lend's pages may read is named by the worker and by the shim, which share no module. It could ride on the shim's address as `l=` and `c=` do. |
| 64 | 10 | crates/svrngs/src/share.rs:99 and :111, at 983547e | not fixed | well_known.txt's kind is parsed by `well_known_kind` and compared as text by `accounts_of`. They agree on every row today; `rows` giving a `Kind` would make one reading. |
| 49 | 8 | as recorded, and crates/svrngs-node/src/door.rs:548-550, at 983547e | not fixed | Grown: the door reads `<label>.<host>` from an `Origin` by its first label, a fourth hand reading of the lend's address, the first in Rust. |
| 57 | 1, 6 | surface.rs:169; web/lend.js:112 and :289; web/src/http.rs:244, at 983547e | not fixed | As recorded. |
| 58, 48, 42 | 8 | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: the plant's Apps at cli.rs:524 for :525 and door.rs:1078 for :1079. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)", so web/'s exact tier is could-not-judge. |
| I5 | 5, 7 | `code dry-report` near tier | Worse. Plant B, a 12-line Rust copy with eight locals renamed, scores 0.786, below an unrelated pair in the same file (0.791): missed at 0.8. The last binary found a 50-line copy at 0.942. |
| I10 | 6 | `code dry-report` header | Still present: "1223 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Still present; every scoped run here used a trailing slash. |
| I14 | 6, 7 | `code dry-report` near tier over JS | Worse. Plant D, an identical 18-line JS body, scores 0.823–0.827, missed at 0.9; its spans are off by three lines (judge :41–64 for :44–61, judgeAgain :171–191 for :174–192). Whole files copied (apps/jellyfin/check/ beside demo/) pair at 0.97–0.998, so neighbours in the span, not the body, decide the score. |
| I16 | 6 | `code dry-report --scope apps/` exact tier | New. The scope holds Python, which exported, and JS, which did not; the tier said "**0** exact-clone groups", a judged result, with Plant D, an identical JS body, in scope. It judged the Python and said nothing of the JS. web/, all JS, says could-not-judge. |

The row's check (LINT): `scripts/check.sh` at 983547e on a clean tree
exited 0 in 6:09 (zsh `time`; load 2.29 at its start, 12 cores). Every
step `passed`: principles pinned, lean and RUST_ARCH, app nouns gate, app
nouns, rustfmt, clippy, tests, canaries, web rustfmt and clippy, browser,
joining in a browser, conformance canary (every route passed on a
computer and with the wildcard; on the floor `/`-absolute paths, local
storage and a post's Origin `needs the wildcard`, the rest passed), app
laws (log /tmp/u5audit2/check.log).

## u5-11: WebKit's worker gone during the canary's upload, 2026-10-06

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 65 | 2, 6 | apps/web-conformance/checks.js `upload`; web/sw.js, at b41941c | **fixed in 861fd5a** (u5-12) | What ended the worker was memory: WebKit's `blob.stream()` handed the 64 MiB body over as one chunk, the runtime copied it into wasm memory, which never shrinks, and the content process holding the floor's two workers grew 135-320 MB per upload until, past 1 GB, WebKit ended it ('Suspended WebProcess is exiting due to memory pressure', `demo/runs/u5/2026-10-06-worker-gone/plant-1/worker-process-54770.log.gz`, `plant-4/worker-process-26737.log.gz`). The seeks alone and a fetch after 28 s quiet did not grow it (`seeks-idle/`, `upload-only/`, f05e44d). The lend's worker now hands a body over a 256 KiB slice at a time (web/lend.js `sent`). The plant failed 33f401f's gate (a content process at 1,345 MB, `plant-4/`); the fix passed it twice, 40 of 40 uploads, none to the page's host, at most 670 MB and 509 MB (`fixed-2/` at load up to 45, `fixed-3/` at load 3-6). As first recorded: in WebKit as an iPhone, through a lend, the 64 MiB upload that follows the seeks is sometimes answered `404 ` by the page's static server: the request never reached the lend's worker. Seen 1 of 12 runs in u5-11's tries at a load of 3.4-3.9 (`demo/runs/u5/2026-10-06-seek-stall/tried-closed-whole/verdicts.txt`, wildcard Sam, before b41941c), then 2 of 4 WebKit phone tests in the `tests` step of the second scripts/check.sh at b41941c, floor Bo and wildcard Bo, at a load of 5.3 at its start and 13.5 at its end on 12 cores (`upload-404/check-2.txt.gz`); the same run's `conformance canary` step passed every route. In both of those the worker's trace starts over (the pass read again from the origin's store) 28 s after the seeks ended, and holds no `POST .../upload`: the worker was gone when the upload was sent, and WebKit sent it to the network. Not reproduced on purpose and not explained: what ended the worker (the 64 MiB body, memory, the load) is not shown, since the trace lives in the worker and starts again with it. The next step is a trace that outlives the worker. |

## REVIEW-audit-u5-auto-3 (2026-10-06, range 50dfeb3..2442048)

The range starts at 50dfeb3, the hash REVIEW-audit-u5-auto-2's row
carries (ralph/next/u5/STATE.md:13), and ends at 2442048, the commit that
made this row. It covers u5-11 and the commits beside it (8428cfc,
9ae4630, 94a957a: rows and calls, no code). 5d12b37, after this row was
started, adds u5-12's row only. The range touched crates/svrngs-node
(door.rs, Cargo.toml, tests/conformance.rs), web/ (lend.js,
tests/conformance.mjs, tests/control.mjs, moved in from demo/ by d1140f2),
apps/web-conformance/, apps/jellyfin/check/ and scripts/check.sh (`git
diff --stat 50dfeb3 2442048 -- crates apps web scripts`). This audit's
fixes are 2e323ec and 89ba34a (#66).

The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli`,
which svrngs does not build. The binary used is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-05 22:09,
sha256 f37d4d53…0625, the one the last audit planted against. Its tiers
were planted again here, on symbols this range added.

The clean runs are on this tree at 5d12b37 (2442048 plus the `started`
line and u5-12's row), `git status --short` empty, corpus `svrngs` (`code
index --full`: 2183 chunks in 397 s; `refresh`: rust 9046 symbols, python
352, "typescript: skipped (export failed)"). The plants went into a clone
of 5d12b37, /tmp/u5audit3/svrngs-plant, one commit holding them (2514448
there), corpus `svrngs-plant` (2191 chunks; rust 9063 symbols).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 50dfeb3..2442048`,
summed by /tmp/u5audit3/ledger.py, the last audit's script and rules
unchanged (a `check/` or `tests/` segment is a test; scripts/ is not
counted: check.sh +7 here).

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u5-11-seek-stall | 193 | 7 | 186 | 210 | 17 | 193 |
| total | 193 | 7 | 186 | 210 | 17 | 193 |

- src: web/lend.js +52 (`FIRST` and `ends`, a seek's open range asked
  small then doubled; `ranged` and `counted`, each range traced),
  door.rs +50 (`Carriage`, a request's carriage traced at debug),
  apps/web-conformance serve.mjs +44 (`big.wav`, `played`) and checks.js
  +37 (`seeks`), svrngs-node's Cargo.toml +3 (tracing-subscriber, a
  dev-dependency).
- tests: crates/svrngs-node/tests/conformance.rs +190 (`uplink`, a home's
  link to the phone; `carriage`; `load`; `rate` and `a_rate`, WebKit's
  seeks a rate), web/tests/conformance.mjs +2, apps/jellyfin/check +1.
  web/tests/control.mjs counts +0: git follows it from
  demo/runs/u5/2026-10-06-seek-stall/control.mjs (d1140f2, a move).

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive`
in /tmp/u5audit3:

| at | svrngs-core | repository |
|---|---:|---:|
| 50dfeb3, range start | 2951 / 3000 | 13818 / 14000 |
| 2442048, range end | 2951 / 3000 | 13910 / 14000 |
| 89ba34a, this audit's fixes | 2951 / 3000 | 13911 / 14000 |

lean_check counts no apps/ and no tests/, so of the ledger's +186 it
counts lend.js and door.rs, less comments, and check.sh's +7. The
repository rose 92 lines, to **99.4%** of its ceiling: 89 are left for
u5-12 and what follows; svrngs-core has 49. (Run on the working tree
rather than an archive it says 13931: files git ignores.) A raise is the
operator's; this audit names it and does not make it.

### (2) Clones (`code dry-report`), each tier checked against a plant first

**Plants**, each a copy of a symbol the range added:

- **A** (exact): door.rs `Carriage` and its `impl Drop` (u5-11),
  verbatim, in a `pub mod plant` at the end of door.rs.
- **B** (near, JS): web/lend.js `counted` (u5-11) as `tallied` at the end
  of web/open.js, its six locals and parameters renamed.
- **D** (JS, exact body): apps/web-conformance/serve.mjs `played` (u5-11)
  as `playedAgain` at the end of serve.mjs.
- **N** (noun): the plant's `pub struct Carriage`.

| scope | clean at 0.95: locations / exact / near | plants seen |
|---|---|---|
| crates/svrngs-node/ | 353 / 5 / 1 | **A exact** at 0.95, 0.9 and 0.8: `drop` door.rs:934 == :1122. At 0.9 also near, the two at 0.928 |
| web/ | 85 / **could-not-judge** / 0 | **B half seen**: at 0.95 the inner `pull`s pair at 0.951 (lend.js:200–212 ~ open.js:65–77); `tallied` itself joins them only at 0.8, at 0.800, its span 61–78 for 64–79 |
| apps/ | 55 / **0** / 0 | **D missed** by the exact tier, which said 0 (I16), and by the near tier at 0.95 and 0.9; at 0.8, `played` serve.mjs:81–103 ~ `playedAgain` :211–231 at **0.883**, each span off (84–100, 214–230) |

N: `converge noun Carriage` on the plant gives **definitions (2)**,
door.rs:924 and :1112 (each a line early, I1).

As at the last audit, on the same binary: an identical Rust body is seen
exactly; an identical JS body is not, and the exact tier says 0 for it
where it should say it could not judge; the near tier sees a renamed JS
copy's inner closure at 0.951 and the function around it at 0.800.

**Clean runs.** Scoped at 0.95 (table above), each with a trailing slash
(I11), and unscoped at 0.9: **14 exact groups and 29 near clusters** over
1234 function/method symbols (14 and 29 over 1223 at the last audit).
/tmp/u5audit3/blame.py blamed every side of every cluster (`git blame -w
-M -C -C HEAD`) for lines in 50dfeb3..2442048: **no cluster in any run has
a side added in the range** (`grep -c ^ADDED` on each `.blame`: 0). The
tool excludes `tests/` paths, so conformance.rs and web/tests/control.mjs
were judged only by the substitute and by reading.

**Substitute check, named as a substitute.** /tmp/u5audit3/windows.py
(windows of 5 normalized lines over .rs .js .mjs .py .sh; a hit is new
when either span has a line blamed in the range), on a clone at 5d12b37:
234 hits, **0 new**. On the plant clone: 236 hits, 2 new, A
(door.rs:926-945 == :1114-1133) and D (serve.mjs:85-99 == :215-229); B,
renamed, it cannot see.

Reading the range for what neither pass would show:

- **#66, fixed.** 2e323ec: door.rs `forward` traced an unanswered
  carriage as `StatusCode::BAD_GATEWAY` (:1047) beside the `unanswered`
  closure that answers 502 (:1036-1043); it reads the status off the
  answer it returns now. 89ba34a: serve.mjs `played` read a Range by the
  regex and clamp `big` had (:85-87 and :103-106); one `span`. The
  canary's 16 answers for each of big.bin and big.wav compared the same
  before and after (/tmp/u5audit3/cmp.sh: status, Content-Range,
  Content-Length, Content-Type and the body's sha-256), and differed on a
  plant (an open range ended a byte early: two lines differed).
- **#68.** The phone's link, 16,666,666 bit/s paced, 150 ms each way, a
  second's bytes queued, has two homes: crates/svrngs-node/tests/
  conformance.rs:294-296 (`uplink`) and web/tests/control.mjs:11, whose
  header says it is "the same link the conformance test gives a phone".
  `rate` (conformance.rs) spawns control.mjs and could hand it the three;
  control.mjs is also run by hand from the seek-stall run's README, so
  that changes how it runs. They agree today. Recorded.
- **#62, grown.** Demo A's seek bar has a fourth home,
  apps/web-conformance/checks.js:74 (`again + 250 <= 5_000`); and what
  "plays again" means, `!seeking && !paused && currentTime > at + 0.25`,
  two: checks.js:72 and web/tests/page.mjs:106. The canary's checks run in
  the lent page and share no module with web/tests; they agree today.
- **No change (#67).** web/lend.js:169 `bounded` reads `bytes=(\d+)-(\d*)`
  as serve.mjs `span` does: the lend's worker and the canary, the
  stand-in app it is tested against, are two programs, and one reading
  the other's parser would make the canary not stand in. `Carriage`
  (door.rs) and `ranged`/`counted` (lend.js) trace one range at the two
  ends of one carriage, each where its own end of it is seen.
- **#57, #58, #49, #48, #42, #63, #64, still open.** The range changed
  none of their lines (`git diff -U0 50dfeb3 2442048 -- web/lend.js`:
  hunks at :47, :147-159, :192, :217, :227, :252); lend.js's below :147
  moved down by up to 52.

### (3) Duplicate type names (`code converge noun`)

`git diff 50dfeb3 2442048 -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: one, `struct Carriage`
(crates/svrngs-node/src/door.rs:925). `converge noun Carriage`:
definitions (1), door.rs:924 (I1). `git grep -nwE
'(struct|enum|trait|type|class) Carriage' -- crates apps web`: one,
door.rs:925. No noun is defined twice.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 66 | 8 | crates/svrngs-node/src/door.rs:1036-1047; apps/web-conformance/serve.mjs:85-87 and :103-106, at 2442048 | 2e323ec, 89ba34a | A 502 spelled where the answer that carries it is made; a Range read twice in the canary. One home each now; the canary's answers compared the same. |
| 67 | 8 | the "no change" items in (2) and (3) | no change | The worker's Range and the canary's, two programs; the door's carriage trace and the worker's, two ends; `Carriage`, one definition. |
| 68 | 8 | crates/svrngs-node/tests/conformance.rs:294-296 and web/tests/control.mjs:11, at 89ba34a | not fixed | The phone's link, pace, delay and queue, set in the test that lends and again in the control it compares against. They agree today; `rate` could hand them to the control. |
| 62 | 8 | as recorded, and apps/web-conformance/checks.js:72 and :74, web/tests/page.mjs:106, at 89ba34a | not fixed | Grown: the 5 s bar in the canary's `seeks` too, and "plays again" spelled in the canary and in the browser tests. |
| 65 | 2, 6 | as recorded | u5-12's row | Not this audit's: u5-12 (ralph/next/u5/STATE.md) is its row. |
| 63, 64, 57, 58, 49, 48, 42 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: the plant's Carriage at door.rs:1112 for :1113, the original at :924 for :925. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)", so web/'s exact tier is could-not-judge. |
| I10 | 6 | `code dry-report` header | Still present: "1234 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Still present; every scoped run here used a trailing slash. |
| I14 | 6, 7 | `code dry-report` near tier over JS | Still present: Plant D, an identical 17-line JS body, scores 0.883, missed at 0.9; spans off by up to three lines. |
| I16 | 6 | `code dry-report --scope apps/` exact tier | Still present: "**0** exact-clone groups" with Plant D, an identical JS body, in scope. |

The row's check (LINT): `scripts/check.sh` at 89ba34a on a clean tree
exited 0 in 16:50 (zsh `time`; load 52.18 at its start, 6.92 at its end,
12 cores; this section was written while it ran, and nothing it reads).
Every step `passed`: principles pinned, lean and RUST_ARCH, app nouns
gate, app nouns, rustfmt, clippy, tests, canaries, web rustfmt and
clippy, browser, joining in a browser, conformance canary (every route
passed on a computer and with the wildcard; on the floor `/`-absolute
paths, local storage and a post's Origin `needs the wildcard`), app laws
(log /tmp/u5audit3/check-1.log). WebKit's seeks back to back, the rate
u5-11 made: through a lend 361, 408, 3672, **5160 failed**, 3168, **8188
failed** ms (computer, floor, wildcard; Sam, Bo); straight at the canary
2927-4103, all passed; the seeks' run began at 1-minute load 8.74 (5- and
15-minute 21.76 and 36.04) and ended at 6.92-10.80. "of 56 here since
2026-10-06T16:42:30Z: 14 missed 5 s through a lend, 2 straight; 26 ended
at a load above 12 cores".

Its load was above 12 cores, so it was run again at d67a455 when the
host read 6.25: rc 0 in 13:47, every step `passed` as above (log
/tmp/u5audit3/check-2.log). The host did not stay quiet: the seeks began
at 16.62 and ended at 14.65-28.05, 40.63 by the run's end, from work this
session did not start. Through a lend 360, 479, 2280, **7393 failed**,
4368, **17496 failed** ms; straight at the canary 2207-4105, all passed;
"of 68 here since 2026-10-06T16:42:30Z: 17 missed 5 s through a lend, 3
straight; 34 ended at a load above 12 cores". Both runs kept; in both,
Bo's lend missed on the floor and with the wildcard. This audit judges no
rate: that is u5-12's and the operator's (u5-9 judges Demo A's).

## REVIEW-audit-u5-auto-4 (2026-10-06, range d67a455..7dd8e65)

The range starts at d67a455, the hash REVIEW-audit-u5-auto-3's row
carries (ralph/next/u5/STATE.md:15), and ends at 7dd8e65, the commit that
made this row. It covers u5-12 and bf9d8f0 (the last audit's second
check.sh run, ralph/ only). The range touched web/ (lend.js, store.js,
tests/conformance.mjs, tests/page.mjs), apps/web-conformance/checks.js
and crates/svrngs-node/tests/conformance.rs; no `src/` of any crate
(`git diff --stat d67a455 7dd8e65 -- crates apps web scripts`). This
audit's fix is 153ca1e (#69).

The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli`,
which svrngs does not build. The binary used is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-06 12:30,
sha256 68f2d27b…50f4. That is **not** the binary the last audit planted
against (f37d4d53…0625), so its tiers were planted again.

The clean runs are on this tree at c54f1e8 (7dd8e65 plus this row's
`started` line), `git status --short` empty, corpus `svrngs` (`code
index --full`: 2198 chunks in 415 s; `refresh`: rust 9096 symbols, python
352, "typescript: skipped (export failed)"). The plants went into a clone
of c54f1e8, /tmp/u5audit4/svrngs-plant, one commit holding them (e5f37f4
there), corpus `svrngs-plant` (2207 chunks; rust 9113 symbols).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' d67a455..7dd8e65`,
summed by /tmp/u5audit4/ledger.py, the last audit's script and rules
unchanged (a `check/` or `tests/` segment is a test; scripts/ is not
counted, and the range changed none of it).

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u5-12-worker-gone | 62 | 18 | 44 | 266 | 31 | 235 |
| total | 62 | 18 | 44 | 266 | 31 | 235 |

- src: web/lend.js +35 (`sent`, an upload's body handed to the dial a
  256 KiB slice at a time, 861fd5a; the trace kept in the origin's store
  across the worker's runs, 1300972), apps/web-conformance/checks.js +8
  (`idle`, a fetch after 28 s quiet), web/store.js +1 (its comment names
  `trace <label>`).
- tests: crates/svrngs-node/tests/conformance.rs +174 (`browser_args` and
  `browse`, moved out of `conformance` by ffb9bed; the requests no worker
  answered, shown; `HELD`, `upload_after_seeks`, `tries`),
  web/tests/conformance.mjs +54 (`routes`, each request and who answered
  it, the host's load, WebKit's content processes sampled), web/tests/
  page.mjs +7 (`server.asked`).

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive`
in /tmp/u5audit4:

| at | svrngs-core | repository |
|---|---:|---:|
| d67a455, range start | 2951 / 3000 | 13911 / 14000 |
| 7dd8e65, range end | 2951 / 3000 | 13932 / 14000 |
| 153ca1e, this audit's fix | 2951 / 3000 | 13932 / 14000 |

lean_check counts no apps/ and no tests/: of the ledger's +44 it counts
lend.js and store.js less comments. The repository rose 21 lines, to
**99.5%** of its ceiling: 68 are left; svrngs-core has 49. A raise is the
operator's; this audit names it and does not make it.

### (2) Clones (`code dry-report`), each tier checked against a plant first

Every symbol the range added in product code is JS (lend.js `sent`,
checks.js `idle`); its Rust is all under tests/, which the tool excludes.
**Plants:**

- **A** (exact, Rust): door.rs `Carriage` and its `impl Drop`, verbatim,
  in a `pub mod plant` at the end of door.rs. Not a symbol of this range
  (u5-11's): it shows the Rust exact tier still works on the new binary.
- **B** (near, JS): web/lend.js `sent` (u5-12) as `slices` at the end of
  web/open.js, its five locals and parameters renamed.
- **D** (JS, exact body): web/lend.js `sent` as `sentAgain` at the end of
  web/store.js, verbatim.
- **N** (noun): the plant's `pub struct Carriage`.

`idle` is 6 lines, under the tool's 8-line floor ("distinct locations
considered (≥8 lines)"), so it was not planted; the substitute pass
below judges it.

| scope | clean at 0.95: locations / exact / near | plants seen |
|---|---|---|
| crates/svrngs-node/ | 353 / 5 / 1 | **A exact** at 0.95, 0.9 and 0.8: `drop` door.rs:934 == :1123. At 0.9 also near, door.rs:931–951 ~ :1120–1139 |
| web/ | 87 / **could-not-judge** / 0 | **D half seen**: at 0.95 and 0.9 only the inner `pull`s, lend.js:239–248 ~ store.js:189–197 at 0.996; `sentAgain` itself joins only at 0.8. **B missed** at 0.95 and 0.9; at 0.8 `slices` (open.js:62–75) and its `pull` join a cluster of six at 0.804–0.996 |
| apps/ | 56 / 0 / 0 | not planted this time (I16 below is the last audit's) |

N: `converge noun Carriage` on the plant gives **definitions (2)**,
door.rs:924 and :1113, each a line early (I1: they are at :925 and
:1114).

The spans are off again (I14): `sent` is lend.js:237–247, its `pull`
:242–245; the tool says :239–248. `slices` starts at open.js:65 and
`sentAgain` at store.js:187; it says :62 and :184.

**Clean runs.** Scoped at 0.95 (table above), each with a trailing slash
(I11), and unscoped at 0.9: **14 exact groups and 29 near clusters** over
1238 function/method symbols (14 and 29 over 1234 at the last audit).
/tmp/u5audit4/blame.py blamed every side of every cluster (`git blame -w
-M -C -C HEAD`) for lines in d67a455..7dd8e65: **no cluster in any run has
a side added in the range** (`grep -c ^ADDED` on each `.blame`: 0, over
43 clusters unscoped and 6 in crates/svrngs-node/).

**Substitute check, named as a substitute.** /tmp/u5audit4/windows.py
(windows of 5 normalized lines over .rs .js .mjs .py .sh, tests/
included; a hit is new when either span has a line blamed in the range),
on this tree at c54f1e8: 230 hits, **0 new**. On the plant clone: 232
hits, 1 new, D (lend.js:238-244 == store.js:188-194). A is not new
because blame follows its copy back to u5-11, before the range; B,
renamed, it cannot see.

Reading the range for what neither pass would show:

- **#69, fixed.** 153ca1e: conformance.rs's paragraph naming #65's run
  (:663-678 at 7dd8e65) ran straight into `HELD`'s own (:679-686), so
  both were `const HELD`'s doc and `mod upload_after_seeks` had none.
  Swapped, a blank line between; the file's sorted lines compared equal
  before and after, and it builds.
- **#70.** The host's load line, `echo $(date -u +%FT%TZ) $(uptime)`, the
  u5 queue's rule for a run that times seeks, has two homes:
  crates/svrngs-node/tests/conformance.rs:371-378 (`load`, at each
  conformance run's start) and web/tests/conformance.mjs:55 (`load`, at
  each lend's checks' start and end). `tries` reads the second's lines
  back (`seen["load"]`, conformance.rs:776-777 at 153ca1e) and
  scripts/check.sh's awk splits the first's on "averages?: ". They agree
  today. Recorded.
- **#71.** The browser half and the Rust half of the conformance run
  agree on words, not fields: conformance.mjs:64 writes each request's
  `by the worker` / `not by the worker` and conformance.rs:516 finds the
  missed ones by `ends_with("not by the worker")`; page.mjs:32 writes the
  page's host's lines, conformance.mjs:128 drops its files by
  `^\S+ GET \S+ 200$` and conformance.rs:779 judges an upload lost by
  `contains(" POST ")`. And a request that failed outright
  (`requestfailed`, conformance.mjs:65, `failed: <error>`) is in the kept
  JSON but not among the requests the run prints as unanswered by the
  worker (conformance.rs:512-518), though no worker answered it either.
  A field (who answered: the worker, the network, nobody) set once in
  conformance.mjs would give one reading. Not fixed: it changes the kept
  JSON's shape, which demo/runs/u5/2026-10-06-worker-gone/ holds.
- **#57, grown.** web/lend.js:52: the trace a worker's earlier runs kept,
  when the store cannot read it, becomes none (`() => []`), with no line;
  beside the cookies and the app's worker (:128, :376), which #57 names.
- **No change (#72).** checks.js `idle` (:93-98) and `upload` (:83-90)
  end alike, a fetch, its text, `{ ok: res.ok && text === want, saw }`:
  three lines, each check's own route and answer, and the canary's checks
  are a stand-in's, outside lean_check's count. lend.js `sent` and
  `counted` (:214-227) are both a `ReadableStream` with a `pull`: one
  reads a Blob by slices, the other wraps a stream to trace it; a failure
  inside `sent` reaches the dial's fetch, which traces it (lend.js:310).
  The trace's 2,000 lines are `LINES` (lend.js:48) and a comment in
  apps/jellyfin/check/trio.mjs:109, which is still true.
- **#68, #62, #63, #64, #58, #49, #48, #42, still open.** The range
  changed none of their lines (`git diff -U0 d67a455 7dd8e65 --
  web/lend.js`: hunks at :28, :46, :50, :58, :65, :229, :295, :302,
  :396); lend.js's below :229 moved down by up to 35.

### (3) Duplicate type names (`code converge noun`)

`git diff d67a455 7dd8e65 -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: **none** (grep exit 1). The range
added functions, a `const` and a `mod`, no noun, so `converge noun` has
nothing of the range to look up; it was run on the plant (N above). No
noun is defined twice.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 69 | 3 | crates/svrngs-node/tests/conformance.rs:663-687, at 7dd8e65 | 153ca1e | #65's run's doc ran into `HELD`'s, so the constant carried both and the module none. Moved, lines unchanged. |
| 70 | 8 | crates/svrngs-node/tests/conformance.rs:371-378 and web/tests/conformance.mjs:55, at 153ca1e | not fixed | The host's load line written by two programs, read back by `tries` and by check.sh's awk. They agree today; one could hand the other its line. |
| 71 | 8, 1 | web/tests/conformance.mjs:64, :65, :128; web/tests/page.mjs:32; crates/svrngs-node/tests/conformance.rs:512-518, :779, at 153ca1e | not fixed | Who answered a request is a phrase both halves spell, and the page's host's lines are parsed by a regex and by `contains`; a request that failed outright is kept but not shown among those no worker answered. One field set in conformance.mjs would be one reading; it changes the kept JSON. |
| 57 | 1, 6 | as recorded, and web/lend.js:52, at 153ca1e | not fixed | Grown: earlier runs' trace that the store cannot read becomes none, with no line. |
| 72 | 8 | the "no change" items in (2) | no change | `idle` and `upload`'s three-line ending; `sent` and `counted`, two streams that do different things; the trace's line count. |
| 68, 62, 63, 64, 58, 49, 48, 42 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present on the new binary: Carriage at door.rs:924 for :925, the plant's at :1113 for :1114. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)", so web/'s exact tier is could-not-judge, and the range's only product symbols are JS. |
| I10 | 6 | `code dry-report` header | Still present: "1238 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Not retested; every scoped run here used a trailing slash. |
| I14 | 6, 7 | `code dry-report` near tier over JS | Still present: Plant D, a verbatim 11-line JS function, is seen at 0.95 only through its inner `pull`, and itself at 0.8; Plant B, renamed, only at 0.8; spans off by up to three lines. |
| I16 | 6 | `code dry-report --scope apps/` exact tier | Not retested on this binary: nothing was planted in apps/. |

The row's check (LINT): `scripts/check.sh` at 153ca1e on a clean tree,
**run a part at a time, not as one process**: one run took 13:47-16:50
at the last audit, longer than this session's foreground limit of ten
minutes, and it may run nothing in the background. /tmp/u5audit4/part.sh
holds check.sh's `step` and every step's command, copied (a `diff` of the
two scripts' `step` lines differs only in `tests`), with check.sh's
`cargo test --quiet` split by target: the workspace less svrngs-node;
svrngs-node's lib and its short targets; its doc tests; `browser`;
`conformance`. Every part exited 0, every step `passed`, 2026-10-06
22:56:49Z-23:12:47Z (logs /tmp/u5audit4/check-*.log): principles pinned,
lean and RUST_ARCH, app nouns gate, app nouns, rustfmt, clippy, the five
parts of tests, canaries, web rustfmt and clippy, browser, joining in a
browser, conformance canary (every route passed on a computer and with
the wildcard; on the floor `/`-absolute paths, local storage and a post's
Origin `needs the wildcard`), app laws. What this does not show is the
one exit of check.sh itself; that is owed to whoever next has a
foreground long enough, or runs it by hand.

WebKit's seeks back to back, the rate (not a gate), ran twice, once in
`tests` and once in the canary's own step. The host's 1-minute load was
above its 12 cores through both, from work this session did not start
(45.14 when the parts began; 12.85-30.46 across the first seeks, 18.43-
36.17 across the second). First, through a lend: computer 397, 389;
floor 2375, 4296; wildcard **6119 failed**, 3337 ms (Sam, Bo); straight
at the canary 1872-4032, all passed. Second: computer 413, 392; floor
**22896 failed**, 4704; wildcard 3119, 2569; straight 1896-3912, all
passed. check.sh's count: "of 126 here since 2026-10-06T16:42:30Z: 29
missed 5 s through a lend, 5 straight; 86 ended at a load above 12
cores". Both kept, misses included; the host was not quiet again before
this was written (31.80 at 23:13Z). This audit judges no rate: u5-9 and
the operator judge Demo A's.

## REVIEW-audit-u6-auto-1 (2026-10-07, range 90d31d8..4bfe1b8)

The range starts at 90d31d8, the hash REVIEW-audit-u5-auto-4's row
carries (ralph/next/u5/STATE.md:16), the last `REVIEW-audit-` row; the u6
queue has none before this one. It ends at 4bfe1b8, the commit that made
this row, and covers u6-00, u6-0, u6-1 and u6-2. a73dbfc (the operator's,
after 4bfe1b8: apps/jellyfin/ROOM.md and demo/b.md, prose only) is outside
it. The range touched crates/svrngs-core/, crates/svrngs/,
crates/svrngs-node/, web/ and apps/jellyfin/ (`git log --format='%h %s'
90d31d8..4bfe1b8 -- crates apps web scripts`: beb2f0c, 697e233, 57592a3,
1976fa6, e019551, 4567181). This audit's fix is a7afc9e (#73).

The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli`,
which svrngs does not build. The binary used is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-06 19:25,
sha256 7f5fbf4c…d2d1, **not** the one the last audit planted against
(68f2d27b…50f4), so its tiers were planted again. The clean runs are on
corpus `svrngs` at a73dbfc, the tree clean (`code index`, incremental: 26
files, 457 chunks; `refresh`: SCIP at HEAD). The plants went into a clone
of a73dbfc, /tmp/u6audit1/svrngs-plant, one commit holding them (d5ef2db
there), corpus `svrngs-plant` (`code index --full`: 2259 chunks in 450 s;
`refresh`: rust 9272 symbols, python 403, "typescript: skipped (export
failed)").

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 90d31d8..4bfe1b8`,
summed by /tmp/u6audit1/ledger.py: the last audit's rules (a `check/` or
`tests/` segment, or a lean_check test file, is a test; scripts/ is not
counted, and the range changed none of it), but a commit counts toward the
row whose `done` comes next after it, since the u6 queue writes no
`started` commit. Each product commit's subject names that same unit,
apart from 1976fa6 (`fix(jellyfin)`, inside u6-2's span).

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u6-00-drop-floor | 94 | 275 | -181 | 58 | 174 | -116 |
| u6-0-inventory | 0 | 0 | 0 | 0 | 0 | 0 |
| u6-1-room-door | 267 | 5 | 262 | 516 | 0 | 516 |
| u6-2-jellyfin-screen | 34 | 0 | 34 | 563 | 2 | 561 |
| total | 395 | 280 | 115 | 1137 | 176 | 961 |

- u6-00 src: cli.rs −96, group.rs −35, svrngs-core link.rs −27, web/app.js
  −15, lend.js −4, open.js −2, store.js −1, door.rs −1 (the floor gone,
  `checked_page` and the page origin kept).
- u6-1 src: svrngs-node room.rs +179 (new), cli.rs +46 (`room`), node.rs
  +21 (`label`, `room_address`, `Host::room`), origin.rs +8 (`reach`),
  surface.rs +5 (`room` in `/v1/lends`), lib.rs +3. Tests: tests/room.rs
  +363, room_tests.rs +106, crates/svrngs/tests/new.rs +47.
- u6-2 src: apps/jellyfin/ROOM.md +34, prose, in the ledger because it is
  under apps/. Tests: apps/jellyfin/check/screen.mjs +334, screen.py +115,
  screen.sh +110, person.mjs +2.

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive` in
/tmp/u6audit1:

| at | svrngs-core | repository |
|---|---:|---:|
| 90d31d8, range start | 2951 / 3000 | 13932 / 14000 |
| 4bfe1b8, range end | 2931 / 3000 | 13976 / 14000 |
| a73dbfc, the operator's after it | 2931 / 3000 | 13976 / 14000 |

On this worktree lean_check said 13996 at a73dbfc and 13995 at a7afc9e:
20 lines more than the archive, all of them build output (#74). The
repository rose 44 lines over the range, u6-00's floor having given back
what u6-1's door took, to **99.8%** of its ceiling. 24 lines are left in
a clean tree, 5 on this worktree; svrngs-core has 69 left. A raise is the
operator's; this audit names it and does not make it.

### (2) Clones (`code dry-report`), each tier checked against a plant first

**Plants**, each a symbol of the range:

- **A** (exact, Rust): room.rs `from_this_network` (u6-1), verbatim and
  named the same, in a `pub mod plant` at the end of door.rs (:1113).
- **B** (near, Rust): room.rs `pick` (u6-1) as `choose` at the end of
  surface.rs (:188), its locals and parameters renamed.
- **D** (JS, verbatim body): apps/jellyfin/check/screen.mjs `until` (u6-2)
  as `untilAgain` at the end of drive.mjs (:84).
- **P** (Python, verbatim body): apps/jellyfin/check/screen.py `elsewhere`
  (u6-2) as `elsewhere_again` at the end of judge.py (:91).
- **N** (noun): room.rs `pub struct Room` in the plant module.

| scope | clean at 0.95: locations / exact / near | plants seen |
|---|---|---|
| crates/svrngs-node/ | 367 / 5 / 1 | **A exact** at 0.95, 0.9, 0.8 (door.rs:1113 == room.rs:161), and near from 0.9 at door.rs:1110–1130 ~ room.rs:158–178. **B missed** at 0.95, 0.9 and 0.8; it joins `pick` at 0.7, in one cluster of 254 symbols (cosine 0.700–0.932) |
| crates/svrngs/ | 64 / 0 / 0 | not planted |
| crates/svrngs-core/ | 421 / 7 / 0 | not planted |
| web/ | 85 / **could-not-judge** / 0 | not planted: the range added no web/ symbol |
| apps/ | 70 / 0 / 0 | **D missed** at 0.95, 0.9, 0.8; it joins `until` at 0.7 in a cluster of 9 (0.706–0.828). **P missed** at 0.95 and 0.9; seen at 0.8 as a pair, cosine 0.842. The exact tier says 0 groups, not could-not-judge, with D in scope (I16) |

All four planted functions are in the tool's symbol set (`--threshold 0.0
--min-lines 1` scoped to each file lists `choose`, `pick`, `untilAgain`
and `until`), so the misses come from the near tier's similarity, not from
symbols left out (I13 does not apply).

N: `converge noun Room` on the real corpus gives **definitions (1)**,
room.rs:78 (it is at :79, I1); the plant was not run through converge,
because (3) below needed none.

**Clean runs.** Scoped at 0.95 (the table), each with a trailing slash
(I11), and unscoped at 0.9: **14 exact groups and 28 near clusters** over
1266 function/method symbols (14 and 29 over 1238 at the last audit).
/tmp/u6audit1/blame.py blamed every side of every cluster (`git blame -w
-M -C -C HEAD`) for lines in 90d31d8..4bfe1b8: **no cluster in any run has
a side added in the range** (`grep -c ^ADDED` on each `.blame`: 0, over 42
clusters unscoped, 6 in crates/svrngs-node/, 7 in crates/svrngs-core/,
none elsewhere).

**Substitute check, named as a substitute**, since the near tier saw none
of B, D and P at 0.9: /tmp/u6audit1/windows.py (windows of 5 normalized
lines over .rs .js .mjs .py .sh, tests included; a hit is new when either
span has a line blamed in the range), unchanged from the last audit. On
this tree at a73dbfc: 253 hits, **13 new**, every one in tests: the
`.send().await.unwrap().json().await.unwrap()` chain of a reqwest call in
crates/svrngs-node/tests/room.rs:194-200 and :325-330, the same chain in
conformance.rs:114-118, lend_origin.rs:176-181 and record_api.rs:230-236,
:349-354 (#76). On the plant clone: those 13 and 3 more, A
(door.rs:1113-1125 == room.rs:161-173), D (drive.mjs:85-91 ==
screen.mjs:98-104) and P (judge.py:92-101 == screen.py:38-47). B, renamed,
it cannot see.

Reading the range for what neither pass would show:

- **#73, fixed.** a7afc9e: u6-1 copied `unshare`'s refusal of a lend's
  name into `room` (crates/svrngs/src/cli.rs:308 and :315 at 4bfe1b8), and
  `--name` (:210) spelled it a third time with its flag first. Now
  `lend_name` (cli.rs:373 at a7afc9e). `svrngs` built before and after on
  seven arguments, among them `unshare a.b`, `room ok a.b` and `share 8080
  --name a.b`: the outputs compared equal.
- **#74.** scripts/lean_check.py:33-35 leaves out `target/` only at the
  root (`rel.startswith(("apps/", "target/"))`), so web/target/, the page's
  wasm build, is counted: six `out/private.rs` files of serde, serde_core
  and thiserror under web/target/wasm32-unknown-unknown/release/build/, 20
  lines (a per-file comparison of `git archive a73dbfc` and this worktree:
  cli.rs aside, those six files are the whole difference). The budget's
  verdict therefore depends on what was built: 13996 here against 13976 in
  the archive, with 24 lines of headroom. Another dependency's build
  script writing a `private.rs` would break check.sh with no code changed.
  Not fixed: it changes the gate's count, so it needs a plant of its own,
  and the budget is the operator's.
- **#75.** crates/svrngs-node/src/room.rs:127-129 at a7afc9e: a lend that
  `room` was asked for, and that is lent, but whose key `label` cannot
  read, is skipped with `continue` and no trace. `Said::Room` then lists
  only what was served, so the person sees nothing for it. The same silent
  drop was already in `/v1/lends` (surface.rs:171, `host.origin(...)?`,
  before the range). Neither can happen while every key comes from the
  record as actor hex; a debug line at each would say so if one ever does.
  Not fixed: it adds a trace, which this audit leaves to a unit.
- **#76, no change.** The 13 new windows above are reqwest's call and
  decode chain in five test files, the crate's idiom, each with its own
  request. node.rs:250 names the room door's `Asking` clone `room` and
  stores it as `asking: room` (:270), named for its use and documented at
  the field (:120).
- **#57, #68, #70, #71, still open.** The range changed none of their
  lines.

### (3) Duplicate type names (`code converge noun`)

`git diff 90d31d8 4bfe1b8 -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: `pub struct Room` and `pub enum
RoomError` (room.rs, u6-1). `converge noun`:

- `Room`: **definitions (1)**, crates/svrngs-node/src/room.rs:78 (at :79).
- `RoomError`: **definitions (1)**, crates/svrngs-node/src/room.rs:96 (at
  :97).

No noun is defined twice.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 73 | 8 | crates/svrngs/src/cli.rs:210, :308, :315, at 4bfe1b8 | a7afc9e | A lend's name refused in three places with one message, the third copied by u6-1. Now `lend_name`; output compared equal. |
| 74 | 6, 7 | scripts/lean_check.py:33-35, and web/target/wasm32-unknown-unknown/release/build/*/out/private.rs, at a7afc9e | not fixed | The size budget counts the page's build output (20 lines here), so its verdict depends on what was built; 5 lines of headroom on this worktree, 24 in a clean tree. |
| 75 | 1, 6 | crates/svrngs-node/src/room.rs:127-129, and crates/svrngs-node/src/surface.rs:171, at a7afc9e | not fixed | A named, lent lend whose key does not read is dropped with no line, at the room door and in `/v1/lends`. |
| 76 | 8 | the "no change" items in (2) | no change | The tests' reqwest chains; the `room` local in node.rs. |
| 57, 68, 70, 71 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present on the new binary: `Room` at room.rs:78 for :79, `RoomError` at :96 for :97. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)"; web/'s exact tier is could-not-judge. |
| I10 | 6 | `code dry-report` header | Still present: "1266 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Not retested; every scoped run used a trailing slash. |
| I14 | 6, 7 | `code dry-report` near tier and spans | Still present, now also over Rust: B, a renamed 14-line Rust function, is missed down to 0.8 and joins only at 0.7 in a cluster of 254. D, a verbatim 9-line JS body, joins only at 0.7. Spans are off: `from_this_network` 1110–1130 for 1113–1125, `pick` 39–58 for 42–55, `elsewhere` 34–50 for 37–47. |
| I16 | 6 | `code dry-report --scope apps/` exact tier | Retested: "0 exact-clone groups" with D, a verbatim JS body, in scope, where web/ says could-not-judge for the same language. |
| I17 | 6, 7 | `code dry-report` over Python | New: P, a verbatim 11-line Python body under another name, is not an exact group and is missed by the near tier at 0.95 and 0.9; it is seen at 0.8, cosine 0.842. The 5-line window pass sees it. |

The row's check (LINT): `scripts/check.sh`'s lint steps at a7afc9e on a
clean tree, /tmp/u6audit1/part.sh `lint`, 2026-10-07T04:05:19Z: principles
pinned, lean and RUST_ARCH, app nouns gate, app nouns, rustfmt, clippy,
every one `passed`, failed=0 (/tmp/u6audit1/check-lint.log). The rest of
check.sh at a7afc9e, **a part at a time, not as one process** (one run is
longer than this session's ten-minute foreground). part.sh is the last
audit's, with `--test room` added to the short targets because u6-1 added
tests/room.rs. Every part exited 0 and every step `passed`, 04:05:37Z to
04:27:40Z (logs /tmp/u6audit1/check-*.log): the three test parts, browser
tests, conformance tests, canaries, web rustfmt and clippy, browser,
joining in a browser, the conformance canary (every route passed on a
computer and with the wildcard) and app laws. What this does not show is
one exit of check.sh itself, which is owed to whoever next has a
foreground long enough. The host's 1-minute load was 16-75 on 12 cores
throughout, from work this session did not start. WebKit's seeks back to
back, the rate and not a gate: through a lend, computer 456, 479 and
wildcard 4080, 3359, all passed; straight at the canary 2111-2929, all
passed. check.sh's count: "of 36 here since 2026-10-07T01:30:06Z: 8 missed
5 s through a lend, 5 straight; 28 ended at a load above 12 cores". This
audit judges no rate.

## #77: a join that cannot reach the issuer says why to nobody (the seat, 2026-10-07)

Setting up the operator's Demo B on this host (`apps/jellyfin/check/screen.sh
up`, PAGE=https://group.svrnme.sh/), Dee's `svrngs join <svrngs:// link>`
printed "the computer that sent this link cannot be reached; it may be off,
so try again later" and exited within 60 s, while Sam's node was up and Alex
had joined by the same kind of link six seconds before. Sam's log shows no
attempt; Dee's `join.log` is empty at the default level, so which hop failed
(address lookup, relay, the dial) is unrecorded. Retried by hand at once with
`RUST_LOG=info,svrngs=debug,svrngs_node=debug`, it joined in 2 s. Principle
1: the refusal's branch carries no event a default run keeps. Not fixed.

## REVIEW-audit-ceiling-auto-1 (2026-10-08, range cd58f97..824ce20)

The range starts at cd58f9755, the hash REVIEW-audit-u6-auto-1's row
carries (ralph/next/u6/STATE.md:7), the last `REVIEW-audit-` row; the
ceiling queue has none before this one. It ends at 824ce20, the commit that
made this row: 42 commits, u6-9 (Demo B's dry run), the #74 fix (910e373)
and ceiling-1 to ceiling-4. The range's product code is in
crates/svrngs-core/, crates/svrngs-runtime/, crates/svrngs-node/ and
apps/jellyfin/; it touched crates/svrngs/ and web/ only in tests, and
scripts/ in 910e373 (lean_check.py +15/−1) and 2f838b5 (check.sh +6, the
ceiling canary). This audit fixes nothing; why each finding is left is in
its row.

The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli`,
which svrngs does not build. The binary used is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-07 18:30,
sha256 8bfb8a59…, **not** the one the last audit planted against
(7f5fbf4c…), so its tiers were planted again. The clean runs are on corpus
`svrngs` at 824ce20, the tree clean (`code index`, incremental: 31 files,
495 chunks in 33 s; `refresh`: SCIP at HEAD). The plants went into a clone
of 824ce20, /tmp/ceilaudit1/svrngs-plant, one commit holding them (64a91f7
there), corpus `svrngs-plant` (`code index`: 2327 chunks in 184 s;
`refresh`: rust 9552 symbols, python 436, "typescript: skipped (export
failed)").

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' cd58f9755..824ce20`,
summed by /tmp/ceilaudit1/ledger.py. A commit counts toward the unit its
subject names in parentheses, else toward the next `done` commit's unit
(0245640, 4b2cdfa: u6-9; the loop's own commits change no product code).
Product code is crates/, apps/, web/ and bridge/; a test is a `tests/` or
`check/` segment, or a file lean_check counts as a test (`*_tests.rs`,
`tests_*.rs`, `tests.rs`, `*.test.mjs`); lockfiles are left out.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| u6-9 | 8 | 5 | 3 | 535 | 114 | 421 |
| #74 (910e373) | 0 | 0 | 0 | 0 | 0 | 0 |
| ceiling-1 | 12 | 2 | 10 | 327 | 26 | 301 |
| ceiling-2 | 142 | 197 | -55 | 422 | 63 | 359 |
| ceiling-3 | 40 | 2 | 38 | 83 | 0 | 83 |
| ceiling-4 | 77 | 48 | 29 | 299 | 47 | 252 |
| total | 279 | 254 | 25 | 1666 | 250 | 1416 |

- u6-9 src: apps/jellyfin/ROOM.md +8/−5, prose under apps/. Tests: tv.mjs
  +123 and screen.mjs −102, the move of 52d8168; night.mjs +232, night.py
  +107, screen.sh +46, check.sh +9/−4, person.mjs +6/−3, web/tests/page.mjs
  +6/−3 (`device`).
- ceiling-1 src: svrngs-runtime journal.rs +12/−2, the digest and offer
  folds' debug lines (b47db37). Tests: scale.rs +327/−26.
- ceiling-2 src: svrngs-core sync.rs +78/−29 (`DigestFold`), lib.rs +3/−1;
  svrngs-runtime journal.rs +57/−63, register.rs −99 (its tests moved to
  register_tests.rs, ef7251e), lib.rs +2/−3. Tests: sync_tests.rs +114,
  register_tests.rs +100, tests_write_index.rs +45, ring_sync/tests.rs +59,
  runtime tests.rs +32/−35, scale.rs +70/−22.
- ceiling-3 src: journal.rs +40/−2 (`Passed`, `Checked`). Tests:
  tests_write_index.rs +51, scale.rs +32.
- ceiling-4 src: journal.rs +37 (the kept standing); register.rs +22/−21;
  svrngs-node −29/+18 over eight files, `&Ed25519Verifier` dropped from
  every `standing` and `standing_anywhere` call (61de5bc). Tests: scale.rs
  +204/−34 (the canary, the line and its predictions), register_tests.rs
  +87/−1, three test files +8/−12.

`python3 scripts/lean_check.py <tree>` (rc 0), trees from `git archive` in
/tmp/ceilaudit1 (lean_check walks a tree outside git whole, and says so):

| at | svrngs-core | repository |
|---|---:|---:|
| cd58f97, range start | 2931 / 3000 | 13975 / 14000 |
| 824ce20, range end | 2957 / 3000 | 13955 / 14000 |

This worktree at 824ce20 says the same, 2957 and 13955: since 910e373 the
budget counts what git calls source, so the 20 lines of build output #74
found no longer count. svrngs-core rose 26 lines (`DigestFold`) and has 43
left; the repository fell 20 and has 45 left, **99.7%** of its ceiling
(#80). A raise is the operator's; this audit names it and does not make it.

### (2) Clones (`code dry-report`), each tier checked against a plant first

**Plants**, each a symbol of the range:

- **A** (exact, Rust): journal.rs `Held::catch_up` (ceiling-2 to -4, 48
  lines), verbatim with the two doc lines above it, in an `impl Plant` at
  the end of register.rs (:190).
- **B** (near, Rust, across crates): svrngs-core sync.rs
  `DigestFold::digest` (ceiling-2, 27 lines) as `summary` in an `impl
  PlantB` at the end of svrngs-runtime oplog.rs (:776), its locals renamed
  (`floors`→`marks`, `actor`→`who`, `seqs`→`runs_of`, `mark`→`top`,
  `view`→`seen`).
- **D** (JS, verbatim body): apps/jellyfin/check/tv.mjs `playOn` (u6-9) as
  `playOnAgain` at the end of night.mjs (:235).
- **N** (noun): `pub struct DigestFold` again at the end of register.rs
  (:181).

| scope | clean at 0.95: locations / exact / near | plants seen |
|---|---|---|
| crates/svrngs-runtime/ | 122 / 1 / 0 | **A exact** at 0.95, 0.9, 0.8 (journal.rs:128 == register.rs:190, both lines right), and near from 0.95 at journal.rs:125–179 ~ register.rs:187–239. **B not in scope**: its original is in svrngs-core, and a scoped run compares within its scope (I12) |
| crates/svrngs-core/ | 428 / 7 / 0 | not planted |
| crates/svrngs-node/ | 368 / 5 / 1 | not planted |
| crates/svrngs/ | 65 / 0 / 0 | not planted |
| apps/ | 79 / 0 / 1 | **D near** at 0.95, 0.9, 0.8 (night.mjs:232–246 ~ tv.mjs:109–123). The exact tier says 0 groups, not could-not-judge, with D in scope (I16) |
| web/ | 85 / **could-not-judge** / 0 | not planted: the range added no web/ symbol outside tests |
| unscoped | — | **B missed** at 0.9 and 0.8; it joins `digest` only at 0.7, in a cluster of 5 symbols (cosine 0.706–0.873) with view.rs `fold_pairs` and two tests (I14) |

N: `converge noun DigestFold` on the plant corpus gives **definitions
(2)**, sync.rs:207 and register.rs:180 (at :208 and :181, I1).

**Clean runs**, scoped at 0.95 (the table), each with a trailing slash.
/tmp/ceilaudit1/blame.py blamed every side of every cluster (`git blame -w
-M -C -C HEAD`) for lines in cd58f97..824ce20. **One cluster has sides
added in the range**, and it is not a clone:

- apps/: `playOn` tv.mjs:109–123 ~ `menu` tv.mjs:112–123, cosine 0.959,
  both blamed to 52d8168 (u6-9's move). `menu` is the closure inside
  `playOn` (tv.mjs:115), so the tool paired a function with a part of
  itself (I9). No change.

Every other side (13 exact groups and 1 near cluster over core, runtime and
node, the same ones the last audits read) is blamed before the range.

**Substitute check, named as a substitute**, since the near tier sees B
only at 0.7: /tmp/ceilaudit1/windows.py, windows of 5 normalized lines over
the tracked .rs .js .mjs .py .sh (tests included, demo/ left out); a hit
is new when either span has a line blamed in the range. At 824ce20: 156
hits, **0 new**. On the plant clone, against cd58f97..64a91f7: 158 hits,
**2 new**, A (journal.rs:128-175 == register.rs:190-237) and D
(night.mjs:236-245 == tv.mjs:113-122). B, renamed, it cannot see.

Reading the range for what neither pass would show:

- **#79.** crates/svrngs-core/src/sync.rs:234-237 at 824ce20:
  `DigestFold::digest` marks every actor stale when a seal or a correction
  moves the sealed floors, so the next digest folds every view again, and
  nothing traces that branch. The cost ceiling-2 removed comes back for
  that one call, and ceiling-1's account sees only the journal's line
  (journal.rs:304, `restarted`), which says false. svrngs-core traces
  elsewhere (`tracing` in its Cargo.toml:20; membership.rs, admit.rs).
  Not fixed: it adds a trace, which this audit leaves to a unit, as #75 was.
- **#81.** web/tests/page.mjs:60, :72-73 at 824ce20, test code: `launch`'s
  `device`, added by u6-9 (2f3f2c9), looks its name up in `pw.devices`, and
  a name Playwright does not have gives `undefined`, so the browser
  launches as no device and the run says nothing. An unknown engine at :61-63
  is refused with exit 1. Demo B's phones pass 'Pixel 7'
  (apps/jellyfin/check/screen.sh:110, through night.mjs:37), which exists. Not fixed: refusing a name that is not there changes
  behaviour on that input.
- **#74, fixed in the range** by 910e373; the archive and this worktree now
  agree (above).
- **#57, #68, #70, #71, #75, #77, still open.** The range changed none of
  their lines: surface.rs changed only at :36 and :88 (61de5bc), page.mjs only at
  :57-73.

### (3) Duplicate type names (`code converge noun`)

`git diff cd58f9755 824ce20 -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: `DigestFold` (sync.rs, ceiling-2),
`Passed` and `Checked` (journal.rs, ceiling-3), `Doors` (runtime
tests.rs, ceiling-2), and `Says`, `Hold`, `Line`, `Counter`, `Cost`
(svrngs-node tests/scale.rs, ceiling-1 and -4). `converge noun` on each,
beside `git grep -nP '\b(struct|enum|trait|type|class) <Name>\b' HEAD`:

| noun | converge | git grep |
|---|---|---|
| DigestFold | definitions (1), sync.rs:207 (at :208) | 1 |
| Passed | definitions (1), journal.rs:93 (at :94) | 1 |
| Checked | definitions (1), journal.rs:99 (at :100) | 1 |
| Doors | definitions (1), tests.rs:25 (at :26) | 1 |
| **Says** | **definitions (1)**, tests_sealing.rs:16 | **2**: crates/svrngs-core/src/tests_sealing.rs:17 `struct Says(bool)` and crates/svrngs-node/tests/scale.rs:356 `enum Says` |
| Hold, Line, Counter, Cost | definitions (0), "defined only in test code" | 1 each, scale.rs:423, :434, :452, :508 |

- **#78, no change.** `Says` twice, both test-only, in two crates and two
  concepts: svrngs-core's is a verifier stub with a fixed answer
  (tests_sealing.rs:14-17), scale.rs's the words a fold's debug line uses
  (scale.rs:353-362). Neither can import the other (an integration test
  of svrngs-node and a `#[cfg(test)]` module of svrngs-core). The tool
  counted one of them (I18).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 78 | 8 | crates/svrngs-core/src/tests_sealing.rs:17 and crates/svrngs-node/tests/scale.rs:356, at 824ce20 | no change | `Says` defined twice, test-only, two concepts in two crates. |
| 79 | 1 | crates/svrngs-core/src/sync.rs:234-237, at 824ce20 | not fixed | A moved floor makes the digest fold every view again, and that branch traces nothing; the journal's line says `restarted=false`. |
| 80 | — | scripts/lean_check.py BUDGET, at 824ce20 | not fixed (operator's) | 43 lines left in svrngs-core, 45 in the repository (13955 / 14000). |
| 81 | 6 | web/tests/page.mjs:60, :72-73 (and :61-63, the engine refused), at 824ce20 | not fixed | An unknown `device` name launches as no device, silently; an unknown engine is refused. |
| 74 | 6, 7 | scripts/lean_check.py | 910e373 | Fixed in the range. |
| 57, 68, 70, 71, 75, 77 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Half fixed on this binary: dry-report's exact tier now gives the right line (`catch_up` journal.rs:128, plant A register.rs:190). converge noun is still one line high: `DigestFold` sync.rs:207 for :208, the plant register.rs:180 for :181, `Passed` :93 for :94. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". |
| I9 | 6, 7 | `code dry-report` near tier | Still present: `menu`, a closure inside `playOn` (tv.mjs:115), is reported as `playOn`'s near clone, 0.959. |
| I10 | 6 | `code dry-report` header | Still present: "1299 function/method symbols" in every scope. |
| I11 | 6 | `code dry-report --scope` | Better: `--scope crates/svrngs/` now says it leaves out svrngs-core, -node and -runtime, "which share it only as a string prefix". Not retested without the slash. |
| I12 | 5, 6 | `code dry-report --scope` | Still present: B, whose original is in another crate, is in no scoped report. |
| I14 | 6, 7 | `code dry-report` near tier | Still present: B, a renamed 27-line Rust body, is missed unscoped at 0.9 and 0.8 and joins only at 0.7 in a cluster of 5. |
| I16 | 6 | `code dry-report --scope apps/` exact tier | Still present: "0 exact-clone groups" with D, a verbatim JS body, in scope. |
| I18 | 6 | `code converge noun` | New, I8's rule seen from converge: a definition in a `src/*_tests.rs` file counts as production and one under `tests/` does not, so `Says`, defined twice, gives definitions (1), and scale.rs's nouns give (0) beside a sibling that gives (1). |

## REVIEW-audit-mw-auto-1 (2026-10-08, range ea4b7ed..e9ed277)

The range starts at ea4b7ed12, the hash REVIEW-audit-ceiling-auto-1's row
carries (ralph/next/ceiling/STATE.md:15), the last `REVIEW-audit-` row; the
mw queue has none before this one. It ends at e9ed277, the commit that made
this row: 17 commits, the last audit's `done` (cdee0c0), the queue (5e877c9,
3b4c608) and mw-0 to mw-3. The range's product code is all under
apps/jellyfin/: the middleware, apps/jellyfin/mw/ (a Cargo workspace of its
own, apps/jellyfin/mw/Cargo.toml:6), and its check, apps/jellyfin/check/.
`git diff ea4b7ed..e9ed277 -- crates web | wc -l` is 0: the spike's fence
held, so there is no crate dir under crates/ for (2) to scope. This audit
fixes one finding (#84) and records the rest.

The row cites `docs/ARCH_PRINCIPLES.md`, which is at
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli`,
which svrngs does not build. The binary used is
~/dev/commonwealth-ai/target/debug/sovereign-cli, built 2026-10-08 06:36,
sha256 5f5e2d74…, **not** the one the last audit planted against
(8bfb8a59…), so its tiers were planted again. No embedder was running
(`code index` refused: "no embedder"), so this session ran `llama-server`
on the corpus's own model (qwen-embedding-0.6b.gguf, the path
~/.svrnmesh/config.toml:18 names) on :8080 for its length. The clean runs
are on corpus `svrngs` at e9ed277, the tree clean (`code index`,
incremental: 12 files, 131 chunks in 13 s; `refresh`: SCIP at HEAD). The
plants went into a clone of e9ed277, /tmp/mwaudit1/svrngs-plant, one commit
holding them (7035261 there), corpus `svrngs-plant` (`code index --full`:
2448 chunks in 324 s; `refresh`: rust 9535 symbols, python 613,
"typescript: skipped (export failed)").

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' ea4b7ed..e9ed277`,
summed by /tmp/mwaudit1/ledger.py (REVIEW-audit-ceiling-auto-1's, its unit
pattern `mw-\d`). The same rules: a commit counts toward the unit its
subject names, else toward the next `done` commit's; product code is
crates/, apps/, web/ and bridge/; a test is a `tests/` or `check/` segment
or a lean_check test file name; lockfiles are left out (Cargo.lock, +1106).
Tests inline in a src file (`#[cfg(test)] mod tests` in page.rs:86,
person.rs:202, swap.rs:99, viewer.rs:376) count as src, as before.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| mw-0 | 0 | 0 | 0 | 0 | 0 | 0 |
| mw-1 | 1686 | 299 | 1387 | 272 | 1 | 271 |
| mw-2 | 522 | 39 | 483 | 205 | 32 | 173 |
| mw-3 | 54 | 9 | 45 | 0 | 0 | 0 |
| total | 2262 | 347 | 1915 | 477 | 33 | 444 |

- mw-1 src: viewer.rs +747 (62560f0, upstream's `provision` unchanged),
  then +176/−293 (717f433) and +3 (0d3feb6); forward.rs +257, person.rs
  +259, people.rs +97, main.rs +11 then +101/−6, Cargo.toml +26 and +9.
  Tests: check/mw.py +211, check.sh +61/−1 (705d217).
- mw-2 src: forward.rs +132/−15, swap.rs +160, page.rs +118, people.rs
  +70/−22, main.rs +29/−2, person.rs +13 (7e0e1c5). Tests: judge.py
  +81/−7, check.sh +37/−8, person.mjs +26/−6, scan.py +22/−10, tokens.py
  +35, drive.mjs +4/−1 (43c44e2).
- mw-3 src: apps/jellyfin/README.md +45, prose; Cargo.toml and main.rs
  +3/−3 and +6/−6, comments (f4e706f).

`python3 scripts/lean_check.py <tree>` (rc 0) on `git archive` trees in
/tmp/mwaudit1:

| at | svrngs-core | repository |
|---|---:|---:|
| ea4b7ed, range start | 2957 / 3000 | 13955 / 14000 |
| e9ed277, range end | 2957 / 3000 | 13970 / 14000 |

The worktree at e9ed277 says the same. The 15 lines are
demo/runs/mw/2026-10-08/mw-2/run.sh (+23, the run's rerun script); the
middleware's 1915 lines count nowhere, as apps/ is outside the budget
(scripts/lean_check.py:14, :49). The repository has 30 lines left,
**99.8%** of its ceiling (#80, still open, the operator's).

### (2) Clones (`code dry-report`), each tier checked against a plant first

**Plants**, each a symbol of the range:

- **A** (exact, Rust, inside the middleware): swap.rs `inward` (mw-2, 25
  lines and its 3 doc lines), verbatim, at the end of page.rs (:120).
- **B** (near, Rust, across the two workspaces): forward.rs `send` (mw-1,
  28 lines) as `dial_once` at the end of svrngs-node's lib.rs (:95), its
  names changed (`jellyfin`→`service`, `stream`→`tcp`, `sender`→`tx`,
  `conn`→`link`, which also made `connect` `linkect`; "mw:"→"door:").
- **D** (Python, verbatim body): check/mw.py `socket_through` (mw-1) as
  `socket_again` at the end of judge.py (:164).
- **N1, N2** (nouns): `pub(crate) struct Signed` again at the end of the
  middleware's main.rs (:136); `pub struct PersonKey(String)` at the end
  of svrngs-node's lib.rs (:127).

| scope | clean at 0.95: locations / exact / near | plants seen |
|---|---|---|
| apps/jellyfin/mw/ | 72 / **could-not-judge** ("no SCIP function symbols in scope") / 0 | **A missed**: the exact tier cannot judge, and the near tier has 0 clusters with A in scope (73 locations) |
| apps/ | 162 / 0 / 1 (`playOn` ~ `menu`, I9) | **D missed** at 0.95 (it is at 0.940) |
| crates/svrngs-node/ | not run clean: the range has no symbol there | B not in scope's clusters (I12) |
| unscoped 0.95 | 14 exact / 12 near | A, B, D all missed |
| unscoped 0.9 | 14 / 36 | **D seen**, judge.py:162–199 ~ mw.py:66–104, 0.940; A and B missed |
| unscoped 0.8 | 14 / 208 | D seen; A and B missed |
| unscoped 0.7 (plant corpus only) | 14 / 72 | A and B seen only inside one cluster of **846 symbols** (cosine 0.700–0.978), beside forward.rs `send` and swap.rs `inward` (I20, I14) |

N1: `converge noun Signed` on the plant corpus gives **definitions (0)**
where `git grep` finds two (main.rs:136, people.rs:46). N2: `PersonKey`
gives **definitions (1)**, the plant at lib.rs:126 (at :127, I1), where
`git grep` finds two (I19).

**Clean runs.** /tmp/mwaudit1/blame.py (the last audit's, over
/tmp/mwaudit1/dry-*.txt) blamed every side of every cluster, 843 sides,
for lines in ea4b7ed..e9ed277. Scoped at 0.95, apps/jellyfin/mw/ and apps/
have **0** clusters with a side added in the range. Unscoped (the plants
show the scoped near tier cannot see across workspaces), the clusters with
an added side:

- 0.95, **1**: person.mjs `watch` (:18–43, 43c44e2) ~
  demo/runs/u5/2026-10-05-jellyfin/drive.mjs `watch` (:37–62), 0.992.
  The other side is a recorded run's script, kept as it ran (AGENTS.md 7);
  no change.
- 0.9, **5**: the above; judge.py `mw_bars` (:51–108) ~ `bars`
  (:105–148), 0.939, and their closures `why` (:58–69) ~ `why`
  (:114–123), 0.947 — **#84**; two pairs of tests inside viewer.rs's
  forked `mod tests` (:449–471 ~ :470–497, 0.917; :573–588 ~ :587–602,
  0.908), upstream's as they landed (62560f0); no change.
- 0.8, **17**, among them the only cluster crossing the two workspaces:
  forward.rs `upgrade_of` (:116–130) and `drop_hop` (:126–140) ~ door.rs
  `upgrade_of` (:416–430) and `keep_upgrade` (:427–436), 0.810–0.854 —
  **#82**; and person.rs's test `headers` (:204–216) ~ door/tests.rs `map`
  and `identity`, 0.837–0.866, test builders of the same headers; no
  change. The rest pair symbols of one middleware file with each other.

**Substitute check, named as a substitute**, since A is invisible to both
tiers: /tmp/ceilaudit1/windows.py, windows of 5 normalized lines over the
tracked .rs .js .mjs .py .sh. At e9ed277: 161 hits, **5 new**: judge.py's
imports (:34–38) == night.py's and screen.py's, and three in viewer.rs
(:285–289 == :311–315, :300–307 == :324–331, :456–461 == :480–485),
upstream's `call(...)` sequences and tests as forked; none to change. On
the plant clone, against ea4b7ed..7035261: 163 hits, **7 new**, the same
five and A (page.rs:124–142 == swap.rs:37–55) and D (judge.py:166–197 ==
mw.py:70–101). B, renamed, it cannot see.

Reading the range for what neither pass would show:

- **#82.** apps/jellyfin/mw/src/forward.rs:39-48 and :129-137 against
  crates/svrngs-node/src/door.rs:142-151 and :387-409, at e9ed277: two
  lists of the headers that belong to one hop, which disagree. The
  middleware's drops `proxy-authenticate` and keeps `trailer`; the door's
  drops `trailer` and keeps `proxy-authenticate`; and the door also drops
  every header a `Connection` names (RFC 9110 §7.6.1), which `drop_hop`
  does not. `upgrade_of` is the same rule twice (forward.rs:119-127,
  door.rs:415-423), the middleware's comparing with
  `eq_ignore_ascii_case` and the door's through `compare_name`
  (door.rs:362-368, which first drops every byte outside printable
  ASCII). Principle 8: two implementations of one rule with no
  equivalence test. They are two programs on purpose (DESIGN §9: svrngs
  ships the middleware no SDK, and the spike's fence kept crates/ apart),
  so the body cannot be shared. Not fixed: aligning either list changes
  what reaches Jellyfin or a browser. **Closed in a81178d** (mwp-3):
  forward.rs is deleted, its proxy and hop list with it; the door is the
  one proxy and its list the one list (`ls apps/jellyfin/mw/src/`:
  answer, main, page, people, person, viewer; `grep -n HOP` over them
  finds nothing).
- **#57, #68, #70, #71, #75, #77, #79, #81, still open.** The range changed
  no line under crates/ or web/.

### (3) Duplicate type names (`code converge noun`)

`git diff ea4b7ed e9ed277 -- crates apps web`, added lines matching
`(struct|enum|trait|type|class) Name`: 16 nouns, every one in
apps/jellyfin/mw/src/. `converge noun` on each, beside `git grep -nP
'\b(struct|enum|trait|type|class) <Name>\b' HEAD -- crates apps web`
(/tmp/mwaudit1/converge.txt):

| noun | converge | git grep |
|---|---|---|
| Ask, At, DeviceFrom, Known, NotTheDoors, On, People, PersonKey, Plant, Server, Signed | definitions (0), "check the spelling, or the name is defined only in test code" | 1 each, all in apps/jellyfin/mw/src/ |
| **Body** | definitions (1), admit.rs:765 | **2**: apps/jellyfin/mw/src/forward.rs:24 `type Body = BoxBody<…>` and crates/svrngs-core/src/admit.rs:766, a struct local to one function |
| **Device** | definitions (1), lib.rs:517 | **2**: apps/jellyfin/mw/src/person.rs:77 `struct Device` (a device as Jellyfin's `MediaBrowser` header names one) and crates/svrngs-core/src/lib.rs:518 `enum Device` (what holds a standing key) |
| **Origin** | definitions (1), link.rs:173 | **2**: apps/jellyfin/mw/src/viewer.rs:385, test-only, upstream's fake Jellyfin; crates/svrngs-core/src/link.rs:174 |
| **Viewer** | definitions (1), limit.rs:85 | **2**: apps/jellyfin/mw/src/viewer.rs:53 (a person's Jellyfin account) and crates/svrngs-node/src/door/limit.rs:86 (one person's use of one lend) |
| **Who** | definitions (1), door.rs:599 | **2**: apps/jellyfin/mw/src/person.rs:36 `struct Who` (the key and name the door's headers carry) and crates/svrngs-node/src/door.rs:600 `enum Who` (how the dialer reached the door) |

- **#83, no change.** Five names defined twice, each pair two concepts in
  two programs that cannot import each other (DESIGN §9; the middleware
  is its own workspace). The nearest, `Who`, is the door's dialer on one
  side and what the door writes into `x-svrngs-key` and `x-svrngs-person`
  on the other; the middleware's names those headers from the door's
  (person.rs:5-8 cites door.rs:103-111, which resolve at e9ed277). The
  tool counted none of the five's middleware halves (I19).

### Fixed here

- **#84.** apps/jellyfin/check/judge.py:73-79 and :137-142, at e9ed277:
  `mw_bars` and `bars` each read scan.py's word on a run the same way,
  which person's bytes held another's secret and which secrets were
  searched (`secrets_searched` with `lend_kept`), two spellings of one
  rule (principle 8). Now one function, `scanned` (judge.py:54), that both
  call (:85, :148). The two `why` closures the near tier paired judge
  opposite things (mw-0's bar fails a person asked to sign in, u5-4's one
  never asked) and stay apart. Behaviour kept, shown by
  /tmp/mwaudit1/replay.sh: judge.py over every recorded pair, the mw-2 run
  (8 pairs) and u5's two Jellyfin runs (9 and 12), 122 lines with every
  verdict both functions give, sha1 820df814… before and after. The
  replay watched failing first: `held` planted inverted in `scanned` made
  28 of its lines differ; a first plant, `lend_kept` dropped, changed
  none, since no recorded scan's `lend_kept` holds a secret
  `secrets_searched` does not, so it was not counted as the gate.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 82 | 8 | apps/jellyfin/mw/src/forward.rs:39-48, :119-137 and crates/svrngs-node/src/door.rs:142-151, :387-409, :415-423, at e9ed277 | mwp-3, `change(jellyfin): the middleware becomes the lend's sign-in` — forward.rs deleted; the door's list is the one | The middleware's and the door's hop-by-hop lists disagree (`trailer`, `proxy-authenticate`, and headers a `Connection` names), with no equivalence test between the two programs. |
| 83 | 8 | Body, Device, Origin, Viewer, Who: the sites in (3) | no change | Five names defined in both the middleware and svrngs, each two concepts in two programs. |
| 84 | 8 | apps/jellyfin/check/judge.py:73-79 and :137-142, at e9ed277 | this audit | Two judges read scan.py's word two ways; now `scanned`, output identical on every recorded pair. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 30 lines left in the repository (13970 / 14000). |
| 57, 68, 70, 71, 75, 77, 79, 81 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present on this binary: the plant `PersonKey` at lib.rs:127 is given as :126; `Viewer` limit.rs:86 as :85, `Who` door.rs:600 as :599. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)". |
| I9 | 6, 7 | `code dry-report` near tier | Still present: `menu`, a closure inside `playOn` (tv.mjs:115), is reported as `playOn`'s near clone, 0.959. |
| I10 | 6 | `code dry-report` header | Still present: "1383 function/method symbols" in every scope. |
| I12 | 5, 6 | `code dry-report --scope` | Still present: a scoped run compares within its scope, so the one cluster across the two workspaces (#82) is in no scoped report. |
| I14 | 6, 7 | `code dry-report` near tier | Still present: B, a renamed 28-line Rust body, is missed unscoped down to 0.8 and is seen at 0.7 only in a cluster of 846. |
| I17 | 6, 7 | `code dry-report` over Python | Better: D, a verbatim 35-line Python body under another name, is seen at 0.9 (0.940); still missed at the default 0.95. |
| I19 | 6 | `code converge noun`, and dry-report's exact tier | New: a Rust crate in its own Cargo workspace (apps/jellyfin/mw/Cargo.toml:6) has no SCIP symbols, so converge gives a noun defined only there "definitions (0) … check the spelling", N1 defined twice there (0), and N2 defined there and in svrngs-node (1), where each should say could-not-judge for that crate. The exact tier does say could-not-judge for the scope. |
| I20 | 6, 7 | `code dry-report` near tier over Rust | New: A, a verbatim 25-line Rust function with its doc, is missed at 0.95, 0.9 and 0.8, scoped and not, and is seen at 0.7 only in a cluster of 846; with the exact tier blind there (I19), a verbatim copy inside the middleware is invisible to the tool. The window pass sees it. |

## REVIEW-audit-mwp-auto-1 (2026-10-08, range 9bdcd1f..c487961)

The range starts at 9bdcd1fb5, the hash REVIEW-audit-mw-auto-1's row
carries (ralph/next/mw/STATE.md:17), the last `REVIEW-audit-` row; the mwp
queue has none before this one. It ends at c487961, the commit that made
this row: 11 commits, the last audit's `done` (3c5dbae), the design and
queue (b764b6a, a21b657), and mwp-0 to mwp-3. Product code changed in
crates/svrngs-node/, crates/svrngs/ and apps/jellyfin/ (`git diff --stat
9bdcd1f c487961 -- crates apps web bridge`: 29 files, +1654 −902); web/
and bridge/ are untouched. This audit fixes two findings (#85, #86) and
records the rest.

As before, the row's `docs/ARCH_PRINCIPLES.md` is
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli, which svrngs does not
build: sha256 5f5e2d74…, **the binary REVIEW-audit-mw-auto-1 planted its
tiers against** (its plants A, B, D, N1, N2, above), so those readings
stand for it and the plants were not run again; the 5-line window pass
stands in where they showed the tiers blind (I19, I20). The embedder was
`llama-server` on the corpus's model (qwen-embedding-0.6b.gguf,
~/.svrnmesh/config.toml:18) on :8080, run for this session; its first
start (`-np 4 -c 32768`) died with exit 133 mid-index, and the next
incremental index then reported complete with stale positions (I21), so
every clean run below is on a `code index --full` of the clean tree at
c487961 (2476 chunks in 328 s) and a `refresh` after it (SCIP idle,
graph 4 s old).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 9bdcd1f..HEAD`, summed
by /tmp/mwpaudit1/ledger.py (the last audit's, its unit pattern
`mwp-\d`), with its rules: a commit counts toward the unit its subject
names, else toward the next `done` commit's; product code is crates/,
apps/, web/, bridge/; a test is a `tests/` or `check/` segment or a
lean_check test file name; lockfiles left out.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| mwp-0 | 0 | 0 | 0 | 0 | 0 | 0 |
| mwp-1 | 538 | 34 | 504 | 481 | 11 | 470 |
| mwp-2 | 87 | 13 | 74 | 199 | 2 | 197 |
| mwp-3 | 296 | 837 | −541 | 68 | 20 | 48 |
| total | 921 | 884 | 37 | 748 | 33 | 715 |

- mwp-1 src (8572711): door/sign_in.rs +388, door.rs +85/−15, cli.rs
  +30/−12, share.rs +18/−5, lend.rs +16/−2, node.rs +1. Tests:
  tests/door/sign_in.rs +346, door/sign_in/tests.rs +81, lend/tests.rs
  +21, share tests +18/−4 and +10/−6, five one-line fixture additions.
- mwp-2 src (d0f1f34): door/sign_in.rs +59/−11, door.rs +28/−2. Tests:
  tests/door/sign_in.rs +120/−2, door/sign_in/tests.rs +79.
- mwp-3 src (a81178d), all apps/jellyfin/: forward.rs −374 and swap.rs
  −160 deleted; answer.rs +135 new; person.rs +30/−123, people.rs
  +45/−83, page.rs +32/−44, main.rs +23/−25, Cargo.toml +6/−7, README
  +25/−21. Tests: check.sh +51/−11, mw.py +13/−7, judge.py +3/−1,
  person.mjs +1/−1.

`python3 scripts/lean_check.py <tree>` (HEAD's script, rc 0) on `git
archive` trees in /tmp/mwpaudit1:

| at | svrngs-core | repository |
|---|---:|---:|
| 9bdcd1f, range start | 2957 / 3000 | 13970 / 14400 |
| c487961, range end | 2957 / 3000 | **14398 / 14400** |

The 428 lines: mwp-1 375 and mwp-2 51 (their commit bodies, 8572711 and
d0f1f34: 14,345 then 14,396), and 2 from
demo/runs/mwp/2026-10-08/run.sh (the only counted code file changed
after d0f1f34, `git diff --stat d0f1f34 HEAD`). DESIGN §3's bar 3
predicted about 320; fence 3's estimate was ~270, ~400 with traces and
refusals: past it by 28, inside the 430 the raise left, as mwp-1's body
said and why (the parse's refusals and the streaming body under rustfmt).
**2 lines are left**: any change below that adds a counted line is the
operator's raise first (#80).

### (2) Clones (`code dry-report`)

Scoped, one run each for the crate dirs the range touched, at 0.95:

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-node/ | 391 | 5 (lend.rs `as_str` ×3, `try_from`, `fmt`; api.rs `addr`; api.rs/iroh.rs `drop`) | 1 (declared.rs `harden_file` ×2) | 0 |
| crates/svrngs/ | 65 | 0 | 0 | 0 |
| apps/jellyfin/mw/ | 52 | **could-not-judge** ("no SCIP function symbols in scope", I19) | 0 | 0 |
| apps/ | 144 | 0 | 2 (tv.mjs `playOn`~`menu`, I9; judge.py `why`~`why`, #84's) | 0 |

Since a scoped run cannot see across scopes (I12), the whole corpus too,
at 0.95 (14 exact, 15 near), 0.9 (14, 36) and 0.8 (14, 214).
/tmp/mwpaudit1/blame.py (the last audit's) blamed every side, 885, for a
line in 9bdcd1f..c487961: **0** clusters with an added side at 0.95 and
0.9; at 0.8, **9**, each read:

- door.rs `lend_for` (:738–853) ~ `forward` (:953–1071) ~ `forward`
  (:1072–1197), 0.834–0.839. door.rs has one `forward` (:956–1197): the
  tool split it in two and paired the halves (I22). What is real is its
  size, 155 lines at 9bdcd1f and 242 at c487961, the sign-in deciding at
  five places in it — **#87**.
- person.rs `Who::of` (:54–72) ~ its test helper `key` and three tests
  (:136–179), 0.807–0.869: a parse and its tests; no change.
- lend.rs `declare` (:394–442) ~ its test (lend/tests.rs:102–141), 0.810;
  share.rs `lend_of` (:134–175) ~ its test (share/tests.rs:78–121), 0.866;
  sign_in.rs `first_in` (:304–319) ~ its test (sign_in/tests.rs:84–104),
  0.828: code and its test; no change.
- lend.rs `read` (:333–366) ~ `all` (:365–391), 0.863: one line of `read`
  is the range's (`sign_in: file.sign_in`), `all` calls `read`; no change.
- people.rs `start` (:42–67) ~ `signed` (:77–107), 0.823: the owner's loop
  and the step it runs; no change.
- sign_in/tests.rs `a_secret_split_anywhere_is_swapped` (:65–84) ~
  `a_streamed_page_gets_its_script_first_and_its_secrets_swapped`
  (:133–160), 0.820: two tests cutting at every byte, one with a page;
  no change.
- person.rs `Device::of` (:84–102) ~ `Device::one` (:100–113), 0.830: a
  person's device and the plant's one device; no change.
- sign_in.rs `said` (:357–366) ~ `compressed` (:363–373), 0.859: two
  header reads, one giving a string and one a yes or no; no change.

**Window pass** (/tmp/ceilaudit1/windows.py . 9bdcd1f..HEAD, 5
normalized lines, tests included): 167 hits, **2 new**:

- crates/svrngs-node/tests/door/sign_in.rs:120–125 == :148–153: the
  stand-in service wrote its head and two chunks by hand twice — **#85**,
  fixed.
- crates/svrngs/src/share/tests.rs:45–49 == crates/svrngs/tests/share.rs
  :160–164: two `Share` literals, each a test's own input; no change.

Reading the range for what neither pass would show:

- **#87.** crates/svrngs-node/src/door.rs:956–1197, `forward`: 155 lines
  at 9bdcd1f, 242 now. The sign-in is decided at :992 (asked), :1012 and
  :1018 (inward), :1103 (outward headers, a 401 forgotten), :1146 (the
  script) and :1163 (the body), each a `match`/`map` on `signed` beside
  the one-secret path. Principle 8's "collapse the drive" would make the
  sign-in a value with an inward and an outward step in sign_in.rs, and
  `forward` call each once. Not fixed: it moves ~90 lines and adds
  signatures, past the 2 lines left (#80), and it is no change in
  behaviour to land alone.
- **#88.** crates/svrngs-node/src/door/sign_in.rs:167–180: the one task
  that keeps every answer awaits `ask_of` (:176) inside its loop, so
  while one person's first ask is out (up to `ASK_WITHIN`, 30 s, :51),
  every request through every lend with a sign-in waits, a kept answer's
  too. DESIGN's contract asks "one person at a time, so a person's first
  requests at once ask once" (docs/DESIGN.md:466-467): the letter is
  kept; its spirit is one ask per person, which needs no wait across
  people or lends. Read from the code, not reproduced by a run. Not
  fixed: a change in behaviour, with lines past the budget; the trigger
  is a lend whose sign-in is slow or down while another person is in.
- **#89.** crates/svrngs-node/src/door/sign_in.rs:307–316 (`first_in`)
  and :323–333 (`tag`'s nonce) against web/lend.js:338–339 and :255:
  where a script goes first in a page, and the nonce it carries, decided
  twice, in Rust in the door and in JavaScript in the phone's worker.
  They agree on every case the door's unit test gives
  (sign_in/tests.rs:87–101, "lend.js's cases", DESIGN :572), but nothing
  runs both on one input (principle 8: a golden equivalence test). Not
  fixed: the two run in two places that cannot share a body; a golden
  test would need a JS runner in the crate's tests.
- **#90.** `svrngs-stand-in` is spelled at
  crates/svrngs-node/src/door/sign_in.rs:48 (`STAND_IN`) and
  apps/jellyfin/mw/src/page.rs:22 (`STAND_IN`, citing the door's), and
  read at apps/jellyfin/check/person.mjs:235. Its one home is the
  contract (docs/DESIGN.md:493-494), as the identity headers' is (#83);
  the middleware cannot link the door (DESIGN §3 call 6: no SDK). If the
  two spellings parted, the page's stored token would go unswapped and
  Jellyfin answer 401: the mw-2 pairs catch it, but they are not in
  scripts/check.sh (`grep -ci jellyfin scripts/check.sh`: 0). No change.
- **#57, #68, #70, #71, #75, #77, #79, #81, still open.** Not touched by
  the range's lines (#82 closed in a81178d, recorded above).

### (3) Duplicate type names (`code converge noun`)

`git diff 9bdcd1f c487961 -- crates apps web bridge`, added lines
matching `(struct|enum|trait|type|class) Name`: 13. `converge noun` on
each beside `git grep -nP '\b(struct|enum|trait|type|class) <Name>\b'
HEAD -- crates apps web bridge` (/tmp/mwpaudit1/converge.txt). `Data`
and `Error` are `hyper::body::Body`'s associated types in `Swapping`
(sign_in.rs:401-402), and `Refusal`'s other `type` lines are
`Command::Refusal` (svrngs-core/src/command.rs:39); none is a noun of
its own. Once each: `Answers` (a test's, tests/door/sign_in.rs:38;
converge 0), `At` (answer.rs:27; converge 0, I19), `Secret`, `SignIns`,
`Swapping`, `Whose` (sign_in.rs; converge 1 each). More than one:

| noun | converge | git grep | the two |
|---|---|---|---|
| **Refusal** | 2: cli.rs:333, sign_in.rs:125 | 2: crates/svrngs/src/cli.rs:334, crates/svrngs-node/src/door/sign_in.rs:126 | why the command stopped, in a person's words; why a lend's sign-in signed nobody in, told to the lender |
| **Answer** | 2: sign_in.rs:64, node.rs:538 | 2: sign_in.rs:65, crates/svrngs-node/src/node.rs:539 | a sign-in's answer, parsed; a struct inside `ask_lends`, a lender's list of lends |
| **Wire** | 2: project.rs:59, sign_in.rs:74 | 2: crates/svrngs-core/src/project.rs:60, sign_in.rs:75 | the ops a projection sends; a sign-in's answer as JSON before it is parsed |
| **Ask** | 1: sign_in.rs:146 | **2**: apps/jellyfin/mw/src/people.rs:28, sign_in.rs:147 | the middleware's ask of its owner of accounts; the door's of its keeper of answers |
| **Signed** | 1: sign_in.rs:138 | **2**: apps/jellyfin/mw/src/people.rs:36, sign_in.rs:139 | Jellyfin's user, device and token; the door's answer and every secret held for the lend |

- **#91, no change.** Each pair is two concepts. `Ask` and `Signed` are
  the two ends of one contract in two programs (as #83's five), and
  converge counts neither middleware half (I19).

### Fixed here

- **#85** (c3327a3). crates/svrngs-node/tests/door/sign_in.rs:120–125
  and :148–153, at c487961: the stand-in service wrote a head, two chunks
  100 ms apart and the terminator by hand, twice. Now `chunked`
  (:88; c3327a3's body says :85, where its doc's line before it is) for
  both. 3 passed before and after; the fixture still splits where the
  tests need, each watched failing on a plant then reverted (no holding
  back in `swapped`: "a person received a secret"; `first_in` deciding
  before `<head` can come: "the person's script first in <head>"). No
  counted line.
- **#86** (31eee93). docs/DESIGN.md:441–600, the sign-in's contract,
  cited door.rs and the spike's swap.rs and forward.rs by path:line as at
  252bf22; mwp-1 moved door.rs's lines and mwp-3 deleted the two files,
  so at HEAD they resolved to nothing or to other lines (principle 4).
  Each resolves at 252bf22; the contract now says so, one clause.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 85 | 8 | crates/svrngs-node/tests/door/sign_in.rs:120-125 and :148-153, at c487961 | c3327a3 | The stand-in service's chunked writer, twice; now `chunked`. |
| 86 | 4 | docs/DESIGN.md:441-600 against crates/svrngs-node/src/door.rs and apps/jellyfin/mw/src/{swap,forward}.rs, at c487961 | 31eee93 | The contract's path:lines resolved only at 252bf22; it now says so. |
| 87 | 8 | crates/svrngs-node/src/door.rs:992, :1012, :1018, :1103, :1146, :1163 | not fixed (budget, #80) | `forward` grew 155 → 242 lines, the sign-in decided at five places beside the one secret's. |
| 88 | 8 (spirit of DESIGN :466) | crates/svrngs-node/src/door/sign_in.rs:167-180, :176 | not fixed (behaviour, budget) | One person's first ask, up to 30 s, holds every request through every sign-in lend, kept answers too. |
| 89 | 8 | crates/svrngs-node/src/door/sign_in.rs:307-316, :323-333 and web/lend.js:338-339, :255 | not fixed | Where a script goes first and its nonce, decided in Rust and in JS, held by mirrored cases, not a golden test. |
| 90 | 8 | crates/svrngs-node/src/door/sign_in.rs:48 and apps/jellyfin/mw/src/page.rs:22 | no change | `svrngs-stand-in` spelled in both programs; DESIGN :493 its home; checked only by the Jellyfin run, not check.sh. |
| 91 | 8 | Refusal, Answer, Wire, Ask, Signed: the sites in (3) | no change | Five names defined twice, each two concepts. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 2 lines left in the repository (14398 / 14400). |
| 57, 68, 70, 71, 75, 77, 79, 81 | as recorded | as recorded | not fixed | Untouched in the range. |

The row's check (LINT): `scripts/check.sh` at 8e773b0 on a clean tree,
all 14 steps passed and the seeks' rate printed, exit 0,
2026-10-08T19:05:40Z (/tmp/mwpaudit1/check.out).

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: every sign_in.rs noun one line early (`Ask` :146 for :147, `Refusal` :125 for :126, `Wire` :74 for :75), and cli.rs `Refusal` :333 for :334. |
| I9 | 6, 7 | `code dry-report` near tier | Still present: tv.mjs `menu` inside `playOn`, 0.952. |
| I12 | 5, 6 | `code dry-report --scope` | Still present: 0 added-side clusters scoped; the 9 at 0.8 are seen only unscoped. |
| I19 | 6 | `code converge noun`, dry-report's exact tier | Still present: apps/jellyfin/mw/ exact tier could-not-judge; `At` gives definitions (0), `Ask` and `Signed` (1) where git grep finds 1, 2, 2. |
| I21 | 6, 7 | `code index` (incremental) after an embedder failure | New: an incremental run whose embedder died (exit 133) said "22 file(s) failed to re-index"; the next incremental run said "Incremental refresh complete … 22 file(s) changed — 280 chunk(s) embedded", yet dry-report then gave door.rs's symbols at e9ed277's lines (`router` :163–174, where it is :172–183 after `--full`; person.rs's `key` :212–220, a function of a file 179 lines long). A completed incremental index is not proof of a current one. Cause not isolated. |
| I22 | 6, 7 | `code dry-report` near tier | New: one long function is two symbols of one name: door.rs `forward` (:956–1197) given as `forward` :953–1071 and `forward` :1072–1197, and those paired as near clones of each other and of `lend_for`, 0.834–0.839. |

## REVIEW-audit-mwp-auto-2 (2026-10-08, range 673db84..edb4072)

The range starts at 673db84cb, the hash REVIEW-audit-mwp-auto-1's row
carries (ralph/next/mwp/STATE.md:18), the last `REVIEW-audit-` row. It ends
at edb4072, the commit that made this row: 7 commits, the last audit's
`done` (f573fa9), the queue's correction of mwp-4's check (30dac6a), and
mwp-4 (a57852d, a9ac3e1, 28c430a, e46c7ba). Product code changed only in
apps/web-conformance/ and crates/svrngs-node/tests/conformance.rs: `git
diff --stat 673db84 edb4072 -- 'crates/*/src' web/ bridge/` is empty. This
audit fixes three findings (#92, #93, #94) and records the rest.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 5f5e2d74…, **the
binary REVIEW-audit-mw-auto-1 planted its tiers against**, so its plants'
readings stand and were not run again. The embedder was `llama-server` on
the corpus's model (qwen-embedding-0.6b.gguf, ~/.svrnmesh/config.toml:18)
on :8080, for this session. Its first start (`-np 1 -c 8192`, so n_batch =
n_ubatch = 512, its log says) exited 133 during `code index . --full`, as
the last audit's did, and the failed `--full` left the corpus without its
chunks (I23); started again with `-b 8192 -ub 8192`, the `--full` at
edb4072 finished: 2482 chunks in 154 s, then `refresh` (SCIP graph 8 s
old by `project watch status`).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 673db84..HEAD`, summed
by /tmp/mwpaudit2/ledger.py (the last audit's, unchanged), its rules as
before: a commit counts toward the unit its subject names, else the next
`done`'s; product code is crates/, apps/, web/, bridge/; a test is a
`tests/` or `check/` segment or a lean_check test file name.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| mwp-4 | 97 | 4 | 93 | 130 | 10 | 120 |
| total | 97 | 4 | 93 | 130 | 10 | 120 |

- mwp-4 src (a9ac3e1), all apps/web-conformance/: answer.mjs +48 new,
  serve.mjs +35/−4, checks.js +14. By lines of code (no blank, no
  comment): answer.mjs 30, serve.mjs +22/−2, checks.js +10.
- mwp-4 tests: crates/svrngs-node/tests/conformance.rs +10/−6 (a57852d)
  and +120/−4 (a9ac3e1).

The ledger counts apps/ as product; lean_check does not count apps/
(scripts/lean_check.py:49, `rel.startswith(("apps/", "target/"))`). On
`git archive` trees in /tmp/mwpaudit2:

| at | svrngs-core | repository |
|---|---:|---:|
| 673db84, range start | 2957 / 3000 | 14398 / 14400 |
| edb4072, range end | 2957 / 3000 | **14398 / 14400** |

mwp-4 added no counted line, as its bar asked (DESIGN §3 call 6, bar 4).
**2 lines are left** (#80).

### (2) Clones (`code dry-report`)

Scoped, at 0.95, for the dirs the range touched:

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-node/ | 391 | 5 (as at the last audit) | 1 (declared.rs `harden_file` ×2) | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| apps/web-conformance/ | 11 | **could-not-judge** ("no SCIP function symbols in scope", I19's for JS) | 0 | 0 |
| apps/ | 145 | 0 | 2 (tv.mjs `playOn`~`menu`, I9; judge.py `why`~`why`) | 0 |

The tool leaves out every `tests` path segment by design ("First-party
production code only"), so the range's one Rust file is outside it; it
says could-not-judge for that scope, rightly. Unscoped: 0.95 (14 exact, 15
near), 0.9 (14, 36), 0.8 (14, 214), the counts the last audit had.
/tmp/mwpaudit2/blame.py blamed every side, 885, for a line in
673db84..HEAD: **0** clusters with an added side at any threshold.

**Window pass** (/tmp/mwpaudit2/windows.py . 673db84..HEAD, 5 normalized
lines, tests included): 162 hits, **4 new**, all one:
crates/svrngs-node/tests/conformance.rs:677–681 (the test asking the
canary for each person's token) == lend_origin.rs:177–181,
record_api.rs:232–236, room.rs:196–200 and :326–330: reqwest's
`.send().await.unwrap().json().await.unwrap()`, each a different request;
no change.

Reading the range for what neither pass would show:

- **#92, fixed (29bfaf9).** apps/web-conformance/serve.mjs:141 and :216,
  at edb4072: the record `/seen` keeps of a request, a page's and a
  WebSocket's, built twice on one line each, and a9ac3e1 added `signed`
  to both alike. One line each is below the window pass's 5. Now
  `saw(req, path)` (:60).
- **#93, fixed (826a713).** crates/svrngs-node/tests/conformance.rs, in
  `conformance` and in a9ac3e1's `two_people_signed_in`: the verdict line
  "{:<9} {:<8} {who:<4} {route:<34} {verdict}" formatted twice, its
  columns what scripts/check.sh:46's `cut -c1-10,25-` reads. Now
  `verdict_line` (:240), its doc naming that reader.
- **#94, fixed (9d0be57).** docs/DESIGN.md:633–634, bar 4's landing:
  "its accounts 32 more in `serve.mjs`" — 22 there and the page's check
  10 in checks.js (the ledger above). The total stood; where now said.
- **#90, updated.** `svrngs-stand-in` is spelled a third time, at
  apps/web-conformance/answer.mjs:18, beside
  crates/svrngs-node/src/door/sign_in.rs:48 and
  apps/jellyfin/mw/src/page.rs:22. The canary's spelling is held by
  scripts/check.sh through `two_people_signed_in`: planted as
  `svrngs-stand-ix`, `two_people_signed_in::chromium` failed, "not signed
  in as themselves … stored svrngs-stand-ix; by query ," (reverted).
  Jellyfin's is still held only by the mw-2 pairs. No change.
- The test writes its lend's file by hand (conformance.rs:672–674, `port`,
  `kind`, `limit`, `sign_in`), as conformance.rs:180–183, room.rs:79,
  lend_origin.rs:56 and surface.rs:51 do; a renamed field breaks them, as
  a reader of the format should. No change.
- **#57, #68, #70, #71, #75, #77, #79, #81, #87, #88, #89, #91, still
  open.** The range changed no line of crates/'s sources or of web/.

### (3) Duplicate type names (`code converge noun`)

`git diff 673db84 edb4072 -- crates apps web bridge`, added lines matching
`\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: **none** (grep exit 1). The
one type the range touched, `Canary` (conformance.rs:86, its doc
changed): converge "definitions (0) … defined only in test code"; `git
grep` one definition. No noun with more than one definition.

### Fixed here

- **#92** (29bfaf9) and **#93** (826a713). `cargo test -p svrngs-node
  --test conformance`, before (clean tree at edb4072) and after: 6 passed
  each (171 s, 181 s). `cmp` of target/conformance/*.txt: five files
  byte-identical; wildcard-webkit.txt differs only in "seeks back to
  back", a rate that never fails (u5-11), sam's and bo's lines trading
  passed and failed. Plant for `saw`: `signed: ''`;
  `two_people_signed_in::chromium` failed, "requests signed in as
  another"; reverted.
- **#94** (9d0be57). The line counts are those in (1).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 92 | 8 | apps/web-conformance/serve.mjs:141 and :216, at edb4072 | 29bfaf9 | The `/seen` record of a request built twice; now `saw`. |
| 93 | 8 | crates/svrngs-node/tests/conformance.rs `conformance` and `two_people_signed_in`, at edb4072 | 826a713 | The verdict line check.sh's `cut` reads, formatted twice; now `verdict_line`. |
| 94 | 4 | docs/DESIGN.md:633-634 | 9d0be57 | Bar 4's accounts said to be 32 lines of serve.mjs; 22 there, 10 in checks.js. |
| 90 | 8 | apps/web-conformance/answer.mjs:18, crates/svrngs-node/src/door/sign_in.rs:48, apps/jellyfin/mw/src/page.rs:22 | no change | `svrngs-stand-in` now spelled three times; the canary's held by check.sh (planted), Jellyfin's by the mw-2 runs alone. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 2 lines left (14398 / 14400). |
| 57, 68, 70, 71, 75, 77, 79, 81, 87, 88, 89, 91 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I9 | 6, 7 | `code dry-report` near tier | Still present: tv.mjs `menu` inside `playOn`, 0.952. |
| I19 | 6 | dry-report's exact tier | Still present for JS: apps/web-conformance/ exact tier could-not-judge, as web/'s was. |
| I21 | 6, 7 | `code index` and its embedder | Narrowed, not isolated: `llama-server` with n_batch = n_ubatch = 512 exited 133 during `--full` again (/tmp/mwpaudit2/llama.log ends mid-task, no assertion line); with `-b 8192 -ub 8192` the same `--full` finished. One run each: a chunk past 512 tokens is the likely cause, unproven. |
| I23 | 6 | `code index --full` | New: a `--full` whose embedder died ("✗ Indexing failed", rc 1) left ~/.svrnmesh/indexes/svrngs/ with only code_index_state.json and scip_graph.db: the previous index is deleted before the new one is whole, so a failed rebuild leaves nothing to judge with, not the last good index. |

## si-1-one-sign-in (2026-10-08): #87, #88, #90 closed

- **#87, closed (ae898f0).** `forward` decides the sign-in once — `lend.sign_in`
  to a `sign_in::Signing`, the lend's one secret or the person signed in at
  its port — and calls `Signing::inward` once and `Signing::outward` once
  (door/sign_in.rs). The grep, over `forward` at HEAD
  (`awk '/^async fn forward\(/{s=1} s{print NR": "$0} s && /^}/{exit}'
  crates/svrngs-node/src/door.rs | grep -nE "sign_in|signing|signed|STAND_IN"`):
  door.rs:993 (decided), :1012 (`inward`), :1106 (`outward`), :1120 (the
  body read as `outward` said). `forward` 242 lines to 171. Behaviour kept
  in its own commit: `cargo test -p svrngs-node --lib door` 18 passed and
  `--test door` 20 passed, no test touched.
- **#88, closed (effb427).** The keeper sends each ask out on its own task
  and a person's further first requests wait on the ask already out. Red
  before the fix (tests/door/sign_in.rs
  `one_person_s_slow_sign_in_holds_nobody_else_s`, bo's ask held 3 s):
  cy's first requests through the same lend and another, "200 after
  2.910681667s" and "200 after 2.910584667s"; green after, each under 1 s,
  bo's two first requests at once asked once. Plant (no join on an ask
  out) caught by two tests.
- **#90, closed (6028168).** The door's ask names the stand-in,
  `x-svrngs-stand-in` (sign_in.rs `STAND_IN_HEADER`), and both answerers
  store what it names; DESIGN §3 call 6 says so. `git grep -n
  "svrngs-stand-in" -- ':!*.md' ':!demo' | grep -v x-svrngs-stand-in`:
  sign_in.rs:50 alone outside tests (tests/door/sign_in.rs:21, page.rs's
  `#[cfg(test)]` :92, :96) and the Jellyfin checks that judge what a page
  stored (apps/jellyfin/check/person.mjs:235; mw.py's probe asks as the
  door does). The canary holds the answerer's read (planted
  `x-svrngs-stand-ix`: both `two_people_signed_in` failed), and the
  Jellyfin run holds the middleware's (demo/runs/si/2026-10-08/).
- **#95, new, not fixed.** apps/jellyfin/check/person.mjs `watch` counts
  a new `<video>` in the watched person's page as not left alone, and
  records the count, not the element. In si-1's run both WebKit pairs
  counted one while the person's film never stopped (back 0 s, still 0
  ms); the WebKit computer pair alone counted one at 37b4a4c, before
  si-1, 2 of 2, and at 6028168 1 of 2; no earlier recorded WebKit pair
  did (demo/runs/si/2026-10-08/README.md). Whether it is a second player
  or a false count, the instrument cannot say (principle 7).

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 87 | 8 | crates/svrngs-node/src/door.rs `forward` | ae898f0 | The sign-in decided at one site, a value with an inward and an outward step; `forward` 242 → 171 lines. |
| 88 | 8 (spirit of DESIGN call 6) | crates/svrngs-node/src/door/sign_in.rs `SignIns::spawn` | effb427 | One person's slow sign-in holds nobody else's request; a person's first requests at once still ask once. |
| 90 | 8 | crates/svrngs-node/src/door/sign_in.rs:50 | 6028168 | `svrngs-stand-in` spelled once outside tests, named to each answerer in the door's ask. |
| 95 | 7 | apps/jellyfin/check/person.mjs `watch` | not fixed | A new `<video>` counted, never described; counted in WebKit today before and after si-1. |

## REVIEW-audit-compose-auto-1 (2026-10-08, range 9a013f4..731c469)

The range starts at 9a013f471, the hash REVIEW-audit-mwp-auto-2's row
carries (ralph/next/mwp/STATE.md:20), the last `REVIEW-audit-` row; this
queue has none before this one. It ends at 731c469, the commit that made
this row: 26 commits, the last audit's `done` (78b3520), the operator's
design and budget (d697b18, 508356d), the queue (37b4a4c, ba5760e), si-1
(ae898f0, effb427, 6028168, 7ac7cf0, dd2a501, 1d9d5f3), mn-0 (c963f93),
mn-1 (47dab7e, a40fe81, fa5f076, c966cbc, 7a1e94b, 86b092d, 83e2335) and
mn-2 (931b9ed, f9a4c9b), with their `done`s. Product code outside apps/
changed only in si-1: `git diff --stat fab9b8c 731c469 -- crates web
bridge scripts/check.sh` is empty, as the spikes' fence asks. This audit
fixes one finding (#96) and records the rest.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli. **It is not the binary
the last three audits used**: sha256 35d97ed2… (built 15:05), not
5f5e2d74…. No commonwealth-ai commit from 11:00 to its build names a path
matching `dry|converge|code_index|scip|embed` (`git log --since
"2026-10-08 11:00" --until "2026-10-08 15:06" --name-only`), and every
reading below on code the range did not touch is the last audit's; its
tiers were planted again all the same (below). The embedder was
`llama-server` on the corpus's model (qwen-embedding-0.6b.gguf) on :8080,
`-b 8192 -ub 8192` as I21 found needed, for this session. `code index .
--full` at 731c469, tree clean: 2550 chunks in 486 s; the SCIP graph
written after the range's last product commit (scip_graph.db 17:54,
c966cbc 15:47).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 9a013f4..731c469`,
summed by /tmp/composeaudit1/ledger.py (the last audit's, its unit
pattern `(si|mn|su|mnp)-\d`), its rules as before: a commit counts toward
the unit its subject names, else the next `done`'s; product code is
crates/, apps/, web/, bridge/; a test is a `tests/` or `check/` segment
or a lean_check test file name. #87 is in `si-1`'s row (ae898f0 names
no unit), #88 and #90 apart, by their subjects.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| si-1 (#87, the move) | 121 | 80 | 41 | 5 | 3 | 2 |
| si-1 (#88) | 51 | 29 | 22 | 107 | 29 | 78 |
| si-1 (#90) | 43 | 32 | 11 | 5 | 0 | 5 |
| mn-0 | 0 | 0 | 0 | 0 | 0 | 0 |
| mn-1 | 300 | 0 | 300 | 1127 | 1 | 1126 |
| mn-2 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 515 | 141 | 374 | 1244 | 33 | 1211 |

- si-1 src: door.rs +10/−80 and door/sign_in.rs +111 (ae898f0);
  sign_in.rs +51/−29 (effb427); +11/−4 there and apps/jellyfin/mw/ +26/−23,
  apps/web-conformance/answer.mjs +6/−5 (6028168). `forward` 242 lines
  (9a013f4, door.rs:956–1197) to 171 (731c469, :957–1127).
- si-1 tests: tests/door/sign_in.rs +107/−29 (effb427), +5 (6028168);
  apps/jellyfin/check/mw.py +5/−3 (7ac7cf0).
- mn-1 src, all apps/movie-night/ (47dab7e): app.js 180, match.js 76,
  index.html 20, style.css 13, plant.js 6, nouns.txt 5.
- mn-1 tests, apps/movie-night/check/ (a40fe81, c966cbc): mn.mjs 390,
  check.sh 314, judge.py 292, doors.py 99, tokens.py 31. The run's data
  and run.sh are under demo/, not product.

lean_check on `git archive` trees in /tmp/composeaudit1 (apps/ is not
counted, scripts/lean_check.py:49):

| at | svrngs-core | repository |
|---|---:|---:|
| 9a013f4, range start | 2957 / 3000 | 14398 / 14400 |
| 731c469, range end | 2957 / 3000 | **14466 / 14750** |

si-1 spent 68 counted lines; the spikes none. **284 lines are left**
(#80).

### (2) Clones (`code dry-report`), each tier checked against a plant

**Plants**, uncommitted in the working tree, the corpus indexed `--full`
over them (2552 chunks, 425 s; `refresh`: SCIP +14 symbols) then again
without them (2550, 363 s), so the corpus stands at 731c469:

- **E** (exact): sign_in.rs `swap_headers` (:428–437, 10 lines; si-1's
  file), verbatim, as `plant_exact` at the file's end.
- **N** (near): the same body as `plant_near`, its names changed
  (`headers`→`map`, `from`→`old`, `to`→`new`, `n`→`count`,
  `value`→`v`, `out`→`swapped_bytes`).

| scope, threshold | E | N |
|---|---|---|
| crates/svrngs-node/src/door/sign_in.rs, 0.95 | **missed**: 0 exact groups (21 locations, the two plants among them: 1454 → 1456 symbols) | **missed** |
| crates/svrngs-node/, 0.95 | **missed**: the same 5 exact groups as clean | missed |
| sign_in.rs, 0.9 and 0.8 | E ~ N only, 0.937; never with `swap_headers` | seen with E alone |
| sign_in.rs, 0.7 | `swap_headers` :425–441 ~ E :576–592 ~ N :589–602, 0.727–0.937 | seen |
| window pass (below), no range | **seen**: sign_in.rs:429–437 == :580–588 | missed (renamed; it cannot see one, as before) |

So for Rust, the dry-report tiers' zeros below are weak evidence: the
exact tier missed a verbatim body under a new name (each of the 5 groups
it finds pairs symbols of one name: `as_str`, `fmt`, `addr`, `drop`,
`try_from`), and the near tier's spans take their neighbours' lines
(`swap_headers` :425–441 for :428–437; E's span runs into N's, I9), so a
verbatim copy scored 0.727. A verbatim copy is the window pass's to find,
and it did (I24).

**Clean runs.** Scoped, at 0.95, for each dir the range touched:

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-node/ | 394 | 5 (as at the last audit) | 1 (declared.rs `harden_file` ×2) | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| apps/jellyfin/mw/ | 52 | **could-not-judge** (I19) | 0 | 0 |
| apps/web-conformance/ | 11 | **could-not-judge** (I19) | 0 | 0 |
| apps/movie-night/ | 49 | 0 | 0 | 0 |
| apps/jellyfin/check/ | 57 | 0 | 2 (`playOn`~`menu`, I9; judge.py `why`~`why`) | 0 |
| apps/ | 194 | 0 | 2 (the same) | 0 |

Unscoped: 0.95 (14 exact, 15 near), 0.9 (14, 38), 0.8 (14, 222).
/tmp/composeaudit1/clusters.py blamed every side (919) for a line in
9a013f4..731c469 (`git blame -w -M -C -C`). **0** at 0.95; at 0.9, **2**,
both in Movie Night's check; at 0.8, 10 (the 2 among them):

| at | cluster | cosine | read |
|---|---|---|---|
| 0.9, 0.8 | judge.py `bar4_before` :132–152 ~ `bar4_after` :149–172 (a40fe81) | 0.911 | Two bars, before and after the list; different reads of the run. No change. |
| 0.9, 0.8 | doors.py `when` :26–33 ~ `fields` :30–37 (a40fe81) | 0.903 | Two one-line functions in overlapping 8-line spans (I9). No change. |
| 0.8 | door.rs `lend_for` :739–854 ~ `forward` :954–1051 ~ `forward` :1052–1127 (ae898f0) | 0.828–0.837 | I22 still: one function given as two symbols. No change. |
| 0.8 | mn.mjs `choose` :185–220, `play` :218–248, `back` :246–275 (a40fe81) ~ person.mjs `judge`, `seek`, trio.mjs `over`, a u5 drive.mjs | 0.801–0.998 | The tool pairs the wrong functions; `play` is a fork of person.mjs's play inside `enter`, which it cannot see (#97). |
| 0.8 | mn.mjs `join`, `listed`, `group`, `openFromList` :62–109 | 0.848–0.881 | Neighbours in one driver; `join` is a fork of person.mjs:185–192 (#97). |
| 0.8 | mn.mjs `computer`, `limit`, `once` :329–378; `phone` ~ `asleep` :276–333 | 0.835–0.845 | One driver's journeys; see the window pass. No change. |
| 0.8 | match.js `copyOf`, `same`, `films` :38–75 (47dab7e) | 0.814–0.850 | Three different functions, overlapping spans (I9). No change. |
| 0.8 | app.js `card` :138–160 ~ `render` :156–174 (47dab7e) | 0.814 | A card and the page that lists cards. No change. |
| 0.8 | people.rs `signed` :65–80 ~ sign_in.rs `forget` :351–360, `signed` :358–368 (effb427) | 0.809–0.822 | An app's actor handle and the door's; different messages, two programs. No change. |

**Window pass** (/tmp/composeaudit1/windows.py . 9a013f4..731c469, 5
normalized lines, tests included): 167 hits, **2 new**:

- apps/movie-night/check/mn.mjs:280–285 == :368–373 (a40fe81): `phone`
  and `once` begin alike (browser, join, open from the list); `once` is
  `phone` cut at its first settle, for the limit's count. No change.
- crates/svrngs-node/tests/door/sign_in.rs:56–61 == :121–126 (effb427):
  two fixture servers' accept loops, the answerer's and the service's.
  No change.

Reading the range for what neither pass would show:

- **#97, new, not fixed.** apps/movie-night/check/mn.mjs re-drives what
  apps/jellyfin/check/person.mjs `enter` (:146) does inline, though it
  already imports `record`, `shown` and `timed` from it (mn.mjs:38):
  the press of Play and its second tap (mn.mjs `play` :221, "(person.mjs's)";
  person.mjs:245–266), the group page's join (mn.mjs `join` :65;
  person.mjs:185–192) and the stand-in read from what the page stored
  (mn.mjs:209–211; person.mjs:233–235). The held rule (paused, ready, under
  0.5 s, for 3 s; 30 s in all) is a threshold in two homes. Not fixed here:
  `enter` exports none of these steps, and a change to it is proven only
  by the Jellyfin runs (demo/runs/mwp/2026-10-08/run.sh,
  apps/movie-night/check/check.sh), which this audit did not run.
- **#90's spellings.** `svrngs-stand-in` outside tests is still
  sign_in.rs:50 alone; mn.mjs:211 is a fourth check that judges what a
  page stored by it, as person.mjs:235 does. A renamed stand-in fails
  both loudly. No change.
- **#57, #68, #70, #71, #75, #77, #79, #81, #89, #95, still open.** #89's
  sites in sign_in.rs moved with the file; its finding stands.

### (3) Duplicate type names (`code converge noun`)

`git diff 9a013f4 731c469 -- crates apps web bridge`, added lines
matching `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: three, all in
crates/svrngs-node/src/door/sign_in.rs.

| noun | converge | git grep (`(struct\|enum\|trait\|type\|class)[[:space:]]+Name\b`) |
|---|---|---|
| Signing | definitions (1), sign_in.rs:153 | 1 |
| Read | definitions (1), sign_in.rs:162; kin `Reads` | 1 |
| **Told** | **definitions (2)**: door.rs:855, door/sign_in.rs:264 | **2**: door.rs:856 `struct Told` (a request's share and held use; at 9a013f4:855), sign_in.rs:265 `type Told` (effb427, the sender a request waits on) |

(converge's lines one early, I1.) **#96**: one name, two meanings, in a
module and its child, where `super::Told` is in scope; unlike #91's pairs,
which sit in two programs. Fixed (8024f76): sign_in.rs's is `Waiter`.

### Fixed here

- **#96** (8024f76). `git grep -nw Waiter -- crates apps web bridge`
  outside sign_in.rs: none. `cargo test -p svrngs-node --test door`: 21
  passed, the four `sign_in::` tests among them.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 96 | 8, 3 | crates/svrngs-node/src/door/sign_in.rs:265 and door.rs:856, at 731c469 | 8024f76 | Two `Told`s, two concepts, a module and its child; the child's is `Waiter`. |
| 97 | 8 | apps/movie-night/check/mn.mjs:65, :209–211, :221; apps/jellyfin/check/person.mjs:185–192, :233–235, :245–266 | not fixed | Movie Night's driver forks the Jellyfin driver's join, stand-in read and Play with its held rule; `enter` exports none of them. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 284 lines left (14466 / 14750). |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95 | as recorded | as recorded | not fixed | Untouched in the range, or moved with their file. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: `Told` door.rs:855 for :856, sign_in.rs:264 for :265. |
| I9 | 6, 7 | `code dry-report` near tier | Still present, and now measured on a plant: a verbatim 10-line copy scored 0.727 against its original, the original's span (:425–441) three lines either side of the function (:428–437). |
| I19 | 6 | dry-report's exact tier | Still present: apps/jellyfin/mw/ (Rust) and apps/web-conformance/ could-not-judge. |
| I22 | 6, 7 | `code dry-report` near tier | Still present: door.rs `forward` (:957–1127) as `forward` :954–1051 and :1052–1127. |
| I24 | 6, 7 | `code dry-report` exact tier | New, on binary 35d97ed2…: a verbatim body under another name (plant E, `swap_headers` as `plant_exact`, svrngs-node, SCIP symbols present) is no exact group; every group it finds pairs one name. Its "0 exact" in Rust is not evidence of no copy; the window pass caught E. Cause not isolated (the hash may take the name; unproven, one plant). |

The row's check: `scripts/check.sh` at 7985cfc, all 14 steps passed
(principles pinned through app laws; the WebKit seek rate "failed, a
rate" for wildcard, which never fails the step, u5-11), exit 0.

## REVIEW-audit-compose-auto-2 (2026-10-09, range 34d5272..0b5cac1)

The range starts at 34d52729a, the hash REVIEW-audit-compose-auto-1's row
carries (ralph/next/compose/STATE.md:19), the last `REVIEW-audit-` row. It
ends at 0b5cac1, the commit that made this row: 17 commits (`git rev-list
--count 34d5272..0b5cac1`), the last audit's `done` (766be45), su-0
(cf56039), su-1 (55dca40, 0095684, 305efde, c0c64ff, 3847360), su-2
(5153bdc, 62582ee) and mnp-1 (1f08560, e02a203, 11dbb9b), with their
`done`s (`git log --format='%h %s' 34d5272..0b5cac1`). Product code outside apps/ changed in su-1's one
header (55dca40, door.rs +29/−4, the fence's exception) and in mnp-1.
This audit fixes one finding (#98) and records two (#99, #100).

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli: **the binary the last
audit planted against**, sha256 35d97ed2… (built 2026-10-08 15:05), so its
tiers' plants (I24, I9) stand and were not run again. The embedder was
`llama-server` on the corpus's model (qwen-embedding-0.6b.gguf,
~/.svrnmesh/config.toml:18) on :8080, `-b 8192 -ub 8192` (I21), for this
session. `code index . --full` at 0b5cac1, tree clean: 2630 chunks in
152 s; the SCIP graph (scip_graph.db 00:44) written after the range's last
product commit (e02a203, 00:13).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 34d5272..0b5cac1`,
summed by /tmp/composeaudit2/ledger.py (the last audit's, unchanged), its
rules as before: a commit counts toward the unit its subject names, else
the next `done`'s; product code is crates/, apps/, web/, bridge/; a test is
a `tests/` or `check/` segment or a lean_check test file name.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| su-0 | 0 | 0 | 0 | 0 | 0 | 0 |
| su-1 | 435 | 9 | 426 | 781 | 6 | 775 |
| su-2 | 0 | 0 | 0 | 0 | 0 | 0 |
| mnp-1 | 175 | 55 | 120 | 254 | 11 | 243 |
| total | 610 | 64 | 546 | 1035 | 17 | 1018 |

- su-1 src: door.rs +29/−4 (55dca40, `x-svrngs-app`); the rest apps/
  (0095684, 305efde): apps/taste/taste.mjs +169/−2, nouns.txt +4;
  apps/movie-night/picks/ picks.js +196/−3, index.html 26, picks.css 8,
  match.js, plant.js, style.css 1 each.
- su-1 tests: tests/door/main.rs +67 (55dca40); apps/taste/check/ su.mjs
  261, judge.py +220/−5, check.sh 186, other/ 46; apps/movie-night/check/
  check.sh +1/−1.
- mnp-1 src: door.rs +89/−25, sign_in.rs +1/−1, origin.rs +6/−7, room.rs
  +1/−2, web/lend.js +11/−10 (1f08560); door.rs +23/−4, node.rs +5/−3,
  ring_sync.rs +39/−3 (e02a203).
- mnp-1 tests: tests/door/main.rs +43, origins.rs +27, newcomer.rs +125,
  tests/browser/main.rs +33/−5, web/tests/ +26/−6.

lean_check on `git archive` trees in /tmp/composeaudit2 (apps/ is not
counted, scripts/lean_check.py:49):

| at | svrngs-core | repository |
|---|---:|---:|
| 34d5272, range start | 2957 / 3000 | 14466 / 14750 |
| 0b5cac1, range end | 2957 / 3000 | **14564 / 14750** |

su-1's header and mnp-1 spent 98 counted lines; **186 are left** (#80).

### (2) Clones (`code dry-report`)

**Scoped**, at 0.95, for each dir the range touched:

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-node/ | 399 | 5 (as at the last audit: `as_str` ×3, `drop`, `fmt`, `try_from`, `addr`) | 1 (declared.rs `harden_file` ×2) | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| apps/movie-night/ | 60 | 0 | 0 | 0 |
| apps/taste/ | 35 | 0 | 0 | 0 |
| web/ | 85 | **could-not-judge** (I19) | 0 | 0 |
| apps/ | 240 | 0 | 5 | **3**, all su.mjs (below) |

**Unscoped**: 0.95 (14 exact, 18 near), 0.9 (14, 43), 0.8 (14, 235).
/tmp/composeaudit2/clusters.py blamed every side (`git blame -w -M -C -C`)
for a line in 34d5272..0b5cac1: **3** clusters at 0.95, **5** at 0.9, 17 at
0.8.

| at | cluster | cosine | read |
|---|---|---|---|
| 0.95, 0.9, apps/ | apps/taste/check/su.mjs `browser` :40–59 ~ apps/movie-night/check/mn.mjs `browser` :44–64 (0095684) | 0.974 | **#99.** Forked, by su.mjs's own heading: "the group's page (mn.mjs's)" (:58). |
| 0.95, 0.9, apps/ | su.mjs `join` :57–75 ~ mn.mjs `join` :62–80 | 0.994 | **#99**; mn.mjs's is itself #97's fork of person.mjs:185–192. |
| 0.95, 0.9, apps/ | su.mjs `listed` :71–82 ~ mn.mjs `listed` :76–88 | 0.956 | **#99.** |
| 0.9 | su.mjs `openFromList` :86–98 ~ mn.mjs :94–109; su.mjs `group` :78–90 ~ mn.mjs :85–97 | 0.904, 0.902 | **#99.** |
| 0.8 | the same five, with su.mjs `openPicks` :130–148, and su.mjs `showing`, `settled`, `picks`, `past` :98–132 ~ mn.mjs `showing` :108–122 | 0.801–0.994 | #99 for the group's page; `showing` reads another page's elements in each, and the rest are su's own. |
| 0.8 | apps/movie-night/picks/picks.js `kept` :24–47 ~ apps/movie-night/app.js `kept` :27–51; picks.js `lends` :43–54 ~ app.js `libraries` :47–60; picks.js `asked` :55–73 ~ app.js `ask` :63–79 (0095684) | 0.802–0.890 | Tonight's Picks' `find` and `refusal`, counted by su-2 as Movie Night's plumbing again (demo/runs/su/2026-10-09/README.md §9, 5) and queued as mnp-2. The store's layout they read is **#100**. |
| 0.8 | apps/taste/check/other/other.js `kept` :7–21 ~ web/store.js `lendDb` :139–152 (0095684) | 0.824 | **#100**: the other app reads svrngs's store as the two pages do. |
| 0.8 | door.rs `lend_for` :829–955 ~ `forward` :1055–1157 ~ `forward` :1158–1235 (1f08560, 55dca40, e02a203) | 0.827–0.836 | I22 still: one `forward` given as two symbols. No change. |
| 0.8 | door.rs `through_pass` :709–729 ~ `through` :811–831 (1f08560) | 0.881 | The holders' and the members' axum handlers, twins since before the range (34d5272:637, :724); mnp-1 changed only `err(` to `Refused::Closed.say(` in each. They differ in `Who` and their words; a shared body saves ~6 lines and costs a parameter. No change. |
| 0.8 | door.rs `say` :180–188 and `said` :185–198 ~ apps/jellyfin/mw/src/answer.rs `said` :58–70 (1f08560, e02a203) | 0.808–0.845 | svrngs's refusal and the middleware's answer: two programs. No change. |
| 0.8 | api.rs `to_a_lend` :86–101 ~ origin.rs `lent` :54–66 (1f08560) | 0.827 | A middleware that routes by `Host` and the handler it routes to. No change. |
| 0.8 | picks.js `rank` :137–162 ~ `start` :158–193; su.mjs `yes` ~ `diary`; apps/taste/check/judge.py `verdict` ~ `get` | 0.818–0.887 | Neighbours in one file, overlapping spans (I9). No change. |
| 0.8 | apps/movie-night/check/judge.py `count` :205–225 ~ apps/taste/check/judge.py `count` :48–76; `load` :28–36 ~ :15–25 | 0.854, 0.826 | Two spikes' counts by two rules written before each (mn-0 §4, su-0 §6); `load` 3 and 5 lines, one returns None on an empty file, the other on an unreadable one. No change. |

**Window pass** (/tmp/composeaudit2/windows.py . 34d5272..0b5cac1, 5
normalized lines, tests included): 178 hits, **3 new**:

- apps/movie-night/check/mn.mjs:54–81 == apps/taste/check/su.mjs:49–76 and
  mn.mjs:83–100 == su.mjs:77–92 (0095684): #99, 44 lines verbatim but for
  names.
- crates/svrngs-node/src/door.rs:713–721 == :815–823 (1f08560): the twin
  handlers above. No change.

Reading the range for what neither pass would show:

- **#100, new, not fixed.** svrngs's lend store and a lend's address are
  spelled in three app files: IndexedDB `svrngs-lend`, store `kept`, key
  `pass <label>` and `baseOf` at apps/movie-night/app.js:28, :32, :39
  (mn-1), apps/movie-night/picks/picks.js:25, :29, :35 and
  apps/taste/check/other/other.js:6, :11, :14 (0095684); their one home is
  web/store.js:137–152 and web/lend.js:80, :96. A change to the store's
  layout or the lend's host breaks all three with nothing red in svrngs's
  own tests. This is mn's candidate 1 (demo/runs/mn/2026-10-08/README.md
  §9), which mnp-2 (a) makes production; the row names Movie Night's `find`
  only, and picks.js and other.js are its sites too. Not fixed: a spike
  changes no product, and the cut is mnp-2's.
- **The refusal kinds across the Rust/JS line.** door.rs `LendRefusal`
  (ex-`Refused`) spells eight kinds; web/lend.js `say` (:252, :267, :272,
  :311) spells three of them for the worker's own answers and reads `pass`
  (:320). As #50 judged `x-svrngs-pass`: a protocol constant across two
  programs, each kind exercised end to end by the page tests 1f08560 cites
  (both engines). No change.
- **ring_sync.rs `AskRound::now`** (e02a203): no trace of its own; its one
  caller, door.rs `holder_of`, traces before and after with `ended` and
  whether the signer stands. Every branch is at debug. No change.
- **#57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97 still
  open.** #97 grows by #99.

### (3) Duplicate type names (`code converge noun`)

`git diff 34d5272 0b5cac1 -- crates apps web bridge`, added lines
matching `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: three.

| noun | converge | git grep (`(struct\|enum\|trait\|type\|class)[[:space:]]+Name\b`) |
|---|---|---|
| **Refused** | **definitions (2)**: svrngs-core command.rs:66, svrngs-node door.rs:148 | **3**: command.rs:67 `Refused<R>` (why a commit wrote nothing), door.rs:149 (1f08560, why svrngs answered in a lend's place), web/src/http.rs:47 (why a request got no response; another workspace, #50) |
| Rounds | definitions (1), ring_sync.rs:161 | 1, :162 |
| AskRound | definitions (1), ring_sync.rs:167 | 1, :168 |

(converge's lines one early, I1; web/ unseen by it, as at #50.) **#98**:
svrngs-node names core's `Refused<R>` at its root (`pub use
svrngs_runtime::*`, lib.rs:33) and imports it as `CommitRefused` to tell it
apart (rail.rs:60, redeem.rs:58; svrngs cli.rs:77), so `crate::Refused`
and `crate::door::Refused` were two concepts in one crate — #35's pattern,
fixed then by renaming the node's (c0e063f). Fixed (9999bff): the door's
is `LendRefusal`, beside AppendRefusal, RedeemRefusal, ShareRefusal.

### Fixed here

- **#98** (9999bff). A rename: no variant, string or header changed.
  `cargo test -p svrngs-node --lib --test door`: 90 passed, 24 passed, the
  mnp-1 door tests among them. git grep for a `Refused` definition: core's
  and web/'s only.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 98 | 8 | crates/svrngs-node/src/door.rs:149 and crates/svrngs-core/src/command.rs:67 (named at svrngs-node's root, lib.rs:33), at 0b5cac1 | 9999bff | A second `Refused` in svrngs-node, as #35; the door's is `LendRefusal`. |
| 99 | 8 | apps/taste/check/su.mjs:40–98 (`browser`, `join`, `listed`, `group`, `openFromList`); apps/movie-night/check/mn.mjs:44–109, at 0b5cac1 | not fixed | Scoped use's driver forks Movie Night's group-page driver, 44 lines verbatim but for names (window pass), itself #97's fork of person.mjs. mn.mjs exports nothing; a shared driver is proven only by both apps' Jellyfin runs, which this audit did not run. |
| 100 | 8 | apps/movie-night/app.js:28, :32, :39; apps/movie-night/picks/picks.js:25, :29, :35; apps/taste/check/other/other.js:6, :11, :14; web/store.js:137–152, web/lend.js:80, :96, at 0b5cac1 | not fixed (mnp-2 a) | svrngs's lend store (`svrngs-lend`, `kept`, `pass <label>`) and a lend's address spelled in three app files beside their home. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 186 lines left (14564 / 14750). |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97 | as recorded | as recorded | not fixed | Untouched in the range; #97 grown by #99. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present: `Refused` door.rs:148 for :149, command.rs:66 for :67; `Rounds` :161 for :162, `AskRound` :167 for :168. |
| I9 | 6, 7 | `code dry-report` near tier | Still present: su.mjs `browser` :40–59 runs into `join` :57–75; picks.js `rank` :137–162 into `start` :158–193. |
| I19 | 6 | dry-report's exact tier | Still present: web/ and crates/svrngs-node/tests/ could-not-judge. |
| I22 | 6, 7 | `code dry-report` near tier | Still present: door.rs `forward` as :1055–1157 and :1158–1235. |
| I24 | 6, 7 | `code dry-report` exact tier | Unchanged binary; its "0 exact" in Rust is still no evidence of no copy. The window pass ran over every file. |

The row's check: `scripts/check.sh` at a9a8789, all 14 steps passed
(principles pinned through app laws; the WebKit seek rate "failed, a
rate" for wildcard and sam's 5016 ms first seek "failed" in the app laws'
table, neither failing its step, u5-11), exit 0.

## REVIEW-audit-compose-auto-3 (2026-10-09, range 22d1a2c..2e6edc7)

The range starts at 22d1a2c6b, the hash REVIEW-audit-compose-auto-2's row
carries (ralph/next/compose/STATE.md:24), the last `REVIEW-audit-` row. It
ends at 2e6edc7, the commit that made this row: 9 commits (`git rev-list
--count 22d1a2c..2e6edc7`), the last audit's `done` (ffd5fbc) and mnp-2
(ef75c07, 44784a4, a9e333f, 8566b86, be796ca, e09fefa), with its `done`
(93685ba) (`git log --format='%h %s' 22d1a2c..2e6edc7`). Product code
changed in ef75c07 (svrngs-node and web/) and 44784a4 (apps/movie-night/)
alone. This audit fixes two findings (#101, #102) and records two (#103,
#104), and narrows #100.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli, **the binary the last two
audits used**, sha256 35d97ed2… (`shasum -a 256`, built 2026-10-08 15:05),
so its tiers' plants (I24, I9) stand and were not run again. The embedder
was `llama-server` on the corpus's model (qwen-embedding-0.6b.gguf,
~/.svrnmesh/config.toml:18) on :8080, `-b 8192 -ub 8192` (I21), for this
session. `code index . --full` at 2e6edc7, tree clean: 2639 chunks in
406 s (/tmp/composeaudit3/index-full.out); the SCIP graph (scip_graph.db
02:07) written after the range's last product commit (44784a4, 01:22).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 22d1a2c..2e6edc7`,
summed by /tmp/composeaudit3/ledger.py (the first audit's, unchanged), its
rules as before: a commit counts toward the unit its subject names, else
the next `done`'s; product code is crates/, apps/, web/, bridge/; a test is
a `tests/` or `check/` segment or a lean_check test file name.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| mnp-2 | 142 | 58 | 84 | 349 | 36 | 313 |
| total | 142 | 58 | 84 | 349 | 36 | 313 |

- mnp-2 src: surface.rs +50, origin.rs +4, node.rs +1/−1, web/lend.js
  +42/−1, web/app.js +22/−13, web/open.js +3/−1 (ef75c07);
  apps/movie-night/app.js +20/−42 (44784a4, `find` cut to one fetch).
- mnp-2 tests: tests/browser/main.rs +147, tests/surface.rs +75,
  web/tests/reads.mjs +76 (ef75c07); apps/movie-night/check/ judge.py
  +35/−22, mn.mjs +15/−12, check.sh +1/−2 (44784a4).
- The last audit's `done` and this row's commit: none.

lean_check on `git archive` trees in /tmp/composeaudit3 (apps/ and tests
not counted, scripts/lean_check.py:49):

| at | svrngs-core | repository |
|---|---:|---:|
| 22d1a2c, range start | 2957 / 3000 | 14565 / 14750 |
| 2e6edc7, range end | 2957 / 3000 | 14643 / 14750 |
| decf376, this audit's fixes | 2957 / 3000 | **14644 / 14750** |

mnp-2 spent 78 counted lines, #101 1; **106 are left** (#80).

### (2) Clones (`code dry-report`)

**Scoped**, at 0.95, for each dir the range touched:

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-node/ | 400 | 5 (as before: `as_str` ×3, `fmt`, `addr`, `drop`, `try_from`) | 1 (declared.rs `harden_file` ×2) | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| web/ | 87 | **could-not-judge** (I19) | 0 | 0 |
| web/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| apps/movie-night/ | 59 | 0 | 0 | 0 |
| apps/ | 239 | 0 | 5 (#99's three, tv.mjs and judge.py's, as before) | 0 |

**Unscoped**: 0.95 (14 exact, 18 near), 0.9 (14, 43), 0.8 (14, 234).
/tmp/composeaudit3/clusters.py blamed every side (`git blame -w -M -C -C`,
HEAD 2e6edc7) for a line in 22d1a2c..2e6edc7: **0** clusters at 0.95,
**1** at 0.9, 10 at 0.8.

| at | cluster | cosine | read |
|---|---|---|---|
| 0.9, 0.8 | apps/movie-night/check/judge.py `bar4_before` :140–162 ~ `bar4_after` :165–182 (44784a4) | 0.916 | Two bars, before and after the list, each its own verdict; a neighbour overlap (I9) and a shared `people(run)` loop. No change. |
| 0.8 | surface.rs `reads` :202–235 (ef75c07) ~ node.rs `people_lends` :618–647, among 12 symbols (`lends`, `standings`, `lends_in`, register.rs `standing`…) | 0.801–0.884 | **#101**: `reads` copied `people_lends`'s prelude, the groups an actor and this computer share and this computer's person in them. Fixed (70077c4). The rest of the cluster reads standings and lends each its own way, as at the last audits. |
| 0.8 | web/app.js `open` :168–201, `enter` :199–238, `opening` :261–282 ~ web/open.js `start` :34–60 (ef75c07) | 0.818–0.855 | The group's page opening a lend and the lend's start page taking it: two halves of one handoff, neighbours in app.js. No change. |
| 0.8 | crates/svrngs-node/src/api.rs `to_a_lend` :86–101 ~ origin.rs `lent` :54–70 (ef75c07, +4) | 0.821 | As at the last audit: a middleware routing by `Host` and its handler. No change. |
| 0.8 | apps/movie-night/check/judge.py `bar1`, `bar2_3`, `bar4_*`, `bar5` ~ `main`; `count` ~ apps/taste/check/judge.py `count` (44784a4) | 0.809–0.916 | Each bar its own verdict; the two counts two spikes' rules, as at the last audit. No change. |
| 0.8 | apps/movie-night/check/mn.mjs `choose`, `play`, `back` ~ person.mjs `judge`, `seek` ~ demo/ drive.mjs; mn.mjs `computer` ~ `limit` ~ `once` (44784a4) | 0.801–0.998 | #97's fork, which 44784a4 touched (a copy chosen by its lender); neighbours otherwise. No change here. |
| 0.8 | apps/movie-night/app.js `card` ~ `render`; `listed` ~ picks/picks.js `group` (44784a4) | 0.820, 0.832 | Neighbours; `listed` asks one library, `group` lists the group's. No change. |

**Window pass** (/tmp/composeaudit3/windows.py . 22d1a2c..2e6edc7, 5
normalized lines, tests included): 191 hits, **4 new**, all one:
crates/svrngs-node/tests/browser/main.rs:872–876 (`kept_page`, ef75c07) ==
tests/door/origins.rs:234–238, :352–356, :408–412 and tests/door/sign_in.rs:120–124:
the five lines that bind a listener and keep its heads, before each
fake answers its own way (a range, a stream, an upgrade, an echo). Each
answers differently and lives in another test binary but origins.rs, which
browser/main.rs includes (:19–20); no change for the shared lines. Reading
`kept_page` beside them found **#102**: it was `page_origin` (:432–451, the
same file) with the heads kept. Fixed (484fea5).

Reading the range for what neither pass would show:

- **#103, not fixed.** `Host::standings` gives `None` when the groups held
  cannot be listed (node.rs:407–413, a warn); `people_lends` and, copied
  from it, `reads` read that as no group (`unwrap_or_default`, node.rs:627,
  surface.rs:205 at 2e6edc7), so a computer that could not read its own
  groups answers a member 403 "this computer lends nothing to you" and a
  lent page 404 "nothing is lent here": an absence answered as a refusal
  (AGENTS.md 6). Since #101 it is one line, node.rs `shared_with`. Not
  fixed: a 503 there changes what both answer, and no test plants an
  unlistable rail (`grep -rln "namespaces()" crates/svrngs-node/tests`: none); `Node::lends` (node.rs:313) does the same for the
  node's own round.
- **#104, not fixed.** What a lent page is told of the lends it reads is
  decided and shaped in two programs: which lends it reads, web/app.js:177
  (`l.reads` holds `o.kind`, not itself, for the handoff's pass) and
  surface.rs:226 (the same, by label); and the answer's shape `{kind, name,
  lender, address}` and `unanswered`, web/lend.js:363–371 `told` and
  surface.rs:229, :234. Each is held by its own test (tests/browser
  `a_lent_page_is_told_…`, tests/surface `a_lent_page_asks_…`), neither
  against the other's answer. As #50 judged a protocol constant across two
  programs, but this is a rule, not a name: one home would be svrngs-web's
  wasm deciding the page's half from the Rust both programs build. Not
  small; not fixed. `READS_PATH`/`READS` (surface.rs:52, lend.js:53) cite
  each other, as #50's header does. No change for them.
- **#100 narrowed.** 44784a4 cut Movie Night's own read of svrngs's lend
  store: apps/movie-night/app.js:20 asks GET /svrngs-reads. Still spelled:
  apps/movie-night/picks/picks.js:20–49 (`svrngs-lend`, `kept`, `pass
  <label>`, `baseOf`), apps/taste/check/other/other.js:6–14 and
  apps/taste/check/su.mjs:171–172. picks.js is served at Movie Night's
  origin, so /svrngs-reads answers it too, but it names each lend by its
  key (:49), which the answer leaves out by design. Not fixed: an app's
  change needs its run.
- **web/open.js `start`** (ef75c07, :54–56): `back` resolved against the
  page and kept only at its own origin, else `/`; no trace of its own, as
  the lines around it have none. The worker's `opening` and `told` trace
  each answer (lend.js:366, :377). No change.
- **#57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97, #99 still
  open**, none touched in the range but #97 (mn.mjs, 44784a4, its forked
  lines kept).

### (3) Duplicate type names (`code converge noun`)

`git diff 22d1a2c 2e6edc7 -- crates apps web bridge`, added lines
matching `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: one, a type alias,
and no `struct`, `enum` or `trait`.

| noun | converge | git grep (`(struct\|enum\|trait\|type\|class)[[:space:]]+KeptHeads\b`) |
|---|---|---|
| KeptHeads | definitions (0): "defined only in test code" (its scope leaves out `tests`) | 1, tests/browser/main.rs:896 at 2e6edc7 (:465 at 484fea5) |

No noun with more than one definition. The alias names the
`Arc<Mutex<Vec<String>>>` nine test sites spell unnamed, eight fakes' returns and a field
(tests/door/origins.rs:97, :230, :348, :404; sign_in.rs:51, :116;
door/main.rs:194; limit.rs:35; crates/svrngs/tests/share.rs:83); a name
in one file of nine is no second concept. No change.

### Fixed here

- **#101** (70077c4). `Host::shared_with`: the groups an actor stands in
  and this computer's person in the first it stands in too, for
  `people_lends` and `reads`; `standings` private again. `cargo test -p
  svrngs-node --lib --test surface --test door`: 90, 24, 4 passed;
  `--test browser`: 10 passed. Counted +3. Its signature failed
  scripts/check.sh's clippy step at 4669120 (`clippy::type_complexity`,
  node.rs:434); `Groups` names the type for it and `standings` (decf376),
  counted −2.
- **#102** (484fea5). `page_origin` is `kept_page(html).0`; 17 test lines
  fewer. `--test browser`: 10 passed, both engines.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 101 | 8 | crates/svrngs-node/src/surface.rs:205–211 and node.rs:627–632, at 2e6edc7 | 70077c4, decf376 | `reads` copied `people_lends`'s groups-shared prelude; one method decides it. |
| 102 | 8 | crates/svrngs-node/tests/browser/main.rs:868–894 and :432–451, at 2e6edc7 | 484fea5 | `kept_page` forked `page_origin` in the same file. |
| 103 | 6 | crates/svrngs-node/src/node.rs `shared_with` (ex-:627, surface.rs:205), and :313, at 484fea5 | not fixed | Groups that could not be listed are read as none: a member refused 403, a lent page 404, where svrngs could not judge. |
| 104 | 8 | web/app.js:177 and crates/svrngs-node/src/surface.rs:226; web/lend.js:363–371 and surface.rs:229, :234, at 2e6edc7 | not fixed | Which lends a page reads, and what it is told of them, decided and shaped in the page and the node, each held by its own test. |
| 100 | 8 | apps/movie-night/picks/picks.js:20–49; apps/taste/check/other/other.js:6–14, su.mjs:171–172, at 2e6edc7 | narrowed (44784a4) | Movie Night's app.js no longer reads svrngs's lend store; Tonight's Picks and scoped use's other app still do. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 106 lines left (14644 / 14750). |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99 | as recorded | as recorded | not fixed | Untouched in the range but #97's lines, kept. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I9 | 6, 7 | `code dry-report` near tier | Still present: judge.py `bar4_before` :140–162 runs into `bar4_after`; mn.mjs `choose`/`play`/`back` overlap. |
| I19 | 6 | dry-report's exact tier | Still present: web/ could-not-judge; a `tests` scope is always could-not-judge, its segment excluded, so #102 was found by reading. |
| I21 | 6, 7 | `code index` and its embedder | Unchanged: `-b 8192 -ub 8192` finished `--full`. |
| I24 | 6, 7 | `code dry-report` exact tier | Unchanged binary; its "0 exact" in Rust is still no evidence of no copy. The window pass ran over every file. |

The row's check: `scripts/check.sh` started at decf376 (08:52, its code
as at 55fff43), all 14 steps passed (principles pinned through app laws;
the WebKit seek rate "failed, a rate" for wildcard and sam's 9892 ms
through the lend "failed" in the app laws' table, neither failing its
step, u5-11), exit 0 (/tmp/composeaudit3/check.out). Its first run, at
4669120, failed clippy (decf376 above). While it ran the operator raised
the budget to 15,200 (aaecbdb, 08:59) and queued sup (5e87364, 9a452e4,
df842e8), none touching crates/, web/ or apps/ (`git diff --stat 55fff43
df842e8`); against it, #80's 14,644 leaves 556.

## REVIEW-audit-sup-auto-1 (2026-10-09, range 341bc2b..8020f03)

The range starts at 341bc2b8e, the hash REVIEW-audit-compose-auto-3's row
carries (ralph/next/compose/STATE.md:26), the last `REVIEW-audit-` row. It
ends at 8020f03, the commit that made this row: 21 commits (`git rev-list
--count 341bc2b..8020f03`): the last audit's `done` (8000a60), two state
commits (ee06adc, c0bc0f3), core-0 (fffdd8d, 0008f75), asleep-1 (9c1da3c,
9ac5ab6, 9fbdb30), sup-1 (790e25b, c255814, b482e08), sup-2 (1c3d662,
a57d80f, d9cb3c4, a754b73, 50ecc20), each unit's `done`, and this row's
(`git log --reverse --format='%h %s' 341bc2b..8020f03`). This audit fixes
two findings (#105, #106), records one (#107), narrows #103, and says
where #80's budget stands before sup-3.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli: **a new binary**, sha256
c9b653d3… (`shasum -a 256`, built 2026-10-09 12:03), not the last three
audits' 35d97ed2…, so its tiers were planted again (below). The embedder
was `llama-server` on the corpus's model (qwen-embedding-0.6b.gguf,
~/.svrnmesh/config.toml:18) on :8080, `-b 8192 -ub 8192` (I21), for this
session. The /tmp scripts the last audits cite were gone; this audit's
are /tmp/supaudit1/ledger.py, windows.py and clusters.py, their rules the
ones the last sections state, but for blame (below).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 341bc2b..8020f03`,
summed by /tmp/supaudit1/ledger.py: a commit counts toward the unit its
subject names, else the next `done`'s (so ee06adc and c0bc0f3 are
core-0's; neither touches product code); product code is crates/, apps/,
web/, bridge/; a test is a `tests/` or `check/` segment or a lean_check
test file name.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| core-0-unused | 19 | 607 | −588 | 69 | 679 | −610 |
| asleep-1-none-held | 175 | 92 | 83 | 213 | 2 | 211 |
| sup-1-app-named | 291 | 67 | 224 | 324 | 17 | 307 |
| sup-2-yes-stop-home | 960 | 79 | 881 | 744 | 0 | 744 |
| total | 1445 | 845 | 600 | 1350 | 698 | 652 |

- core-0 src: svrngs-core attest.rs −146, introduce.rs −363, lib.rs
  +12/−93, sig.rs +5/−4; svrngs-node rail.rs +2/−1 (fffdd8d).
- asleep-1 src: ring_sync.rs +83/−51 (9ac5ab6, the move) and +60/−26
  (9fbdb30); node.rs +16/−3, surface.rs +3/−2 (9c1da3c); door.rs +11/−8,
  node.rs +2/−2 (9fbdb30).
- sup-1 src: svrngs-core pass.rs +30/−17, door.rs +7/−6, web/src/lib.rs
  +9/−9, web/app.js +1/−1, svrngs-node Cargo.toml +2 (790e25b); origin.rs
  +147/−16, door.rs +62/−13, node.rs +20/−2, lend.rs +9, room.rs +3/−2,
  surface.rs +1/−1 (c255814).
- sup-2 src: door.rs +40/−9, sign_in.rs +17/−11 (1c3d662); lend.rs
  +15/−1, svrngs cli.rs +24/−5, share.rs +3, surface.rs +5/−1, door.rs +1
  (a57d80f); door/choices.rs +338, door.rs +20/−14, lend.rs +19/−1,
  node.rs +1 (d9cb3c4); origin.rs +29/−18 (a754b73, the move); home.rs
  +232, home/home.js +109, home/index.html +15, home/home.css +7,
  choices.rs +26, node.rs +23/−12, origin.rs +15/−3, surface.rs +9/−2,
  api.rs +5/−2, cli.rs +5, lib.rs +2 (50ecc20).
- Tests: tests/home.rs +434 (50ecc20), tests/door/choices.rs +211 and
  door/choices/tests.rs +45 (d9cb3c4), tests/lend_origin.rs +196 and
  door/main.rs +39/−2 (c255814), +43/−10 (790e25b), newcomer.rs +108
  (9fbdb30), tests/surface.rs +89 (9c1da3c), svrngs tests/share.rs +48
  (a57d80f); svrngs-core tests_introduce.rs −599, attest_tests.rs −51,
  tests.rs +46, svrngs-runtime tests.rs +23/−29 (fffdd8d).

lean_check on `git archive` trees in /tmp/supaudit1 (apps/ and tests
not counted, scripts/lean_check.py:49):

| at | svrngs-core | repository |
|---|---:|---:|
| 341bc2b, range start | 2957 / 3000 | 14644 / 15200 |
| 826ce52, core-0 done | 2628 / 3000 | 14315 / 15200 |
| 8cb33fc, asleep-1 done | 2628 / 3000 | 14366 / 15200 |
| bcc7ef7, sup-1 done | 2635 / 3000 | 14529 / 15200 |
| 8020f03, sup-2 done, range end | 2635 / 3000 | **15162 / 15200** |
| b4aab80, this audit's fixes | 2635 / 3000 | 15162 / 15200 |

core-0 made 329 counted lines of room (the queue's estimate: about 286);
asleep-1 spent 51, sup-1 163, **sup-2 633**. **38 are left** (#80), with
sup-3 to come: the phone's ask in web/app.js, web/lend.js, web/open.js and
web/src/lib.rs, all counted. The design's estimate for the whole queue
was about 650 (ralph/next/sup/STATE.md's fence 7); sup-1 and sup-2 have
spent 796. By the queue's own fence a row that would pass the budget
stops for the operator; sup-3 is likely to be that row.

### (2) Clones (`code dry-report`), each tier checked against a plant

**Plants**, the binary being new: a clone of the repository at 8020f03
(/tmp/supaudit1/plant), two functions appended to door/choices.rs, the
range's largest new file:

- **E** (exact): `write` (:171–182, 12 lines), verbatim, as `plant_exact`.
- **N** (near): the same body as `plant_near`, its names changed
  (`path`→`file`, `said`→`given`, `kept`→`all`, `person`→`who`,
  `choices`→`what`, `partial`→`tmp`, `text`→`body`, `e`→`err`).

Indexed `--full` (corpus `svrngs-supplant`, 2702 chunks, 482 s); `project
refresh` there then embedded it again as corpus `plant`, named for the
directory (2700 chunks in 8m; I25), and wrote its SCIP graph:

| scope, threshold | E | N |
|---|---|---|
| door/choices.rs, 0.95 | **missed**: 0 exact groups, 0 near | missed |
| crates/svrngs-node/, 0.95 | **missed**: the same 5 exact groups as clean, 1 near (`harden_file`) | missed |
| choices.rs and svrngs-node/, 0.9 | E ~ N only, 0.912; never with `write` | seen with E alone |
| choices.rs and svrngs-node/, 0.8 | E ~ N only, 0.914; `write` in another cluster | seen with E alone |
| window pass, no range | **seen**: choices.rs:172–181 == :368–377 | missed (renamed) |

So I24 stands on the new binary: the exact tier missed a verbatim body
under a new name, every group it finds pairing one name, and the near
tier's spans take their neighbours' lines (`write` :168–185 for :171–182,
E :364–381 running into N :378–392; I9). Its zeros in Rust are weak
evidence; the window pass found E. The plants were reverted (`git
checkout`) and the clone's corpus rebuilt clean at 8020f03 (`project
refresh --rebuild-index`, 2700 chunks in 511 s; its SCIP graph at
8020f03): 1537 symbols, the plants' 2 gone, choices.rs's 17 locations.

**Clean runs.** Scoped, at 0.95, for each crate dir the range touched:

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-core/ | 392 | 7 (`is_empty` ×3, `as_str` ×3, `len`, `lines`, `key` (tests), `signed_by_link`, `fmt`) | 0 | 0 |
| crates/svrngs-node/ | 447 | 5 (`as_str` ×4, `fmt` ×3, `drop`, `addr`, `try_from`) | 1 (declared.rs `harden_file` ×2, as before) | 2: `as_str` door.rs:731 and `fmt` :737 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| crates/svrngs-runtime/ | 122 | 1 (`open`, tests) | 0 | 0 |
| crates/svrngs/ | 66 | 0 | 0 | 0 |
| crates/svrngs/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| web/ | 87 | **could-not-judge** (I19) | 0 | — |

The two with a side added: `PersonKey`'s `as_str` and `Display`
(door.rs:731–739 at 8020f03, 1c3d662), three lines each, beside
`LendName`'s, `Kind`'s and `RefusedPath`'s in lend.rs: a newtype's
accessors, as each audit since u3 judged them. No change.

**Unscoped**: 0.95 (14 exact, 18 near), 0.9 (14, 43), 0.8 (14, 233).
/tmp/supaudit1/clusters.py blamed every side (`git blame -w` at 8020f03)
for a line in 341bc2b..8020f03: **2** groups at 0.95 (the two above),
**5** at 0.9:

| at | cluster | cosine | read |
|---|---|---|---|
| 0.9 | origin.rs `kept_to_reads` :99–138 (c255814) ~ surface.rs `reads` :203–243 | 0.912 | Both judge "the page of `app` reads `label`" by one rule, `Offered::reads_one`, over the groups `Host::shared_with` gives (#101's one decider); `kept_to_reads` asks two lenders per request and refuses, `reads` asks all and lists, and its doc cites the other. One rule, two callers. No change. |
| 0.9 | door/sign_in.rs `of` :334–353 ~ `forget` :351–366 (1c3d662, `PersonKey`) | 0.926 | Two senders on one task's channel, neighbours (I9). No change. |
| 0.9 | door/choices.rs `load` :78–90 ~ `read` :87–98 (d9cb3c4) | 0.918 | `read` calls `load`; neighbours (I9). No change. |

**Window pass** (/tmp/supaudit1/windows.py . 341bc2b..8020f03, 5
normalized lines over crates/, web/, apps/ in .rs, .js, .mjs, .py, tests
included). Its first run blamed with `-M -C -C`, as clusters.py did at the
last audits, and found 6 new; but `-C -C` credits a line copied from
another file to the commit it was copied from, so a copy made in the
range reads as old. With `git blame -w`: 318 window hits in 118 spans,
**21 new**. Product code, 2:

- door/choices.rs:269–274 == :300–305: `choose`'s and `stop_with_pass`'s
  axum signatures (`State(door)`, `dialer`, `Segment(raw)`, `headers`,
  `body`). No change.
- lend.rs:228–244 == svrngs share.rs:67–76: `Lend`'s and `Share`'s
  fields, `asks` added to each by a57d80f, as `reads` and `sign_in` were:
  the declaration and what the command was told, as before. No change.

Tests, 19: test set-ups copied, #31's case, but three are not across
binaries:

- **#106.** The 13 lines that bind an endpoint as a key and bridge it to
  a node's door, six times in the one `door` binary: door/main.rs:929,
  :1066, door/choices.rs:110, door/newcomer.rs:46, :177, door/sign_in.rs
  :195 (four added in the range). Fixed (1b246ea): `bridge_as`.
- **#106.** lend_origin.rs:139–159 == :436–456: sup-1 copied the u5-2
  test's two computers in the same file. Fixed (b4aab80):
  `alex_lets_sam_in`.
- The rest cross binaries, #31's case, not moved for #31's reason:
  tests/home.rs:154–174 (50ecc20) is lend_origin's two computers again,
  and :10–22, :30–38, :405–412 its header, `start` and a `Roster` seed;
  the `Fixture { act: RailAct::Admit … }` literal at
  tests/door/newcomer.rs:152, surface.rs:369, lend_origin.rs:448 and
  home.rs:166, the four of 28 `RailAct::Admit` sites in tests that blame
  into the range (`git grep -n "RailAct::Admit" 8020f03 -- crates/*/tests`,
  each line `git blame -w`);
  newcomer.rs:42–59 == :173–190, :67–79 == :198–207 (two tests' own
  arrangements, after #106 their bridges one call); surface.rs:282–290 ==
  :359–367 and two more there; svrngs share/tests.rs:46–50 ==
  join_page.rs, phone_dry.rs, share.rs (the `Share` literal, `asks: None`
  added to each).

Reading the range for what neither pass would show:

- **#105.** door/choices.rs:66 (d9cb3c4) named the body `{app, said}` a
  private `Person`, the core's noun (svrngs-core lib.rs:251), and
  home.rs:187 (50ecc20) spelled the same body again as `json!({ "app":
  app, "said": said })` to post it to a lender's door: one wire shape,
  two homes in one crate. Fixed (838f364): `Saying`, both ways.
- **#107, not fixed.** How svrngs-node writes a file whole is decided
  three times, each its own partial name: api.rs:51–53 `write_address`
  (`.api.partial`), lend.rs:461–463 the declaration
  (`.<name>.json.partial`), and door/choices.rs:176–181 `write`
  (`<name>.json.partial`, not hidden, sup-2). One function would rename
  two of the partials on disk, a change of behaviour, and save about 3
  counted lines; not done in an audit.
- **#103 narrowed** (50ecc20): `Host::shared_with` (node.rs:448–461) now
  answers svrngs's `fault` 500 when the groups held cannot be listed,
  and tests/home.rs `groups_that_cannot_be_read_are_said_so` watched it
  fail first. Still: `Node::lends` (node.rs:327–329,
  `standings().unwrap_or_default()`), which room.rs:109 and the bare
  command (svrngs cli.rs:914) read, so they say "lends nothing" where
  svrngs could not judge.
- **#104 still open**, as the loop's ee06adc keeps in view for sup-3:
  surface.rs `reads` (:229–240 at 8020f03) now adds `asks` and `ask` to
  each lend it tells a page; web/lend.js `told` does not yet, and is
  sup-3's.
- **Glassbox.** The new branches trace: home.rs `only_itself` (:129) on
  every answer, choices.rs `check` (:142–143) on every judgment, `member`,
  `found`, `stop` and `keep` on each refusal, origin.rs `kept_to_reads`
  on each of its three outcomes.
- **#57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97, #99, #100
  still open**, none touched in the range (`git diff --stat 341bc2b
  8020f03 -- apps` is empty).

### (3) Duplicate type names (`code converge noun`)

`git diff 341bc2b 8020f03 -- crates apps web bridge`, added lines
matching `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: `PersonKey`,
`Said`, `Stopping`, `Choice`, `Choices`, `Person`, `App`, `Choosing`,
`Of`, `Progress`, `Reach`, `Got` (tests), and the aliases `Kept`, `Done`,
`Whose`, `Rounds`. `converge noun --corpus-id plant` (the clean corpus,
its SCIP graph at 8020f03), beside `git grep -nP
"\b(struct|enum|trait|type|class)\s+<Name>\b" -- crates apps web bridge`:

| noun | converge | git grep | read |
|---|---|---|---|
| Said | 2: svrngs cli.rs:662, svrngs-node door/choices.rs:39 | 3: cli.rs:663, choices.rs:40, web/app.js:69 (`class Said extends Error`) | Three concepts: what a verb tells the person, what a person says of an app, a page's thrown words. svrngs never imports choices' (`grep -rn "choices::" crates/svrngs/src`: none). No change. |
| Person | 2: svrngs-core lib.rs:250, choices.rs:65 | 2: lib.rs:251, choices.rs:66 | **#105**: the door's request body under the core's noun. Fixed (838f364), `Saying`. |
| PersonKey | 1: door.rs:721 | 2: door.rs:722, apps/jellyfin/mw/src/person.rs:24 | One key, two programs either side of the door's `x-svrngs-key` header (door.rs:1184, the `PersonKey` the door resolves); the middleware parses what the door says, as #50 judged a protocol's two ends. No change. |

Every other noun: one definition (converge and git grep agree; `Got` is
tests/home.rs:69 alone, converge's scope leaving out `tests`). converge
reported each line one early (I1).

### Fixed here

- **#105** (838f364). `Saying` in door/choices.rs, `Serialize` and
  `Deserialize`, read by the door and serialized by home.rs `say`; the
  JSON object is the same (serde_json's map sorted either way). `cargo
  test -p svrngs-node --test home --test door --lib`: 2, 26, 91 passed.
  Counted 0.
- **#106** (1b246ea, b4aab80). `bridge_as` in tests/door/main.rs for six
  sites, `alex_lets_sam_in` in tests/lend_origin.rs for two: 72 test
  lines fewer (`git diff --shortstat 838f364 b4aab80`: +59/−131). `--test door` 26,
  `--test lend_origin` 3 passed; clippy `--tests` clean.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 105 | 8, 3 | crates/svrngs-node/src/door/choices.rs:66 and crates/svrngs-node/src/home.rs:187, at 8020f03 | 838f364 | The choice body `{app, said}` shaped twice in one crate, the door's under the core's noun `Person`. |
| 106 | 8 | crates/svrngs-node/tests/door/main.rs:929, :1066, choices.rs:110, newcomer.rs:46, :177, sign_in.rs:195; tests/lend_origin.rs:139 and :436, at 8020f03 | 1b246ea, b4aab80 | Test set-ups copied within one binary, so #31's reason did not hold. |
| 107 | 8 | crates/svrngs-node/src/api.rs:51–53, lend.rs:461–463 and door/choices.rs:176–181, at b4aab80 | not fixed | A file written whole through a partial renamed into place, decided three times, three partial names. |
| 103 | 6 | crates/svrngs-node/src/node.rs:327–329, at b4aab80 | narrowed (50ecc20) | `shared_with` now says svrngs could not judge; `Node::lends`, read by the room and the bare command, still reads unlistable groups as none. |
| 104 | 8 | web/lend.js:363–371 and crates/svrngs-node/src/surface.rs:229–240, at 8020f03 | not fixed (sup-3's) | What a page is told of its reads, shaped in the page and the node; sup-2 added `asks` and `ask` to the node's half. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 38 lines left (15162 / 15200), sup-3's counted web/ changes to come. |
| 31 | 8 | tests/home.rs:154–174 ~ tests/lend_origin.rs; the `RailAct::Admit` fixture across binaries | not fixed | Grown by the range: home.rs's two computers, four more `Admit` literals. |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still present on c9b653d3…: every definition a line early (`PersonKey` :721 for :722, `Said` :662 for :663). |
| I9 | 6, 7 | `code dry-report` near tier | Still present: spans take neighbours' lines (`write` :168–185 for :171–182; `load` ~ `read`, `of` ~ `forget`). |
| I19 | 6 | dry-report's exact tier | Still present, and said: web/ and every `tests` scope "could-not-judge … not a pass". |
| I24 | 6, 7 | `code dry-report` exact tier | **Still present on the new binary** c9b653d3…: plant E, a verbatim `write` renamed, is no exact group, scoped to its file or its crate. The window pass caught it. |
| I25 | 6 | `project refresh` and corpus ids | New: `project refresh` names the corpus after the directory, not the `--corpus-id` the index was built under, so a clone at /tmp/supaudit1/plant was embedded twice (as `svrngs-supplant` and `plant`). Then `refresh --rebuild-index` there deleted `plant/chunks.lance` and, its promotion refused ("both hold corpus data named documents.lance", a `plant-partition-local` left by the first refresh), left the new index in `plant-partition-local`; dry-report then said "no chunk index … run `svrn project init`", an error, not a zero. The partition was moved into place by hand (`mv`), its symbols checked: 1537, the plants gone. |
| I26 | 6, 7 | the `svrngs` corpus's SCIP graph | New: ~/.svrnmesh/indexes/svrngs/scip_graph.db, written 15:25 (after 50ecc20, 15:09), gave `PersonKey` at door.rs:707, its line at 1c3d662 (:707) or a57d80f (:708, I1), and no `Stopping` or `Choice`, which d9cb3c4 added: a graph from an older tree under a newer mtime. Cause not isolated; this audit judged from its own corpus at 8020f03. |

## REVIEW-audit-sup-auto-2 (2026-10-10, range 30d975a..204b3d4)

The range starts at 30d975a39, the hash REVIEW-audit-sup-auto-1's row
carries (ralph/next/sup/STATE.md:25), the last `REVIEW-audit-` row. It
ends at 204b3d4, the commit that made this row: 15 commits (`git rev-list
--count 30d975a..204b3d4`): the last audit's `done` (c3c9637), sup-3
(caf017e, 85986c2), sup-4 (da3004d, 517225a, e30975c, 7cbf217, 2f30f10,
c0c999a, 478fedf, 5dc463e, d4868ee), each unit's `done`, and this row's
(`git log --reverse --format='%h %s' 30d975a..204b3d4`). No product
source under crates/ changed in it (`git diff --stat 30d975a 204b3d4 --
'crates/*/src'` is empty): sup-3 is web/ and a browser test, sup-4 apps/
and a run. This audit fixes two findings (#108, #109), records two (#110,
#111), closes #100 and narrows #104.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli: **a new binary** again,
sha256 384ef1ca… (built 2026-10-09 19:22), not the last audit's
c9b653d3…, so its tiers were planted again (below). The embedder was
`llama-server` on qwen-embedding-0.6b.gguf (~/.svrnmesh/config.toml:18)
on :8080, `-b 8192 -ub 8192` (I21), for this session. Scripts:
/tmp/supaudit2/ledger.py, windows.py, clusters.py, the last audit's
unchanged.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 30d975a..204b3d4`,
summed by /tmp/supaudit2/ledger.py (its rules the last section's: a
commit counts toward the unit its subject names, else the next `done`'s;
product code is crates/, apps/, web/, bridge/; a test is a `tests/` or
`check/` segment or a lean_check test file name):

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| sup-3-phone-ask | 186 | 26 | 160 | 319 | 0 | 319 |
| sup-4-taste-rerun | 83 | 170 | −87 | 578 | 315 | 263 |
| total | 269 | 196 | 73 | 897 | 315 | 582 |

- sup-3 src: web/lend.js +11/−6 (caf017e, the move) and +56/−1, web/app.js
  +107/−16, web/index.html +10/−2, web/vercel.json +2/−1 (85986c2).
- sup-4 src, all apps/: taste/taste.mjs +22/−70, movie-night/picks/picks.js
  +58/−97, picks/index.html +1/−1 (da3004d); one line each in taste.mjs
  and picks.js (e30975c).
- Tests: tests/browser/main.rs +182, web/tests/ask.mjs +129, page.mjs +8
  (85986c2); apps/taste/check/ su.mjs +196/−113, judge.py +213/−126,
  check.sh +98/−54, other/other.js +69/−20, other/index.html +3/−2
  (517225a, e30975c, 7cbf217, c0c999a).

lean_check on `git archive` trees in /tmp/supaudit2 (apps/ and tests not
counted, scripts/lean_check.py:49):

| at | svrngs-core | repository |
|---|---:|---:|
| 30d975a, range start (the raise to 15,350) | 2635 / 3000 | 15162 / 15350 |
| 502d0e6, sup-3 done | 2635 / 3000 | 15272 / 15350 |
| 872a821, sup-4 done, range end | 2635 / 3000 | 15283 / 15350 |
| 3c29dda, this audit's fixes (tests only) | 2635 / 3000 | 15283 / 15350 |

sup-3 spent 110 counted lines (web/app.js, web/lend.js; vercel.json and
index.html are not code to lean_check, scripts/lean_check.py:29). sup-4's
apps/ cut counts nothing, and its 11 are demo/runs/sup/2026-10-10/run.sh
(`git diff --stat 502d0e6 872a821 -- '*.rs' '*.js' '*.mjs' '*.ts' '*.py'
'*.sh' ':!apps'`: that file alone, 11 of its 22 lines code). **67 are
left** (#80). The queue spent 796 + 121 = 917 counted lines against the
design's estimate of about 650 (fence 7), core-0's 329 of room and the
operator's two raises (aaecbdb, 30d975a) between.

### (2) Clones (`code dry-report`), each tier checked against a plant

**Plants**, the binary being new: a clone of the repository at 204b3d4
(/tmp/supaudit2/plant), one commit there (4607a9c) appending, since the
range's code is JavaScript:

- **E** (exact, JS): sup-3's `stopping` (web/lend.js:422–440), verbatim,
  as `plantExact`.
- **N** (near, JS): the same body as `plantNear`, its names changed
  (`label`→`which`, `h`→`held`, `lend`→`found`, `body`→`bytes`,
  `headers`→`hs`, `e`→`err`).
- **R** (exact, Rust, a positive control for the tier the last audit
  planted): door/choices.rs `write` (:172–183) verbatim as `plant_exact`.

Indexed `code index … --corpus-id plant --full` (2726 chunks; I25 again,
below), its SCIP graph by `refresh` there (rust 10790 symbols, python 996,
"typescript: skipped (export failed)"):

| scope, threshold | E | N | R |
|---|---|---|---|
| web/lend.js, 0.95 | **missed**: exact tier "could-not-judge" (I19), 0 near | missed | — |
| web/lend.js and web/, 0.9 | E ~ N only, 0.935; never with `stopping` | seen with E alone | — |
| web/lend.js, 0.8 | **seen**: `stopping`, E, N, `asking` one cluster, 0.870–0.935 | seen | — |
| door/choices.rs, 0.95 / 0.9 / 0.8 | — | — | **missed** at each: 0 exact groups, and `plant_exact` (in SCIP, choices.rs:366–377) in no near cluster even at 0.8, where `write` is |
| window pass | **seen**: lend.js:423–438 == :610–625 | missed (renamed) | **seen**: choices.rs:173–182 == :368–377 |

So on 384ef1ca the exact tier still misses a verbatim Rust body (I24) and
judges no JavaScript (I19, said so), and the near tier pairs a verbatim
JS copy with its original only at 0.8, its spans running into their
neighbours (`stopping` :419–443 for :422–440; I9). Its zeros here are weak
evidence; the window pass is the instrument that saw E and R.

**Runs.** The scoped and unscoped reports below are on the planted
corpus, not a second clean one: the plants are appended at the ends of
their files, so every other line is 204b3d4's, and they sit in no group or
cluster at 0.95 or 0.9 but with each other (above). Planting only adds
symbols, so a clone with a side added in the range is in these reports if
it is in a clean one. Scoped, for each dir the range touched:

| scope | locations | exact | near 0.95 | near 0.9 | with a side added in the range |
|---|---:|---|---:|---:|---|
| crates/svrngs-node/ | 448 | 5 (`as_str` ×4, `fmt` ×3, `drop`, `addr`, `try_from`), as before | 1 (`harden_file`), as before | 6 | 0 (no src changed) |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — | — |
| web/ | 96 | **could-not-judge** (I19) | 0 | 2 | lend.js `asking` ~ `stopping` (0.903), and E ~ N |
| apps/taste/ | 44 | 0 (judge.py's only: no JS in SCIP, I27) | 0 | 0 | 0 |
| apps/movie-night/ | 57 | 0 (likewise) | 0 | 2 (judge.py `bar4_before` ~ `bar4_after`, doors.py `when` ~ `fields`, neighbours, I9) | 0 |

**Unscoped**: 0.95 (14 exact, 16 near), 0.9 (14, 43), 0.8 (14, 233), 1438
locations. /tmp/supaudit2/clusters.py blamed every side (`git blame -w` at
204b3d4) for a line in 30d975a..204b3d4: **1** group at 0.95, **4** at
0.9, 15 at 0.8:

| at | cluster | cosine | read |
|---|---|---|---|
| 0.95 | crates/svrngs-node/src/home/home.js `uses` :65–88 (sup-2) ~ web/app.js `uses` :255–283 (85986c2) | 0.956 | **#110.** Not seen scoped (I12: home.js is under crates/, app.js under web/). |
| 0.9 | web/lend.js `asking` :406–422 ~ `stopping` :419–443 (both 85986c2) | 0.903 | **#111.** |
| 0.9 | apps/movie-night/check/mn.mjs `browser` :45–65 ~ apps/taste/check/su.mjs `browser` :58–77; `join` :63–81 ~ :75–92 | 0.915, 0.913 | The spike's driver, forked from Movie Night's (su.mjs:76's own words, "mn.mjs's"), sup-4 touching 6 and 3 of its lines: #97's case. No change. |
| 0.8 | home.js `uses` ~ app.js `asking` ~ `uses`; home.js `get`, `said` ~ app.js `choose` | 0.806–0.956 | #110 again: the phone's ask and the computer's. |
| 0.8 | the rest, 11 | 0.802–0.935 | Neighbours (I9: app.js `reach` ~ `joining`, judge.py `ranked` ~ `asked_once` ~ `unranked`, su.mjs `yes` ~ `again` ~ `otherPerson`, `settled` ~ `picks`, `stop` ~ `homeStop`), app.js `open` ~ `enter` ~ `opening` ~ open.js `start`, picks.js `refused`, `rank` ~ other.js `rank`, and judge.py and su.mjs ~ Movie Night's (#97). No change. |

**Window pass** (/tmp/supaudit2/windows.py . 30d975a..204b3d4, as the
last audit's, `git blame -w`): 290 window hits in 117 spans, **9 new**,
none in product code:

- **#108.** crates/svrngs-node/tests/browser/main.rs:1156–1165 ==
  tests/door/main.rs:74–83: sup-3's `holders_bridge` is #106's
  `bridge_as` with the ALPN fixed. Separate test binaries, #31's reason
  for leaving copies; but tests/common/mod.rs is the file each binary may
  compile (`#[path = "../common/mod.rs"]`, door/main.rs:18), so the
  reason does not hold. Fixed (2dc268a).
- **#109.** web/tests/ask.mjs:40–44 == handover.mjs:71–75 ==
  reads.mjs:28–32 == runtime.mjs:69–73: the five steps that open the
  group's page as the person, ask.mjs the fourth copy; page.mjs is the
  module the drivers share. Fixed (3c29dda), `held`.
- tests/browser/main.rs:616–621 == :900–905 == :1018–1023 and :645–649
  == :824–828 == :924–928 == :1037–1041: three tests' set-up (site,
  relay, bo, two computers, cy joined) and four `drive(engine, "<x>.mjs",
  json!({ site, key, token, open … }))` headers, each with its own fields
  after. A helper would return a seven-part tuple; the test reads better
  as it is. No change.
- web/tests/ask.mjs:23–30 == reads.mjs:17–24 and :37–42 == :25–30: a
  driver's prologue (args, `serve`, `launch`, its console kept) and, before
  #109, the steps `held` now is. No change.
- apps/movie-night/check/judge.py:19–24 == apps/taste/check/judge.py:10–15
  (the imports) and mn.mjs:58–75 == su.mjs:70–87 (#97's fork). No change.

After the fixes (`windows.py . 30d975a..HEAD`): 278 hits in 113 spans, 7
new; the two fixed gone, handover.mjs and runtime.mjs's imports and their
lines around `held` now counted new only because `held`'s line is.

Reading the range for what neither pass would show:

- **#110, not fixed.** The computer's home page and the phone's group's
  page each ask the person and list what apps use, in the same words, by
  the same reading of a choice: crates/svrngs-node/src/home/home.js
  `uses` :68–85 and `ask` :88–96 (sup-2), web/app.js `uses` :258–281 and
  `asking` :229–256 (sup-3) — "uses" / "was stopped using" on `c.said ===
  'yes'`, "cannot be asked now", "That app, or what it asks to use, is not
  lent in your group(s) now." The decision is the door's either way
  (door/choices.rs `check`); the pages only say it. One home would be a
  module both pages load, but the home page answers only itself (fence 3,
  home.rs's `only_itself`) and the group's page is static files on the page
  host: not small, and a choice of the operator's about where the page
  shell lives. Recorded.
- **#111, not fixed.** web/lend.js `asking` :409–419 and `stopping`
  :422–440 (sup-3), with `answered` :290–297 before them, each hold the
  pass (`holding()`), refuse 503 `pass` without it, and refuse 403
  `closed` a lend it does not read, in the same words. The refusal is
  one function already (`say`); the guards differ (`asking` refuses the
  page's own lend, `answered` a navigation without a pass); one guard
  returning a pass or a Response would save about 2 counted lines. Not
  done in an audit.
- **The choice's body** `{app, said}`, one shape in Rust since #105
  (`Saying`), is spelled at its JavaScript ends: web/app.js:221
  (`JSON.stringify({ app, said })`), web/lend.js:430 (`'{"said":"stop"}'`,
  the app the pass's), home.js:28 (`{ lend, app, said }` to its own
  origin's `/choices`, home.rs's own body). Two ends of a wire, as #50
  judged; each held by a test that posts it (tests/browser/main.rs ask
  test, tests/home.rs). No change.
- **#104 narrowed** (85986c2): web/lend.js `told` (:374–389) now gives a
  page `asks` and `ask` in the shape surface.rs `reads` gives on a
  computer. Still two homes, the page's and the node's, each held by its
  own test (web/tests/reads.mjs and ask.mjs; tests/surface.rs).
- **#100 closed** (da3004d): Tonight's Picks reads its lends from
  `/svrngs-reads` (picks.js:26) and stops through `svrngs-stop` (:27),
  no longer svrngs's lend store. apps/taste/check/other/other.js:26 still
  opens `svrngs-lend`, on purpose: it is the plant, an app's page reading
  what its origin keeps to say a yes for itself (its header, :5–8), which
  svrngs refuses.
- **Glassbox.** sup-3's branches trace: app.js `asking` on both outcomes
  (:232, :236), its yes kept (:248), `uses` each failure (:276), `post`
  each refusal (:114); lend.js `asking` and `stopping` each branch
  (:414, :417, :427, :432, :437).
- **The browser test's one failure.** With #108 in place, one run of
  `--test browser -- a_lent_page_asks` failed in Chromium (148 s, while
  the plant corpus embedded; its output not kept). After it Chromium alone
  passed (43.98 s) and both engines passed in three runs (54.65 s, 68.09
  s, 65.10 s, the embedder busy in two), then the whole binary, 12 passed
  (152.81 s), after #109. One unexplained failure in five runs is not a
  rate (AGENTS.md 7); the next run that fails keeps its output.
- **#103, #107 still open**, nothing under crates/*/src changed in the
  range; **#57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97, #99
  still open**, as recorded.

### (3) Duplicate type names (`code converge noun`)

`git diff 30d975a 204b3d4 -- crates apps web bridge`, added lines
matching `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: **none** (the
command's output is empty). The range added functions in JavaScript and
tests, no type, so no noun was run through `converge noun`.

### Fixed here

- **#108** (2dc268a). `bridge_as` in tests/common/mod.rs, called by the
  door's tests and the browser test (`Alpn::Lend`); `holders_bridge`
  deleted. The browser's copy dialled `addr().ip_addrs()`, the shared one
  `loopback_sockets`, both this computer with no relay. `git diff
  --shortstat`: +35/−53. fmt, clippy `--tests` clean; `--test door` 26
  passed; the browser test as above.
- **#109** (3c29dda). page.mjs `held`, called by four drivers; drive.mjs's
  copy differs (a 15 s goto, its failure named) and stays. +19/−24;
  `--test browser` 12 passed.

### The check

scripts/check.sh at 12afe52 failed one step, `conformance canary`: wildcard
Chromium Sam's "64 MiB uploaded, its length told", "nothing within 60000
ms". The upload was not lost: the worker's trace in the same failure says
"upload handed its body to the dial whole, 60128 ms", then "upload answered
200" (Bo's took 45 s), at a load of 65 on 12 cores (`uptime` after: 64.23
over 15 min). Neither move touches that test: conformance.rs and
conformance.mjs call neither `bridge_as` nor `held`. Run alone at a load of
27-72, the step passed (`cargo test -p svrngs-node --test conformance`: 6
passed, 227.51 s). One run, not a rate; a 64 MiB upload's 60 s window is
close to what this host gives under load.

Run again at 45b2ce3 (`REVIEW-audit-sup-auto-2-3.await.log`, ended 23:09
local), it failed one other route of the same step: wildcard Chromium Sam's
"seeks back to back", "the last played again in 6580 ms", at a load of
93.79 then 50.59 on 12 cores (the failure's `load` lines), the upload now
passed. That is the miss chk-1 (e31bdce, 23:13) makes could-not-judge past
3 a core (conformance.rs:252-264); the run began before it landed, so it
ran without it. check.sh is run again at HEAD.

Run again at b841412 (`REVIEW-audit-sup-auto-2-4.await.log`, 23m37s),
scripts/check.sh exited 0: every step `passed`, none failed, could-not-judge
or never-ran (the log's 48 lines; `grep -E '^(failed|could-not|never)'`
finds none). All 26 conformance routes passed, both Chromium seeks among
them (the log's :26-27 and :39-40); chk-1's could-not-judge line was not
needed. One run, at a load near
70-90 on 12 cores (`uptime` after it: 69.87 78.63 91.39), not a rate.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 108 | 8 | crates/svrngs-node/tests/browser/main.rs:1155 and tests/door/main.rs:69, at 204b3d4 | 2dc268a | sup-3's `holders_bridge`, #106's `bridge_as` again, the shared tests/common unused for it. |
| 109 | 8 | web/tests/ask.mjs:40, handover.mjs:71, reads.mjs:28, runtime.mjs:69, at 204b3d4 | 3c29dda | Four drivers open the group's page as the person by the same five steps. |
| 110 | 8, 3 | crates/svrngs-node/src/home/home.js:68–96 and web/app.js:229–281, at 204b3d4 | not fixed | The computer's and the phone's pages each ask and list what apps use, in the same words by the same reading; two hosts, the home page answering only itself. |
| 111 | 8 | web/lend.js:409–419 and :422–440 (and :290–297), at 204b3d4 | not fixed | The worker's pass-and-lend guard written three times, its refusals in the same words. |
| 104 | 8 | web/lend.js:374–389 and crates/svrngs-node/src/surface.rs `reads`, at 204b3d4 | narrowed (85986c2) | The page's half now gives `asks` and `ask`; still two homes, each held by its own test. |
| 100 | 8 | apps/movie-night/picks/picks.js:26–27, at 204b3d4 | closed (da3004d) | Tonight's Picks off svrngs's lend store; other.js's read is the plant's. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 67 lines left (15283 / 15350). |
| 31, 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 103, 107 | as recorded | as recorded | not fixed | Untouched in the range but #97 (su.mjs, 517225a; its fork kept). |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I9 | 6, 7 | `code dry-report` near tier | Still present on 384ef1ca…: `stopping` :419–443 for :422–440, `plantExact` :606–630 running into `plantNear`; a verbatim JS copy and its original cluster only at 0.8. |
| I12 | 5, 6 | `code dry-report --scope` | Still present: home.js ~ app.js (0.956) is in no scoped report. |
| I19 | 6 | dry-report's exact tier | Still present, and said: web/ and every `tests` scope "could-not-judge … no SCIP function symbols in scope". |
| I24 | 6, 7 | `code dry-report` exact tier | **Still present on 384ef1ca…**: plant R, `write` verbatim as `plant_exact`, in SCIP (scip_graph.db `symbols`: choices.rs 366–377) and in no exact group; and in no near cluster at 0.8, where `write` is in one. The window pass caught it. |
| I25 | 6 | `code index` and corpus ids | **Worse than recorded**: `code index --corpus-id plant --full` exited 0 and printed "✓ Indexed 2726 chunks" while its log said "finalise_solo_ingest failed — partition-of-self left in place … both hold corpus data named documents.lance" (the last audit's `plant` corpus in the way): a success-shaped line over a failed promotion. `refresh` then found no `_corpus_meta.json`, discarded the partition ("starting fresh") and embedded again. The last audit's documents.lance and texts were moved aside (/tmp/supaudit2/old-plant-index), and `code index --full` run again promoted (2726 chunks, 774 s). |
| I27 | 6 | dry-report's exact tier over mixed languages | New: apps/taste/ and apps/movie-night/ report "**0** exact-clone groups", not could-not-judge, though only their Python is in SCIP (`refresh`: "typescript: skipped (export failed)"); their .mjs and .js are unjudged under a zero. |

## lean-4-once-and-follow-ups (a) (2026-10-10): #49, #58, #64, #107, #111 closed; #42, #48, #63, #89 said why not

The row's part (a) took the open findings in counted code whose fix is a
move. Each commit's body holds its run; `git log --format='%h %s'
73a234d..e978122` lists them.

- **#107, closed (dfa0b31).** One `declared::write_whole(path, text)`,
  `.<its name>.partial` renamed into place, for the API's address, a
  lend's declaration and its choices; each keeps its refusal. The
  choices' partial is now hidden, the one difference #107 named. Not
  owner-only, as the row's letter asked: the three files are 0644 under
  the umask today (`ls -la ~/.svrngs/api`), and `api` is read by apps the
  node did not start (api.rs:40-44); narrowing them is a change, a
  follow-up, not a move. `declared::write_owner_only` stays beside it.
- **#58, closed (4a6e233).** `Label::of_hex(lender, name)` in svrngs-core
  beside `Label`, read by the node (node.rs `origin`, `room_address`;
  surface.rs; room.rs) and the page (web/src/lib.rs `label`); node.rs's
  `label` is gone.
- **#64, closed (77602a7).** share.rs `rows` gives a parsed `Kind`; both
  readers take it. #64's "a change of behaviour" did not hold: a row whose
  kind does not parse never matched `accounts_of`'s parsed `Kind` either.
- **#111, closed (0c25cdf).** The worker's refusals at the pass and the
  lend, `unheld()` and `closed()`, each in one set of words at all seven
  sites. The three guards stay three: each decides differently (`lent`
  redirects a navigation and does not trace; `asking` refuses its own
  lend).
- **#49, closed for the page's files (e978122).** store.js's
  `lendOrigin` and `labelOf`, read by app.js, lend.js and open.js. The
  door's `own_origin` (Rust, an `Origin`'s first label) and origin.rs
  (`<label>.localhost`) are other languages and hosts, each one home in
  its crate.
- **#42, not fixed.** issuer.rs `key`/`addr` and web/src/lib.rs's issuer
  read decide differently — the command `expect`s where the page refuses,
  and the page with a bare key falls back to n0's lookup and relays where
  `addr` dials the key alone — and one home needs a crate both workspaces
  link that names iroh (svrngs-core does not, being `no_std` and
  iroh-free). A change of shape, not a move.
- **#48, not fixed.** The three reads of the person's 32 bytes in
  web/src (join.rs `person`, lib.rs `bind`, lib.rs `pass`) are each the
  array's own rule, `<[u8; 32]>::try_from`, into a key type of its own,
  refused in words and a trace of its own (`JOIN: REFUSED`, `LAYER key:
  REFUSED`, none); fence 4 keeps each, and a function over `try_from`
  would decide nothing more and save no line. `pass`'s refusal has no
  trace (#48's last note): a glassbox gap, a change, not a move.
- **#63, not fixed.** `x-svrngs-page-cookies` is named by the worker
  (lend.js) and by the shim (shim.js), which runs as a classic script in
  a lent page and shares no module with it; carrying the name on the
  shim's address changes the address every lent page loads — a change —
  and saves no line.
- **#89, not fixed.** Where a script goes first and its nonce are decided
  in Rust in the door and in JavaScript in the phone's worker: two
  runtimes that cannot share a body. A golden test over both is a change
  to the tests, not a move.

Pairs of SURVEY.md's "written twice" not taken, each said in its commit:
rail.rs `append_act` and `push_answer`'s early returns (their tests read
a `Response`; 732285d); `through_pass` and `through`, which take the
dialer as different `Who`s and refuse in their own words (0aa120e);
sync.rs's mismatching unit, which decides differently from the two walks
written once (6cad0b9); `Origin`, `RefusedPath` and `Secret`, which have
an `as_str` and no `Display` (7a111b2).

## REVIEW-audit-lean-auto-1 (2026-10-10, range 8b508d0..06bece5)

No `REVIEW-audit-` row in this queue came before it, so the range starts
where the queue does: 8b508d0, REVIEW-audit-sup-auto-2's `done` and the
parent of 7e29266, the commit that made ralph/next/lean/STATE.md. It ends
at 06bece5, the commit that made this row: 37 commits (`git rev-list
--count 8b508d0..06bece5`), lean-1 to lean-4 and their `done`s. This audit
fixes one finding (#112), records one (#113), and closes #110.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli: **a new binary**, sha256
b604bcb6… (built 2026-10-10 14:10), not the last audit's 384ef1ca…, so its
tiers were planted again (below). The embedder was `llama-server` on
qwen-embedding-0.6b.gguf (~/.svrnmesh/config.toml:18) on :8080, `-b 8192
-ub 8192` (I21), for this session. Scripts: /tmp/leanaudit/ledger.py (the
last audit's, its unit pattern `lean-N`), windows.py and clusters.py
unchanged.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 8b508d0..06bece5`,
summed by /tmp/leanaudit/ledger.py (a commit counts toward the unit its
subject names, else the next `done`'s; product code is crates/, apps/,
web/, bridge/; a test is a `tests/` or `check/` segment or a lean_check
test file name, so lean-2's `iroh/tests.rs`, `oplog_tests.rs`,
`tests_support.rs` count as tests):

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| lean-1-product-only | 0 | 0 | 0 | 26 | 2 | 24 |
| lean-2-tests-out | 7 | 782 | −775 | 780 | 0 | 780 |
| lean-3-unreached | 46 | 478 | −432 | 228 | 223 | 5 |
| lean-4-once-and-follow-ups | 505 | 549 | −44 | 298 | 8 | 290 |
| total | 558 | 1809 | −1251 | 1332 | 233 | 1099 |

- lean-1: no product line; web/tests/serve.mjs +24 (5a8872e, the page
  server out of demo/runs/) and the counting rule in scripts/ (fa98e42,
  not product code).
- lean-2 (dd65852): iroh.rs −403, oplog.rs −314, journal.rs −31, op.rs −34
  into iroh/tests.rs +399, oplog_tests.rs +311, journal_tests.rs +35,
  tests_support.rs +35 — a move, +780 test against −782 src.
- lean-3 (1c6fe9d, 4f6037d, bfce2b7): guest/tunnel.rs −137,
  ring_checkpoint.rs −42, guest.rs −44 and −70, view.rs −32, lib.rs −14
  and −26, sync.rs −25, node.rs −20; tests rewritten on what the product
  uses (+228/−223).
- lean-4 (a), 13 commits, src −97 net (732285d to e978122); (b) origin.rs
  +25/−1 (9cdfda5); (c) question.js +31, home.js +8/−11, app.js +9/−19,
  home.rs +8, sw.js +1/−1 (7b10b4a); (d) surface.rs +8/−4, lend.js
  +5/−4, picks.js +3/−5 (c42939a). Tests: home.rs +189, browser/main.rs
  +73, web/tests/home.mjs +27.

lean_check on `git archive` trees in /tmp/leanaudit (each tree's own
scripts/lean_check.py, and HEAD's where the rule changed):

| at | svrngs-core | repository |
|---|---:|---:|
| 8b508d0, range start | 2635 / 3000 | 15284 / 15350 (13426 by lean-1's rule) |
| c5b5648, lean-1 done | 2635 / 3000 | 13428 / 15350 |
| b77d996, lean-2 done | 2619 / 3000 | 12879 / 15350 |
| 73a234d, lean-3 done | 2547 / 3000 | 12615 / 15350 |
| f22f7cb, lean-4 (a) done | 2540 / 3000 | 12496 / 15350 |
| 06bece5, range end | 2540 / 3000 | 12526 / 15350 |
| 0ca47dc, this audit's fix (#112) | 2540 / 3000 | 12528 / 15350 |

The queue took 2,758 counted lines off (15284 → 12526): 1,856 by lean-1's
rule, 549 by lean-2, 264 by lean-3, 119 by lean-4 (a); lean-4 (b) to (d)
spent 30. **2,822 are left** (#80), against the survey's estimate of about
3,000 the product does not carry (ralph/next/lean/SURVEY.md).

### (2) Clones (`code dry-report`), each tier checked against a plant

**Plants**, the binary being new: a clone of the repository at 06bece5
(/tmp/leanaudit/leanplant), one commit there (faac109) appending copies of
two symbols the range added:

- **R** (exact, Rust): door.rs `remove_counted` (:505–517, 0aa120e),
  verbatim, as `plant_exact`.
- **RN** (near, Rust): the same body as `plant_near`, its names changed
  (`headers`→`map`, `goes`→`drops`, `gone`→`dropped`, `n`→`count`,
  `name`→`key`).
- **J** (exact, JS): question.js `uses` (:22–32, 7b10b4a), verbatim, as
  `plantExact`.
- **JN** (near, JS): the same body as `plantNear`, its names changed.

Indexed under a corpus id of its own, `leanplant`, so no earlier audit's
corpus was in the way (I25): `code index … --corpus-id leanplant --full`
("✓ Indexed 2675 chunks in 416s", rc 0), then `refresh` there (rust 10695
symbols, python 996, "typescript: skipped (export failed)"); the SCIP graph
holds `plant_exact` (door.rs 1316–1328) and `remove_counted` (504–516).

| scope, threshold | R | RN | J | JN |
|---|---|---|---|---|
| door.rs, 0.95 | **missed**: 0 exact groups, 0 near | missed | — | — |
| door.rs, 0.9 | R ~ RN only, 0.920; never with `remove_counted` | seen with R alone | — | — |
| door.rs, 0.8 | **missed** against its original: `remove_counted` clusters with its neighbours `strip_hop_by_hop`, `upgrade_of`, `keep_upgrade` (0.821–0.868), R with RN alone | as at 0.9 | — | — |
| question.js, 0.95 | — | — | **missed**: exact tier "could-not-judge" (I19), 0 near | missed |
| question.js, 0.9 | — | — | J ~ JN only, 0.940 | seen with J alone |
| question.js, 0.8 | — | — | **seen**: `uses` ~ J, one cluster | seen |
| window pass | **seen**: door.rs:506–516 == :1318–1328 | missed (renamed) | **seen**: question.js:23–29 == :34–40 | missed (renamed) |

So on b604bcb6 the exact tier still misses a verbatim Rust body that is in
SCIP (I24) and judges no JavaScript (I19, said so); the near tier pairs a
verbatim JS copy with its original only at 0.8, and a verbatim Rust copy
with its original at no threshold run, its spans running into their
neighbours (`plant_exact` 1314–1332 for 1316–1328, `remove_counted`
502–520 for 504–516; I9). Its zeros are weak evidence; the window pass saw
both exact plants.

**Runs**, on the planted corpus as the last audit's were (the plants are
appended at their files' ends, every other line 06bece5's, and they sit
in no group or cluster but with each other and, at 0.8, J with `uses`).
Scoped, for each dir the range touched, /tmp/leanaudit/clusters.py blaming
every side (`git blame -w` at faac109) for a line in the range:

| scope | locations | exact | near 0.95 | near 0.9 | a side touched in the range |
|---|---:|---|---:|---:|---|
| crates/svrngs-core/ | 373 | 4 (`lines`, `is_empty`, `signed_by_link`, tests' `key`) | 0 | 9 | 1 at 0.9: `decide` ×4, tests_support.rs:300–308 (2 lines, 4f6037d) |
| crates/svrngs-node/ | 421 | 1 (`addr`), down from the last audit's 5: `as_str` ×4, `fmt` ×3 (7a111b2), `drop` (ad82d1e), `try_from` gone | 0 | 9 | 4 at 0.9: R ~ RN, J ~ JN, `kept_to_reads` ~ `reads`, `live_push` ~ `live_drain` |
| crates/svrngs-runtime/ | 123 | 1 (tests' `open`) | 0 | 0 | 0 |
| crates/svrngs/ | 66 | 0 | 0 | 0 | 0 |
| web/ | 93 | **could-not-judge** (I19) | 0 | 0, was 2 (#111 closed in 0c25cdf) | 0 |
| apps/movie-night/ | 57 | 0 (I27) | 0 | 2 | 0 |
| apps/taste/ | 44 | 0 (I27) | 0 | 0 | 0 |
| apps/jellyfin/ | 109 | 0 (I27) | 2 | 5 | 0 (check/ only, lean-1's one line) |

**Unscoped**: 0.95 (9 exact, 15 near), 0.9 (9, 41), 0.8 (9, 221), 1390
locations. The row asks for every clone with a side that is a symbol
ADDED in the range. The names defined at 06bece5 and not at 8b508d0, over
the product's .rs and .js (tests and check/ apart): `appOf`, `closed`,
`harden`, `labelOf`, `lendOrigin`, `of_hex`, `question`, `readable`,
`remove_counted`, `tell_share`, `unasked`, `uri_of`, `write_whole`, with
question.js's `uses` and `named`, moved there from the pages. Of these,
those in a group or cluster:

| at | cluster | cosine | read |
|---|---|---|---|
| 0.95, 0.9 | home.js `uses` :65–85 ~ web/app.js `uses` :250–273 | 0.951 | **#113**: the two pages' callers of question.js `uses` since 7b10b4a, each side touched by it. |
| 0.8 | question.js `uses` :19–34 ~ home.js `uses`, `ask` ~ app.js `asking`, `uses` (and J, JN) | 0.809–0.951 | #113 again: the callers and the shared body. |
| 0.8 | pass.rs `of_hex` :100–109 ~ `of`, `parse`, `lender`, web/src/lib.rs `label`, pass_tests.rs `key`, `lender` | 0.803–0.858 | `of_hex` reads hex into `of`; lib.rs `label` calls `of_hex` (4a6e233). Neighbours and a caller (I9). No change. |
| 0.8 | door.rs `remove_counted` :502–520 ~ `strip_hop_by_hop`, `upgrade_of`, `keep_upgrade` | 0.821–0.868 | `strip_hop_by_hop` calls `remove_counted` (door.rs:498); the rest neighbours (I9). No change. |
| 0.8 | declared.rs `write_whole` :130–141 ~ `write_owner_only` :116–129 ~ `harden` :138–155 | 0.809–0.811 | Neighbours; `write_owner_only` stays beside `write_whole`, lean-4 (a) said why (#107's close). No change. |

The rest with a side touched in the range, though no side added: 0.9
origin.rs `kept_to_reads` ~ surface.rs `reads` (0.916; one rule, two
callers, as REVIEW-audit-sup-auto-1 judged); rail.rs `live_push` ~
`live_drain` (0.916, two three-line handlers whose spans run together,
I9); svrngs-core `decide` ×4 (0.918–0.948; tests_support.rs's two are the
test fixtures lean-3 B moved there). No change.

**Window pass** (/tmp/leanaudit/windows.py . 8b508d0..06bece5): 282 window
hits in 114 spans, **24 new**. In product code, 3:

- door.rs:749–757 == :892–900: `through_pass` and `through`'s signatures
  and first refusal, each in its own words; lean-4 (a) left them two, said
  in 0aa120e (REVIEW-audit-sup-auto-2's last paragraph). No change.
- door/choices.rs:267–272 == :293–298: `choose` and `stop_with_pass`'s
  axum extractor lists. No change.
- svrngs-runtime journal.rs:418–423 == journal_tests.rs:20–25: `seal`'s
  `Author` literal and the test fixture's, moved out of journal.rs by
  lean-2. No change.

The other 21 are tests: lean-2's moved modules now beside the integration
tests they resemble (iroh/tests.rs:62–72 == tests/common/mod.rs:11–21,
:50–56 == tests/accept.rs:23–29), lean-3 B's rewritten set-up in each
integration test (door/main.rs:44–67 == lend_origin.rs:32–53 ==
room.rs:38–59; canaries.rs:22–34 == group_not_mesh.rs:26–38; and the
like), lean-4 (b)'s new test beside the home test (home.rs:160–177 ==
:462–479 == lend_origin.rs:134–151), and web/tests/home.mjs's prologue
(:13–18 == reads.mjs:22–27). Each binary's set-up, #31's case. No change.

Reading the range for what neither pass would show:

- **#112, fixed (0ca47dc).** c42939a built a page's stop address in
  surface.rs `reads` with "/svrngs-stop" (surface.rs:230 at 06bece5), which
  origin.rs `lent` parsed with its own literal (origin.rs:199): one path,
  two spellings in one crate. `surface::STOP_SUFFIX` beside `READS_PATH`
  now. web/lend.js's (`STOP` :65, `told` :390) is the phone's, another
  language and host, as `READS` is.
- **#110, closed (7b10b4a)**: one wording, question.js, for both pages'
  question and what apps use; **#113** is what is left of it.
- **#113, not fixed.** home.js `uses` :68–82 and web/app.js `uses`
  :253–270 each walk the lends that ask, get their choices, and on a
  failure say `unasked(l)`: the home page from its node's `/choices` with
  the error's message after the line, the group's page by a dial, the
  error traced and not shown. And "That app, or what it asks to use, is
  not lent in your group(s) now." is spelled in each, home.js:87 ("groups",
  the page of all of them) and app.js:228 ("group", the page of one). The
  loop into question.js would take each page's fetch and failure as
  arguments and give them one failure line, a change of what a page says,
  not a move; each saves about 3 lines. Recorded.
- **Glassbox.** lean-4 (b) traces both of `readable`'s branches
  (origin.rs:226, :229 at 06bece5); (d)'s `stop` is built where `ask` is,
  inside the branch `reads` traces (surface.rs:235–236 at 06bece5). lean-2 and lean-3 moved and
  deleted without dropping a trace: their commits name each refusal kept.
- **#57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97, #99, #103,
  #104 still open**, as recorded; #42, #48, #63 said why not in lean-4 (a).

### (3) Duplicate type names (`code converge noun`)

`git diff 8b508d0 06bece5 -- crates apps web bridge`, added lines matching
`\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: `struct OnStream`, `enum
Probe`, `enum Other` (and `type Error = LendError`, an associated type).
All three are lean-2's moves: OnStream from iroh.rs:520 to
iroh/tests.rs:10, Probe from oplog.rs:492 and Other from oplog.rs:546 to
oplog_tests.rs:34 and :88 (`git grep` at each end). `code converge noun
<Name> --corpus-id leanplant` for each: "definitions (0), first-party
production … the name is defined only in test code". The SCIP graph has
them (OnStream iroh/tests.rs:9; Probe oplog_tests.rs:33; Other
oplog_tests.rs:87 and svrngs/tests/new.rs:458, two test-local enums in
two crates' tests). As a positive control, `converge noun Said` gives
"definitions (2)": cli.rs:662 and door/choices.rs:39, a pair `converge
census --local` sets aside (neither reached from another crate) and
REVIEW-audit-e0's census already named. **No noun added in the range has
more than one production definition.**

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 112 | 8 | crates/svrngs-node/src/surface.rs:230 and origin.rs:199, at 06bece5 | 0ca47dc | The stop's `/svrngs-stop` suffix built in one file and parsed in another, two literals. |
| 113 | 8, 3 | crates/svrngs-node/src/home/home.js:68–87 and web/app.js:228–270, at 06bece5 | not fixed | The two callers of question.js `uses` walk, fetch and fail alike; the "not lent" line spelled in each, "groups" and "group". |
| 110 | 8, 3 | home.js and web/app.js | closed (7b10b4a) | One wording in question.js; #113 the rest. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 2,822 lines left (12528 / 15350). |
| 31, 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 103, 104 | as recorded | as recorded | not fixed | Untouched in the range but by moves that kept them. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I9 | 6, 7 | `code dry-report` near tier | Still present on b604bcb6…: `plant_exact` 1314–1332 for 1316–1328, `remove_counted` 502–520 for 504–516; a verbatim Rust copy and its original in no cluster even at 0.8. |
| I19 | 6 | dry-report's exact tier | Still present, and said: web/ and question.js "could-not-judge … no SCIP function symbols in scope". |
| I24 | 6, 7 | `code dry-report` exact tier | **Still present on b604bcb6…**: plant R, `remove_counted` verbatim as `plant_exact`, both in SCIP (`symbols`: door.rs 504–516, 1316–1328, kind `unknown`, as 9,543 of the graph's rust symbols are), in no exact group. The window pass caught it. |
| I27 | 6 | dry-report's exact tier over mixed languages | Still present: apps/ scopes report "**0** exact-clone groups", their JS unjudged. |

## REVIEW-audit-folders-auto-1 (2026-10-09, range 2bf0e1b..b2a76db)

No `REVIEW-audit-` row precedes this one in its queue
(ralph/next/folders/STATE.md), so the range starts where the queue did:
2bf0e1b, the commit that made it, on the branch `folders` cut from main
at b482e08. It ends at b2a76db, the commit that made this row: 18 commits
(`git rev-list --count 2bf0e1b..b2a76db`), gf-0 (e7715bd), gf-1 (bbf00e1,
88c4362, 33bb0b7, 1490f61, aee58ae, 362a6bd, 75e34ae, 0360181, 575a911,
e49a006, dfd00c3), gf-2 (71490c4, 8251050), each unit's `done` and this
row's (`git log --reverse --format='%h %s' 2bf0e1b..b2a76db`). Product
code changed in gf-1 alone. This audit fixes one finding (#114), records
three (#115-#117), and grows #31 and #103. These were #108-#111 on
`folders`, numbered past main's #105-#107; main went on to use #108-#111
itself (REVIEW-audit-sup-auto-2), so syp-3's merge renumbered this
branch's #108-#116 as #112-#120, after main's last; main then used #112
and #113 (REVIEW-audit-lean-auto-1), so syp-5's merge renumbered them
again as #114-#122, each row of a table below saying its numbers on
`folders`.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 c9b653d3…
(`shasum -a 256`, built 2026-10-09 12:03), **the binary main's
REVIEW-audit-sup-auto-1 planted against the same day**, so its tiers'
readings (I24, I9) stand and were not planted again. The embedder was
`llama-server` on the corpus's model (qwen-embedding-0.6b.gguf,
~/.svrnmesh/config.toml:18) on :8080, `-b 8192 -ub 8192` (I21), for this
session. The worktree is its own corpus, `svrngs-folders`, so main's
`svrngs` was not touched: `code index . --corpus-id svrngs-folders
--full` at b2a76db, tree clean, 2726 chunks in 358 s
(/tmp/gfaudit1/index-full.out); `project refresh` there (the corpus named
after the directory, I25, the same name): rust 10707 symbols, python
1274, "typescript: skipped (export failed)" (I4). The scripts are
main's sup audit's, copied to /tmp/gfaudit1: ledger.py with gf's unit
names, windows.py, clusters.py.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' 2bf0e1b..b2a76db`,
summed by /tmp/gfaudit1/ledger.py: a commit counts toward the unit its
subject names (`gf-1` is gf-1-build-run), else the next `done`'s; product
code is crates/, apps/, web/, bridge/; a test is a `tests/` or `check/`
segment or a lean_check test file name.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| gf-0-rule | 0 | 0 | 0 | 0 | 0 | 0 |
| gf-1-build-run | 2129 | 134 | 1995 | 444 | 9 | 435 |
| gf-2-verdict | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 2129 | 134 | 1995 | 444 | 9 | 435 |

- gf-1 src, crates: svrngs-core folder.rs +144, lib.rs +20/−2, admit.rs
  +13/−1, alpn.rs +7/−1, membership.rs +1/−1 (bbf00e1); svrngs-node
  folder.rs +572 (88c4362) and +83/−104 (33bb0b7, the trim), node.rs
  +13/−3, iroh.rs +9, Cargo.toml +5, lib.rs +1 (88c4362); svrngs cli.rs
  +50/−1, Cargo.toml +3/−2 (1490f61). Crates: +921/−115.
- gf-1 src, apps/jellyfin/folders/ (not counted, scripts/lean_check.py:49):
  driver.cjs +232, jf_setup.py +118, rangeserver.py +116, the first run's,
  unchanged (362a6bd); analyze.py +387, run.sh +241, probe.py +41, strm.py
  +39, and the forks' changes +34/−19 (75e34ae). Apps: +1208/−19.
- gf-1 tests: svrngs-node tests/folders.rs +272 (aee58ae), +36/−9
  (575a911, rustfmt); svrngs-core folder_tests.rs +80 (bbf00e1); svrngs
  tests/share.rs +56 (1490f61).
- gf-0 and gf-2 wrote demo/ and docs/ alone.

lean_check on `git archive` trees in /tmp/gfaudit1 (apps/ and tests not
counted):

| at | svrngs-core | repository |
|---|---:|---:|
| 2bf0e1b, range start (b482e08's code) | 2635 / 3000 | 14529 / 15200 |
| f2f6945, gf-1 done | 2763 / 3000 | 15184 / 15200 |
| b2a76db, range end | 2763 / 3000 | 15184 / 15200 |
| b8d2b97, this audit's fix (the tree) | 2741 / 3000 | **15163 / 15200** |

gf-1 spent 128 of the core's and 655 of the repository's, against gf-0's
estimate of about 430 (its README §7 counts them by part); #114 gives
back 21. **37 are left** on this branch (#80). Main's own count has
moved since the cut (its 30d975a raised the budget to 15,350 for sup-3);
the merge's budget is the operator's, as the queue's fence 6 says.

### (2) Clones (`code dry-report`)

**Scoped**, at 0.95, for each dir the range touched
(/tmp/gfaudit1/clean-*_0.95.md):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-core/ | 407 | 7 (as at main's sup audit: `is_empty` ×3, `as_str` ×3, `key` (tests), `len`, `signed_by_link`, `lines`, `fmt`) | 0 | 0 |
| crates/svrngs-node/ | 424 | 5 (`as_str` ×3, `addr`, `fmt`, `try_from`, `drop`) | 1 (declared.rs `harden_file` ×2, as before) | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| crates/svrngs/ | 67 | 0 | 0 | 0 |
| crates/svrngs/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| apps/jellyfin/folders/ | 36 | 0 | 0 | 0 |
| apps/ | 275 | 0 | 5 (#99's three, tv.mjs and judge.py's, as before) | 0 |

**Unscoped**: 0.95 (14 exact, 18 near), 0.9 (14, 41), 0.8 (14, 239).
/tmp/gfaudit1/clusters.py blamed every side (`git blame -w` at b2a76db)
for a line in 2bf0e1b..b2a76db: **0** groups at 0.95, **0** at 0.9, 11
at 0.8 (/tmp/gfaudit1/clusters.out). converge and dry-report give lines
one early (I1), corrected here:

| at | cluster | cosine | read |
|---|---|---|---|
| 0.8 | svrngs-core folder.rs `FileHash::parse` :24-28, `hex` :30-32 ~ hash.rs `ContentHash::from_hex` :81-84, op.rs `from_hex` :89-91; folder.rs `serialize`/`deserialize` :35-46 ~ hash.rs `serialize` :110-114 | 0.805-0.847 | **#114**: `FileHash` was a second BLAKE3-256 type beside the core's one decider. Fixed (b8d2b97). op.rs's is `OpId`'s, another value. |
| 0.8 | folder.rs `ShareFolder::decide` :75-86 ~ record.rs, register.rs, tests_support.rs `decide`, command.rs `confirm` | 0.812-0.948 | Each `Command` its own act; the trait's shape. No change. |
| 0.8 | folder.rs `folders` :114-131 ~ `holders` :128-142 | 0.883 | Neighbours (I9); `holders` calls `folders`. #116 is about `holders`, not this pair. |
| 0.8 | membership.rs `moves_membership` :370-384 among 40 symbols | 0.801-0.928 | One line in range, `Folder` added to its `false` arm. No change. |
| 0.8 | svrngs-node folder.rs `router` ~ `list` ~ `ask` :516-551 | 0.808-0.832 | Neighbours, one route's two handlers. No change. |
| 0.8 | folder_tests.rs `file` ~ `folder` ~ the first test | 0.859-0.874 | Neighbours. No change. |
| 0.8 | apps/jellyfin/folders/ driver.cjs `quiet` ~ `login`, `ffmpegInContainer` ~ `rangeLogLines`; analyze.py `event` ~ `window` ~ `result`, `union` ~ `overlap` | 0.815-0.862 | Neighbours (I9), the run's instruments. No change. |

**Window pass** (/tmp/gfaudit1/windows.py . 2bf0e1b..b2a76db, 5
normalized lines over crates/, web/, apps/ in .rs, .js, .mjs, .py, tests
included): 307 window hits in 111 spans, **1 new**; with .cjs and .sh
added (windows-cjs.py, since the range's driver and run script are
those), 309 in 113, **3 new**:

- tests/folders.rs:17-22 == tests/surface.rs:17-22: the imports and
  `signing`, then `seed_file` (folders.rs:21-31, surface.rs:21-34). Two
  test binaries: **#31's case, grown**. `signing` is now in 11 binaries
  (`grep -rn "fn signing(" crates/*/tests`), `seed_file` in 2; folders.rs
  `Group` (:50) is conformance.rs `Group`'s (:130) shape, three nodes
  each, the third. Not moved, for #31's reason.
- apps/jellyfin/folders/run.sh:177-181 == :193-197: the byte flipped and
  flipped back, the same inline Python twice. The run's script, whose data
  are at 575a911 (e49a006); a change to it is proven only by the run
  again. No change here.
- run.sh:138-142 == apps/jellyfin/check/check.sh:118-122 ==
  apps/movie-night/check/check.sh:102-106: Jellyfin's network.xml, its
  loopback lines, each run's Jellyfin of its own (the queue's fence 3).
  Three runs' set-ups, #99's kind; no change.

Reading the range for what neither pass would show:

- **#114, fixed.** `FileHash` (svrngs-core folder.rs:18-46 at b2a76db)
  re-made `ContentHash` (hash.rs:44): one algorithm, one wire form,
  a second parse, serde and hex. iroh-blobs names a raw blob by the
  BLAKE3 of its bytes, so the record's file hash is a `ContentHash`.
- **#115, not fixed.** `Folders::all` (svrngs-node folder.rs:137-144)
  reads groups that cannot be listed, and a group whose admission fails,
  as none (`namespaces().unwrap_or_default()`, `admission(ns).ok()`), with
  no trace. Every reader inherits it: `/v1/folders` (`list`, :526-531)
  answers "no folders", the endpoint (`bytes`, :385, :394-403) answers 404
  `x-svrngs-refused: not-shared`, and the provider (`shares`, :219-224)
  refuses "no standing folder of this computer's names the hash" where
  svrngs could not judge: #103's case, and the queue's own fence ("a
  refusal is never an empty answer"). Not fixed: each answer changes, and
  no test plants an unlistable rail. For the production rows (the
  verdict's candidates 2 and 3 rewrite these readers).
- **#116, not fixed.** Who holds a hash is decided twice: `Admission::
  holders` (svrngs-core folder.rs:103-111), read only by its test, and
  the endpoint's own list across groups (svrngs-node folder.rs:404-408),
  which keeps a key once per group: Ana sharing in two groups is asked
  twice for a piece she did not answer, and each ask waits
  `ANSWER_WITHIN`. The verdict names the fold (its §9, candidate 2);
  the duplicate ask is this audit's. Not fixed: folding them changes who
  is asked.
- **#117, not fixed.** `svrngs share --folder` (svrngs cli.rs:548-563)
  reads the node's API file with `unwrap_or_default()` (:549), so an
  unreadable file and an absent one both say `NotRunning`, the first
  untrue; and the answer's body with `unwrap_or_default()` (:560), so a
  refusal whose body could not be read is "the folder was refused: ",
  empty. Small, but each changes what the command says.
- **Glassbox.** The new branches trace: `share`'s link skipped and each
  file taken (svrngs-node folder.rs:165, :175), `write`'s each namespace
  (:206, :209), `hold`'s each holder (:268, :281), the provider's every
  request (:341, :345, :360), `bytes`'s refusals (:395, :420, :455) and
  each answer (`Served`, :513), the core's `folders` (svrngs-core
  folder.rs:97-98), the CLI's ask (cli.rs:556,
  :561). Two do not, recorded with their fix's home: the 416 for a range
  outside the file (svrngs-node folder.rs:410-416), and `all`'s skipped
  groups (#115).
- **Nouns of one program each.** `Plant` is folder.rs:48's and
  apps/jellyfin/mw/src/main.rs:38's, each its program's own planted
  failures (converge sees the first alone); the spike's drops in
  production (verdict §9). The 60 s after which an unanswered holder is
  asked first again (folder.rs:253) has one home, unnamed. No change.
- **#57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97, #99, #100,
  #104 still open**, none touched in the range (`git diff --stat 2bf0e1b
  b2a76db -- web apps/movie-night apps/taste apps/jellyfin/check
  apps/jellyfin/mw` is empty).

### (3) Duplicate type names (`code converge noun`)

`git diff 2bf0e1b b2a76db -- crates apps web bridge`, added lines
matching `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: `FileHash`,
`SharedFile`, `SharedFolder`, `ShareFolder` (svrngs-core); `Folders`,
`FolderError`, `FolderAsk`, `Plant`, `Served` (svrngs-node); `Group`
(tests/folders.rs); `H` (apps/jellyfin/folders/rangeserver.py); and
`type Refusal = Infallible`, the `Command` trait's associated type, no
noun. `converge noun <Name> --corpus-id svrngs-folders`
(/tmp/gfaudit1/nouns.out), beside `git grep -nP
"\b(struct|enum|trait|type|class)\s+<Name>\b" -- crates apps web bridge`:

| noun | converge | git grep | read |
|---|---|---|---|
| Plant | 1: svrngs-node folder.rs:47 | 2: folder.rs:48, apps/jellyfin/mw/src/main.rs:38 | Two programs' planted failures, each its own closed set. No change. |
| Group | 0: "defined only in test code" | 3: tests/folders.rs:50, tests/conformance.rs:130, web/src/join.rs:50 | Two test fixtures, three nodes each (#31), and the join page's group, another concept. No change. |
| FileHash | 1: svrngs-core folder.rs:19 | 1: folder.rs:20 | One name, but the concept of `ContentHash` (hash.rs:44): **#114**, by reading, which converge cannot see. Fixed. |

Every other noun: one definition (converge and git grep agree, converge
one line early, I1).

### Fixed here

- **#114** (b8d2b97). `SharedFile::hash` is a `ContentHash`; `FileHash`
  deleted. The record's spelling is unchanged (folder_tests.rs
  `a_hash_is_spelled_as_hex_and_nothing_else_parses`, its type swapped,
  passes); `ContentHash::from_hex` trims (hash.rs:82), the one difference,
  for input svrngs never writes. `cargo test -p svrngs-core`: 138 passed;
  `-p svrngs-node --lib --test folders`: 90, 3; `-p svrngs --test
  share`: 5. Counted −21. The verdict's §9 citations into both folder.rs
  files, and its budget (with DESIGN §8 and §9's row, about 800 to about
  780), moved with it.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 114 (was #112 on `folders`; #108 before syp-3's merge) | 8 | crates/svrngs-core/src/folder.rs:18-46 and crates/svrngs-core/src/hash.rs:44-126, at b2a76db | b8d2b97 | A shared file's hash was a second BLAKE3-256 type, `FileHash`, beside the core's `ContentHash`. |
| 115 (was #113 on `folders`; #109 before syp-3's merge) | 6, 1 | crates/svrngs-node/src/folder.rs:137-144, read at :219-224, :385 and :526-531, at b8d2b97 | not fixed | Groups that cannot be listed, or admitted, read as no folders: the list says none, the endpoint and the provider refuse "not shared", untraced. |
| 116 (was #114 on `folders`; #110 before syp-3's merge) | 8 | crates/svrngs-core/src/folder.rs:103-111 and crates/svrngs-node/src/folder.rs:404-408, at b8d2b97 | not fixed (the verdict's candidate 2) | Who holds a hash decided twice, the core's unused; the endpoint's list repeats a holder per group, so an unanswering holder is waited on once per group. |
| 117 (was #115 on `folders`; #111 before syp-3's merge) | 6 | crates/svrngs/src/cli.rs:549 and :560, at b8d2b97 | not fixed | `share --folder` reads an unreadable API file as "not running" and an unreadable refusal as an empty one. |
| 118 (was #116 on `folders`; #112 before syp-3's merge) | 1 | crates/svrngs-node/src/folder.rs:410-416, at b8d2b97 | not fixed (the verdict's candidate 3 rewrites the endpoint) | A range outside the file is answered 416 with no trace. |
| 103 | 6 | as recorded; and crates/svrngs-node/src/folder.rs:137-144 (#115) | grown | The same absence read as none, in the folders' reader. |
| 31 | 8 | crates/svrngs-node/tests/folders.rs:17-31, :50 and tests/surface.rs:17-34, tests/conformance.rs:130 | not fixed | Grown by gf-1: `signing` (11 binaries), `seed_file` (2), a three-node `Group` (2). |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 37 lines left on this branch (15163 / 15200); the merge's raise is the operator's. |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 104 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, `dry-report` spans | Still present: `FileHash` :19 for :20, `Plant` :47 for :48, `parse` :21-31 for :24-28. |
| I4 | 6 | `project refresh` | Still present: "typescript: skipped (export failed)". |
| I9 | 6, 7 | `code dry-report` near tier | Still present: each new 0.8 cluster but #114's is a function and its neighbour. |
| I19 | 6 | dry-report, `tests` scopes | Still present, and said: "could-not-judge"; the window pass found #31's growth. |
| I24 | 6, 7 | `code dry-report` exact tier | The same binary as main's sup audit's plant, which it missed; its zeros in Rust are weak evidence. The window pass ran over every file. #114 was found by reading; the 0.8 tier paired it after. |

The row's check: `scripts/check.sh` at b764f8a, tree clean, all 14 steps
passed (principles pinned through app laws; the ceiling canary 0.0331 s
of processor against 0.1 s; in the conformance table wildcard's WebKit
seeks back to back "failed, a rate" once, and in the rate's table
computer/bo straight at the canary 5738 ms and wildcard/sam through the
lend 23566 ms "failed", none failing its step, u5-11), exit 0
(/tmp/gfaudit1/check.out).

## REVIEW-audit-sync-auto-1 (2026-10-10, range e4e5437..8171ed6)

No `REVIEW-audit-` row precedes this one in its queue
(ralph/next/sync/STATE.md), so the range starts where the queue did:
e4e5437, the commit that made it (the budget's raise, e8042bd, sits just
before and touches scripts/lean_check.py alone). It ends at 8171ed6, the
commit that made this row: 14 commits (`git rev-list --count
e4e5437..8171ed6`), sy-1 (67abfa3, 6a260fc, e7d0ca9, db706d3, 162d298,
ea0f3a5, e3b3df9, 4926df0), main's chk-1 cherry-picked (a9659e1, from
e31bdce; main's REVIEW-audit-sup-auto-2, b841412, audits it there), sy-2
(09793b5, 1b5706e), each unit's `done` and this row's. Product code
changed in sy-1 and, in a test file, chk-1. This audit fixes one finding
by a move (6982059, #119's reading given one home), records four
(#119-#122), and grows #115 and #116: on `folders`, #117-#120 and #113,
#114 from syp-3's merge, and #113-#116 and #109, #110 before it,
renumbered by syp-3's and syp-5's merges (REVIEW-audit-folders-auto-1's
opening says how).

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli — **a new binary**, sha256
7dfd54da… (`shasum -a 256`, built 2026-10-09 23:19), not c9b653d3…, the
one planted before, so its exact tier was planted again here (I24 below).
The embedder was `llama-server` on the corpus's model
(qwen-embedding-0.6b.gguf, ~/.svrnmesh/config.toml:18) on :8080, `-b 8192
-ub 8192`, for this session. The corpus is the worktree's own,
`svrngs-folders`: `code index . --corpus-id svrngs-folders`, incremental
from b2a76db, 38 files changed, 414 chunks embedded in 42 s
(/tmp/syaudit1/index.out); `project refresh`: rust 11039 symbols, python
1462, "typescript: skipped (export failed)" (I4). The scripts are the
folders audit's, copied to /tmp/syaudit1: ledger.py with sy's unit names
and chk-1's, clusters.py, and windows-sy.py (windows.py over crates, web,
apps, demo and scripts, .cjs and .sh included).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' e4e5437..8171ed6`,
summed by /tmp/syaudit1/ledger.py (/tmp/syaudit1/ledger.out): a commit
counts toward the unit its subject names, else the next `done`'s; product
code is crates/, apps/, web/, bridge/; a test is a `tests/` or `check/`
segment or a lean_check test file name.

| unit | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|
| sy-1-build-run | 510 | 46 | 464 | 386 | 10 | 376 |
| chk-1 (main's, cherry-picked) | 0 | 0 | 0 | 101 | 12 | 89 |
| sy-2-verdict | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 510 | 46 | 464 | 487 | 22 | 465 |

- sy-1 src: svrngs-core synced.rs +156 (6a260fc), +1/−1 (db706d3), lib.rs
  +16/−2, admit.rs +19/−9, membership.rs +4/−1 (6a260fc); svrngs-node
  folder.rs +58/−27 (e7d0ca9, the move) and +27/−4, synced.rs +216, node.rs
  +4 (db706d3); svrngs cli.rs +9/−2 (db706d3).
- sy-1 tests: svrngs-core synced_tests.rs +130, folder_tests.rs +1/−1
  (6a260fc); svrngs-node tests/folders.rs +120, tests/sy_dee.rs +118
  (db706d3) and +17/−9 (162d298).
- chk-1: tests/conformance.rs +101/−12 (a9659e1).
- Outside product code but inside the budget (scripts included):
  demo/runs/sy/2026-10-09/run.py, 180 counted; scripts/check.sh +1.

lean_check on `git archive` trees in /tmp/syaudit1 (by file,
/tmp/syaudit1/bylean.out): svrngs-core synced.rs +111, admit.rs +9,
lib.rs +9, membership.rs +3; svrngs-node synced.rs +180, folder.rs +40,
node.rs +4; svrngs cli.rs +7; run.py +180; check.sh +1: **+544**.

| at | svrngs-core | repository |
|---|---:|---:|
| e4e5437, range start | 2741 / 3000 | 15163 / 15750 |
| 8bf279d, sy-1 done | 2873 / 3000 | 15707 / 15750 |
| 8171ed6, range end | 2873 / 3000 | 15707 / 15750 |
| 6982059, this audit's move (the tree) | 2873 / 3000 | **15704 / 15750** |

sy-1 spent 132 of the core's and 544 of the repository's; **46 are left**
on this branch (#80).

### (2) Clones (`code dry-report`)

**Scoped**, at 0.95, for each dir the range touched
(/tmp/syaudit1/clean-*0.95.md):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---|---:|---|
| crates/svrngs-core/ | 418 | 7, as at the folders audit | 0 | 0 |
| crates/svrngs-node/ | 432 | 5, as at the folders audit | 1 (declared.rs `harden_file` ×2) | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — |
| crates/svrngs/ | 67 | 0 | 0 | 0 |
| demo/runs/sy/ | 20 | 0 | 0 | 0 |

**Unscoped**: 0.95 (14 exact, 18 near), 0.9 (14, 42), 0.8 (14, 246).
clusters.py blamed every side (`git blame -w` at 8171ed6) for a line in
e4e5437..8171ed6 (/tmp/syaudit1/clusters.out): **0** groups at 0.95,
**1** at 0.9, 9 at 0.8. Every one with a side added in the range:

| at | cluster | cosine | read |
|---|---|---|---|
| 0.9, 0.8 | svrngs-core synced_tests.rs `two_edits` :69-90 ~ `two_edits_of_one_version_are_both_kept_…` :87-107 (at 0.8 with `an_add_then_an_edit_…` :44-71, `a_delete_beside_an_edit_…` :104-115, `deleting_the_conflict_s_copy_…` :112-126) | 0.843-0.908 | A helper and the tests that call it, each a different history; the shape of a test of the layout. No change. |
| 0.8 | svrngs-core synced.rs `SyncFolder::decide` :62-73 ~ folder.rs, record.rs, register.rs, tests_support.rs `decide`, command.rs `confirm` | 0.803-0.948 | Each `Command` its own act; the trait's shape, as at the folders audit. No change. |
| 0.8 | svrngs-node synced.rs `step` :77-142 ~ `apply` :142-216 | 0.876 | Neighbours (I9), one calling the other. No change. |
| 0.8 | svrngs-node synced.rs `both_ways` :50-68 ~ `placed` :65-81 | 0.868 | Neighbours (I9). No change — but `placed`'s body was `all`'s (folder.rs:153-159), which neither tier paired: fixed by a move, 6982059. |
| 0.8 | svrngs-node folder.rs `router` ~ `list` ~ `ask` :591-606 | 0.823-0.847 | One route's handlers; `ask` grew the `BothWays` arm. No change. |
| 0.8 | synced_tests.rs `synced` :17-32 ~ `placed` :29-43; `now` :5-13 ~ `entry` :9-21 | 0.826-0.833 | Neighbouring test fixtures. No change. |
| 0.8 | demo/runs/sy/2026-10-09/run.py `s1` :88-104 ~ `s2` :101-116 ~ `s3` :113-131; `holds` :69-77 ~ `write` :74-83 | 0.810-0.851 | The run's bars, each its own scenario, and neighbours. Its data are at e3b3df9; a change is proven only by the run again. No change. |

dry-report gives lines one early (I1); the spans above are as it printed
them.

**Window pass** (/tmp/syaudit1/windows-sy.py . e4e5437..8171ed6,
/tmp/syaudit1/windows-sy.out): 567 window hits in 139 spans, **2 new**:

- svrngs-core folder.rs:8-16 == synced.rs:13-21: the two modules' `use`
  lines. No change.
- tests/folders.rs:333-337 == :370-374: two tests' set-up, a group and
  two synced directories. No change.

**The instrument, planted** (this binary is new): a copy of
svrngs-core synced.rs `beside` (11 lines) as `pub fn planted_beside`,
uncommitted; `code index` (3 files, 7 chunks), `project refresh` (rust
11047), `dry-report --scope crates/svrngs-core/`
(/tmp/syaudit1/plant-core.md): 1611 symbols, 419 locations (one more
than unplanted, so the copy was considered), **7 exact groups, 0 near —
the plant in neither**. Reverted and reindexed (`git checkout`, index
1 file, refresh rust 11038). The exact tier's zeros are no evidence here
(I24); the window pass and reading carry the clone question.

Reading the range for what neither pass would show:

- **`placed` re-wrote `all`, fixed by a move (6982059).** sy-1's `placed`
  (svrngs-node synced.rs:68-72 at 8171ed6) read every group's admission
  as `all` does (folder.rs:153-159): every namespace, an unlistable rail
  read as none, an admission that fails skipped. Both call
  `Folders::admissions` (folder.rs:160-165 at 6982059). Behaviour kept:
  `cargo test -p svrngs-node --test folders --test sy_dee`, 5 passed, 1
  ignored (the spike's instrument). −3 lines.
- **#119, not fixed.** That reading is now the synced directory's, where
  an absence deletes: if the rail cannot list its groups, or a group's
  admission fails, `placed` (synced.rs:68) holds none of that group's
  files, and `apply` deletes every file it last wrote or took that is
  unchanged on disk, "the record placing nothing there" (synced.rs:162-165
  at 6982059), untraced as to why. No delete is sent — the entry leaves
  `held`, so the next step finds no file and no held entry — and the
  files come back when the rail reads again and a member serves them.
  Read, not run. #115's case, grown from a wrong refusal to a member's
  files taken off their disk; the queue's fence ("a member that did not
  answer is never an empty folder"). Not in the verdict's candidates
  (demo/runs/sy/2026-10-09/README.md §9; its 9 names the kept state, not
  the record). Not fixed: the apply's answer changes.
- **#120, not fixed.** The run's plant variable, `SVRNGS_GF_PLANT`, has
  two readers: folder.rs:109-117 parses it into `Plant` at start, and
  node.rs:756 reads it again, by string, on every dial, for `open-gate`,
  which `Plant` does not name — while folder.rs:51 says it is "read once
  at start". Principle 8. The plants drop in production (verdict §9's
  line table), and this with them; recorded so the drop takes both.
- **#121, not fixed.** `both_ways` (synced.rs:53-65) spawns a loop that
  nothing owns or can stop: asked twice for one directory (or for two
  directories of one name), two loops step at once over one
  `synced-<name>.json` (:77) and one folder, each reading and rewriting
  it. RUST_ARCH's ownership tree; the verdict's candidate 9 keeps the
  directories in the data dir, and is where one owner per directory
  belongs. Read, not run.
- **#122, not fixed.** The apply's skips do not trace: a held file the
  record no longer places but changed on disk, kept (synced.rs:162, the
  `else` of the delete); a placed file not written because the disk
  changed since it was last taken (:181); a file written whose mtime
  cannot be read, left unheld (:206-208); and the step's same-bytes touch
  (:96). Each is a decision ("never touching a file changed on disk",
  the module's doc, :10-11) that AGENTS.md 1 wants at debug. Small, but
  the trace's text is the row's to write with candidate 9's base.
- **#116, grown.** `apply` builds a third list of who may hold a file
  (synced.rs:153-158): every standing member of every group held here,
  beside the endpoint's (folder.rs) and the core's unused
  `Admission::holders`. The verdict's §8 first part and candidate 2 name
  the many-groups half; the three homes are this audit's.
- **The provider pays for a layout per request.** `shares` (folder.rs:270)
  builds every group's synced layout on each request, before the shared
  folders' answer is known, and `layout` traces at debug each time. Not a
  principle; for candidate 2's rewrite of `shares`. No finding.
- **Glassbox otherwise.** The new branches trace: the core's `layout`
  (svrngs-core synced.rs:148), the step's failure (svrngs-node
  synced.rs:59), each change sent (:129), each delete (:164), each
  write and its two failures (:196, :201, :204), the gate's plant
  (node.rs:757).
- **#57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97, #99, #100,
  #104, #117, #118 still open**, none touched in the range (`git diff
  --stat e4e5437 8171ed6 -- web apps crates/svrngs/src/cli.rs` touches
  cli.rs's `--both-ways` alone, not #117's lines).

### (3) Duplicate type names (`code converge noun`)

`git diff e4e5437 8171ed6 -- crates apps web bridge demo scripts`, added
lines matching `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`: `Entry`,
`Change`, `Placed`, `SyncFolder`, `Written` (a type alias), svrngs-core;
`Held`, svrngs-node; and `type Refusal = Infallible`, the `Command`
trait's, no noun. `converge noun <Name> --corpus-id svrngs-folders`
(/tmp/syaudit1/nouns.out), beside `git grep -nP` for each:

| noun | converge | git grep | read |
|---|---|---|---|
| Held | 3: svrngs-node surface.rs:72, synced.rs:31, svrngs-runtime journal.rs:80 | 3: surface.rs:73, synced.rs:32, journal.rs:81 | Three private types, three concepts: a group as `/v1/groups` lists it, the journal's cache of what it folded, what a synced path was last taken or written as. One name, no shared meaning; nothing to fold. No change. |

Every other noun: one definition (converge and git grep agree, converge
one line early, I1).

### Fixed here

- **#119's reading, one home** (6982059). `all` and `placed` call
  `Folders::admissions`; behaviour kept, as above. #119 itself, the
  delete, stays open.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 119 (was #117 on `folders`; #113 before syp-3's merge) | 6, 1 | crates/svrngs-node/src/folder.rs:160-165 (`admissions`) and crates/svrngs-node/src/synced.rs:68, :162-165, at 6982059 | not fixed (the reading folded, 6982059) | A group that cannot be listed or admitted reads as placing nothing, and the synced directory then deletes every file it held there that is unchanged on disk, untraced as to why. |
| 120 (was #118 on `folders`; #114 before syp-3's merge) | 8 | crates/svrngs-node/src/folder.rs:109-117 and crates/svrngs-node/src/node.rs:756, at 6982059 | not fixed (drops with the plants) | The plant variable read twice, the second time on every dial and for a value `Plant` does not name, against folder.rs:51's "read once at start". |
| 121 (was #119 on `folders`; #115 before syp-3's merge) | 10 (RUST_ARCH ownership) | crates/svrngs-node/src/synced.rs:53-65 and :77, at 6982059 | not fixed (the verdict's candidate 9) | `both_ways` spawns an unowned, unstoppable loop per ask; two asks for one name step at once over one kept state. |
| 122 (was #120 on `folders`; #116 before syp-3's merge) | 1 | crates/svrngs-node/src/synced.rs:96, :162, :181, :206-208, at 6982059 | not fixed | The apply's skips — a changed file kept, a placed file not written, an mtime unread — and the step's same-bytes touch are untraced. |
| 116 (was #114 on `folders`; #110 before syp-3's merge) | 8 | as recorded; and crates/svrngs-node/src/synced.rs:153-158, at 6982059 | grown (the verdict's candidate 2) | A third list of who holds a file: every standing member of every group. |
| 115 (was #113 on `folders`; #109 before syp-3's merge) | 6, 1 | as recorded; folder.rs:160-165 at 6982059 | grown into #119 | The same absence, now one helper's, read by the synced directory too. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 46 lines left on this branch (15704 / 15750). |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 104, 115, 116 | as recorded | as recorded | not fixed | Untouched in the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, `dry-report` spans | Still present: each noun one line early (`Held` :31 for :32, `Entry` :24 for :25). |
| I4 | 6 | `project refresh` | Still present: "typescript: skipped (export failed)". |
| I9 | 6, 7 | `code dry-report` near tier | Still present: of the 9 clusters with a side in the range, all are neighbours, a trait's shape or tests of one shape; the one real re-write (`placed` ~ `all`) was not paired at 0.8. |
| I19 | 6 | dry-report, `tests` scopes | Still present, and said: "could-not-judge". |
| I24 | 6, 7 | `code dry-report` exact and near tiers | **Present in the new binary (7dfd54da…)**: an 11-line byte-identical copy of `beside`, considered (419 locations against 418), found by neither tier at 0.95. |

## REVIEW-audit-syp-auto-1 (2026-10-11, range 7442463..baa3061)

No `REVIEW-audit-` row precedes this one in its queue
(ralph/next/syp/STATE.md), so the range starts where the queue did:
7442463, the commit that made it. It ends at baa3061, the commit that
made this row. `git rev-list --count 7442463..baa3061` is 183, of which
82 are main's, carried by syp-3's merge (fb98c92, main at 265ffa9) and
syp-5's (05eee01, main at 8966ac8) and audited on main by its own rows
(REVIEW-audit-sup-auto-2, REVIEW-audit-lean-auto-1). This audit reads
the other 101 (`git rev-list --count 7442463..baa3061 ^main`): 99
commits and the two merges. Units: syp-1, syp-2, syp-3, perf-1, syp-4,
perf-2, syp-5 and syp-6. The audit's own commits: dedc224 (a doc moved),
5d36346 and 35a9ef2 (test set-up and helpers folded), cf21f71 and
734018d (#122's skips traced).
It closes #115, #116, #118, #119, #120 and #121, all by the range's
units, and #122 by this audit's two commits. It records no new finding.
#117 stays open, at new lines.

As before, `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 b604bcb6…
(`shasum -a 256`, built 2026-10-10 14:10). Main's REVIEW-audit-lean-auto-1
already planted this binary (its I9, I24), and it was planted again here
(below). The embedder was `llama-server` on the corpus's model
(qwen-embedding-0.6b.gguf, ~/.svrnmesh/config.toml:18) on :8080, `-b 8192
-ub 8192`, for this session.

**The corpus is not the worktree's.** `code index . --corpus-id
svrngs-folders` (no `--full`) found 527 files changed since 6982059 and
rebuilt whole. Its promotion was then refused: "promote_single_shard:
cannot merge …/svrngs-folders-partition-local into …/svrngs-folders —
both hold corpus data named documents.lance". That left `svrngs-folders`
with no `chunks.lance` (dry-report: "no chunk index … run `svrn project
init` first") and the new index in `svrngs-folders-partition-local`
(I25). REVIEW-audit-sup-auto-1 repaired the same refusal with a `mv` by
hand, but this session's tool permissions refused the `mv`, so
`svrngs-folders` is left as described, for the operator: move its
`documents.lance` aside, then run `svrn code finalize svrngs-folders`.
The corpus read here is instead baa3061's tree, `git archive baa3061`
into /tmp/sypaudit1/tree-baa3061 (the corpus is named after its
directory, I25): `code index . --corpus-id tree-baa3061`, 3039 chunks in
364 s (/tmp/sypaudit1/index.out). `refresh` gave rust 12260 symbols,
"typescript: skipped (export failed)" and "python: skipped (export
failed)" (I4; /tmp/sypaudit1/refresh.out). The corpora `tree-baa3061`
and `tree-plant` stay in ~/.svrnmesh/indexes, about 23 MB each. The
scripts are the sync audit's, copied to /tmp/sypaudit1: ledger.py with
this queue's unit names and extra `git log` arguments, and clusters.py
and windows-sy.py with `^main` added to their `rev-list`, so that a line
blamed on main's commits is not counted as this range's.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
7442463..baa3061 ^main`, summed by /tmp/sypaudit1/ledger.py
(/tmp/sypaudit1/ledger.out). A commit counts toward the unit its
subject names, else toward the next `done`'s. Product code is crates/,
apps/, web/ and bridge/. A test is a `tests/` or `check/` segment, or a
lean_check test file name. The merges are not in this table; their
lines are in the count below.

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| syp-1-no-stream | 4 | 76 | 439 | −363 | 21 | 265 | −244 |
| syp-2-plants-out | 4 | 42 | 91 | −49 | 183 | 1 | 182 |
| syp-3-merge-main (adf104f; the merge apart) | 6 | 37 | 24 | 13 | 0 | 0 | 0 |
| perf-1-diagnose-and-propose | 12 | 76 | 12 | 64 | 185 | 0 | 185 |
| syp-4-production | 19 | 775 | 339 | 436 | 1213 | 216 | 997 |
| perf-2-make-it-fast | 37 | 576 | 194 | 382 | 486 | 80 | 406 |
| syp-5-rerun-and-merge (the merge apart) | 7 | 0 | 0 | 0 | 5 | 3 | 2 |
| syp-6-restart-and-retry | 9 | 112 | 26 | 86 | 197 | 26 | 171 |
| total | 98 + this row's | 1694 | 1125 | 569 | 2290 | 591 | 1699 |

- syp-1 src: 12d0ba2 alone, which removed streaming. svrngs-core
  folder.rs −116 (the file gone), admit.rs +8/−18, lib.rs +3/−16,
  membership.rs +1/−4; svrngs-node folder.rs +33/−264; cli.rs +17/−16.
- syp-2 src: 4867dcf, the plants out. folder.rs +7/−46, synced.rs
  +35/−41, node.rs −4.
- syp-4 src, the most of any unit: folder.rs +264/−96 (7102542, 2254580,
  db60756, ea9d95f, 7aaafed), synced.rs +344/−188 (9600be0's +71/−36
  reverted by 7a5dca4's +36/−71), core synced.rs +79/−8 (af50f20,
  `SyncPath`), cli.rs +53/−40.
- perf-2 src: synced.rs +302/−119, folder.rs +236/−73, svrngs-runtime
  lib.rs +35/−3 (6910f25, 7732812, the rail's teller).
- syp-6 src: synced.rs +66/−22 (4a79316, the retry), node.rs +34
  (c50d7f2, `asked_to_stop`), folder.rs +9 (31750ee, the dial's trace),
  cli.rs +1/−3.

The size budget, scripts/lean_check.py at HEAD run on a `git archive`
tree of each commit (/tmp/sypaudit1/lean-points.out). HEAD's rule leaves
`demo/runs/` apart (lean-1), so the early figures are not those the
units' own commits printed:

| at | svrngs-core | repository |
|---|---:|---:|
| 7442463, range start | 2873 / 3000 | 13673 / 15350 |
| 36bd119, syp-1 done | 2771 | 13366 |
| 8bf59cc, syp-2 done | 2771 | 13327 |
| fb98c92, syp-3's merge (main's side) | 2732 | 13375 |
| 607add5, syp-3 done | 2732 | 13383 |
| 619a1e4, perf-1 done | 2732 | 13440 |
| b16a2b9, syp-4 done | 2776 | 13730 |
| af109e9, perf-2 done | 2776 | 13982 |
| 05eee01, syp-5's merge (main's side) | 2719 | 13778 |
| 94fbea3, syp-6 done | 2719 | 13828 |
| baa3061, range end | 2719 | 13828 |
| 734018d, this audit's traces | 2719 | **13848 / 15350** |

The range nets +155 (/tmp/sypaudit1/bylean.out, by file): the branch's
units +311 (syp-1 −307, syp-2 −39, syp-3 +8, perf-1 +57, syp-4 +290,
perf-2 +252, syp-6 +50) and the merges −156 (+48, −204), main's lean
cuts. svrngs-node synced.rs +314 and core synced.rs +44 are this queue's
largest growths; core folder.rs −81 and svrngs-node folder.rs +4 net are
streaming's removal and the synced folder's growth in one file. **1,502
left** (#80).

### (2) Clones (`code dry-report`)

**Scoped**, at 0.95, for each crate dir the range's branch commits
touched (/tmp/sypaudit1/clean-*0.95.md):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 397 | 5 | 0 | 1: `as_str` (below) |
| crates/svrngs-node/ | 452 | 1 | 0 | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — (I19) |
| crates/svrngs/ | 69 | 0 | 0 | 0 |
| crates/svrngs/tests/ | 0 | **could-not-judge** | **could-not-judge** | — (I19) |
| crates/svrngs-runtime/ | 125 | 1 | 0 | 0 |

**Unscoped**: 0.95 (8 exact, 46 near), 0.9 (8, 73), 0.8 (8, 265).
clusters.py blamed every side (`git blame -w` at baa3061, this range's
branch commits only) (/tmp/sypaudit1/clusters.out). Groups with a side
in the range: **32** at 0.95, of which 31 pair perf-1's driver with
perf-2's; 35 at 0.9; 42 at 0.8. Every one in product code:

| at | cluster | cosine | read |
|---|---|---|---|
| exact | `as_str` core synced.rs:67-69 ~ link.rs:186-188 ~ svrngs-node lend.rs:91-93 | — | A newtype's `&self.0`, three types; RUST_ARCH's parse-don't-validate shape. No change. |
| 0.8 | svrngs-node synced.rs `kept_files` :336-353 ~ `kept_at` :350-363 ~ `held_everywhere` :361-376 | 0.851-0.912 | Already one: both call `kept_files`. Neighbours (I9). No change. |
| 0.8 | folder.rs `synced_groups` :363-375 ~ `read_at` :377-390 | 0.844 | One calls the other. No change. |
| 0.8 | folder.rs `start` :138-227 ~ `provider` :613-683 | 0.809 | The store's set-up and its provider's event loop. No change. |
| 0.8 | cli.rs `group_named` :1166-1189 ~ `new_group` :1142-1165; `new_link` :1185-1222 ~ `mint` :1226-1254 | 0.817-0.828 | Neighbours, each its own verb. Reading them found `new_link`'s doc above `group_named` (7102542 put it between them): moved back, dedc224. |
| 0.8 | core synced.rs `as_str`, `try_from`, `from` :64-85 | 0.818-0.844 | `SyncPath`'s conversions. No change. |
| 0.8 | node.rs `standings` (1 line in range), runtime lib.rs `ingest_all`, `ops_missing_from_within`, membership.rs `moves_membership` (1 line each) | 0.80-0.94 | A line each, in clusters of one shape (rail delegations, a trait's arms). No change. |
| 0.8 | synced_tests.rs `synced` ~ `placed`, `now` ~ `entry` (1 line each) | 0.820-0.834 | Test fixtures, as at the sync audit. No change. |

**Run drivers.** 31 of the 32 groups at 0.95 pair
demo/runs/perf/2026-10-10/run.py with
demo/runs/perf/2026-10-10-make-it-fast/run.py, most at cosine 1.000.
perf-2 copied perf-1's driver unchanged (e5a9847) and changed the copy
in later commits. The windows also pair syp-5's and syp-1's drivers
(no-stream ~ production ~ rerun `main`/`setup`). Each driver is the
code that ran beside its data, kept byte for byte (#61), and
`demo/runs/` is outside the budget (lean-1). No change.

**Window pass** (/tmp/sypaudit1/windows-sy.py . 7442463..baa3061,
/tmp/sypaudit1/windows-syp.out): 866 window hits in 165 spans, 23 new:
7 in the perf drivers, 2 in the syp drivers, 14 in tests, **0 in product
source**. Of the tests:

- **kept.rs:210-236 == :322-348 == :423-449**, three tests each starting
  Cy in the test's process with one 20-line block, and the group's seed
  built four times (member.rs:65-70 and three in kept.rs; member.rs:69-73
  == kept.rs:451-455). **Folded, 5d36346**: `cy_here`, `member::seed`,
  `member::hold_seed`, behaviour kept (`cargo test -p svrngs-node --test
  kept --test perf`: 3 passed, 4 ignored, as before). −42 test lines.
- **folders.rs:225-229 == :303-307**: syp-2's `get` (4f46785,
  folders.rs:212-233) wrote out again the body of sy-1's `ask_ben`
  closure (db706d3, :298-311), both in one file (35a9ef2's body says
  :211-232 and :297-310, one line early each); and **folders.rs:165-171
  == kept.rs:193-199**, `noise`, in two test binaries. **Folded,
  35a9ef2**: the test calls `get`, and `noise` lives in tests/common
  (`cargo test -p svrngs-node --test folders --test kept`: 19 passed,
  and 3 passed with 1 ignored, as before). −28 test lines. kept.rs's
  `get` (:183-187 == folders.rs:225-229) stays its own: it takes the
  holder as a member process's `Addr`, folders.rs's as a `Node`.
- No change to the rest: folders.rs:247-251 and ten more like it are
  the tests' two-directory set-up (`Group::new`, `both_ways` twice);
  :414-421 == :1229-1236 is two tests' log subscriber; :360-366 ==
  :699-705, :439-444 == :458-463 and :1295-1301 == :1324-1330 are an
  assertion's message, a `get` call and a wait; and kept.rs:28-34 ==
  perf.rs:40-46 is each binary's member-process entry.

**The instrument, planted** in /tmp/sypaudit1/tree-plant, a copy of
tree-baa3061 with two plants in svrngs-node synced.rs: a byte-identical
copy of `held_everywhere` (10 lines) as `planted_held_everywhere`, and
`kept_files` with its names changed as `planted_kept_files`. Steps:
`code index` 3042 chunks, `refresh` rust 12276 symbols, `dry-report
--scope crates/svrngs-node/` (/tmp/sypaudit1/plant-node.md). It
considered 454 locations against 452 unplanted, so both plants were
seen, and found **1 exact group, 0 near: neither plant**. At 0.8
(plant-node-0.8.md), both plants sit in one 5-symbol cluster with
`kept_files`, `kept_at` and `held_everywhere`, at cosine 0.831-0.920,
never 1.0, and no exact group pairs the verbatim copy. The same holds
for Python: the perf drivers' `phases` are byte-identical (`diff` of
perf/2026-10-10/run.py:186-215 against
2026-10-10-make-it-fast/run.py:208-237 prints nothing), and they are in
no exact group. I24 and I9 are still present, so the exact tier's zeros
are no evidence; the window pass and the reading carry the clone
question.

### Reading the range

- **#115, #119: closed by the range.** syp-1 removed the folder list and
  the endpoint (12d0ba2). syp-4 (a) (7102542) replaced `admissions` with
  `Host::standings`. A step now reads who stands and the layout before
  anything is sent or deleted, and stops by the group's name
  (svrngs-node synced.rs:389-391 at baa3061: "so a group that cannot be
  read stops the step by its name"). The provider serves nothing of a
  group it cannot read, said at warn (folder.rs:442-457, node.rs:456,
  :466). Held by `a_group_whose_record_cannot_be_read_is_said_so_never_read_as_none`
  (tests/folders.rs:783 at baa3061), red with the old reading.
- **#116: closed.** The core's `Admission::holders` and the endpoint's
  holder list went with streaming (12d0ba2). The synced directory's own
  list became `holders` (synced.rs:329-335), which reads
  `Host::standings`. One home.
- **#118: closed by removal**: the endpoint and its 416 (12d0ba2).
- **#120: closed.** syp-2 removed the plants (4867dcf). `git grep -n
  "PLANT\|Plant" -- crates` outside tests finds none. apps/jellyfin/mw's
  `Plant` belongs to that app.
- **#121: closed in its failure.** `both_ways` now refuses a second
  directory for one (group, folder), and a second (group, folder) for
  one directory, and answers an ask it has had as before (synced.rs:172-181,
  db60756). So one loop runs per kept entry, held by
  `one_directory_keeps_one_group_s_folder` (tests/folders.rs:1048 at baa3061). The
  loop's task is still detached (`keep_in_step`, synced.rs:243). No verb
  stops a sync, so nothing needs its handle yet; the verb that unsyncs
  will need one.
- **#122: fixed here**, cf21f71 and 734018d. Five silent decisions now
  trace: a touch with the bytes held, a held file the record no longer
  places kept because it changed, a placed file not written over a
  changed disk, and a written or re-pointed file whose mtime cannot be
  read. Each condition is unchanged, split where it was joined. +20
  lines.
- **#117, not fixed, new lines.** `share --folder` still reads an
  unreadable API file as "" and so refuses "not running"
  (cli.rs:580, :589-592), and reads an unreadable answer as an empty
  refusal (:595). It is not fixed here because the refusal the person
  sees would change.
- **syp-6 (a)'s retry for a file no member will give.** The retry runs
  each 15 s for as long as any placed file is unwritten
  (synced.rs:66-79, :268-278), each wake a step: who stands, the layout,
  and the apply over every placed file. perf-2's idle bar was rerun with
  every file written (f86a6fa, 0.4% of a core), not with one that never
  arrives. Not a principle, and each wake traces at debug (:320). No
  finding; it is for the run that measures a holder gone for good.
- **syp-6 (b) is glassbox.** `asked_to_stop` says Ctrl-C, SIGTERM, and
  each failure to wait on them (node.rs:116-137). The dial traces the
  addresses iroh holds (folder.rs:551-559). The command is the one
  caller that starts a node (`git grep -n "Node::start" -- crates apps`
  outside tests: cli.rs:623), so no other path leaves an endpoint open
  on SIGTERM.
- **dedc224's body is wrong**: it says `cargo doc -p svrngs --no-deps`
  was clean, but that printed two warnings. Both predate this audit:
  issuer.rs:18 links to the private `DRAWS`, and identity.rs:4 to
  `commonwealth_core::ids::NodePubkey`. None is in cli.rs. Said here
  rather than amended.
- **Still open, none touched by the range's branch commits** (their
  files, `git log --no-merges --name-only 7442463..baa3061 ^main`, are
  core admit, alpn, folder, lib, membership, synced; node folder, node,
  synced, Cargo.toml; runtime lib; the command's cli.rs and Cargo.toml;
  and tests): #31, #57, #68, #70, #71, #75, #77, #79, #81, #89, #91,
  #95, #97, #99, #100, #103 (node.rs `shared_with`, untouched: 7102542
  and 4867dcf changed `start`, `endpoint`, `standings`' visibility and
  the gate's plant), #104 and #113 (main's).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 7442463..baa3061 ^main --
crates apps web bridge` matching `\b(struct|enum|trait|type)\s+[A-Z]\w*`
are: `Look`, `NotASyncPath`, `Nothing`, `Addr`, `Asking`, `FolderAsk`,
`Layout`, `Member`, `Said`, `SyncPath`, `Found`, `Holders`, `KeepAsk`,
`ReadAt`, `Syncing`, `Teller`, and `type Error`, an associated type. No
merge added one (`git diff --cc` of fb98c92 and 05eee01). Each was run
through `converge noun <Name> --corpus-id tree-baa3061`
(/tmp/sypaudit1/nouns.out), beside `git grep -nP` at baa3061:

| noun | converge (production) | git grep (all) | read |
|---|---|---|---|
| Said | 2: cli.rs:739, door/choices.rs:39 | 3: those and tests/folders.rs:1078 (the range's) | Neither production `Said` is the range's. The range's is a test's log sink, `io::Write` over a buffer. The two older ones are distinct: the command's words to a person, and a person's yes or stop to an app. No change. |
| Asking | 1: node.rs:657 | 2: and tests/folders.rs:1107 (the range's) | The people route's state against a test's fixture: Ana's node with Ben a hand-driven holder. Private to each. No change. |
| Member | 1: svrngs-runtime register.rs:40 | 2: and tests/kept.rs:39 (the range's) | A key standing in a group against a test's child process, killed on drop. No change. |

Every other noun has one definition. `Addr` is in tests alone
(member.rs:32), which converge's scope leaves out ("definitions (0)").
converge's lines are one early (I1: `Look` :81 for :82, `Nothing` :600
for :601, `SyncPath` :32 for :33).

### Fixed here

- dedc224: `new_link`'s doc moved back above it. Comments only.
- 5d36346: kept.rs's three in-process Cy set-ups and the members' seed
  each written once. Tests only, behaviour kept.
- 35a9ef2: folders.rs's `ask_ben` closure is its `get`; `noise` written
  once, in tests/common. Tests only, behaviour kept.
- cf21f71, 734018d: #122's five skips traced, decisions unchanged
  (`cargo test -p svrngs-node --test folders --test kept`: 19 passed,
  and 3 passed with 1 ignored, after each).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 122 (was #120 on `folders`; #116 before syp-3's merge) | 1 | crates/svrngs-node/src/synced.rs:462, :559, :584, :591-594, :615, at baa3061 | cf21f71, 734018d | The apply's skips and the step's same-bytes touch were untraced. |
| 121 (was #119 on `folders`; #115 before syp-3's merge) | 10 (RUST_ARCH ownership) | crates/svrngs-node/src/synced.rs:172-181, :243, at baa3061 | closed in its failure (db60756) | One loop per directory and folder now. The loop's task stays detached until a verb stops a sync. |
| 120 (was #118 on `folders`; #114 before syp-3's merge) | 8 | — | closed (4867dcf) | The plants are gone from product code. |
| 119 (was #117 on `folders`; #113 before syp-3's merge) | 6, 1 | crates/svrngs-node/src/synced.rs:389-391, at baa3061 | closed (7102542) | A group that cannot be read stops the step by its name before anything is deleted. |
| 118 (was #116 on `folders`; #112 before syp-3's merge) | 1 | — | closed (12d0ba2) | The endpoint and its 416 were removed. |
| 117 (was #115 on `folders`; #111 before syp-3's merge) | 6 | crates/svrngs/src/cli.rs:580, :589-592, :595, at baa3061 | not fixed | `share --folder` reads an unreadable API file as "not running", and an unreadable answer as an empty refusal. |
| 116 (was #114 on `folders`; #110 before syp-3's merge) | 8 | crates/svrngs-node/src/synced.rs:329-335, at baa3061 | closed (12d0ba2, 7102542) | Who holds a file has one home: `Host::standings`. |
| 115 (was #113 on `folders`; #109 before syp-3's merge) | 6, 1 | crates/svrngs-node/src/folder.rs:442-457, at baa3061 | closed (12d0ba2, 7102542) | No list or endpoint is left to read an absence as none. The provider refuses, said. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 1,502 lines left (13848 / 15350). |
| 31, 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 103, 104, 113 | as recorded | as recorded | not fixed | Untouched by the range's branch commits. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` lines | Still present: one line early. |
| I4 | 6 | `refresh` | Still present, and grown on a tree outside git: "typescript: skipped (export failed)" and "python: skipped (export failed)". Yet dry-report paired the run drivers' Python functions, so its Python symbols came from somewhere other than the export it called skipped. |
| I9 | 6, 7 | `code dry-report` near tier | Still present on b604bcb6…: a verbatim copy and its original reach only cosine 0.920. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said: "could-not-judge". |
| I24 | 6, 7 | `code dry-report` exact tier | Still present on b604bcb6…: a 10-line verbatim Rust copy, and two 30-line byte-identical Python functions, each considered and each in no exact group. |
| I25 | 6 | `code index`'s automatic full rebuild | Still present, reached without `--full`: past 500 changed files, `code index` rebuilds whole, refuses its own promotion over the corpus's `documents.lance`, and leaves the corpus with no chunk index. dry-report then errors rather than reporting a zero. `svrngs-folders` is left so (above). |

## REVIEW-audit-v1-auto-1 (2026-10-11, range ec703b5..4e1d3a9)

No `REVIEW-audit-` row precedes this one in its queue
(ralph/next/v1/STATE.md), so the range starts where the queue did:
ec703b5, the commit that made it. It ends at 4e1d3a9, the commit that
made this row. `git rev-list --count ec703b5..4e1d3a9` is 21, no merge
among them (`--merges`: 0), all on main. One unit, u7-removal, and its
`done` (cebb39a). The audit's own commits: f2ed770 (docs), f941db2,
14ce2a7, e07b98b, 45af469 and 0414547 (moves, behaviour kept), and
3e9c51f (traces, decisions unchanged). It opens #123 and #124, grows
#31, moves #117, narrows #103, and closes none.

The row's `docs/ARCH_PRINCIPLES.md` is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli` is
~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 b604bcb6…
(`shasum -a 256`, built 2026-10-10 14:10), the binary the last two
audits planted; the row is corrected in ralph/next/v1/STATE.md beside
this section. The embedder was `llama-server` on the corpus's model
(qwen-embedding-0.6b.gguf, ~/.svrnmesh/config.toml:18) on :8080, `-b 8192
-ub 8192`, for this session. The corpus is 4e1d3a9's tree, `git archive`
into /tmp/v1audit1/tree-4e1d3a9: `code index . --corpus-id tree-4e1d3a9`,
3177 chunks in 216 s (/tmp/v1audit1/index.out); `refresh` gave rust
12865 symbols and "typescript: skipped (export failed)", "python: skipped
(export failed)" (I4; /tmp/v1audit1/refresh.out). The corpora
`tree-4e1d3a9` and `tree-plant1` stay in ~/.svrnmesh/indexes, about 23 MB
each. The scripts are the syp audit's (/tmp/sypaudit1), copied to
/tmp/v1audit1: ledger.py with this queue's unit name, and clusters.py and
windows.py without their `^main`, since this range is main's own.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
ec703b5..4e1d3a9`, summed by /tmp/v1audit1/ledger.py
(/tmp/v1audit1/ledger.out). A commit counts toward the unit its subject
names, else toward the next `done`'s; product code is crates/, apps/,
web/ and bridge/; a test is a `tests/` or `check/` segment, or a test
file name (`*_tests.rs`, `tests.rs`, `*.test.mjs`).

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| u7-removal | 20 | 1090 | 274 | 816 | 1440 | 101 | 1339 |
| REVIEW-audit-v1-auto-1 (the row's own commit) | 1 | 0 | 0 | 0 | 0 | 0 | 0 |

By crate (/tmp/v1audit1/byfile.out): svrngs-core +488/−190, net 298;
svrngs-node +297/−10, net 287; the command +148/−6, net 142;
svrngs-runtime +154/−67, net 87; web/ +3/−1. By file, the most:
svrngs-node removal.rs +227 (2289372, a473f1d), cli.rs +142 (d0abac4),
core membership.rs +130 (f915bc2's +245/−119, the keeper and the cut),
core register.rs +88 (5f5a585, the two commands), core admit.rs +58
(45b338b, f915bc2, 995dd13, 5f5a585), runtime register.rs +48 and
journal.rs +39 (f915bc2, 995dd13, 2d927be, 3bcf840).

The size budget, scripts/lean_check.py at HEAD over a `git archive` tree
of each commit (`python3 scripts/lean_check.py <tree>`,
/tmp/v1audit1/lean-points.out):

| at | svrngs-core | repository |
|---|---:|---:|
| ec703b5, range start | 2719 / 3000 | 13848 / 15350 |
| 888a3ee, (a) the holes planted | 2720 | 13849 |
| 45b338b, (b) forks void nothing | 2720 | 13849 |
| f915bc2, (c) removal is the keeper's | 2850 | 13994 |
| 995dd13, (d) #16 and the hole in a chain | 2854 | 14000 |
| 5f5a585, the two commands | 2939 | 14085 |
| 3bcf840, the kept fold one `Register` | 2939 | 14126 |
| 2289372, the record API | 2939 | 14295 |
| d0abac4, the command | 2939 | 14410 |
| cf6ea9c, the words, the doors, the folder | 2939 | 14450 |
| a473f1d, the round woken | 2939 | 14457 |
| 4e1d3a9, range end | 2939 | 14457 |
| 0414547, this audit's last | **2937 / 3000** | **14463 / 15350** |

The range nets +609 by the budget's count, svrngs-core +220 of it. **63
lines are left in svrngs-core and 887 in the repository** (#80). The core
is the one near its edge: f915bc2 and 5f5a585 took 215 of the 281 it had.

### (2) Clones (`code dry-report`)

**Scoped**, at 0.95, for each dir the range touched
(/tmp/v1audit1/clean-*0.95.md; every side blamed by `git blame -w` at
4e1d3a9 against the range's commits, /tmp/v1audit1/clusters.out):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 431 | 6 | 0 | 1: `standing` (below) |
| crates/svrngs-runtime/ | 131 | 1 | 0 | 0 |
| crates/svrngs-node/ | 462 | 1 | 0 | 0 |
| crates/svrngs-node/tests/ | 0 | **could-not-judge** | **could-not-judge** | — (I19) |
| crates/svrngs/ | 72 | 0 | 0 | 0 |
| crates/svrngs/tests/ | 0 | **could-not-judge** | **could-not-judge** | — (I19) |
| web/ | 93 | **could-not-judge** | 0 | 0 (I4: no JavaScript symbols for the exact tier) |

**Unscoped**: 0.95 (9 exact, 46 near; 1 group with a side in the range),
0.9 (9, 76; 7), 0.8 (9, 265; 25). Every group with a side the range
added:

| at | cluster | cosine | read |
|---|---|---|---|
| exact | `standing` core membership_invite_tests.rs:65-67 ~ removal_tests.rs:36-38 | — | Byte for byte, 888a3ee's copy of a test helper. **Folded, 45af469**: one `standing` in tests_support.rs. |
| 0.9 | core register.rs `decide` :174-185 ~ record.rs `decide` :58-66 ~ tests_support.rs `decide` :284-292, :300-308 | 0.922-0.940 | `HandOnTo::decide` beside other commands' one-act decides: the `Command` trait's shape. No change. Reading them found the walk "keys standing as this person" spelled three times (register.rs:128-132, :167-169, record.rs:37-41): **folded, 0414547**, `Membership::keys_of`. |
| 0.9 | removal_tests.rs `a_hand_on_moves_the_keeper…` :271-301 ~ `a_removed_key_cannot_remove…` :301-332; register_tests.rs :138-155 ~ :152-166; membership_tests.rs `admit_act` ~ removal_tests.rs `admit_of` | 0.917-0.922 | Tests of one shape, each a different case; `admit_act` signs where `admit_of` builds the act unsigned. No change. |
| 0.9 | demo/runs/u7/2026-10-11/run.py `say` ~ perf drivers' `say`; `poll_lends` ~ `one` | 0.940-1.000 | Run drivers, kept byte for byte beside their data (#61); `demo/runs/` is outside the budget. No change. |
| 0.8 | core membership.rs `membership` :186-295 ~ `walk` :333-470 | 0.879 | One calls the other: the fixpoint and its one walk. No change; see #124. |
| 0.8 | core admit.rs `admit` ×3 (:443-563, :564-700, :701-813) | 0.810-0.826 | Chunks of one function. No change. |
| 0.8 | core register.rs `decide` :123-150 ~ record.rs, register.rs `decide` | 0.860-0.862 | As at 0.9. |
| 0.8 | runtime register.rs `standing_in` :63-76 ~ `membership_in` :109-129; `standing` ~ node.rs `standings`, `shared_with`, `standing_peers` | 0.80-0.878 | `standing_in` is a test-only projection of `register_in`. Reading `standing` beside its neighbour found `RingRail::register` (:173-180) repeating `standing`'s held-and-seeded check with no trace: **folded and traced, 3e9c51f** (`seeded`). |
| 0.8 | runtime journal.rs `standing` :310-321 ~ `register` :318-326 | 0.828 | Both ask one `folded`. No change. |
| 0.8 | node removal.rs `status` ~ `into_response`, `router` ~ `decides`, `remove` in an 82-symbol cluster of signing helpers | 0.80-0.84 | Neighbours, each its own. No change. |
| 0.8 | node surface.rs `groups` ~ rail.rs `standing_of`, `log_answer`; home.js `groups` ~ `list`; ring_sync.rs `new` ~ `wake`; cli.rs `run` ~ `serve`, `folder` ~ folder.rs `ask` | 0.80-0.86 | A line or two in range each, in clusters of one shape. No change. |
| 0.8 | demo/runs/u7/2026-10-11/phone.mjs `opening`, `open` ~ apps' check drivers, web/app.js `open` | 0.80-0.91 | A run's driver (#61). No change. |

**Window pass** (/tmp/v1audit1/windows.py . ec703b5..4e1d3a9,
/tmp/v1audit1/windows.out): 854 window hits in 171 spans, 18 new: 2 in
product source, 16 in tests, 0 in the run drivers.

- **origin.rs:298-303 == removal.rs:183-188**: a `Contact` built from a
  key alone, the fifth such in product code (node.rs:620, origin.rs:301,
  rail.rs:576, removal.rs:186, ring_sync/journal.rs:40). **Folded,
  e07b98b**: `Contact::by_key`, −8 lines.
- folder.rs:25-29 == removal.rs:23-27: the two modules' axum imports. No
  change.
- **tests/removal.rs:19-49 == canaries.rs:22-54 == group_not_mesh.rs:26-59**:
  2289372 wrote out again the node-start block (`signing`, `key`,
  `no_relay`, `no_seed`, `start`), the third binary to hold all five
  (record_api.rs:27-53 holds four, its `start` calling `start_on`);
  `signing` alone is in 13 (`grep -l "fn signing(seed: u8)"` over
  crates/svrngs-node/tests). #31, grown; not folded here, since one home
  for them is a change to 13 test binaries, not the range's one.
- No change to the rest: core membership_tests.rs:63-68 == :199-203,
  register_tests.rs:242-246 == :303-307, removal_tests.rs:251-255 ==
  :311-315 and runtime tests.rs:1014-1019 == :1042-1047 are a test's
  set-up written in the test; browser/main.rs:231-241 == :372-378 ==
  :1253-1263 are its page set-up, and :1330-1335 the record API's POST,
  as in conformance.rs, record_api.rs, room.rs and lend_origin.rs;
  group_not_mesh.rs:92-96 == removal.rs:60-64 is a seed written; and
  svrngs/tests/remove.rs:165-171 == share.rs:673-679 the "svrngs stopped"
  ending both command tests share.

**The instrument, planted** in /tmp/v1audit1/tree-plant1, a copy of
tree-4e1d3a9 with two plants of removal.rs's `tell` (35 lines): P1, its
body byte for byte as `planted_tell` in removal.rs; P2, byte for byte under
its own name `tell`, in a new `mod planted` (planted.rs). `code index`
then `refresh` (rust 12892 symbols), `dry-report --scope
crates/svrngs-node/` at 0.95 and 0.8 (/tmp/v1audit1/plant-node*.md): 464
locations against 462, so both were seen. **P2 is an exact group** (`tell`
planted.rs:6 ~ removal.rs:169, 34 lines) and a near one at 0.954. **P1
is in no exact group**, and in the near tier only at 0.8, at cosine
0.933 with `tell`. So the exact tier finds a verbatim copy only when it
keeps its name (I24, refined below): the scoped tables' "0 exact" says
nothing of a renamed copy, and the window pass, which compares bodies,
carries that question. It found the one product copy the range made
(`Contact`, above), which neither tier did.

### Reading the range

- **Docs on the wrong item, fixed in f2ed770.** f915bc2 put `names`
  between `moves_membership`'s doc and its body (core membership.rs:469-
  477 at 4e1d3a9), and 3bcf840 put `Register` under `Member`'s doc
  (runtime register.rs:38-41). Moved back. journal.rs's `Held` said the
  acts the register reads are "kept in the log's order"; since f915bc2 a
  newly named key's acts are added from the whole log after the tail's,
  and one after its `Remove` in the same tail is pushed twice. Harmless,
  since `ActSet::of` reads a set, and now said.
- **Two `Asked`, fixed in f941db2.** `converge noun Asked`: core
  joining.rs:71, the issuer's answer to a link's ask (svrngs-node
  re-exports it as `redeem::Asked`), and 2289372's removal.rs:44, the
  command's request. The range's is now `RemovalAsk`, after `FolderAsk`.
- **`removed_anywhere`, moved in 14ce2a7 and traced in 3e9c51f.**
  cf6ea9c wrote it in node.rs as `standing_anywhere`'s loop again, for the
  same two callers, reading groups that cannot be listed and a register
  that cannot be read as "nobody removed", with no line (AGENTS.md 6,
  ARCH 6's smell: "an `Err` collapsed into a success-shaped value"). It
  is now `RingRail::removed_anywhere` beside `standing_anywhere`, and
  says both at warn, as that one does. Its answers are unchanged. #103
  narrowed by one site.
- **`RingRail::register`'s silent empty register, fixed in 3e9c51f.** A
  group not held, local-only or without a seed answered
  `Register::default()` with no line, where `standing`, the same check,
  says each. One `seeded` now makes the check for both and says it.
- **Silent branches in the node and the core, traced in 3e9c51f**:
  removal.rs's name that is no `Person` (in `remove` and `hand_over`),
  `tell`'s key that is no iroh key, and core `HandOnTo::decide`'s
  refusal. Each answer is unchanged.
- **#123, not fixed.** `hand_over`'s `NotDecider` names who decides by
  reading the register again, `.ok()` (removal.rs:217), so an unreadable
  register becomes `decides: None`, which the command words as "Only
  whoever decides who is removed can …" (cli.rs, `Said::Removal`). That
  is the same words as a register naming no keeper's person. And
  surface.rs:108-110 reads an unreadable register as `removed: false`;
  the `standing` read beside it says its own failure at warn, but they
  are two reads. Not fixed: telling "could not read" apart changes what
  the person sees.
- **#124, not fixed.** `membership`'s fixpoint (core membership.rs:254-
  264) stops after `ops.len() + 1` walks whether or not the cuts
  settled, and returns the last walk as the answer. The only sign is the
  round's debug line, `settled=false`, the same line every unsettled round
  prints. Not reproduced: no test or run here reached the cap, and
  whether cuts can cycle is not shown. Not fixed: saying it as
  could-not-judge changes the fold's type, in a core with 63 lines left.
- **#117 moved.** d0abac4 moved `share --folder`'s POST into
  `ask_running` (cli.rs:683-706), which `remove` and `handover` share: the
  unreadable API file read as "" (:688) and the unreadable answer read as
  "" (:703) now serve all three. For `remove` and `handover` the empty
  answer fails to parse and is refused as "cannot be changed on this
  computer", said at debug (cli.rs, `removal`), so it is not lost there.
- **`tell` is detached** (removal.rs:159, `tokio::spawn`), its handle
  dropped. It is bounded (one request per removed computer, never
  retried) and traces each, so no finding; it is the shape #121 named.
- **A person's words** (fence 5): the range's strings a person reads
  hold none of key, member, keeper, ring, act or journal as a word
  (`grep -iwE "key|keys|member|members|keeper|ring|act|acts|journal"` over
  the quoted strings of cli.rs's `Said::Removal` arm and the `remove` and
  `handover` help, and over door.rs's, home.js's and web/app.js's strings
  that say "removed" or "no longer": no line). "decides who is removed"
  stands for the keeper.
- **This audit's own misstatements**, not amended: e07b98b's body gives
  the four test binaries' counts in the wrong order (`cargo test`
  printed canaries 1, group_not_mesh 3, record_api 9, removal 3), and
  3e9c51f's "as before" for the core and runtime counts rests on the
  commit changing no test file (`git show --stat 3e9c51f`), not on a run
  at its parent.
- **Still open, none touched by the range** (its files, `git log
  --no-merges --name-only ec703b5..4e1d3a9`, are core admit, command, lib,
  membership, register, sync; runtime journal, register; node door,
  home.js, lib, node, removal, ring_sync, surface; cli.rs; web/app.js;
  tests): #57, #68, #70, #71, #75, #77, #79, #81, #89, #91, #95, #97,
  #99, #100, #104, #113. #79 (core sync.rs:234-237) is in a file
  45b338b touched, at one line of `Compaction`'s doc ("either branch of
  a fork"; `git show 45b338b -- crates/svrngs-core/src/sync.rs`), not
  that branch.

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges ec703b5..4e1d3a9 -- crates apps
web bridge` matching `\b(struct|enum|trait|type)\s+[A-Z]\w*` are:
`HandOnRefusal`, `Outcome`, `RemovalRefusal`, `Asked`, `Cut`, `HandOnTo`,
`Register`, `Removal`, `Folded`, and `type Refusal` twice (the `Command`
trait's associated type). Each was run through `converge noun <Name>
--corpus-id tree-4e1d3a9` (/tmp/v1audit1/nouns.out):

| noun | converge (production) | read |
|---|---|---|
| Asked | 2: core joining.rs:71, svrngs-node removal.rs:44 | Two meanings under one name, both public in svrngs-node: the issuer's answer and the command's request. **Renamed, f941db2**: `RemovalAsk`. |

Every other noun has one definition. `Folded` has none: a type alias
2d927be added and 3bcf840 removed. converge's lines are still one early
(I1: `Asked` :71 for :72 and :44 for :45, `Cut` :145 for :146,
`Register` :42 for :43).

### Fixed here

- f2ed770: three docs, comments only.
- f941db2: `removal::Asked` is `RemovalAsk`. A rename; the wire unchanged.
- 14ce2a7: `removed_anywhere` moved to `RingRail`, beside
  `standing_anywhere`. `cargo test -p svrngs-node --test removal`: 3
  passed.
- 3e9c51f: six silent branches traced, and `RingRail::register` through
  `standing`'s one check. Decisions unchanged. svrngs-core 2939, the
  repository 14457 → 14476.
- e07b98b: `Contact::by_key`, five sites. 14476 → 14465.
- 45af469: the core tests' `standing`, once. Tests only.
- 0414547: `Membership::keys_of`, three sites. svrngs-core 2939 → 2937,
  the repository 14465 → 14463.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 124 | 6, 1 | crates/svrngs-core/src/membership.rs:254-264, at 0414547 | not fixed | The cut fixpoint stops at `ops.len() + 1` walks settled or not, and answers with the last walk, said only by the round's ordinary debug line. Not reproduced. |
| 123 | 6 | crates/svrngs-node/src/removal.rs:217; crates/svrngs-node/src/surface.rs:108-110, at 0414547 | not fixed | An unreadable register is read as "nobody decides" in the hand-on's refusal, worded as a person would read a register with no keeper; and as "not removed" on the home page. |
| 117 | 6 | crates/svrngs/src/cli.rs:688, :703, at 0414547 | not fixed, moved (d0abac4) | `ask_running`, now `share --folder`'s, `remove`'s and `handover`'s, reads an unreadable API file as "not running" and an unreadable answer as an empty one. |
| 103 | 6 | crates/svrngs-node/src/node.rs `Node::lends`, as recorded | narrowed (3e9c51f) | `removed_anywhere`'s unlistable groups now said at warn; `Node::lends` still reads them as none. |
| 31 | 8 | crates/svrngs-node/tests/removal.rs:19-49 ~ canaries.rs:22-54 ~ group_not_mesh.rs:26-59 ~ record_api.rs:27-53 | not fixed | Grown by 2289372: a third binary holding the node-start block whole; `signing` in 13. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 63 lines left in svrngs-core (2937 / 3000), 887 in the repository (14463 / 15350). |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 104, 113 | as recorded | as recorded | not fixed | Untouched by the range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` lines | Still present: one line early. |
| I4 | 6 | `refresh` | Still present: "typescript: skipped (export failed)", "python: skipped (export failed)". The web/ scope's exact tier says could-not-judge, which it is. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said: "could-not-judge". |
| I24 | 6, 7 | `code dry-report` exact tier | Still present on b604bcb6…, and narrowed by this plant: a 35-line verbatim copy under its own name, in another file, is an exact group; the same copy renamed is in none. The tier keys on the name with the body, so "0 exact" is no evidence against a renamed copy. |
| I9 | 6, 7 | `code dry-report` near tier | Still present: the renamed verbatim copy reaches cosine 0.933, and the same-named one 0.954, against their original. |

## REVIEW-audit-v1-auto-2 (2026-10-11, range 53a5bd4..39d1c9b)

The previous `REVIEW-audit-` row is REVIEW-audit-v1-auto-1, its hash
53a5bd4 (ralph/next/v1/STATE.md:14); the range ends at 39d1c9b, the
commit that made this row. `git rev-list --count 53a5bd4..39d1c9b` is
32, no merge among them (`--merges`: 0), all on main. Three units:
v1-said-not-substituted, v1-keeper-office and v1-onboarding. The audit's
own commits: ebdc110 (the row's two paths), 0b42088 (a move, behaviour
kept), 70c4ed7 (a trace, decisions unchanged) and 5bfcc40 (a test move).
It closes #123 and #124, grows #31, and opens none.

The instrument is ~/dev/commonwealth-ai/target/debug/sovereign-cli,
sha256 b604bcb6… (`shasum -a 256`), the binary the last audit used. The
embedder was `llama-server` on qwen-embedding-0.6b.gguf on :8080
(/tmp/v1audit2/llama.out). The corpus is 39d1c9b's tree, `git archive`
into /tmp/v1audit2/tree-39d1c9b: `code index . --corpus-id tree-39d1c9b`,
3248 chunks in 544 s (/tmp/v1audit2/index.out); `refresh` gave rust
13224 symbols and "typescript: skipped (export failed)", "python: skipped
(export failed)" (I4; /tmp/v1audit2/refresh.out). The scripts are the
last audit's, copied to /tmp/v1audit2.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
53a5bd4..39d1c9b`, summed by /tmp/v1audit2/ledger.py
(/tmp/v1audit2/ledger.out), counted as the last audit counted them.

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| v1-said-not-substituted | 10 | 107 | 33 | 74 | 244 | 13 | 231 |
| v1-keeper-office | 6 | 137 | 27 | 110 | 220 | 37 | 183 |
| v1-onboarding | 14 | 512 | 24 | 488 | 719 | 32 | 687 |
| REVIEW-audit-v1-auto-1's done, this row's own commit | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 32 | 756 | 84 | 672 | 1183 | 82 | 1101 |

By file, the most: web/install.sh +129 (090d9e1), apps/jellyfin/mw
viewer.rs +180 net (e877024, its tests in the same file), cli.rs +91 net
(e1bf5e0, 20ede44), runtime register.rs +52 net (01393a7, `removals_in`),
core membership.rs +28 net (7e54170, 441db92).

The size budget, scripts/lean_check.py at HEAD over a `git archive` tree
of each commit (/tmp/v1audit2/lean-points.out):

| at | svrngs-core | repository |
|---|---:|---:|
| 53a5bd4, range start | 2937 / 3000 | 14463 / 15350 |
| 7e54170, cuts that cycle said unsettled (#124) | 2964 | 14510 |
| 441db92, a hand-on survives its giver's removal | 2971 | 14517 |
| 01393a7, every removal shown to the group | 2971 | 14579 |
| e1bf5e0, `join --no-run` | 2971 | 14630 |
| 090d9e1, install.sh | 2971 | 14730 |
| 20ede44, the group's folder at ~/svrngs/<group> | 2971 | 14834 |
| 39d1c9b, range end | 2971 | 14834 |
| 5bfcc40, this audit's last | **2971 / 3000** | **14835 / 15350** |

The range nets +371 by the budget's count, svrngs-core +34 of it. **29
lines are left in svrngs-core and 515 in the repository** (#80).

### (2) Clones (`code dry-report`)

Scoped, at 0.95, for each dir the range touched
(/tmp/v1audit2/clean-*_0.95.md; every side blamed against the range's
commits, /tmp/v1audit2/clusters-scoped.out):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 435 | 5 | 0 | none |
| crates/svrngs-node/ | 464 | 1 | 0 | none |
| crates/svrngs-runtime/ | 135 | 1 | 0 | none |
| crates/svrngs/ | 74 | 0 | 0 | none |
| apps/jellyfin/mw/ | 61 | could-not-judge (I4's kin: no SCIP symbols) | 0 | none |
| apps/jellyfin/check/ | 57 | could-not-judge | 2 | none |
| web/ | 94 | could-not-judge (I4) | 0 | none |
| crates/svrngs-node/tests/, crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | — |

**No exact or near clone at the instrument's threshold has a side the
range added.** Below it, whole tree (/tmp/v1audit2/clusters.out): at
0.90, two groups — removal_tests.rs's
`a_hand_on_survives_its_givers_removal` (:418-443) ~
`a_hand_on_that_made_no_keeper_is_cut_as_before` (:445-470), cosine
0.902, two tests stating two cases; and the llama run's `up`
(demo/runs/llama/2026-10-11/run.py:47-58) ~ u7's (run.py:29-40), 0.937,
two recorded runs, each kept as run. At 0.80, 21 groups have a range side,
none past 0.937; read, none is one behaviour twice.

The tests scopes are could-not-judge (I19), so a line-window pass stood in
for them (/tmp/v1audit2/windows.out: 867 hits, 180 spans, 13 new). Of the
13, one was fixed here — crates/svrngs/tests/install.rs:241-259 ==
:348-366 at 39d1c9b, 090d9e1's and 20ede44's starts of Sam's computer,
now `sams_computer` (5bfcc40) — and the rest are the same block across
binaries (join_page.rs:93-110, phone_dry.rs:49-68: #31), test fixtures
stating each test's origin (apps/jellyfin/mw/src/viewer.rs:561-568 ==
:658-663, the browser main.rs spans) or register_tests.rs:201-209 ==
:257-265, two tests' set-up of one group.

### Reading the range

- **Never silently substitute** (AGENTS.md 6): the `.ok()`s, `??` and
  `unwrap_or_default` the range added (`git log -p 53a5bd4..39d1c9b --
  crates/*/src apps/*/mw/src web/app.js web/install.sh`) each carry their
  absence: surface.rs's `folders` and node.rs `people_lends`' `removals`
  are `null` after a warn, read by home.js and app.js as "cannot be read
  just now"; membership.rs's `made.get(..).unwrap_or_default()` is "no
  hand-on made a keeper", traced with its count. install.sh's wait is
  bounded at 30 tries and says the log on running out (web/install.sh:120-124).
- **Glassbox** (AGENTS.md 1): runtime register.rs `removals_in` left out
  an uncounted `Remove` with no trace — fixed, 70c4ed7.
- **A person's words** (fence 5): the range's quoted strings in cli.rs,
  home/, surface.rs, removal.rs, web/app.js, web/index.html and
  web/install.sh hold none of key, member, keeper, ring, act or journal as
  a word (`grep -iwE` over them: no line).
- **#123 and #124 are closed** by v1-said-not-substituted: the hand-on's
  refusal has `Outcome::NotDeciderUnread` (cli.rs:1069), the home page
  `removed: null` (surface.rs:93, home.js:57), and the fold
  `Membership::unsettled` (core membership.rs:151, :279).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 53a5bd4..39d1c9b -- crates apps
web bridge` matching `\b(struct|enum|trait|type)\s+[A-Z]\w*` are:
`Started`, `Removing`, `Said`, `Unreadable` twice and `Message`. Each was
run through `converge noun <Name>` on tree-39d1c9b
(/tmp/v1audit2/nouns.out):

| noun | converge (production) | read |
|---|---|---|
| Said | 3: cli.rs:855, svrngs-node door/choices.rs:39, svrngs-runtime register.rs:63 | Three meanings: the command's words, a person's yes or stop of an app, and 01393a7's whether a removal was the cut or an ask. **Renamed, 0b42088**: the range's is `RemovalSaid`; the wire unchanged. |

`Removing` has one definition. `Started` (install.rs), `Unreadable` and
`Message` (svrngs-node tests/removal.rs) are tests' own, 0 in production;
09e1484's `Unreadable(PathBuf)` was replaced inside the range (47c48fa).
converge's lines are still one early (I1: `Said` :855 for :856, :39 for
:40, :63 for :64; `Removing` :53 for :54).

### Fixed here

- 0b42088: runtime `Said` is `RemovalSaid`. A rename; the wire unchanged.
- 70c4ed7: `removals_in`'s silent branch said at debug. Decisions
  unchanged; the repository 14834 → 14835.
- 5bfcc40: install.rs's two starts of Sam's computer, once. Tests only.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 124 | 6, 1 | crates/svrngs-core/src/membership.rs:151, :279 | closed (7e54170, 3c1374c) | An unsettled fixpoint is `Membership::unsettled`, and no caller reads it as standing. |
| 123 | 6 | crates/svrngs/src/cli.rs:1069; crates/svrngs-node/src/surface.rs:93 | closed (09e1484, 233daf0) | An unreadable register is said as itself in the hand-on's refusal and on the home page. |
| 31 | 8 | crates/svrngs-node/tests/removal.rs:19-49 ~ canaries.rs:22-54 ~ group_not_mesh.rs:26-59 ~ record_api.rs:27-53; crates/svrngs/tests/install.rs:35-57 ~ join_page.rs:93-110 ~ phone_dry.rs:49-68 | not fixed | Grown by 090d9e1 and 20ede44: the node-start block in the command's test binaries too, install.rs's two copies now one (5bfcc40). |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 29 lines left in svrngs-core (2971 / 3000), 515 in the repository (14835 / 15350). |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 103, 104, 113, 117 | as recorded | as recorded | not fixed | Not touched by this range's decisions. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` lines | Still present: one line early. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)"; web/'s and apps/jellyfin's exact tiers say could-not-judge, which they are. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said: "could-not-judge". The line-window pass stood in. |

## REVIEW-audit-bash-auto-1 (2026-10-12, range b91134f..feba150)

No earlier `REVIEW-audit-` row exists in this queue, so the range starts
at b91134f, the commit that made the queue (ralph/next/bash/STATE.md),
and ends at feba150, the commit that made this row. `git rev-list --count
b91134f..feba150` is 31, no merge among them (`--merges`: 0), all on
main. Four units: bash-first-minute, bash-limit, bash-person and
bash-devices. Between REVIEW-audit-v1-auto-2's last commit and b91134f
are only b1d8f40 and 9d95533, Demo C's dry run, which touch `demo/` alone.
The audit's own commits: 8849ea0 (the row's two paths), 7b0a59b and
ee4792c (moves in the product, behaviour kept), 4f221f7 and a85a146
(test moves) and 635b782 (a trace, decisions unchanged). It opens #125,
grows and shrinks #31, and closes none.

The instrument is ~/dev/commonwealth-ai/target/debug/sovereign-cli,
sha256 585edd51… (`shasum -a 256`). That is not the last audit's binary
(b604bcb6…), so its exact tier was checked against a plant first (below).
The embedder was the daemon on :9741, which served qwen-embedding-0.6b,
the model the last audit used through llama-server (/tmp/bashaudit1/index.out:
"embeddings via http://localhost:9741/v1/embeddings, model
`qwen-embedding-0.6b`, 1024-d"). The corpus is feba150's tree, `git
archive` into /tmp/bashaudit1/tree-feba150: `code index . --corpus-id
tree-feba150` gave 3408 chunks in 429 s. `project refresh` gave rust
13860 symbols, with typescript and python "skipped (export failed)" (I4;
/tmp/bashaudit1/refresh.out).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
b91134f..feba150`, summed by /tmp/bashaudit1/ledger.py — the last
audit's, its unit pattern changed to `bash-` (/tmp/bashaudit1/ledger.out).
A commit that names no unit counts toward the next `done`'s, so
b4ec55f, a state commit with no product lines, counts toward
bash-first-minute.

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| bash-first-minute | 9 | 181 | 62 | 119 | 297 | 3 | 294 |
| bash-limit | 4 | 176 | 59 | 117 | 310 | 19 | 291 |
| bash-person | 6 | 273 | 111 | 162 | 455 | 21 | 434 |
| bash-devices | 11 | 394 | 135 | 259 | 591 | 30 | 561 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 31 | 1024 | 367 | 657 | 1653 | 73 | 1580 |

By file, the largest src nets: cli.rs +128 (bash-devices: `link --device`,
`own_person`, `mint`) and +81 (bash-person), core membership.rs +112
(a1d0ba9, the person as their first key), node.rs +103
(bash-first-minute, `Unlisted` and `ask_lends`), lend.rs and home.rs +62
each (bash-limit), core register.rs +54 (bash-devices).

The size budget, measured by scripts/lean_check.py at HEAD over a `git
archive` tree of each commit (/tmp/bashaudit1/lean-points.out):

| at | svrngs-core | repository |
|---|---:|---:|
| b91134f, range start | 2971 / 3080 | 14835 / 15350 |
| 42f6adc, bash-first-minute's last | 2971 | 14907 |
| d55ece3, bash-limit's last | 2971 | 14986 |
| 83e4153, bash-person's last | 2981 | 15079 |
| 7187d67, svrngs-core's cap to 3,080 (the operator, 2026-10-12) | 2981 | 15079 |
| a1d0ba9, a person is their first key | 3051 | 15154 |
| 3078eab, the register and the door key on the person | 3051 | 15164 |
| c2e074a, `link --device` | 3055 | 15247 |
| feba150, range end | 3055 | 15247 |
| 635b782, this audit's last | **3046 / 3080** | **15240 / 15350** |

The range nets +412 by the budget's count, +84 of it in svrngs-core.
This audit takes back 7: 9 in the core (7b0a59b), +2 for a trace
(635b782). **34 lines are left in svrngs-core and 110 in the repository**
(#80).

### (2) Clones (`code dry-report`)

**The exact tier was checked against a plant.** The plant was a verbatim
copy of 8558ebe's `change` (home.rs:192-214, its doc line included),
appended to rail.rs in the indexed tree, followed by `refresh`. The scoped
report gave **1 new group**: `change — home.rs:193 == rail.rs:626`
(/tmp/bashaudit1/plant-node.md). With the plant removed and `refresh` run
again, that group was **gone** (/tmp/bashaudit1/unplant-node.md). Both
lines are the `fn`'s own, so the exact tier's off-by-one (I1) is fixed in
this binary. `converge noun` still has it.

Each dir the range touched was run scoped at 0.95
(/tmp/bashaudit1/clean-*_0.95.md). Every side was blamed against the
range's commits (/tmp/bashaudit1/clusters-scoped.out):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 449 | 5 | 0 | none |
| crates/svrngs-node/ | 474 | 1 | 0 | none |
| crates/svrngs-runtime/ | 135 | 1 | 0 | none |
| crates/svrngs/ | 79 | 0 | 0 | none |
| apps/jellyfin/mw/ | 59 | could-not-judge (I4) | 0 | none |
| apps/jellyfin/check/ | 57 | could-not-judge | 2 | none |
| apps/web-conformance/ | 11 | could-not-judge | 0 | none |
| web/ | 97 | could-not-judge (I4) | 0 | none |
| scripts/ | 15 | could-not-judge | 0 | none |
| crates/svrngs-node/tests/, crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | — |

**No exact or near clone at the instrument's threshold has a side the
range added.**

Below the threshold, over the whole tree (/tmp/bashaudit1/clusters.py →
clusters.out), 12 groups at 0.90 have a side in the range. The six with
`demo/runs/` sides are recorded runs, each kept as run (#61). The rest,
each read:

- origin.rs `kept_to_reads` (:114-150) ~ surface.rs `reads` (:234-271),
  0.912. The range touched only `reads`' `Unlisted` read (f3d0c18). Both
  ask the one decider, `Offered::reads_one`, so this is not one rule written
  twice.
- home.js `uses` (:83-103) ~ web/app.js `uses` (:275-298), 0.925. The two
  pages each reach their lender their own way: the home page through its
  own route, the phone by an iroh dial. Both render through the one `used`.
- sign_in.rs `of` ~ `forget` and redeem.rs `router` ~ `ask_route`. These
  are neighbours whose spans overlap (I9), not clones.
- register_tests.rs :384-405 ~ :407-429. Two tests, each stating a case.

At 0.80, the groups with a range side are topic clusters (53 symbols of
redemption and register tests, 39 of seal and sync tests), and none of
them is one behaviour written twice.

The tests scopes are could-not-judge (I19), so a line-window pass stood
in for them: 6 non-trivial lines, whitespace stripped, over test code,
each span blamed against the range (/tmp/bashaudit1/windows.py →
windows.out: 181 hits, 80 spans, 20 new). Two of the 20 were fixed here:

- crates/svrngs-node/tests/home.rs :160-178 == :420-438 == :610-628 ==
  lend_origin.rs:134-152 (the second home.rs copy is 8558ebe's). In
  home.rs this is now one `alex_lets_sam_in`, byte for byte lend_origin's
  (4f221f7).
- crates/svrngs/tests/join_page.rs :92-108 == :265-281 (the second copy is
  c2e074a's), install.rs's `sams_computer` written out. The function is now
  in crates/svrngs/tests/common/mod.rs and used by both binaries (a85a146).

The rest of the 20 are each test stating its own case: the phone's
spawn and read in join_page.rs (:126-135 == :286-295, :179-188 ==
:336-344), the browser main.rs drives (:1505-1514 == :1743-1752), a
six-line `.send().await.unwrap().json()` chain in five binaries,
join_by_link.rs's new-group set-up, and install.rs's two folder asserts
(:379-387 == :446-454).

### Reading the range

- **Never silently substitute** (AGENTS.md 6). Each `.ok()`,
  `unwrap_or`, `unwrap_or_default` and `??` the range added to product
  code (/tmp/bashaudit1/range-src.diff) carries its absence:
  - redeem.rs:378-379: a 409 body this build cannot read becomes "a name
    this build cannot read", `NoAnswer`, never a name.
  - core membership.rs: a refused redemption names no device person; a key
    with no device binding is its own first key.
  - cli.rs `full_name`: a system with no full name falls to `$USER`.
  - iroh.rs `closes_unadmitted`: a connection still open is "not that",
    traced.
- **Glassbox** (AGENTS.md 1). cli.rs `user_name` chose between the full
  name and `$USER` with no trace of which it took. Fixed in 635b782.
- **A person's words** (fence 5). The range's quoted strings in cli.rs,
  share.rs, home/, home.rs, surface.rs, redeem.rs, door.rs, door/limit.rs,
  web/app.js, web/index.html and web/install.sh use none of key, member,
  keeper, ring, act or journal as a word outside a trace (`grep -iwE`
  over the added lines: one hit, a `tracing::debug!` in redeem.rs).
- **One decider** (AGENTS.md 8). bash-person moved the account name to
  the door, and it is decided only there (door.rs:143 `user_name`): no other
  product file slices a key to 16 hex for a name. The mw's
  `Device::of` (person.rs:113) slices it for a Jellyfin device id, which
  is another thing; it is #125.

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges b91134f..feba150 -- crates
apps web bridge scripts` matching `\b(struct|enum|trait|type)\s+[A-Z]\w*`
are: `Asked`, `Change`, `Signs`, `Bos`, `Changing`, `Unlisted`,
`Identity`, `Link` and `Redeemed`. `PersonKey`'s line moved, but the type
was already there at b91134f (door.rs:731). Each name was run through
`converge noun <Name> --corpus-id tree-feba150` (/tmp/bashaudit1/nouns.out):

| noun | converge (production) | read |
|---|---|---|
| Link | 2: core link.rs:194, core membership.rs:195 | a1d0ba9's alias was a second projection of the fold's `Invite`s, beside `Membership::invites`, built from the same ops at the end of the same function. **One map now, 7b0a59b**; the alias is gone. |
| Asked | 2: core joining.rs:72, node.rs:630 | Two meanings: the issuer's answer to a joiner, and f3d0c18's account of how a computer answered what it lends. **Renamed `Answered`, ee4792c.** |
| Change | 2: core synced.rs:101, home.rs:178 | Two meanings: one file's change, and 8558ebe's change a person makes to a lend. **Renamed `LendChange`, ee4792c**; its variants' wire is unchanged. |
| Redeemed | 2: node redeem.rs:88, core membership.rs:199 | Two meanings: what a redemption did, and a1d0ba9's alias for a redemption judged on what it alone says. **Renamed `Judged`, ee4792c.** |

`Signs`, `Changing`, `Unlisted` and `Identity` have one definition each.
`Bos` belongs to a test (door/main.rs:279) and has 0 in production.
converge's lines are still one early (I1: `Asked` :71 for :72, :629 for
:630, and so on).

### Fixed here

- 7b0a59b: the fold reads its `Invite`s once. svrngs-core 3055 → 3046;
  167 + 16 core tests unmodified.
- ee4792c: `Answered`, `LendChange` and `Judged`, renames only.
- 4f221f7: home.rs's `alex_lets_sam_in`, tests only.
- a85a146: `sams_computer` in crates/svrngs/tests/common, tests only.
- 635b782: `user_name`'s choice said at debug; decisions unchanged.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 125 | 7, 6 | apps/jellyfin/mw/src/person.rs:113 (`Device::of`, `svrngs-<16 hex of x-svrngs-key>`); crates/svrngs-node/src/door/sign_in.rs:262 (`Whose`, keyed by `PersonKey`) | fixed (0e04c99; bash-two-at-once) | Since bash-devices, `x-svrngs-key` is the person's key, and the sign-in keeps one answer per person by design (bash-devices (c)). So a person's computer, laptop and phone are one Jellyfin token and one Jellyfin device id. What Jellyfin does with two of them playing at once under one session is unmeasured: demo/runs/devices/2026-10-12/README.md asks about accounts, the sign-in and the limit, never two at once. A run with two of one person's devices playing at once says whether this holds. Run: demo/runs/two-at-once/2026-10-12/ — Ana's laptop and phone each played, seeked and kept their own film, but Jellyfin listed one session for both, its now-playing changing 24 times in 180 s, and the laptop's stop blanked it while the phone played. 0e04c99 tells a lend the device asking (`x-svrngs-device`, door.rs `DEVICE_HEADER`), keeps a sign-in a device, and names the Jellyfin device from it; the same steps after (demo/runs/two-at-once/2026-10-12-b/): two sessions in each of 89 reads, each its own film, one account, the sign-in asked once a device. |
| 31 | 8 | crates/svrngs-node/tests/home.rs `alex_lets_sam_in` ~ lend_origin.rs:131-154, with `signing`, `no_seed` and `start` (home.rs:30-54 ~ lend_origin.rs:31-54) across the two binaries, and removal.rs:19-49 ~ canaries.rs:22-54 ~ group_not_mesh.rs:26-59 ~ record_api.rs:27-53 as before; crates/svrngs/tests/common/mod.rs `sams_computer` ~ phone_dry.rs:48-68 | not fixed | Grown by 8558ebe and c2e074a. Within each file it is now one function: home.rs (4f221f7), and join_page.rs with install.rs through crates/svrngs/tests/common (a85a146). Across svrngs-node's test binaries it is still copies. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 34 lines left in svrngs-core (3046 / 3080) and 110 in the repository (15240 / 15350). Three rows follow: bash-rerun, which adds to product only its driver's fixes, and 5a15d49's bash-same-name and bash-film-load. |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 103, 104, 113, 117 | as recorded | as recorded | not fixed | Not touched by this range's decisions. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code dry-report` exact tier; `code converge noun` | **The dry-report half is fixed** in sha256 585edd51…: the plant was reported at the planted `fn`'s own line (rail.rs:626) and the original at its own (home.rs:193). converge is still one line early. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". The exact tiers of web/, apps/ and scripts/ say could-not-judge, which they are. |
| I9 | 6, 7 | dry-report near tier | Still present: neighbours' spans overlap (sign_in.rs `of` :334-353 and `forget` :351-366; redeem.rs `router` :124-135 and `ask_route` :131-144), and the neighbours pair at 0.91-0.93. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said: "could-not-judge". The line-window pass stood in. |

## REVIEW-audit-bash-auto-2 (2026-10-12, range 98991f0..6fc3de4)

The range starts at 98991f0, REVIEW-audit-bash-auto-1's hash in
ralph/next/bash/STATE.md, and ends at 6fc3de4, the commit that made this
row. `git rev-list --count 98991f0..6fc3de4` is 33, no merge among them
(`--merges`: 0), all on main. Four units: bash-rerun, bash-same-name,
bash-film-load and bash-two-at-once. Two state commits after the last
audit's own (f8b5c8c, b6e03e7) count toward it and hold no product lines.
The audit's own commits: b0e1abf (the relay queue's three files, found
untracked in the tree), 70c0d83 (the row's two paths, as 8849ea0 did for
the last), f4eb063 and 633b755 (test moves, behaviour kept). It opens
#126, grows nothing, and closes none.

The instrument is ~/dev/commonwealth-ai/target/debug/sovereign-cli,
sha256 68746bcf… (`shasum -a 256`, built 2026-10-12 14:56). That is not
the last audit's binary (585edd51…), so its exact tier was checked against
a plant first (below). The embedder was the daemon on :9741 serving
qwen-embedding-0.6b, 1024-d, the last audit's (/tmp/bashaudit2/index.out).
The corpus is 6fc3de4's tree, `git archive` into
/tmp/bashaudit2/tree-6fc3de4: `code index . --corpus-id tree-6fc3de4`
gave 3559 chunks in 595 s. `refresh` gave rust 14044 symbols, with
typescript and python "skipped (export failed)" (I4;
/tmp/bashaudit2/refresh.out).

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
98991f0..6fc3de4`, summed by /tmp/bashaudit2/ledger.py, the last audit's
unchanged (/tmp/bashaudit2/ledger.out). A commit that names no unit counts
toward the next `done`'s. Product is crates/, apps/, web/ and bridge/;
demo/ and scripts/ are apart (demo/runs/c +7781, demo/runs/film-load
+16304, demo/runs/two-at-once +6245, demo/c.md +11 −6, scripts/check.sh
+8 −1).

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-bash-auto-1's state commits | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| bash-rerun | 6 | 0 | 0 | 0 | 0 | 0 | 0 |
| bash-same-name | 5 | 246 | 96 | 150 | 415 | 30 | 385 |
| bash-film-load | 11 | 4 | 1 | 3 | 209 | 63 | 146 |
| bash-two-at-once | 8 | 141 | 40 | 101 | 37 | 13 | 24 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 33 | 391 | 137 | 254 | 661 | 106 | 555 |

bash-rerun added to product nothing, as its row says (a run and its
driver, under demo/runs/). By file, the largest src nets: apps/jellyfin/mw
person.rs +72 (0e04c99, `DeviceKey` and `x-svrngs-device`), cli.rs +68
(c0ba4f7, `--joined`, `joined_of`, `joined_said`), core membership.rs +47
(6fdf37a, `Joined`, `admitted_at`, `one_named`), door.rs +24 and
door/sign_in.rs +2 (0e04c99, 80ef1fb), home.js +9, removal.rs +8, core
register.rs +9. crates/svrngs/Cargo.toml's +4 is chrono, with its reason
(c0ba4f7).

The size budget, measured by scripts/lean_check.py over a `git archive`
tree of each commit (/tmp/bashaudit2/lean-points.out):

| at | svrngs-core | repository |
|---|---:|---:|
| 98991f0, range start | 3046 / 3080 | 15240 / 15350 |
| 5b5b87c, bash-rerun done | 3046 | 15240 |
| 21a936d, bash-same-name done | 3072 | 15331 |
| 3f14a7d, bash-film-load done | 3072 | 15334 |
| 6fc3de4, range end | 3072 | 15343 |
| 633b755, this audit's last | **3072 / 3080** | **15343 / 15350** |

The range nets +103 by the budget's count, +26 of it in svrngs-core;
apps/ is outside the count, so person.rs's +72 is not in it. This audit's
moves are in tests, which the budget does not count. **8 lines are left
in svrngs-core and 7 in the repository** (#80).

### (2) Clones (`code dry-report`)

**The exact tier was checked against a plant.** The plant was a verbatim
copy of c0ba4f7's `removal` (cli.rs:707-746, its doc lines included),
appended to crates/svrngs/src/share.rs in the indexed tree, followed by
`refresh`. The scoped report gave **1 group**: `removal —
crates/svrngs/src/cli.rs:710 == crates/svrngs/src/share.rs:198`, which
/tmp/bashaudit2/clusters.py marked RANGE on cli.rs's side
(/tmp/bashaudit2/plant-cli.md). With the plant removed and `refresh` run
again, the tier found **0** (/tmp/bashaudit2/unplant-cli.md). Both lines
are the `fn`'s own, so I1's dry-report half stays fixed in this binary.

Each dir the range touched was run scoped at 0.95
(/tmp/bashaudit2/clean-*_0.95.md), and every side was blamed against the
range's commits (/tmp/bashaudit2/clusters-scoped.out):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 451 | 5 | 0 | none |
| crates/svrngs-node/ | 475 | 1 | 0 | none |
| crates/svrngs-runtime/ | 135 | 1 | 0 | none |
| crates/svrngs/ | 81 | 0 | 0 | none |
| apps/jellyfin/mw/ | 62 | could-not-judge (I19) | 0 | none |
| apps/jellyfin/check/ | 57 | could-not-judge (I4) | 2 | none |
| web/ | 97 | could-not-judge (I4) | 0 | none |
| scripts/ | 15 | could-not-judge (I4) | 0 | none |
| crates/svrngs-node/tests/, crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | — |

**No exact or near clone at the instrument's threshold has a side the
range added.**

Below the threshold, over the whole tree at 0.90
(/tmp/bashaudit2/clean-all-0.9.md → clusters.out), 56 groups have a side
in the range. 51 are recorded runs under demo/runs/, each kept as it ran
(#61): the rerun's driver beside the dry run's, and two-at-once's -b
driver beside its first. The other five, each read:

- home.js `uses` (:92-112) ~ web/app.js `uses` (:275-301), 0.924. The
  range touched only app.js's (b7126e2, the body read before the dial
  closes). As the last audit read it, the two pages reach their lender
  their own ways, and both render through the one `used`.
- door/sign_in.rs `of` (:336-355) ~ `forget` (:353-368), 0.924. These are
  neighbours whose spans overlap (I9), not clones.
- apps/jellyfin/mw person.rs `as_str` :37-45 ~ :52-60, 0.930: the
  accessors of `PersonKey` and 0e04c99's `DeviceKey`. `DeviceKey::parse`
  asks `PersonKey::parse` (:52), so the spelling of a key is decided once.
- register_tests.rs :426-450 ~ :452-477, and person.rs's tests :240-262 ~
  :264-283. Two tests each, each stating its case.

The tests scopes are could-not-judge (I19), so the last audit's
line-window pass stood in for them: 6 non-trivial lines, whitespace
stripped, over test code, each span blamed against the range
(/tmp/bashaudit2/windows.py → windows.out: 179 hits, 80 spans, 5 new).
Two of the five were fixed here:

- crates/svrngs/tests/remove.rs :169-180 == :253-264 (the second copy is
  b8b700c's): Sam's computer started and trip made, and beside it Bo let
  in by hand. Each is now one function in the file, `sams_trip` and
  `let_bo_in` (f4eb063).
- crates/svrngs-node/tests/browser/main.rs :1482-1488 == :1514-1520 ==
  :1556-1562, and :1513-1518 == :1555-1560 (b8b700c's): the home page's
  address and its `drive`. The same steps stood at four more sites with
  other selectors, and all seven are now `home_page` (633b755).

The other three are tests each stating its own case:
register_tests.rs :329-334 == :706-711, a `Removal` of dee committed in
two tests; and rail/tests.rs :271-277 == record_api.rs :243-248,
bo's `joined` read from the people answer, one in svrngs-node's unit tests
and one in an integration binary, which share no module.

### Reading the range

- **Never silently substitute** (AGENTS.md 6). Each `.ok()`, `unwrap_or`,
  `map_or` and `??` the range added to product code
  (/tmp/bashaudit2/range-src.diff) carries its absence:
  - core `Membership::joined`: a first key with no `Admit` time is the
    seed row's person, `Started`. `walk` starts from the seed and the
    whole order (membership.rs:449-470), so no admitted person lacks a
    time.
  - cli.rs `joined_said`: a time chrono cannot hold is "at a time this
    computer cannot write", never a number.
  - web/app.js `uses`: a body not read on a failed answer is `null`, and
    the next line throws on `!res.ok` before it is read.
  - mw person.rs `Who::of`: no `x-svrngs-device`, or one that is not a
    key, is refused `NoDevice` / `NotADevice`, never taken as the
    person's key.
  - **One that does not hold: #126.** cli.rs `joined_said` writes a time
    in this computer's zone with no offset, and `joined_of` reads it
    back with `.single()`. In the hour a fall-back repeats, two times an
    hour apart are written alike, and reading either back gives `None`.
    The refusal then offers a `--joined` that the command refuses as "not
    when someone joined". Reproduced with both bodies verbatim
    (/tmp/bashaudit2/dst/src/main.rs, `TZ=America/Los_Angeles cargo run`):
    `1793521800 said "2026-11-01 01:30:00" -> read back None`,
    `1793525400 said "2026-11-01 01:30:00" -> read back None`, and
    `1791831600 said "2026-10-12 12:00:00" -> read back Some(1791831600)`.
- **Glassbox** (AGENTS.md 1). Each new decision is traced at debug:
  `one_named` ("membership: the one person a name and when they joined
  name, or when each joined"), `Removal::decide` with its verdict,
  removal.rs's refusal, the sign-in's kept and asked answers by device,
  and the mw's sign-in with `from` the device. `joined_of` refuses in
  words to the person who typed it.
- **A person's words** (fence 5). The range's added quoted strings in
  product code use none of key, member, keeper, ring, act or journal as
  a word outside a trace, a test, the people answer's JSON keys and the
  mw's refusals to the door (`grep -iwE` over the added lines: those
  six hits only).
- **One decider** (AGENTS.md 8). The door alone says which device asks
  (door.rs:135 `DEVICE_HEADER`, set with the other three in `forward`).
  The mw parses the header and does not derive it. The time a person
  joined is written in two places, cli.rs `WHEN` with `joined_said`
  (:1167-1192) and home.js `people` (:60-62), each saying it is the
  other's form. The browser test pins the page's shape
  (main.rs `the_home_page_tells_two_people_of_one_name_apart`,
  `9999-99-99 99:99:99`), but no test pins the value against what
  `--joined` takes. This is #126's second site.

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 98991f0..6fc3de4 -- crates
apps web bridge scripts` matching `\b(struct|enum|trait|type)\s+[A-Z]\w*`
are `DeviceKey` (twice), `Identity`, `Whose` and `Joined`. `Identity` and
`Whose` were already there at 98991f0 and changed shape (four headers; a
device's key). Each was run through `converge noun <Name> --corpus-id
tree-6fc3de4` (/tmp/bashaudit2/nouns.out):

| noun | converge (production) | git grep | read |
|---|---|---|---|
| DeviceKey | 1: door.rs:786 | 2: crates/svrngs-node/src/door.rs:787, apps/jellyfin/mw/src/person.rs:48 | converge does not see the mw (I19, its own workspace). One key, two programs either side of the door's `x-svrngs-device` header, as `PersonKey` was judged (#50; this file, "PersonKey … No change"). The mw parses what the door says. No change. |
| Joined | 1: core membership.rs:131 | 1: :132 | One. |
| Identity | 1: door.rs:141 | 1: :142 | One. |
| Whose | 1: door/sign_in.rs:262 | 1: :263 | One. |

converge's lines are still one early (I1: :786 for :787, :131 for :132).

### Fixed here

- f4eb063: remove.rs's `sams_trip` and `let_bo_in`, tests only. `cargo
  test -p svrngs --test remove`: 4 passed before and after.
- 633b755: browser main.rs's `home_page`, tests only. `cargo test -p
  svrngs-node --test browser the_home_page`: 16 passed (8 tests, two
  engines) before and after.

No product code was moved: the range's product clones are none at the
instrument's threshold, and #126 is a change in behaviour.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 126 | 6, 8 | crates/svrngs/src/cli.rs:1167-1192 (`WHEN`, `joined_of`'s `.single()` at :1176, `joined_said`); crates/svrngs-node/src/home/home.js:60-62 (`people`, `toLocaleString('sv-SE')`) | fixed: 63f24ce (start-findings (a)): `%Y-%m-%d %H:%M:%S %z`, its one home svrngs-node home.rs `WHEN`, which cli.rs reads and home.js writes; crates/svrngs/tests/joined.rs at a fall-back hour in `TZ=America/Los_Angeles`, and the browser test reading each time back by `WHEN` to its `Admit` | When someone joined is written in this computer's zone with no offset, in two places (the command and the home page), and read back by `--joined` only when the zone maps it to a single instant. In the repeated hour of a fall-back night, two people of one name admitted an hour apart are said alike, and the `--joined` the refusal offers is refused (reproduced above, /tmp/bashaudit2/dst). DESIGN §6 says two let in the same second cannot be told apart (docs/DESIGN.md:1353), but not this hour. A fix says the time so it reads back to one instant, for example with its offset, in both places and in one form that a test pins against `joined_of`. A test planted at a fall-back time in a fixed `TZ` shows it red first. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 8 lines left in svrngs-core (3072 / 3080) and 7 in the repository (15343 / 15350). The next queue, relay, carries the operator's raise to 15,800 and 3,130 (ralph/next/relay/CHARTER.md, "The budget is raised for this queue"), taken by its first row that needs it. |
| 31 | 8 | as recorded | not fixed | Not grown by this range. remove.rs and browser main.rs are each one function within their file now. |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 103, 104, 113, 117 | as recorded | as recorded | not fixed | Not touched by this range's decisions. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | The dry-report half is still fixed in sha256 68746bcf…: the plant was reported at the planted `fn`'s own line (share.rs:198). converge is still one line early. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". |
| I9 | 6, 7 | dry-report near tier | Still present: sign_in.rs `of` :336-355 and `forget` :353-368 overlap and pair at 0.924. |
| I19 | 6 | dry-report, `tests/` scopes; converge and the exact tier in apps/jellyfin/mw | Still present, and said as could-not-judge. converge gives `DeviceKey` 1 where git grep finds 2. The line-window pass stood in for the tests scopes. |

## REVIEW-audit-relay-auto-1 (2026-10-13, range 6cc26d0..f3d25cf)

No earlier `REVIEW-audit-` row exists in ralph/next/relay/STATE.md, so the
range starts where this queue does. The queue's first commit, 50643fd,
comes before REVIEW-audit-bash-auto-2's own four commits
(`git log --oneline 50643fd..6cc26d0`: 633b755, f4eb063, 6e0c2cf, 6cc26d0).
That audit already read them, so the range starts after its `done`,
6cc26d0, and ends at f3d25cf, the commit that made this row.
`git rev-list --count 6cc26d0..f3d25cf` is 31, with no merge among them
(`--merges`: 0), all on main. It covers four units: relay-n0-off,
relay-in-record, relay-verb and relay-default.

This audit's own commits are 41808f5 (the row's two paths, as 70c0d83
corrected them for the last audit) and 0551e2d (a test move, behaviour
kept). It opens #127-#131, updates #80, and closes none.

The instrument is ~/dev/commonwealth-ai/target/debug/sovereign-cli,
sha256 68746bcf…, the last audit's binary (`shasum -a 256`). Its exact
tier was still checked against a plant (below). The embedder was the
daemon on :9741 serving qwen-embedding-0.6b, 1024-d. The corpus is
f3d25cf's tree, put into /tmp/relayaudit1/tree-f3d25cf by `git archive`.
`code index . --corpus-id tree-f3d25cf` gave 3677 chunks in 501 s
(/tmp/relayaudit1/index.out). `refresh` gave rust 14470 symbols, with
typescript and python "skipped (export failed)" (I4;
/tmp/relayaudit1/refresh.out).

### (1) Net lines per unit, product code

The counts come from `git log --reverse -M --numstat --format='@@%h %s'
--no-merges 6cc26d0..f3d25cf`, summed by /tmp/relayaudit1/ledger.py.
That is the last audit's script with its unit pattern changed from
`bash-` to `relay-` (/tmp/relayaudit1/ledger.out). A commit that names no
unit counts toward the next `done`. Product is crates/, apps/, web/ and
bridge/. Everything else is counted apart:

- demo/runs/n0-off +1112, demo/runs/relay-in-record +2250 and
  demo/runs/relay-rate +62
- docs/ +177 −34, scripts/ +90 −3, ARCHITECTURE.md +5 −2, ralph/ +5 −4
  and Cargo.lock +1

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| relay-n0-off | 7 | 0 | 0 | 0 | 84 | 0 | 84 |
| relay-in-record | 12 | 314 | 87 | 227 | 194 | 18 | 176 |
| relay-verb | 6 | 270 | 19 | 251 | 379 | 3 | 376 |
| relay-default | 5 | 168 | 74 | 94 | 425 | 45 | 380 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 31 | 752 | 180 | 572 | 1082 | 66 | 1016 |

relay-n0-off added nothing to product, as its row says: a run, and the
`n0_off` test binary its harness drives. The largest src nets, by file:

- crates/svrngs/src/relay.rs +181 (ba93f15, `svrngs relay`)
- cli.rs +115 over three units (`reach`, `DEFAULT_RELAY`, `new --relay`,
  `wait_online`, the relay verb's parse)
- node.rs +82 (42e36ca, e16d52e, 6cd1727, f3ec329: `say_relay`,
  `contact`, each dial by `Contact`)
- web/install.sh +29, group.rs +32 (`write_relay`, `read_relay`)
- core membership.rs +26, register.rs +26 (`RelayAt`), lib.rs +16
  (`RailAct::Relay`, `RELAY_LINE_VERSION`) and relay.rs +12 (`parse`)

web/src/lib.rs nets −2: ab99c34 took the page's n0 path out. The
command's Cargo.toml +7 −3 is iroh-relay's `server` feature and rustls
(db4f8ca, ba93f15).

The size budget, measured by scripts/lean_check.py over a `git archive`
tree of each commit (/tmp/relayaudit1/lean-points.out):

| at | svrngs-core | repository |
|---|---:|---:|
| 6cc26d0, range start | 3072 | 15343 |
| d0300f8, relay-n0-off done | 3072 | 15343 |
| bb6652e, relay-in-record done | 3121 | 15486 |
| f2da6c7, relay-verb done | 3121 | 15663 |
| b4e17ed, relay-default done | 3121 | 15776 |
| f3d25cf, range end | 3121 | 15776 |
| 0551e2d, this audit's last | **3121 / 3130** | **15776 / 15800** |

The caps moved from 3,080 and 15,350 to 3,130 and 15,800 in e05883e, the
operator's raise for this queue. Over the range the budget's count nets
+433, +49 of it in svrngs-core. This audit's move is in tests, which the
budget does not count. **9 lines are left in svrngs-core and 24 in the
repository** (#80). The queue's last row, relay-run, adds nothing to
product but the driver's own fixes.

### (2) Clones (`code dry-report`)

**The exact tier was checked against a plant.** The plant was a verbatim
copy of 9a4f755's `reach` (cli.rs:664-693, its doc lines included),
appended to crates/svrngs/src/share.rs in the indexed tree, followed by
`refresh` (+16 symbols). The scoped report gave **1 group**: `reach —
crates/svrngs/src/cli.rs:668 == crates/svrngs/src/share.rs:199`, 2 copies
× 26 lines (/tmp/relayaudit1/plant-cli.md). With share.rs restored
(`diff` against `git show f3d25cf:` is empty) and `refresh` run again
(−16), the tier found **0** (/tmp/relayaudit1/unplant-cli.md).

Each dir the range touched was run scoped at 0.95
(/tmp/relayaudit1/clean-*_0.95.md). Every side was blamed against the
range's commits by /tmp/relayaudit1/clusters.py, the last audit's with
the range changed (clusters-scoped.out):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs-core/ | 457 | 5 | 0 | none |
| crates/svrngs-node/ | 481 | 1 | 0 | none |
| crates/svrngs-runtime/ | 135 | 1 | 0 | none |
| crates/svrngs/ | 89 | 0 | 0 | none |
| web/ | 98 | could-not-judge (I4) | 0 | none |
| scripts/ | 15 | could-not-judge (I4) | 0 | none |
| crates/svrngs-node/tests/, crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | — |

The seven exact groups are the last audit's, each three lines (api.rs
`addr`, core `is_empty`, `lines`, `signed_by_link`, `as_str`, and two
test helpers). **No exact or near clone at the instrument's threshold
has a side the range added.**

Below the threshold, the whole tree was run at 0.90
(/tmp/relayaudit1/clean-all-0.9.md → clusters.out). It gave 149 groups,
15 with a side in the range. 14 of those are recorded runs: relay-in-record's
driver is relay-n0-off's, copied unchanged before it ran (32785ec), and
each run is kept as it ran (#61). The 15th is home.js `uses` (:92-112)
~ web/app.js `uses` (:284-310) at 0.922. The range touched only
app.js's (42e36ca: the lender's relay passed to `dial`). As the last
audit read it, the two pages reach their lender their own ways. The home
page asks its own computer `/choices?lend=`, and that computer dials
through `Host::contact`. Both render through the one `used`.

The tests scopes are could-not-judge (I19), so the line-window pass
stood in for them, the last audit's script with the range changed
(/tmp/relayaudit1/windows.py → windows.out: 183 hits, 85 spans, 13 new).
One of the 13 was fixed here:

- crates/svrngs/tests/join_by_link.rs :235-244 == :520-529 == :599-608
  == :655-664 == :702-711 == :777-786: Sam makes house with no page and
  no relay. 9a4f755 gave `new_group` a tenth line (`kept`), and rustfmt
  spread each call over ten lines. It is now one function in the file,
  `sams_house` (0551e2d). share.rs :147-157 is the same call in another
  binary, which shares no module with join_by_link.rs.

The other new spans:

- install.rs :18-23 == relay.rs :7-12: each binary's `mod harness`
  header.
- install.rs :256-265 == join_page.rs :128-136, :267-275, and install.rs
  :477-483 == :538-544: each test's own paste and its asserts.
- core membership_tests.rs :84-89 == :269-274: two folds each checked
  over every arrival order.
- browser main.rs :817-822 == :1027-1032: two films, each its own
  `drive`.
- join_by_link.rs :837-843 == new.rs :190-196: a page's group made in two
  binaries.

### Reading the range

- **Never silently substitute** (AGENTS.md 6). Each `.ok()`,
  `unwrap_or`, `map_or`, `??` and `.flatten()` the range added to
  product code was read (/tmp/relayaudit1/range-src.diff):
  - core `relay::parse` traces its refusal before `.ok()`.
  - cli.rs `reach` meets a group that keeps no relay at `DEFAULT_RELAY`,
    which group.rs's header says is the choice.
  - The page's `?? null` and `reach`'s `unwrap_or_else` dial through the
    link's relay when the group names none, and say so
    (`t!("LAYER parse: the group names no relay for …")`).
  - **Two do not hold.** First, #128: `Host::say_relay` lists the groups
    with `.unwrap_or_default()`, and `relay::spawn` asks
    `namespaces_in(..).is_ok_and(..)`. A groups directory that cannot be
    read is then "no groups": nothing is said, untraced, and the relay
    refuses as "this computer is in no group".
  - Second, #129: the relay's `Admits::on_connect` takes `.ok().flatten()`
    of the blocking read. A read that panicked is refused and traced as
    "the key stands in no group held here".
- **Glassbox** (AGENTS.md 1). Each new decision is traced at debug:
  - the fold's `Relay` act and `Membership::relay`
  - `RelayAt::decide` with `writes`
  - `say_relay`'s skip, commit and refusal
  - `Contact::named` with no relay
  - ring sync's wake on the asker's own relay
  - `reach`'s relays and `wait_online`
  - the relay's admit, refuse, start and stop
  - group.rs's kept relay

  `wait_online` also says in a person's words which relay it did not
  reach.
- **A person's words** (fence 5). The range's added quoted strings in
  product code use none of key, member, keeper, ring, act or journal as
  a word outside a trace. The only other hits are an `expect` ("ring's
  protocols", the crypto provider) and deploy-relay.sh's `--ssh-key`,
  which the operator reads (`grep -iwE` over the added lines).
- **One decider** (AGENTS.md 8). Where a computer is reached is folded in
  svrngs-core alone (`RailAct::Relay`, `Membership::relay`, `RelayAt`).
  The node and the page carry the register's `Member::relay`. But
  which group's answer names a key held in several groups is decided
  twice (#127). And a relay as written is read dotless by two functions,
  core's and the node's (#131).
- **Draw the line by what each owns** (principle 12). Since 9a4f755 the
  command never builds n0's path, but svrngs-node still defaults to it,
  and the page compiles it unused (#130).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 6cc26d0..f3d25cf -- crates
apps web bridge scripts` that match `\b(struct|enum|trait|type)\s+[A-Z]\w*`
are `Relay`, `At`, `Admits` (crates/svrngs/src/relay.rs), `RelayAt`
(core register.rs), `Kept` (crates/svrngs/tests/default_relay.rs) and
`type Refusal` (`RelayAt`'s `Command` impl). Each was run through
`converge noun <Name> --corpus-id tree-f3d25cf`
(/tmp/relayaudit1/nouns.out):

| noun | converge (production) | git grep | read |
|---|---|---|---|
| Relay | 1: crates/svrngs/src/relay.rs:46 | 1: :47 | One. Its kin `RelayConfig` (svrngs-node guest.rs:39) is how a node reaches the network; `Relay` is what `svrngs relay` was asked. |
| RelayAt | 1: core register.rs:244 | 1: :245 | One. |
| At | 1: crates/svrngs/src/relay.rs:54 | 2: :55, apps/jellyfin/mw/src/answer.rs:28 | converge does not see the mw (I19). These are two private names in two programs for two things: where a relay is reached, and what every mw answer holds. No change. |
| Admits | 1: crates/svrngs/src/relay.rs:80 | 1: :81 | One. |
| Kept | 1: svrngs-node door/choices.rs:59 | 2: choices.rs:60 (`type`, the kept choices), crates/svrngs/tests/default_relay.rs:31 (the test's trace sink) | converge leaves tests out. These are two private names for two things. The other trace sink, folders.rs:1053 `Said`, is in a test binary that shares no module with this one. No change. |
| Refusal | 2: cli.rs:513, door/sign_in.rs:132 | 2: :514, :133 | Neither is the range's. The range's is `type Refusal = Infallible`, the `Command` trait's associated type, which is no noun (as judged at this file's U7 and v1 audits). |

converge's lines are still one early (I1).

### Fixed here

- 0551e2d: join_by_link.rs's `sams_house`, tests only. `cargo test -p
  svrngs --test join_by_link` gave 10 passed before and after
  (/tmp/relayaudit1/jbl-before.out, jbl-after.out).

No product code was moved. The range's product clones are none at the
instrument's threshold, and #127-#131 each change behaviour, even if
only in a trace.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 127 | 8 | crates/svrngs-node/src/node.rs:595-613 (`standing_peers`: the first group naming the key) and :489-492 (`Host::contact`, by svrngs-runtime register.rs:278 `standing_anywhere`, the first group naming the key); node.rs:872-876 (`Asking`'s `relay`: the first group naming a *relay* for the key) | fixed: 532adb9 (start-findings (b)): `node::relay_of`, the first group naming a relay for the key, asked by the round, the lend asks, a folder's fetch, the origin and the people answer; door/main.rs `a_key_in_two_groups_that_disagree_is_dialled_and_told_by_one_relay`. The live lane (rail.rs `live_peers`) still dials by the relay of the group it offers to, which this finding did not name | Two rules decide which group's relay a key held in several groups is dialled by. The round, the lend asks, a folder's fetch and the origin take the first group that names the key, even when that group names no relay for it. The people answer, which the phone dials by, takes the first group that names a relay. Suppose group A (first) has not yet heard key K's `Relay` act, or holds an older one, and group B names K's relay. Then this computer dials K blind, or through the old relay, while it tells the phone the new one. With n0 off, a blind dial is not made (`Contact::named`'s trace). `say_relay` writes in every group, so the two agree once each group has synced. A fix gives the choice one home, for example in the register beside `standing_anywhere`, with a test of two groups that disagree, red first. |
| 128 | 6, 1 | crates/svrngs-node/src/node.rs:462 (`say_relay`: `self.rail.namespaces().unwrap_or_default()`); crates/svrngs/src/relay.rs:110 (`spawn`: `namespaces_in(root).is_ok_and(..)`) | fixed: bf67398 (start-findings (c)): say_relay traces the listing's error at warn, and `svrngs relay` refuses `GroupsUnread` in a person's words; tests/relay.rs `groups_that_cannot_be_read_are_said_so_never_taken_as_none` | A groups directory that cannot be read (svrngs-runtime lib.rs:147, `RailError::Io`) is taken as no groups. `say_relay` then says nothing, with no trace. `svrngs relay` refuses `RelayForNobody`, "this computer is in no group … join one first", which misdirects whoever runs it. A fix traces and says the read's error, with a test over an unreadable directory, red first. |
| 129 | 6, 1 | crates/svrngs/src/relay.rs:92-99 (`Admits::on_connect`: `stands.await.ok().flatten()`) | fixed: 9bb6df8, after 8d1a700's move (start-findings (d)): `verdict` traces a failed read at warn as "the record could not be read"; relay_tests.rs, a planted panic | A blocking read that panicked (`JoinError`) is refused, and traced as "relay: refused — the key stands in no group held here". The verdict is the safe one. The reason is not: a relay whose record read fails refuses everyone, and says each is in no group. A fix traces the join error apart from the read's `None`. |
| 130 | 12, 6 | crates/svrngs-node/src/guest.rs:63-72 (`RelayConfig::default()`, `n0_services: true`, "the bootstrap default"); crates/svrngs/src/cli.rs:1820-1824 (`link_relay`'s n0 branch); web/src/lib.rs:27-32 (n0's relays and `DotlessLookup` compiled into the page under `#[allow(dead_code)]`); crates/svrngs/src/cli.rs:688-692 (`reach`, `n0_services: false`, the command's only build) | not fixed: a change in behaviour; deciding where n0's path still lives is relay-run's to show | Since 9a4f755 the command never builds n0's path (main.rs asks `cli::reach`). svrngs-node still defaults to it, though, and only tests reach that default: svrngs-node tests/common/member.rs:78 `"n0"`, sy_dee.rs:43, and crates/svrngs/tests/new.rs:250-254. The default is now one nobody chose (DESIGN §10, the operator, 2026-10-12: n0 none of the path). The page compiles code it never calls. "You drew it wrong when the uses go to zero and the ability stays." A fix makes `RelayConfig` have no n0 default, so a caller says which relays it uses, and keeps n0 only where a test or run names it. That could follow relay-run, whose samples show whether anything still reaches n0. |
| 131 | 8, 3 | crates/svrngs-core/src/relay.rs:22-31 (`parse`, b18f187: `raw.parse::<Url>().map(dotless)`, `Option`, its doc "The one reading of a relay a person or a peer wrote"); crates/svrngs-node/src/relay.rs:29-32 (`parse`, ea651e0: `raw.parse::<RelayUrl>().map(dotless)`, `Result`) | fixed: c97b9e3 (start-findings (e)): core's `parse` returns its `url::ParseError`, the node's wraps it; svrngs-core adds no line | A relay as written is read dotless by two functions. They agree today: iroh-base 1.0.2's `RelayUrl::from_str` is `Url::from_str` wrapped (relay_url.rs:38-45), and both drop the dot by core's `dotless`. Core's doc claims to be the one reading, but the command (`--relay`, group.rs `read_relay`, `link_relay`), the node (iroh.rs:188, guest.rs:233) and the page call the node's. A fix has core's return the `url::ParseError`, and the node's wrap it. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 9 lines left in svrngs-core (3121 / 3130) and 24 in the repository (15776 / 15800), the operator's raise for this queue (e05883e). relay-run, the queue's last row, adds nothing to product but its driver's fixes. |
| 31 | 8 | as recorded | not fixed | Not grown by this range. |
| 57, 68, 70, 71, 75, 77, 79, 81, 89, 91, 95, 97, 99, 100, 103, 104, 113, 117, 126 | as recorded | as recorded | not fixed | Not touched by this range's decisions. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | The dry-report half is still fixed in sha256 68746bcf…: the plant was reported at the planted `fn`'s own line (share.rs:199). converge is still one line early (relay.rs:46 for :47, register.rs:244 for :245). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/ and scripts/ exact tiers are could-not-judge. |
| I9 | 6, 7 | dry-report near tier | Still present: sign_in.rs `of` :336-355 and `forget` :353-368 overlap and pair at 0.924. |
| I19 | 6 | dry-report, `tests/` scopes; converge and the exact tier in apps/jellyfin/mw | Still present, and said as could-not-judge. converge gives `At` 1 where git grep finds 2. The line-window pass stood in for the tests scopes. |

## REVIEW-audit-relay-auto-2 (2026-10-13, range 778fd4b..8974e7b)

The previous `REVIEW-audit-` row is REVIEW-audit-relay-auto-1, 778fd4ba4
(ralph/next/relay/STATE.md). The range starts there and ends at 8974e7b,
the commit that made this row. `git rev-list --count 778fd4b..8974e7b` is
10, with no merge among them (`--merges`: 0), all on main. It covers one
unit, relay-run, and the last audit's own `done` (98b32c0).

This audit's own commit before this one is 78cde59: the row's two paths,
corrected as 41808f5 corrected the last audit's. It opens #132 and #133,
updates #130, and closes none.

The instrument is ~/dev/commonwealth-ai/target/debug/sovereign-cli,
sha256 68746bcf…, the last audit's binary (`shasum -a 256`). The embedder
was the daemon on :9741 serving qwen-embedding-0.6b. The corpus is
8974e7b's product: crates/, apps/, web/, scripts/, Cargo.toml and
Cargo.lock, put into /tmp/relayaudit2/prod-8974e7b by `git archive` (267
files). demo/ is left out because the range's demo/ is one recorded run.
`code index . --corpus-id prod-8974e7b` gave 3202 chunks in 426 s
(/tmp/relayaudit2/index.out). `project refresh` gave rust 14477 symbols,
with typescript and python "skipped (export failed)" (I4;
/tmp/relayaudit2/refresh.out).

### (1) Net lines per unit, product code

The counts come from `git log --reverse -M --numstat --format='@@%h %s'
--no-merges 778fd4b..8974e7b`, summed by /tmp/relayaudit2/ledger.py, the
last audit's script unchanged (/tmp/relayaudit2/ledger.out). Product is
crates/, apps/, web/ and bridge/. Everything else is counted apart:

- demo/runs/c/no-n0-2026-10-13 +9186: predictions, the driver (run.py,
  c.mjs, media.sh, copied from dry-2026-10-12-rerun/ and changed as
  PREDICTIONS.md says, 82b21f5), its data and README
- docs/DESIGN.md +22 −9 and ralph/next/relay/STATE.md +4 −3

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-relay-auto-1 (its `done`) | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| relay-run | 8 | 0 | 0 | 0 | 24 | 2 | 22 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 10 | 0 | 0 | 0 | 24 | 2 | 22 |

relay-run added nothing to product, as its row says. Its one test change
is ee76d6d: crates/svrngs/tests/n0_off.rs +24 −2, the ignored
`the_verbs_relay`, which the run's driver started and `scripts/check.sh`
never runs.

The size budget, measured by scripts/lean_check.py over a `git archive`
tree of each end (/tmp/relayaudit2/lean-778fd4b, lean-8974e7b):

| at | svrngs-core | repository |
|---|---:|---:|
| 778fd4b, range start | 3121 | 15776 |
| 8974e7b, range end | **3121 / 3130** | **15776 / 15800** |

The range and this audit move no counted line. **9 lines are left in
svrngs-core and 24 in the repository** (#80), and relay-run was the
queue's last row.

### (2) Clones (`code dry-report`)

The only crate dir the range touched is crates/svrngs, in its tests
alone (`git diff --stat 778fd4b 8974e7b -- crates apps web scripts`:
crates/svrngs/tests/n0_off.rs only).

**The exact tier was checked against a plant**, in the indexed tree,
with `project refresh` after each change:

- relay.rs `run` (:157-180) appended to crates/svrngs/src/share.rs was
  **not** found (0 groups, /tmp/relayaudit2/plant.md). The plant was
  mine and malformed: relay.rs has 181 lines, and `run`'s closing brace is
  :181, so the copy did not parse. This is not the instrument's miss.
- 9a4f755's `reach` (cli.rs:664-693, the last audit's plant) gave **1
  group**: `reach — crates/svrngs/src/cli.rs:668 == crates/svrngs/src/share.rs:199`,
  2 copies × 26 lines (/tmp/relayaudit2/plant2.md).
- relay.rs `spawn` (:106-155, an async fn) gave **1 group**: `spawn —
  crates/svrngs/src/relay.rs:107 == crates/svrngs/src/share.rs:196`,
  2 copies × 49 lines (/tmp/relayaudit2/plant3.md).
- With share.rs restored (`diff` against `git show 8974e7b:` is empty)
  and `refresh` run again (14477 symbols, −20), the tier found **0**
  (/tmp/relayaudit2/unplant.md).

Scoped at 0.95 (/tmp/relayaudit2/clean-*_0.95.md):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs/ | 89 | 0 | 0 | none |
| crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | — |

**No exact or near clone has a side the range added**, and the range
added no product symbol for one to have. The whole-tree pass at 0.90 was
not run: with no product line added, no group could have a side in the
range.

The tests scope is could-not-judge (I19), so the line-window pass stood
in for it: 6 non-trivial lines, whitespace stripped, over test code at
8974e7b. It is the last audit's script with the range and the blamed
commit changed (/tmp/relayaudit2/windows.py → windows.out): **177 window
hits, 81 spans, 0 new.** `the_verbs_relay` repeats no window of
`the_relay` or `the_command` beside it.

The driver repeats the rerun's. `diff` counts 205 changed lines of
run.py's 622, 13 of c.mjs's 229 and 7 of media.sh's. A recorded run is
kept as it ran (#61), so no change.

### Reading the range

- **Never silently substitute** (AGENTS.md 6). relay-run's README names
  two failures, and neither is a row anywhere (`git grep -n
  no-n0-2026-10-13 8974e7b -- ralph/`: none). Both are opened here. #132: a phone shown the group's join page in a lend's place when
  the relay refused the lend's key. #133: a newcomer the relay refused,
  told that the sender's computer did not answer.
- **Cite, don't recall** (AGENTS.md 4). Five of the README's numbers
  were read in its `data/summary.json`, and each matched:
  - `lsof.sampled` 561, with `lsof.n0_hits` and `ours_hits` `[]`
  - `n0_in_traces` empty for all eight
  - `newcomers.*.done` 20548, 18519 and 17933
  - `cy.film.playing` 3200
  - `group.cy.film.stopped`, the join page's words

  ce7aaaf's DESIGN §8 and §11 change rewrites rows in place. It adds no
  section.
- **Draw the line by ownership** (principle 12). relay-run's samples are
  the evidence #130 waited for. Nothing the command builds reached n0: 561
  samples, `n0_hits` empty. Yet svrngs-node's `RelayConfig::default()` is
  still n0's (guest.rs:63-72), reached only by tests (below).
- **Glassbox** (AGENTS.md 1). `the_verbs_relay` traces as the verb does
  (`traced()`) and says its refusal's words before it exits with the
  verb's exit code (n0_off.rs:95-106).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 778fd4b..8974e7b -- crates
apps web scripts demo` match `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`
**0 times** (/tmp/relayaudit2/new-types.out). The range defines no type,
so there is no noun to converge. `the_verbs_relay` uses relay.rs's
`Relay` and `At`, which the last audit ran (one definition each, with
`At`'s second in apps/jellyfin/mw a different thing).

### Fixed here

Nothing. The range has no product line, and its test adds no repeated
span. #132 and #133 each change what a person sees.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 132 | 6, 1 | web/app.js:21-30 (a lend opened from a fresh key holding the person's pass, not their key; web/lend.js:9) and crates/svrngs/src/relay.rs:83-100 (`Admits::on_connect`: a key standing in no group refused, `Access::Deny { reason: None }`) | not fixed: a change in behaviour (who a relay admits, and what the phone says) | A relay that admits only its group lets a phone in and plays it nothing. relay-run saw it (demo/runs/c/no-n0-2026-10-13/README.md, finding 1; DESIGN §11 risk 5). Cy's key `70d8…` was admitted, while the lend's fresh key `2b0c…` was refused 14 times. The film's address showed the join page, "What’s your name? … Join", in the film's place, and the music stopped at "Opening…" (`group.cy.film.stopped`, `music.stopped`). The phone is not told that the relay refused it. svrngs's own relay is `--open`, so a group on the default is untouched. A fix has two parts. The relay admits a key holding a pass signed by a key standing in a group, read from the relay's request if iroh-relay carries one; otherwise how the pass is shown is a decision of its own. And the page says a refusal at the relay as that. Each part needs a test, red first: the browser test with a relay for the group alone. |
| 133 | 6 | crates/svrngs/src/cli.rs:536 ("the computer that sent this link cannot be reached; it may be off, so try again later") and web/app.js:157 ("The computer that sent this link did not answer through …"); crates/svrngs/src/relay.rs:100 (`Access::Deny { reason: None }`: the refusal carries no reason to the client) | not fixed: a change in behaviour (a person's words) | A newcomer refused at the group's relay is told the sender is off. relay-run saw it (README finding 2; `group.fay.said`, `group.gus`). Fay's key `bfd6…` was refused 11 times and Gus's `d892…` 17 times, each traced at the relay. Yet Fay's line said Sam's computer "cannot be reached; it may be off", and Gus's page said it "did not answer", while Sam's computer was up throughout. A person would wait for Sam. A fix first finds whether iroh-relay 1.0.2 hands the client a deny's reason. If it does, the line and the page say that the group's relay refused this computer or phone. If it does not, they say both causes, not just the one. Each needs a test, red first. |
| 130 | 12, 6 | crates/svrngs-node/src/guest.rs:63-72 (`RelayConfig::default()`, `n0_services: true`); reached by crates/svrngs-node/tests/common/member.rs, sy_dee.rs, guest/tests.rs, crates/svrngs/tests/new.rs and default_relay.rs | not fixed: a change in behaviour | Updated from relay-run. With the command's own posture, no process opened an address of n0's or `relay.svrnme.sh`'s in 561 samples, and no trace named n0 (`summary.json`, `lsof`, `n0_in_traces`). The n0 default is reached only by tests, so the fix #130 names (no n0 default in `RelayConfig`, n0 kept only where a test names it) waits on nothing the run left open. Updated again from relay-park (the operator, 2026-10-13): n0's relays and lookup are the command's chosen default again, until the operator's switch. It is chosen in one place, `cli::SHIPPED` (`MeetAt::N0`), and `cli::reach` builds it with `n0_services` on and n0's relays named (crates/svrngs/src/cli.rs `reach`; tests/shipped.rs). It still never comes from `RelayConfig::default()`. So the default is no longer one nobody chose, but svrngs-node still holds a second spelling of it that the command does not use. The fix is unchanged: `RelayConfig` with no default, each caller naming its relays. Under the park it no longer waits on a run. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | Unchanged by this range: 9 lines left in svrngs-core (3121 / 3130), 24 in the repository (15776 / 15800). |
| 127, 128, 129, 131 | as recorded | as recorded | not fixed | Not touched by this range. The range has no product line. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/ and scripts/ exact tiers are could-not-judge. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said as could-not-judge. The line-window pass stood in. |

## REVIEW-audit-relay-auto-3 (2026-10-13, range fc1d7a2..2605549)

The previous `REVIEW-audit-` row is REVIEW-audit-relay-auto-2, fc1d7a203
(ralph/next/relay/STATE.md). The range starts there and ends at 2605549,
the commit that made this row. `git rev-list --count fc1d7a2..2605549` is
15, with no merge among them (`--merges`: 0), all on main. It covers one
unit, relay-park, the last audit's own `done` (f22d34a), and two commits
that queue other work (11eb3d6, ralph/next/start/; b36a4f4,
ralph/next/onair/), which touch only ralph/.

This audit's own commits before this one are 3195b7b (the row's two paths,
corrected as 41808f5 and 78cde59 corrected the earlier rows') and d11355b
(one `has_lookup`, behaviour kept). It opens #134 and #135, and closes none.

The instrument is ~/dev/commonwealth-ai/target/debug/sovereign-cli,
sha256 68746bcf…, the last audits' binary (`shasum -a 256`). The embedder
was the daemon on :9741 serving qwen-embedding-0.6b, 1024-d. The daemon
stopped twice during this audit, a rebuild and then its switch to another
chat model. Its operator, another session, measured embeddings across the
switch at cosine 1.000000 against the local-mode vector for the same text.
The corpus is 2605549's product: crates/, apps/, web/, scripts/,
Cargo.toml and Cargo.lock, put into /tmp/relayaudit3/prod-2605549 by
`git archive` (268 files). The first `code index` broke at 2560 of the
chunks when the daemon stopped (/tmp/relayaudit3/index.out). The second
found the partial index unopenable and started fresh, then broke on the
second stop (index2.out; I28). `refresh`, with no index left, built it: 3210 chunks, rust
14533 symbols, typescript and python "skipped (export failed)" (I4;
/tmp/relayaudit3/refresh.out).

### (1) Net lines per unit, product code

The counts come from `git log --reverse -M --numstat --format='@@%h %s'
--no-merges fc1d7a2..2605549`, summed by /tmp/relayaudit3/ledger.py, the
last audit's script unchanged (/tmp/relayaudit3/ledger.out). A commit
that names no unit counts toward the next `done`, so 11eb3d6 and b36a4f4
fall to relay-park, adding no product line. Product is crates/, apps/,
web/ and bridge/. Everything else is counted apart:

- demo/runs/park/2026-10-13 +946: predictions, the driver, its data and
  README
- docs/DESIGN.md +102 −49 and scripts/ +23 −13 (deploy-page.sh's guard,
  deploy-relay.sh's header)
- ralph/next/start +187, ralph/next/onair +172, ralph/next/relay +37 −4
  and ralph/REVIEW_FINDINGS.md +1 −1

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-relay-auto-2 (its `done`) | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| relay-park | 13 | 99 | 46 | 53 | 175 | 32 | 143 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 15 | 99 | 46 | 53 | 175 | 32 | 143 |

The src lines, by file:

- crates/svrngs/src/cli.rs +81 −32 (e24d60b: `MeetAt`, `SHIPPED`,
  `reach` given it; c2e2c33: `link_relay`'s lookup branch gone,
  `kept_relay`, `join_group` keeping by it)
- core link.rs +9 −8, web/app.js +3 −2, web/src/lib.rs +3 −2, group.rs
  +2 −1: docs and comments, which the budget does not count
- main.rs +1 −1: `cli::SHIPPED` passed to `reach`

The tests are tests/shipped.rs +117 −1 (new), new.rs +43 −19 (the
`link_relay` cases, `kept_relay`'s), default_relay.rs +13 −11 (`reach`
given `MeetAt::Svrngs`) and relay.rs +2 −1.

The size budget, measured by scripts/lean_check.py over a `git archive`
tree of each end (/tmp/relayaudit3/lean-fc1d7a2, lean-2605549), and over
the working tree for this audit's move:

| at | svrngs-core | repository |
|---|---:|---:|
| fc1d7a2, range start | 3121 | 15776 |
| 2605549, range end | 3121 | 15798 |
| d11355b, this audit's move | **3121 / 3130** | **15797 / 15800** |

relay-park adds 22 counted lines. **9 lines are left in svrngs-core and 3
in the repository** (#80).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs (src and tests), crates/svrngs-core
(docs alone), web/ and scripts/ (`git diff --stat fc1d7a2 2605549 --
crates apps web scripts`). crates/svrngs-node/ was run too, since
d11355b touches it.

**The exact tier was checked against a plant.** 2605549's `reach`
(cli.rs:684-730, its doc lines included) was appended to
crates/svrngs/src/share.rs in the indexed tree, followed by `refresh`
(+18 symbols). The scoped report gave **1 group**: `reach —
crates/svrngs/src/cli.rs:694 == crates/svrngs/src/share.rs:205`, 2
copies × 37 lines (/tmp/relayaudit3/plant.md). With share.rs restored
(`diff` against `git show 2605549:` is empty) and `refresh` run again
(−18), each scope was run at 0.95 (/tmp/relayaudit3/clean-*_0.95.md):

| scope | locations | exact | near | with a side added in the range |
|---|---:|---:|---:|---|
| crates/svrngs/ | 90 | 0 | 0 | none |
| crates/svrngs-core/ | 457 | 5 | 0 | none |
| crates/svrngs-node/ | 481 | 1 | 0 | none |
| web/ | 98 | could-not-judge (I4) | 0 | none |
| scripts/ | 15 | could-not-judge (I4) | 0 | none |
| crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | — |

The six exact groups are the earlier audits' three-line ones (`is_empty`,
`key`, `lines`, `signed_by_link`, `as_str`, api.rs `addr`); `git blame`
puts none in the range (link.rs `as_str` :186-188 is b92e6fe9). crates/svrngs/
at 0.90 also gave 0 and 0 (clean-crates_svrngs_0.90.md). **No exact or
near clone at the instrument's threshold has a side the range added.**

The instrument judges whole symbols of eight lines or more. What it
cannot see, the read found: `join_group`'s two lines asking whether the
endpoint has an address lookup (e24d60b, cli.rs:1544-1545) were the
bare-key dial's question in svrngs-node iroh.rs:218-221, spelled again.
Fixed here (d11355b).

The tests scope is could-not-judge (I19), so the line-window pass stood
in for it: the last audit's script with the range and the blamed commit
changed (/tmp/relayaudit3/windows.py → windows.out). It found **179
window hits, 82 spans, 1 new**:

- crates/svrngs/tests/default_relay.rs :113-120 == shipped.rs :69-76:
  a parsed `new` taken apart and handed to `cli::new_group`. The two are
  separate test binaries, and neither includes tests/common (whose
  `sams_computer` brings the browser harness with it). Each test also
  starts its computer its own way, so no change.

### Reading the range

- **One decider** (AGENTS.md 8). Where a build meets its groups is
  decided once, `cli::SHIPPED` (cli.rs:243, `MeetAt`, a closed set,
  principle 9). `reach` carries it into `RelayConfig::n0_services`
  (:727). But `join_group` does not take it. It infers the posture from
  whether the endpoint has an address lookup (:1544), and a test's
  `MemoryLookup` trips that as n0's does (#135). deploy-page.sh reads
  `SHIPPED` by `sed`, as it read `DEFAULT_RELAY` before. A spelling it
  cannot read is refused with its words (`*)`, deploy-page.sh:33), never
  taken as n0's.
- **Never silently substitute** (AGENTS.md 6; fence 2). Each `.ok()`,
  `unwrap_or`, `map_or`, `??` and `.flatten()` the range added to product
  code was read (/tmp/relayaudit3/range-src.diff). The one, `join_group`'s
  `address_lookup().ok()`, takes a closed endpoint as no lookup. It is
  read just after a redeem through the same endpoint succeeded, so it is
  kept in `has_lookup` as it was and named in #135. The parked relay
  leaves one gap, which DESIGN §3 says (docs/DESIGN.md:1044-1046) but no
  row or finding held: after the switch, a joiner keeps n0's relay from a
  link minted before it, and meets that group at n0's (#134).
- **Glassbox** (AGENTS.md 1). Each new branch traces at debug: `reach`'s
  group with no relay and its list with `at` and `unkept`, `link_relay`'s
  relay with `n0`, `kept_relay`'s verdict. deploy-page.sh says which
  posture it publishes.
- **A person's words** (fence 5). The range's added quoted strings in
  product code use none of key, member, keeper, ring, act or journal as
  a word outside a trace (`grep -iwE` over the added lines). The one hit
  is `link_relay`'s `warn!` trace, "the link carries the key alone".
  deploy-page.sh's lines are the operator's.
- **Cite, don't recall** (AGENTS.md 4). The park run's README quotes the
  constant as its data holds it (`data/summary.json`, `data/n0.json`:
  `"pub const SHIPPED: MeetAt = MeetAt::N0;"`), and says its first,
  could-not-judge attempt and why (46b3b59).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges fc1d7a2..2605549 -- crates
apps web scripts demo` match `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`
once: `pub enum MeetAt` (/tmp/relayaudit3/new-types.out). It was run
through `converge noun MeetAt --corpus-id prod-2605549`
(/tmp/relayaudit3/nouns.out):

| noun | converge (production) | git grep | read |
|---|---|---|---|
| MeetAt | 1: crates/svrngs/src/cli.rs:231 | 1: :232 | One. No other enum in crates/, apps/ or web/ names where a computer meets the network (`git grep -nE "enum \w*(Meet\|Posture)\w*"`: only it). |

converge's lines are still one early (I1). **No noun has more than one
definition.**

### Fixed here

- d11355b: `svrngs_node::iroh::has_lookup`, the one answer to whether an
  endpoint resolves a bare key, called by the bare-key dial (iroh.rs:225)
  and `join_group` (cli.rs:1544). `cargo test -p svrngs --test shipped
  --test default_relay` (2 and 2 passed) and `cargo test -p svrngs-node
  --lib bare_key` (1 passed) before and after
  (/tmp/relayaudit3/lookup-before.out, lookup-after.out). The repository
  count went 15798 → 15797.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 134 | 6 (fence 2) | crates/svrngs/src/cli.rs:1885-1889 (`kept_relay`: at svrngs's, `n0` false, so every carried relay is kept, n0's included); crates/svrngs/tests/new.rs:286-287 (pinned: `kept_relay(Some(n0's), false) == Some(n0's)`); docs/DESIGN.md:1044-1046 ("One gap is left open") | not fixed: a change in behaviour, and its moment is the operator's switch | After the switch, n0 can come back as a group's own relay. A link minted by a computer at n0's carries one of n0's relays (`link_relay`, either posture; the park run's link carried `usw1-1`, demo/runs/park/2026-10-13/README.md). A computer built at svrngs's that opens it within the link's day keeps that relay beside the group's seed (`join_group`, `keep_group`). It then meets that group at n0's relay for as long as it holds it, says it there in the record by its `Relay` act, and the group's other computers dial it there. This is traced only at debug (`kept_relay`'s line). DESIGN says the gap but holds no row for it, so the switch would flip with it open. A fix tells n0's relays apart at svrngs's too, since `n0_relays()` is a pure list and needs no lookup built (`svrngs_node::relay::n0_relays`): it refuses or drops them, says so in a person's words, and has a test, red first, that a joiner at svrngs's keeps none of n0's. It lands before the operator flips `SHIPPED`. |
| 135 | 8, 6 | crates/svrngs/src/cli.rs:1542-1545 (`join_group`: the posture inferred, `n0 = has_lookup(node.endpoint())`); crates/svrngs/src/cli.rs:243, :727 (`SHIPPED`, carried by `reach` as `n0_services: at == MeetAt::N0`); crates/svrngs/tests/join_by_link.rs:132, share.rs:82 (a `MemoryLookup` added to an endpoint with `n0_services` off) | not fixed: a change of signature (`join_group`'s 17 callers) | Where a build meets its groups is decided by `SHIPPED`, and `join_group` decides it a second time from a proxy: any address lookup on the endpoint counts as n0's. A test's `MemoryLookup` makes `has_lookup` true with n0 off, so a link carrying one of n0's relays would be dropped there as if at n0's. Today no such test carries n0's relay; their links name 127.0.0.1, and `kept_relay` keeps them. A closed endpoint reads as no lookup (iroh 1.0.2 endpoint.rs:1490-1493, `EndpointError::Closed`), untraced apart from `kept_relay`'s `n0 = false`. A fix passes `join_group` the `MeetAt` (or the `RelayConfig`) the command was built with, so `kept_relay` reads the one decision, with a test, red first: a computer at svrngs's with a `MemoryLookup` keeps a link's relay. |
| 130 | 12, 6 | as recorded | not fixed | Not grown by this range. relay-park updated it (3a474d5). |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 9 lines left in svrngs-core (3121 / 3130) and **3** in the repository (15797 / 15800), after relay-park's +22 and this audit's −1. The next row that adds product code meets the cap. |
| 127, 128, 129, 131, 132, 133 | as recorded | as recorded | not fixed | Not touched by this range's decisions. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (cli.rs:231 for :232). The dry-report half reported the plant at its own `fn` line (share.rs:205). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/ and scripts/ exact tiers are could-not-judge. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said as could-not-judge. The line-window pass stood in. |
| I28 | 6 | `code index` resume | New. When the embedder stopped mid-index, it said "Committed chunks are preserved — re-install to resume" (/tmp/relayaudit3/index.out). Run again, it said "Partial index … could not be opened (… Missing metadata …/_corpus_meta.json); starting fresh" (index2.out). The resume it promised did not happen. The rerun's own words say so, so nothing was silently lost, but the first message is wrong. |

## REVIEW-audit-start-auto-1 (2026-10-13, range eaa401e..308be1a)

The previous `REVIEW-audit-` row is REVIEW-audit-relay-auto-3, whose
`done` is eaa401e (ralph/next/relay/STATE.md); the start queue began on
the commit after it (63f24ce), so both readings of the row give one range.
It ends at 308be1a, the commit that made this row. `git rev-list --count
eaa401e..308be1a` is 29, with no merge among them (`--merges`: 0), all on
main: start-findings, start-measure, start-address and start-llama, each
with its `done`, two commits that queue other work (d959a3c, 0728e14;
ralph/ and docs alone) and the row's own commit.

This audit's own commits before this one are 81bc336 (the row's two paths,
as 3195b7b corrected the last audit's) and 891c050 (one `share_of` in
crates/svrngs/tests/share.rs, behaviour kept). It opens #136 and #137,
updates #80, and closes none.

The instrument is ~/dev/commonwealth-ai/target/debug/sovereign-cli,
sha256 68746bcf…, the last audits' binary (`shasum -a 256`). The embedder
was the daemon on :9741 (`/health`: ok). The corpus is 308be1a's product:
crates/, apps/, web/, scripts/, Cargo.toml and Cargo.lock, put into
/tmp/startaudit1/prod-308be1a by `git archive` (271 files). demo/ is left
out: the range's demo/ is two recorded runs. `code index . --corpus-id
prod-308be1a` gave 3256 chunks in 982 s (/tmp/startaudit1/index.out).
`refresh` gave rust 14837 symbols, typescript and python "skipped (export
failed)" (I4), and "Daemon nudge failed: 404 … prod-308be1a is not
registered" (I29; /tmp/startaudit1/refresh.out).

### (1) Net lines per unit, product code

The counts come from `git log --reverse -M --numstat --format='@@%h %s'
--no-merges eaa401e..308be1a`, summed by /tmp/startaudit1/scripts/ledger.py
(/tmp/startaudit1/ledger.out), written again since /tmp/relayaudit3/ was
gone, to the last audit's rules: product is crates/, apps/, web/ and
bridge/; a file under `tests/`, or named as `scripts/lean_check.py`'s
`TEST_FILE` names one (relay_tests.rs, share/tests.rs, tests_support.rs),
is a test; a commit that names no unit counts toward the next one that
does, so d959a3c and 0728e14 fall to start-address, adding no product
line. Everything else is counted apart:

- demo/runs/start +6693 −1: start-measure's run and start-address's
  restart step, predictions, drivers, data and READMEs
- docs/DESIGN.md +79 −19, ARCHITECTURE.md +4 −2, scripts/lean_check.py
  +2 −2 (c27521f's raise), Cargo.lock +1
- ralph/next/start +23 −11, ralph/next/onair +12 −10,
  ralph/REVIEW_FINDINGS.md +5 −5

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| start-findings | 11 | 121 | 59 | 62 | 333 | 21 | 312 |
| start-measure | 4 | 0 | 0 | 0 | 0 | 0 | 0 |
| start-address | 10 | 79 | 21 | 58 | 297 | 1 | 296 |
| start-llama | 3 | 94 | 14 | 80 | 193 | 5 | 188 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 29 | 294 | 94 | 200 | 823 | 27 | 796 |

The src lines, by file:

- start-findings: svrngs-node node.rs +37 −16 (`relay_of`, `dialled`
  given the relay, `say_relay`'s unread groups traced), svrngs relay.rs
  +43 −17 (`verdict`, `spawn`'s `GroupsUnread`), cli.rs +8 −10 (`WHEN`
  moved to home.rs, `GroupsUnread`), home.rs +7, home.js +6 −3, svrngs-node
  relay.rs +5 −4, svrngs-core relay.rs +8 −6, link.rs +1 −1 and
  membership.rs +1 −1 (#131: no line added there, as fence 6 has it),
  rail.rs +2 −1, svrngs-node Cargo.toml +3 (chrono, a dev-dependency)
- start-address: svrngs-node api.rs +34 −6 (`kept_address`, `Taken`,
  `NoAddress`), cli.rs +26 −8 (`Refusal::Taken`, `Api`, `start` listening
  where it last did), origin.rs +8 −4, home.rs +7 −1 (`address`), home.js
  +2, web/install.sh +2 −2
- start-llama: share.rs +55 −11 (`known_kinds`, `says`, `kept_of`,
  `lend_of` adding the table's paths), well_known.txt +27 (llama-server's
  row and why each path is kept), cli.rs +12 −3 (`Said::Shared` saying
  what the table keeps back)

The size budget, measured by scripts/lean_check.py at HEAD over a `git
archive` tree of each end (/tmp/startaudit1/lean-eaa401e.out,
lean-308be1a.out), and over the working tree after this audit's move:

| at | svrngs-core | repository |
|---|---:|---:|
| eaa401e, range start (its own cap 15,800) | 3121 | 15797 |
| 308be1a, range end | 3121 | 15898 |
| 891c050, this audit's move (tests alone) | **3121 / 3130** | **15898 / 15950** |

The range adds 101 counted lines. **9 lines are left in svrngs-core and
52 in the repository** (#80), before start-home's +50 to 16,000.

### (2) Clones (`code dry-report`)

The range touched crates/svrngs, crates/svrngs-node and crates/svrngs-core
(src and tests) and web/ (`git diff --stat eaa401e 308be1a -- crates apps
web scripts`); scripts/ was run too, for lean_check.py.

**The exact tier was checked against a plant.** 308be1a's `kept_of`
(share.rs:142-157, its doc lines included), added in the range by
f900454, was appended to crates/svrngs/src/cli.rs in the indexed tree,
followed by `refresh` (+5 symbols). The scoped report gave **1 group**:
`kept_of — crates/svrngs/src/cli.rs:1950 == crates/svrngs/src/share.rs:145`,
2 copies × 13 lines (/tmp/startaudit1/plant.md). With cli.rs restored
(`diff` against `git show 308be1a:` is empty) and `refresh` run again
(−5), each scope was run at 0.95 and 0.90
(/tmp/startaudit1/clean-*.md):

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs/ | 98 | 0 | 0 | 0 | none |
| crates/svrngs-core/ | 457 | 5 | 0 | 12 | none |
| crates/svrngs-node/ | 489 | 1 | 0 | 8 | none |
| web/ | 98 | could-not-judge (I4) | 0 | 1 | none |
| scripts/ | 15 | could-not-judge (I4) | 0 | 0 | none |
| crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

The six exact groups are the earlier audits' three-line ones (`is_empty`,
`key`, `lines`, `signed_by_link`, `as_str`, api.rs `addr`). Every
location the reports name, 55, was blamed at 308be1a against the range's
29 commits: none has a line from them. **No exact or near clone at 0.95
or 0.90 has a side the range added.**

The tests scope is could-not-judge (I19), so a line-window pass stood in:
/tmp/startaudit1/scripts/windows.py, written again to the last audit's
rule (8 lines of code, blanks, comments and lone brackets dropped, over
every test file of crates/; a span new when a side overlaps a line the
range added, by `git diff -U0`). At 308be1a it found **94 window hits,
54 spans, 7 new** (/tmp/startaudit1/windows.out); after 891c050, 87, 50
and 4 (windows-after.out):

- crates/svrngs/tests/share.rs :159-169 == :285-294 and :171-179 ==
  :296-304, and address.rs :172-179 == share.rs :169-176: a `Share`
  literal, field for field, which f900454 wrote again for llama-server.
  Fixed in 891c050 (`share_of`); address.rs's is a separate binary (#137).
- svrngs-node src/relay_tests.rs :150-161 == svrngs src/relay_tests.rs
  :9-20 == svrngs tests/relay.rs :103-113: a test's trace sink, `TRACES`
  and `struct Kept`, written three times in the range (c97b9e3, 9bb6df8,
  bf67398) beside default_relay.rs :31-43's, before it (#137).
- crates/svrngs/tests/address.rs :41-54 == share.rs :531-544: `Program`,
  the command run as a process with its output read by line, copied by
  82c7dd1 from share.rs (334c4b8) and changed (`HOME`, `RUST_LOG=debug`,
  no standard input) (#137).

### Reading the range

- **One decider** (AGENTS.md 8). Each of the range's rules has one home:
  when a person joined, `home::WHEN` (svrngs-node home.rs:59), read by the
  command's `joined_of` and written by home.js `people` by hand, which the
  browser test pins (start-findings (a)); which group's relay a key is
  dialled by, `relay_of` (node.rs:686), asked by the round, the asks and
  `contact`; a relay as written, svrngs-core's `relay::parse`, the node's
  wrapping it (#131); the home page's address, `home::address` (home.rs:91),
  the command's two sentences and `ask_address` asking it, install.sh
  spelling it in shell (:156) as tests/install.rs:295 pins; what a kind
  keeps back, `kept_of` (share.rs:145), through `says` (:137), the one
  reader of a row's clause, which `accounts_of` asks too. One reader is
  left beside its rule: the API's address file is read by `kept_address`
  (api.rs:58), refusing one that names no address, and still by hand in
  `ask_running` (cli.rs:891), an unreadable file taken as empty (#136).
- **Never silently substitute** (AGENTS.md 6; fence 2). Each `.ok()`,
  `unwrap_or`, `unwrap_or_default` the range added to product code was
  read (/tmp/startaudit1/range-src.diff). svrngs-core link.rs:140 and
  membership.rs:289 keep the `Option` they had, the refusal traced inside
  `parse`; api.rs's `a.parse().ok()` becomes `NoAddress`, refused; node.rs
  `say_relay`'s `unwrap_or_default` comes after the warn #128 asked for;
  cli.rs `kept_address(..)?.unwrap_or(API)` is `None` only before the
  first start, traced; `joined_of`'s `.ok()` becomes its refusal's words.
  A port another program holds is said with the address and the file to
  delete (`Refusal::Taken`), never moved.
- **Glassbox** (AGENTS.md 1). Each new branch traces: `kept_address`'s
  read, `relay_of`'s group, `verdict`'s three arms (a failed read at
  warn), `spawn`'s unread groups, `kept_of`'s count, `lend_of`'s refused
  count, `app_of`'s mode.
- **A person's words** (fence 3). The range's added quoted strings in
  product code use none of key, member, keeper, ring, act, journal, port,
  namespace or origin as a word a person reads (`grep -iwE` over the added
  lines): the hits are format strings, a trace target and trace fields.
  The command's own words keep "port", as `share <port>` takes one.
- **The nouns gate** (fence 6). Outside well_known.txt, llama-server is
  named in product code only in share.rs:30, a module doc citing the row
  (`git grep -i llama 308be1a -- crates apps web scripts`, tests apart).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges eaa401e..308be1a -- crates apps
web scripts demo` match `\b(struct|enum|trait|type|class)\s+[A-Z]\w*`
four times: `struct Kept` three times and `struct Program` once
(/tmp/startaudit1/new-types.out), all in test code. Each was run through
`converge noun <Name> --corpus-id prod-308be1a`
(/tmp/startaudit1/nouns.out), with `git grep` over every path beside it,
since converge leaves tests out (nouns-grep.out):

| noun | converge (production) | git grep (all) | read |
|---|---|---|---|
| Kept | 1: svrngs-node door/choices.rs:59 | 5: choices.rs:60 (`type Kept`, the choices kept on disk); svrngs-node relay_tests.rs:153, svrngs relay_tests.rs:12, tests/relay.rs:105, tests/default_relay.rs:33 (`struct Kept`, a test's trace sink) | Two concepts. The production alias is one; the four sinks are one concept written four times (#137). |
| Program | 0 | 2: tests/share.rs:507, tests/address.rs:22 | One concept, written twice (#137). |

converge's lines are still one early (I1). **No noun has more than one
definition in production code.** In test code, `Kept` and `Program` each
do (#137).

### Fixed here

- 891c050: crates/svrngs/tests/share.rs's two `Share` literals one
  `share_of(origin, kind)`. `cargo test -p svrngs --test share`: 8 passed
  before and after (/tmp/startaudit1/share-before.out, share-after.out).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 136 | 8, 6 | crates/svrngs/src/cli.rs:891 (`ask_running`: `read_to_string(root.join(API_FILE)).unwrap_or_default()`); crates/svrngs-node/src/api.rs:58-70 (`kept_address`, start-address's reader of the same file) | not fixed: a change in what a person is told | Two readers of where svrngs listens. start-address made `kept_address` the reader that refuses a file naming no address (`NoAddress`, "delete it"). `ask_running`, which a folder's sync and `svrngs remove` ask through (cli.rs:839, :860, :1629), still reads the file by hand: one it cannot read is taken as empty, and one naming no address is pasted before the route as written. Either is refused as `NotRunning`, "svrngs is not running on this computer", so a file svrngs cannot read is said as svrngs not running. A fix has `ask_running` ask `kept_address`: `None` is `NotRunning`, an error `Refusal::Api` in its words, with a test, red first, of a file naming no address said as that. |
| 137 | 8 | the trace sink `TRACES` + `struct Kept`: crates/svrngs-node/src/relay_tests.rs:150-163 (c97b9e3), crates/svrngs/src/relay_tests.rs:9-21 (9bb6df8), crates/svrngs/tests/relay.rs:103-115 (bf67398), crates/svrngs/tests/default_relay.rs:31-43 (before the range); `Program`: crates/svrngs/tests/address.rs:22-75 (82c7dd1) and crates/svrngs/tests/share.rs:507-566; the `Share` literal: address.rs:170-180 and share.rs:43-55 (`share_of`) | not fixed: across two crates and separate test binaries, the copies differing | Test code (no budget line), but the drive is copied: four trace sinks, two `Program`s, two `Share` literals, and `address_in` in address.rs:78 beside share.rs:145. tests/common cannot hold them, since it brings the browser harness with it (common/mod.rs:13). A fix gives the command's tests one support module without the harness, `Program::start` taking its environment and input, and the sinks one file each crate's tests include, each test run before and after. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 9 lines left in svrngs-core (3121 / 3130) and 52 in the repository (15898 / 15950), after the range's +101. start-home's row takes the operator's +50 to 16,000 when it first passes 15,950. |
| 130, 132, 133, 134, 135 | as recorded | as recorded | not fixed | Not touched by this range's decisions. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (choices.rs:59 for :60). The dry-report half reported the plant at its own `fn` line (cli.rs:1950). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/ and scripts/ exact tiers are could-not-judge. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said as could-not-judge. The line-window pass stood in. |
| I29 | 6 | `refresh` over a corpus `code index` made | New: "⚠ Daemon nudge failed: 404 Not Found: … prod-308be1a is not registered; run `sovereign project register` first", then "✓ 14837 symbols" and exit 0 (/tmp/startaudit1/refresh.out). The symbols were exported in process, as the plant's +5 and −5 show, so nothing was lost; the warning names a step the index never asked for. |

## REVIEW-audit-start-auto-2 (2026-10-13, range c6c2212..1ea033b)

The previous `REVIEW-audit-` row is REVIEW-audit-start-auto-1, whose
`done` is c6c2212 (ralph/next/start/STATE.md: its hash f9b41a5, the loop's
"ralph: REVIEW-audit-start-auto-1 done" c6c2212 after it). The range ends
at 1ea033b, the commit that made this row. `git rev-list --count
c6c2212..1ea033b` is 55, with no merge among them (`--merges`: 0), all on
main: start-home, start-group, start-run and start-economize, each with its
`done`, and the row's own commit.

This audit's own commits before this section are a12fd81 (the row's two
paths, as 81bc336 corrected the last audit's), 13aff25 (one `keep_folder`
in the home page's browser tests, behaviour kept), 1dee396 (`ORIGIN_FILE`'s
comment naming its writer where it now is) and b860db4 (the data dir's
`origin` read by `ORIGIN_FILE` in tests/install.rs, so install.sh's
spelling is pinned). It opens #138-#140, updates #137 and #80, and closes
none.

### (1) Net lines per unit, product code

The counts come from `git log --reverse -M --numstat --format='@@%h %s'
--no-merges c6c2212..1ea033b`, summed by /tmp/startaudit2/scripts/ledger.py
(/tmp/startaudit2/ledger.out), to the last audit's rules: product is
crates/, apps/, web/ and bridge/; a file under `tests/`, or named as
`scripts/lean_check.py`'s `TEST_FILE` names one (`*_tests.rs`,
`tests.rs`, `tests_support.rs`), is a test; a commit that names no unit
counts toward the next one that does. Everything else is counted apart:

- demo/runs/start +12870 −76: start-home's walk before and after,
  start-run's story, predictions, drivers, data and READMEs;
  demo/runs/economize +3917: start-economize's survey and its data
- docs/DESIGN.md +129 −46, ARCHITECTURE.md +8 −5, AGENTS.md +3 −1,
  scripts/lean_check.py +9 −6 (4b81e4b's and 54e5e79's raises, 0b5bccf's
  ratchet), scripts/ralph-mark.sh −29 (250b17e), Cargo.lock +2
- ralph/next/start +47 −19, ralph/next/onair +41 −25,
  ralph/claude-settings.json −1

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| start-home | 20 | 399 | 203 | 196 | 392 | 22 | 370 |
| start-group | 9 | 209 | 120 | 89 | 317 | 22 | 295 |
| start-run | 9 | 0 | 0 | 0 | 0 | 0 | 0 |
| start-economize | 16 | 4 | 47 | −43 | 56 | 52 | 4 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 55 | 612 | 370 | 242 | 765 | 96 | 669 |

The src lines, by file:

- start-home: svrngs-node invite.rs +192 −3 (5a0ffed moving `draw`,
  `carried`, `Host::mint` and `own_person` from svrngs's issuer.rs and
  cli.rs, which lose +7 −39 and +23 −76; then `Host::link` and
  `kept_page`), home.rs +58 −3 (`POST /link`, `/folder`, `Linking`,
  `OPENERS`), home.js +71 −60, index.html +22 −6, home.css +1, node.rs
  +11 −2 (`Node::host`, `relay` kept), svrngs group.rs +9 −14 (caafae7's
  `read_origin` asking `kept_page`), svrngs-node Cargo.toml +4
  (ed25519-dalek and getrandom, both in Cargo.lock already), lib.rs +1
- start-group: svrngs-node group.rs +125 −15 (318599c, 31522ac and 51583e7
  moving the seed's keeping, `folder_place`, `kept_relay` and the folder's
  placing from svrngs; then 63dcaee's `Asking::found` and `Made`), cli.rs
  +19 −87, home.rs +16 −2 (`POST /group`), home.js +14 −4, index.html
  +1 −2, node.rs +8, lib.rs +2 and svrngs lib.rs +2 −2 (`pub use
  svrngs_node::group`), web/app.js +8 −4 and index.html +2 −1 (the
  founder's line, 2b2a751), web/install.sh +12 −3 (0f468a3)
- start-economize: cli.rs +3 −17 (`new_link`, `LINK_LIFE_SECS`'s
  re-export), runtime register.rs −12 (`standing_in` into its tests),
  guest.rs −11 (`on_loopback_only`), node.rs +1 −7 (`Node::say_relay`)

The size budget, by scripts/lean_check.py over a `git archive` tree of
each end (/tmp/startaudit2/lean-c6c2212.out, lean-1ea033b.out), and over
the working tree after this audit's commits:

| at | svrngs-core | repository |
|---|---:|---:|
| c6c2212, range start (its cap 15,950, then) | 3121 | 15898 |
| 1ea033b, range end (caps 3,122 and 15,987 since 0b5bccf) | 3121 | 15986 |
| b860db4, this audit's last change (tests and a comment) | **3121 / 3122** | **15986 / 15987** |

The range adds 88 counted lines (the ledger's 242 src lines net, less
comments, blanks and the pages' markup): start-home's and start-group's,
under the raises to 16,000 (4b81e4b) and 16,040 (54e5e79), less what
start-economize cut before its ratchet to where it landed (0b5bccf). **1 line is
left in svrngs-core and 1 in the repository** (#80), and the cap is
strict: a 15,987th line is refused (`lean_check.py`: "BROKEN . has 15987
lines of code, budget 15987"), as this audit found when it tried the trace
#140 asks for.

### The instrument

~/dev/commonwealth-ai/target/debug/sovereign-cli is not the last audits'
binary. While this audit ran, commonwealth-ai's `target/` was emptied and
rebuilt by another session (`ls target/debug` at 18:09: no binaries; its
`cargo build -p sovereign-cli …` started 18:10); the binary this audit
used was linked at 18:14, sha256 67e46f63… (`shasum -a 256`), not
68746bcf…. In it `code refresh` is gone ("Unknown code subcommand:
refresh"); `svrn refresh` does that work. So its exact tier was checked
again against a plant before any clean report was taken as clean (below).

The corpus is 1ea033b's product: crates/, apps/, web/, scripts/,
Cargo.toml and Cargo.lock, put into /tmp/startaudit2/prod-1ea033b by `git
archive` (272 files); demo/ is left out, the range's demo/ being recorded
runs. `code index . --corpus-id prod-1ea033b`, by the old binary before it
was removed, gave 3290 chunks in 1144 s, after the first attempt, stopped
with the session that started it, left a partial index that "could not be
opened (… Missing metadata …); starting fresh" (/tmp/startaudit2/index.out,
index2.out; I28, still present). The embedder was the daemon on :9741,
`qwen-embedding-0.6b`. `refresh` gave rust 15105 symbols, typescript and
python "skipped (export failed)" (I4), "⚠ SCIP DB schema v0 is stale
(current: v3); resetting", and "LanceDB index current — skipping", so the
embeddings the near tier reads are the index's own
(/tmp/startaudit2/refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs, crates/svrngs-node and crates/svrngs-runtime
(src and tests) and web/ (`git diff --stat c6c2212 1ea033b -- crates apps
web scripts`); svrngs-core not at all, so it was not run.

**The exact tier was checked against a plant.** 1ea033b's `kept_page`
(invite.rs:34-48, its doc lines included), added in the range by caafae7,
was appended to crates/svrngs-node/src/home.rs in the indexed tree, then
`refresh` (+5 symbols, /tmp/startaudit2/refresh-plant.out). The scoped
report gave **2 groups**: `kept_page — crates/svrngs-node/src/home.rs:378
== crates/svrngs-node/src/invite.rs:37`, 2 copies × 12 lines, beside api.rs
`addr` (/tmp/startaudit2/plant.md). home.rs was then restored (`diff`
against `git show 1ea033b:` empty) and refreshed again
(refresh-restored.out). The clean reports were taken before the plant, at
0.95 and 0.90 (/tmp/startaudit2/clean-*.md):

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs/ | 77 | 0 | 0 | 0 | none |
| crates/svrngs-node/ | 517 | 1 | 0 | 8 | none |
| crates/svrngs-runtime/ | 135 | 1 | 0 | 0 | none |
| web/ | 98 | could-not-judge (I4) | 0 | 1 | none |
| crates/svrngs/tests/, crates/svrngs-node/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

The two exact groups are the earlier audits' three-line ones (api.rs
`addr`; runtime tests.rs and tests_write_index.rs `open`). Every location
the reports name, 22 (/tmp/startaudit2/locs.txt), was blamed at 1ea033b
against the range's 55 commits: none has a line from them
(/tmp/startaudit2/blame.out; the same filter finds 8 of 8 in invite.rs
:153-160, `Host::link`). **No exact or near clone at 0.95 or 0.90 has a
side the range added.**

The tests scopes are could-not-judge (I19), so the line-window pass stood
in: /tmp/startaudit2/scripts/windows.py, to the last audit's rule (8 lines
of code, blanks, comments and lone brackets dropped, over every test file
of crates/; a span new when a side overlaps a line the range added, by
`git diff -U0`). At 1ea033b it found **95 window hits, 53 spans, 12 new**
(/tmp/startaudit2/windows.out):

- crates/svrngs-node/tests/browser/main.rs :2006-2018 == :2448-2460: a
  folder kept as the group's, `POST /v1/folders`, which bece124 (start-home)
  wrote again from 20ede44. Fixed in 13aff25 (`keep_folder`).
- crates/svrngs-node/tests/conformance.rs :520-530 == crates/svrngs/tests/
  home_group.rs :54-64: a `web/tests/` script run under node and its last
  line read, which 63dcaee (start-group) copied from conformance.rs's
  `browse` (51ac1a7) (#137).
- The other ten are a node started in a test with a `RelayConfig` literal,
  across tests/door/main.rs, lend_origin.rs, room.rs, surface.rs, home.rs,
  folders.rs, browser/main.rs and conformance.rs. Each predates the range
  (`git blame`); afdaa6f (start-economize) changed one or two lines in
  each when it cut `on_loopback_only`, which the rule counts as new (#137).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges c6c2212..1ea033b -- crates apps
web scripts` match `\b(struct|enum|trait|type|class)\s+[A-Z]\w*` three
times: `enum Unminted` (invite.rs:52, 5a0ffed), `struct Linking` (home.rs
:242, ab6ef1f) and `type Made` (group.rs:258, 63dcaee). (The previous
session's /tmp/startaudit2/new-types.out counted demo/ too, where
start-economize's demoted.diff repeats every type; with demo/ left out
there are these three.) Each was run through `converge noun <Name>
--corpus-id prod-1ea033b` (/tmp/startaudit2/nouns.out), with `git grep`
over every path beside it, since converge leaves tests out
(nouns-grep.out):

| noun | converge (production) | git grep (all) | read |
|---|---|---|---|
| Unminted | 1: svrngs-node invite.rs:51 | 1: invite.rs:52 (and its line in demo/runs/economize/…/demoted.diff, a recorded diff) | One. |
| Linking | 1: svrngs-node home.rs:241 | 1: home.rs:242 | One. |
| Made | 1: svrngs-node group.rs:257 | 1: group.rs:258 (`Said::Made`, cli.rs:1058, is a variant) | One. |

converge's lines are still one early (I1). **No noun has more than one
definition.**

### Reading the range

Every hunk of product code the range changed was read
(/tmp/startaudit2/range-src.diff, `git diff c6c2212 1ea033b -- crates
apps web ':!*tests*'`).

- **A move is its own commit** (AGENTS.md 2). Each of start-home's and
  start-group's moves into the node landed apart from the change it made
  room for, its tests named in its body: 5a0ffed (a link), caafae7 (the
  kept page), 318599c, 31522ac and 51583e7 (a group's seed, its folder's
  place, the relay a joiner keeps, the folder placed), each before
  ab6ef1f, bece124 and 63dcaee used it. start-economize's cuts are each
  their own commit with lean_check before and after.
- **One decider** (AGENTS.md 8). A link is made in one place,
  `Host::mint` (crates/svrngs-node/src/invite.rs:110), which `svrngs
  link`, `svrngs new` and the page's `Host::link` all ask; the relay it
  carries, `carried` (:85); a group kept, `Host::keep_group`
  (group.rs:171), which `new`, `join` and the page ask; a folder's place,
  `folder_place` (group.rs:196) through `Folders::place`, which `new`,
  `join` and the page ask; the page a group's https links print on,
  `kept_page` (invite.rs:37), which svrngs's `read_origin`, the page's
  link and `Asking::found` ask. Two things are decided twice:
  - whether a name a group of this computer's has already founds another
    one: `Asking::found` refuses it (group.rs:241-245), `svrngs new` makes
    a second group of that name (cli.rs:1659, `new_group`; pinned by
    crates/svrngs/tests/new.rs:74-79, `new_twice_with_one_name_makes_two_groups_with_two_ids`)
    (#138);
  - the data dir's `origin` file's name: `ORIGIN_FILE`
    (invite.rs:32) and install.sh's `"$dir/origin"` (web/install.sh:102, after b860db4's comment),
    which shell cannot import. Its test read it back by a literal of its
    own, so a change to the constant left a group made on the page keeping
    no page, every test green. Fixed in b860db4: the test reads it by the
    constant, red on a plant.
- **Never silently substitute** (AGENTS.md 6; fence 2). Each `.ok()`,
  `unwrap_or`, `unwrap_or_default` and `catch` the range added to product
  code was read. `carried`'s `relay::parse(r).ok()` (invite.rs:90) moved
  unchanged from cli.rs `link_relay` (5a0ffed), the refusal traced inside
  `parse` (svrngs-core relay.rs:29-31). `kept_page` says a file it cannot
  read as that, never as no page (invite.rs:37-48). `open`'s
  `dirs_of(..).is_ok_and(..)` takes a folder list it cannot read as the
  folder not being the group's, refused in a person's words and traced
  with `listed`. home.js's `show` says a group whose lends cannot be read
  as that. One substitution stays from before the range, rewritten by
  4830603: the page's `/ask` takes a group whose lends cannot be read as
  lending nothing (home.js:44, `lentIn`'s `() => []`; c6c2212's
  `g.lent ?? []`), and says "That app, or what it asks to use, is not lent
  in your groups now" (home.js:153) (#139).
- **Glassbox** (AGENTS.md 1). Each new branch traces — `found`'s made or
  refused (home.rs `found`, `?made`), `open`'s `listed` and opener,
  `Folders::place`'s outcome, `kept_relay`, `carried`, `draw`'s draws,
  `own_person`, `mint`'s refusals, `link`'s made — but one: `Host::link`
  refusing a group this computer does not hold (invite.rs:157-159)
  returns `NotIn`, and home.rs's `link` answers it 403 with no trace
  (home.rs:254-256). The trace is one line, and the repository is at
  15,986 of a strict 15,987 (#140, #80).
- **A person's words** (fence 3). The range's added quoted strings in
  product code use none of key, member, keeper, ring, act, journal, port,
  namespace or origin as a word a person reads (`grep -iwE` over the added
  lines): the hits are routes (`/v1/lends?namespace=`), a file name
  (`ORIGIN_FILE`), trace text and `l.origin`, a URL shown as "its address
  for a program".
- **The nouns gate** (fence 6). Outside well_known.txt, llama-server is
  named in product code only in share.rs:30, a module doc citing the row
  (`git grep -in llama 1ea033b -- crates apps web scripts`, tests apart).
- **The core untouched** (fence 6). `git diff --stat c6c2212 1ea033b --
  crates/svrngs-core` is empty; svrngs-core stays at 3121.

### Fixed here

- 13aff25: crates/svrngs-node/tests/browser/main.rs's two `POST
  /v1/folders` (:2006-2018 == :2448-2460, 20ede44 and bece124) one
  `keep_folder(at, folder)`. `cargo test -p svrngs-node --test browser --
  groups_folder group_s_folder`: 4 passed before and after
  (/tmp/startaudit2/folder-before.out, folder-after.out).
- 1dee396: invite.rs's `ORIGIN_FILE` comment names its writer where
  318599c moved it, the node's group.rs `write_origin`.
- b860db4: tests/install.rs reads the data dir's `origin` by `ORIGIN_FILE`,
  so install.sh's spelling of it is pinned; red on the plant
  `ORIGIN_FILE = "origin-planted"` ("panicked at crates/svrngs/tests/
  install.rs:426:70 … NotFound", /tmp/startaudit2/origin-red.out), green
  without it (origin-green.out).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 138 | 8 | crates/svrngs-node/src/group.rs:241-245 (`Asking::found`: a name one of this computer's groups holds, any case, refused: "You are in Trip already: give this group another name."); crates/svrngs/src/cli.rs:1659 (`new_group`, which asks no such thing) and crates/svrngs/tests/new.rs:74-79 (`new_twice_with_one_name_makes_two_groups_with_two_ids`, which pins two groups of one name) | fixed: 6acfdd6 (v1ui-door (b)): the operator's "Allow; tell them apart" — one rule, group.rs `namesakes`, asked by `svrngs new` and the page's make, a held name never refused and said before and after; a second group's folder placed beside the first's; crates/svrngs/tests/home_group.rs, red on its plants | Two makers of a group decide differently whether a held name founds another. 63dcaee (start-group (c)) made the page refuse it, as its row asked ("a name already held among them"); `svrngs new`, which DESIGN §3 says the page makes groups "as" (docs/DESIGN.md:208), makes a second group of the name, and `link`, `share --folder` and `svrngs remove` then refuse that name as ambiguous (cli.rs `group_named`, `Refusal::Ambiguous`). Which is right is the design's: a fix gives the rule one home both ask — `new_group` asking `found`'s check, and new.rs:74 rewritten to the refusal, red first — or the page's refusal dropped, and DESIGN §3 says which. |
| 139 | 6 | crates/svrngs-node/src/home/home.js:44 (`lentIn`: `get('/v1/lends?…').then((l) => l.lent, () => [])`) and :153 ("That app, or what it asks to use, is not lent in your groups now."); c6c2212's home.js:154 (`all.flatMap((g) => g.lent ?? [])`, before the range) | not fixed: a change in what a person is told | The page's `/ask`, where an app sends a person to say yes, takes a group whose lends cannot be read (its people's computers asleep, the API refusing) as lending nothing, and tells the person the app is not lent in their groups now. Before the range the same substitution was `g.lent ?? []` over `groups()`'s `unread`; 4830603 (start-home) rewrote it as `lentIn`. The main page says the same failure as "What this group lends cannot be read on this computer" (home.js:101). A fix keeps the groups `lentIn` could not read and, when the asked lend is not found among the rest, says that what some group lends could not be read, with a browser test, red first, of an `/ask` against a lend whose group's read fails. |
| 140 | 1 | crates/svrngs-node/src/invite.rs:157-159 (`Host::link`: a group this computer does not hold, `return Err(Unminted::NotIn)`, untraced); crates/svrngs-node/src/home.rs:254-256 (the page's `link` answering it 403, untraced) | fixed: 0db97c8 (events-principles (e)): traced in `Host::link` and in the page's `link`; tests/home.rs `the_page_s_link_for_a_group_not_held_is_refused_and_traced`, red on each | ab6ef1f (start-home) added the page's link with every branch traced but this one: a link asked for a group this computer does not hold is refused with no trace on either side. `svrngs link` refuses the same in `group_named` with a trace (cli.rs, "cli: refused — no group by that name"). A fix is one `tracing::debug!(target: "link", group, …)` before the return; tried here, `scripts/lean_check.py` said "BROKEN . has 15987 lines of code, budget 15987", and rustfmt would not take the line back from `link`'s `home`/`relay` pair (`carried(…)` wrapped to four lines). It waits for a line freed (onair-economize) or the operator's raise. |
| 137 | 8 | as recorded; and now crates/svrngs/tests/home_group.rs:49-65 (63dcaee, `home_page`) == crates/svrngs-node/tests/conformance.rs:517-531 (`browse`) | not fixed: two test binaries, the copies differing | Updated. start-group's `home_page` runs a `web/tests/` script under node and reads its last line, as conformance.rs's `browse` does, word for word but the script and its `expect` texts; both binaries include browser/harness.rs, the one home a `browse(script, args)` would have, `#[allow(dead_code)]` in the binaries that do not call it. The node starts copied across tests/door/main.rs :43-66, lend_origin.rs :31-52, room.rs :37-58, surface.rs :35-42, home.rs :39-47, folders.rs :31-39, browser/main.rs :73-86 and :124-137 and conformance.rs :66-79 predate the range (`git blame`: 4f6037d, a16aeb8, bf6276d, 57592a3, 1c61b37, 50ecc20, 05eee01, 48ec52c, e502de9, 51ac1a7); afdaa6f's cut of `on_loopback_only` changed one or two lines in each, the line-window pass's "new" spans. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 1 line left in svrngs-core (3121 / 3122) and 1 in the repository (15986 / 15987), strict, after the range's +88 and start-economize's ratchet (0b5bccf). onair-economize is queued before any raise (491936f). |
| 130, 132, 133, 134, 135, 136 | as recorded | as recorded | not fixed | Not touched by this range's decisions; #136 named by start-economize's NEEDS_HUMAN.md as a change in what a person is told, folded into onair-economize. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (invite.rs:51 for :52, home.rs:241 for :242, group.rs:257 for :258). The dry-report half reported the plant at its own `fn` line (home.rs:378). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/ and scripts/ exact tiers are could-not-judge. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, and said as could-not-judge. The line-window pass stood in. |
| I28 | 6 | `code index` resume | Still present: the index this audit's first session started, stopped mid-way, was "could not be opened (… Missing metadata …); starting fresh" when run again (/tmp/startaudit2/index2.out). |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present, under its new name `svrn refresh`: "⚠ Daemon nudge failed: 404 Not Found: … prod-1ea033b is not registered", then the in-process rebuild and exit 0 (/tmp/startaudit2/refresh.out). |
| I30 | 7 | the binary at ~/dev/commonwealth-ai/target/debug/sovereign-cli | New: the path these rows name is a build product another session may empty and rebuild at any time; between this audit's index and its reports the binary went from sha256 68746bcf… to 67e46f63…, and `code refresh` with it. A row's instrument named by path alone is not one instrument across a run; this audit checked the new one against a plant. |

## REVIEW-audit-start-auto-3 (2026-10-13, range d7cf438..88d60e2)

The previous `REVIEW-audit-` row is REVIEW-audit-start-auto-2, whose
`done` is d7cf438 (ralph/next/start/STATE.md: its hash a6ab719, the loop's
"ralph: REVIEW-audit-start-auto-2 done" d7cf438 after it). The range ends
at 88d60e2, the commit that made this row. `git rev-list --count
d7cf438..88d60e2` is 9, with no merge among them (`--merges`: 0), all on
main: start-doc's seven commits, its `done` (c63fb6a) and the row's own.

This audit's own commits before this section are 3696cdc (the row's two
paths, as a12fd81 corrected the last audit's) and 9c6f5ce (the reader
run's success cited to its session's result record). It opens #141-#145,
updates #80, and closes none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
d7cf438..88d60e2` (/tmp/startaudit3/ledger.raw), to the last audits'
rules (product is crates/, apps/, web/ and bridge/; tests by
`scripts/lean_check.py`'s `TEST_FILE`). No line of product is in it:
`git diff --stat d7cf438 88d60e2 -- crates apps web bridge scripts
Cargo.toml Cargo.lock` is empty. Counted apart:

- demo/start.md +295 −11 (d6ed674, 099221d), the newcomer's document
- demo/runs/start/2026-10-13-reader +396 −4 and a 1.4 MB gzipped
  transcript (b7a1825, f000fcb, c287f3e, 990ec26, 522719a): PROMPT.md,
  run.sh, README.md and data/
- README.md +8 −6, docs/DESIGN.md +24 (d6ed674)
- ralph/next/start/STATE.md +2 −1 (c63fb6a, 88d60e2)

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| start-doc | 8 | 0 | 0 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 9 | 0 | 0 | 0 | 0 | 0 | 0 |

The size budget, by scripts/lean_check.py over a `git archive` tree of
each end (/tmp/startaudit3/lean-d7cf438.out, lean-88d60e2.out) and over
the working tree after this audit's commits:

| at | svrngs-core | repository |
|---|---:|---:|
| d7cf438, range start | 3121 / 3122 | 15986 / 15987 |
| 88d60e2, range end | 3121 / 3122 | 15986 / 15987 |
| 9c6f5ce, this audit's last change (a run's README) | **3121 / 3122** | **15986 / 15987** |

### (2) Clones (`code dry-report`)

The range touched no crate dir and no `web/` (the empty `git diff --stat`
above), so no scope is owed a report and none was run: not a
could-not-judge, there is no added symbol for a clone to have a side in.
The one program the range added is a run's harness, demo/runs/start/
2026-10-13-reader/run.sh (52 lines of sh, b7a1825), outside every scope
the row names; it was read whole (below). The binary at
~/dev/commonwealth-ai/target/debug/sovereign-cli is the one the last audit
checked against a plant (sha256 67e46f63…, linked 18:14, `shasum -a 256`,
`ls -la`).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges d7cf438..88d60e2` match
`\b(struct|enum|trait|type|class)\s+[A-Z]\w*` no times, demo/ included.
**No new noun, so none to converge.**

### Reading the range

Every line the range added outside the recorded transcript was read:
demo/start.md, the reader's PROMPT.md, run.sh, README.md and NOTES.md,
README.md's "Run it", DESIGN §8's Start entry.

- **Cite, don't recall** (AGENTS.md 4). Every markdown link and
  backticked path in demo/start.md, the reader's README and PROMPT.md and
  README.md resolves (/tmp/startaudit3/cites.out: 33 ok, 0 missing), and
  the four line citations say what is claimed of them: cli.rs:798
  (`prompt_secret`) before :808 (`print!` of `Said::Shared`), cli.rs:1626
  (`user_name`), the story's run.py:78 (`"USER": who.capitalize()`),
  index.html:24 (the `svrngs share` line). One claim was cited to a file
  that does not hold it: the reader README's "exit 0 … (`data/end.json`)",
  where end.json holds seconds only and run.sh's exit holds nothing of
  the session's. Fixed in 9c6f5ce; #145 for the harness.
- **Never silently substitute** (AGENTS.md 6). run.sh refuses to start
  over a svrngs kept at login or an existing /tmp/try, each said
  (run.sh:15-19). The group's token is redacted from what is committed:
  `#<the token>` 14 times in the transcript, no `#` and 40 or more token
  characters left in it or NOTES.md (`gzcat … | grep -oE
  '#[A-Za-z0-9_-]{40,}'`: none); one 8-character prefix the session wrote
  itself, elided (`#AtaAp54K…`), a scratch group's. run.sh's session
  runs `|| true` (#145).
- **Follow-ups that nothing holds** (AGENTS.md 10). The reader's README
  leaves four doubts "follow-ups, not fixed here"
  (demo/runs/start/2026-10-13-reader/README.md:81-102), in no row and no
  finding (`grep -rln` over ralph/ and docs/: only session logs). Each is
  opened below, #141-#144, with its sites, so a later row reads them.
- **The nouns gate** (fence 6). Outside well_known.txt, product code names
  llama-server once at each end, the same line (`git grep -in llama` over
  crates, apps, web and scripts, tests and well_known apart: 1 and 1).
  The range names it only in docs and its run.
- **The core untouched** (fence 6). `git diff --stat d7cf438 88d60e2 --
  crates/svrngs-core` is empty.

### Fixed here

- 9c6f5ce: demo/runs/start/2026-10-13-reader/README.md §3 cites the
  session's success to its result record, the transcript's last line
  (`subtype` success, `is_error` false, 39 turns, 239,282 ms), not to
  end.json, and says run.sh's exit holds nothing of the session's.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 141 | fence 3 | crates/svrngs-node/src/home/index.html:24 (the page's line, `svrngs share <the number after the colon in its address>`, `svrngs share 8080`); crates/svrngs/src/cli.rs:783 (`share::lend_of(…, share::well_known_kind(asked.port))`, refusing a port the table does not know); demo/start.md step 5 (`--kind llama-server`, which the document adds) | not fixed: a change in what a person is told | The one line the home page gives for lending is refused for llama-server on 18089: exit 1, "say what is on port 18089: `svrngs share 18089 --kind <kind>` …" (demo/runs/start/2026-10-13-story/README.md:215-218); start-run and start-doc's reader both met it (reader README:83-88). The refusal says what to type, so nothing is silent, but fence 3 asks the page for the line that works. A fix gives the page a line for a lend of a kind svrngs knows, `--kind` included, or lends from the page itself, with a browser test reading the page's line and running it, red first on today's. |
| 142 | 6 | crates/svrngs/src/cli.rs:1164 (`svrngs` run in a terminal: "Apps on this computer reach your groups at http://{api}", `node.api_addr()`, cli.rs:1411); demo/start.md:184-186 ("A program gets `{"error":"this page answers only itself"}`. It was given the home page's address"); crates/svrngs-node/src/home.rs:165 | not fixed: unread | The reader saw svrngs print an address for apps (http://127.0.0.1:64333) that a newcomer might give a program in place of the one **Copy its address for a program** copies; whether a program given it is answered, refused in a person's words, or answered "this page answers only itself" was not tried (reader README:89-92). Could-not-judge. A fix is a test giving a program each address svrngs prints, and the document's troubleshooting saying what each is for. |
| 143 | 7 | crates/svrngs-node/src/home/home.js:91 (the button copies `l.origin`, no `/v1`); demo/start.md step 9 | not fixed: unread | Python's `openai` answered at the copied address as it is (the story run, the reader's run); whether programs that want a base URL ending in `/v1` are answered there is unread (reader README:93-95). One program, one host: not a claim about programs. A run with a second client, or the document saying to add `/v1` where a program asks for it, waits for a row. |
| 144 | 3, 6 | demo/start.md:15 ("Neither makes an account anywhere"), :260; docs/DESIGN.md §8, the relay-park row (n0's relays the shipped default) | not fixed: the operator's, with the relay switch | Bo's page reached `usw1-1.relay.n0.iroh.link` to ask Sam's computer what the link is for (reader README:96-102); the document says no account is made and says nothing of what leaves the computer. What a link's page reaches is the operator's to decide (own-relay parked, DESIGN §8); once decided, the document says it. |
| 145 | 6 | demo/runs/start/2026-10-13-reader/run.sh:41 (`claude -p … >"$W/transcript.jsonl" 2>"$W/claude.err" \|\| true`) and :42 (end.json, `{"seconds":…}` only) | not fixed: the harness as it ran | The reader's session's exit is dropped: run.sh exits 0 whatever the session did, and end.json keeps only its seconds, so a session that died after writing part of NOTES.md reads as one that ended. This run's success stands on the transcript's result record (9c6f5ce). A fix, at the harness's next run, keeps the session's exit in end.json and says a nonzero one. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | Unchanged by the range: 1 line left in svrngs-core (3121 / 3122) and 1 in the repository (15986 / 15987), strict. |
| 130, 132-140 | as recorded | as recorded | not fixed | Not touched by this range: it changed no product code. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

None new: the range gave the instrument nothing to read. I1, I4, I19,
I28, I29 and I30 stand as REVIEW-audit-start-auto-2 recorded them.

## REVIEW-audit-onair-auto-1 (2026-10-13, range 5c38557..44b1cbc)

The previous `REVIEW-audit-` row is REVIEW-audit-start-auto-3, whose
`done` is 5c38557 (ralph/next/start/STATE.md: its hash 3766145, the loop's
"ralph: REVIEW-audit-start-auto-3 done" 5c38557 after it); this queue has
none before this one, and its first commit, 506ad71 (onair-red (a)), is
5c38557's child. The range ends at 44b1cbc, the commit that made this
row. `git rev-list --count 5c38557..44b1cbc` is 34, with no merge among
them (`--merges`: 0), all on main: onair-red, onair-economize, onair-live
and onair-sign, each with its `done`, and the row's own commit.

This audit's own commits before this section: d829cdc (the row's two
paths, as 3696cdc corrected the last audit's), 6e278f3 (the door's
private `App` named `AppBody`), e3c6a1e (the cli's `Removal` named
`RemoveKind`), d7f0eef (one `declare` and one `at_origin` in
tests/surface.rs), 44576a8 (three core test files ask
`tests_support::key`) and 5d8faef (the listener's guessed payload
fields cut). It opens #146 and #147, updates #137 and #80, and closes
none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
5c38557..44b1cbc`, summed by /tmp/onairaudit1/scripts/ledger.py
(/tmp/onairaudit1/ledger.raw, ledger.out), to the last audits' rules:
product is crates/, apps/, web/ and bridge/; a test as
`scripts/lean_check.py` names one (`/tests/`, `*_tests.rs`, `tests.rs`,
`tests_support.rs`, `*.test.mjs`); a commit that names no unit counts
toward the next one that does (3c43877 to onair-red, 389ffe3 to
onair-sign). New here: apps/ is put apart from crates/ and web/, since
`scripts/lean_check.py:14-16` does not count it, and the vendored
`@noble` code (56e636d, 47 files, +19,793, unchanged from upstream) apart
from both.

crates/ and web/:

| unit | commits | src + | src − | src net | test + | test − | test net |
|---|---:|---:|---:|---:|---:|---:|---:|
| onair-red | 9 | 0 | 0 | 0 | 0 | 0 | 0 |
| onair-economize | 8 | 204 | 218 | −14 | 130 | 9 | 121 |
| onair-live | 7 | 43 | 24 | 19 | 90 | 35 | 55 |
| onair-sign | 9 | 154 | 37 | 117 | 184 | 2 | 182 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 34 | 401 | 279 | 122 | 404 | 46 | 358 |

apps/on-air, not counted: onair-red src +980 −17 and test +60
(laws.test.mjs), onair-live src +2 −2 (listen.mjs reading the sender),
onair-sign src +4 −4 (air.mjs's domain with its `:`) and test +11 −1
(the frozen vector).

The src lines, by file:

- onair-economize: svrngs-node surface.rs +82 −21 (`read_by`, `reaches`,
  `named`), origin.rs +2 −52 (`kept_to_reads` and `not_read` gone),
  node.rs −15 (`Asking::offered_by`), home/question.js +29 −12, home.js
  +4 −13; svrngs cli.rs +71 −85 (`VALUED`, `A_KIND`, `Removal`); web/app.js
  +16 −20
- onair-live: svrngs-node rail.rs +15 −4 (`Live`), ring_routes.rs +28 −20
  (`standing_asker`)
- onair-sign: svrngs-core app.rs +60, sig.rs +44 −7 (the domains moved in
  from joining.rs −7, pass.rs −7, seed.rs −4, view.rs −4), lib.rs +1;
  svrngs-node surface.rs +43 −2 (`SignQuery`, `sign`)

Counted apart: demo/runs/onair +6338 −1 and demo/runs/onair-economize
+102 −1 (predictions, harness runs, check.sh logs); docs/DESIGN.md +98
−19, ARCHITECTURE.md +15 −5; scripts/lean_check.py +15 −15 (a0c3f78's
ratchet, 38ab822's and f61b252's raises); ralph/next/onair +5 −4.

The size budget, by scripts/lean_check.py over a `git archive` tree of
each end (/tmp/onairaudit1/lean-5c38557.out, lean-44b1cbc.out) and over
the working tree after this audit's commits:

| at | svrngs-core | repository |
|---|---:|---:|
| 5c38557, range start | 3121 / 3122 | 15986 / 15987 |
| 44b1cbc, range end | 3149 / 3150 | 16014 / 16015 |
| 5d8faef, this audit's last change (tests, an app, renames) | **3149 / 3150** | **16014 / 16015** |

The range adds 28 counted lines to each: onair-economize's −36 and its
ratchet to 15,951 (a0c3f78), onair-live's +11 under its raise to 15,962
(38ab822), onair-sign's +53 under 16,015 and svrngs-core's +28 under
3,150 (f61b252). **1 line is left in each** (#80), within the queue's
ceilings of 16,287 and 3,182.

### The instrument

~/dev/commonwealth-ai/target/debug/sovereign-cli is the binary
REVIEW-audit-start-auto-2 checked against a plant: sha256 67e46f63…,
linked 18:14 (`shasum -a 256`, `ls -la`), the same at this audit's start
and close. Its exact tier was checked again here anyway, on this corpus.

The corpus is 44b1cbc's product: crates/, apps/, web/, scripts/,
Cargo.toml and Cargo.lock, by `git archive` into
/tmp/onairaudit1/prod-44b1cbc (286 files), with
apps/on-air/page/vendor left out (upstream's code, unchanged) and demo/
left out (recorded runs). `code index . --corpus-id prod-44b1cbc` gave
3377 chunks in 458 s, the embedder the daemon on :9741,
`qwen-embedding-0.6b` (/tmp/onairaudit1/index.out). `refresh` gave rust
15266 symbols, typescript and python "skipped (export failed)" (I4), and
"prod-44b1cbc is not registered" before its in-process rebuild (I29)
(refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-core, crates/svrngs-node, crates/svrngs,
web/ and apps/on-air (`git diff --stat 5c38557 44b1cbc -- crates apps web`),
each reported at 0.95 and 0.90 (/tmp/onairaudit1/clean-*.md).

**The exact tier was checked against a plant.** 44b1cbc's `sign`
(surface.rs:173-191, added by c03b6db) was appended to
crates/svrngs-node/src/rail.rs in the indexed tree, then `refresh` (+9
symbols, refresh-plant.out). The node's report gave **2 groups**: `sign —
crates/svrngs-node/src/rail.rs:639 == crates/svrngs-node/src/surface.rs:173`,
2 copies × 19 lines, beside api.rs `addr` (plant.md); :639 is the
planted `fn`'s own line (rail.rs was 637 lines, then a blank). rail.rs
was restored (`diff` against `git show 44b1cbc:` empty) and refreshed
(−9, refresh-restored.out).

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-core/ | 469 | 5 | 0 | 12 | `key`, app_tests.rs:9 (exact) |
| crates/svrngs-node/ | 519 | 1 | 0 | 7 | none |
| crates/svrngs/ | 77 | 0 | 0 | 0 | none |
| web/ | 99 | could-not-judge (I4) | 0 | 2 | `uses` ~ `choicesOf`, web/app.js (I9) |
| apps/on-air/ | 38 | could-not-judge (I4) | 0 | 0 | none |
| crates/svrngs-node/tests/, crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

Every location the reports name, 56 (/tmp/onairaudit1/locs.txt), was
blamed at 44b1cbc against the range's 34 commits (blame.out); the filter
finds 16 of 16 in surface.rs :175-190, `sign`. Three have a line from
the range:

- `key` — crates/svrngs-core/src/app_tests.rs:9 (57dbade, 3 of 3) ==
  pass_tests.rs:4 == sig_tests.rs:6, an exact group: each is
  `tests_support::key` (tests_support.rs:24) written again. Fixed in
  44576a8: the three ask the one.
- `uses` web/app.js:285-307 (14 of 23) ~ `choicesOf` :288-303 (14 of 16),
  0.903: `choicesOf` is a closure inside `uses`, two spans of one
  function (I9). No clone.

The tests scopes are could-not-judge (I19), so the line-window pass stood
in: /tmp/onairaudit1/scripts/windows.py, the last audits' rule. At
44b1cbc it found **101 window hits, 54 spans, 3 new** (windows.out):

- crates/svrngs-node/tests/surface.rs :312-322 == :478-487 and :333-346
  == :504-517: cd80ad1's test (onair-economize (a)) copied mnp-2's
  `declare` closure and its request at a lend's origin; a third copy of
  each in `a_lent_page_is_not_held…` predates the range. Fixed in
  d7f0eef: one `declare` and one `at_origin`, the silent lender's test
  red on its plant after the move.
- crates/svrngs-node/tests/door/main.rs :289-298 (aa052a1, before the
  range) == crates/svrngs-node/tests/record_api.rs :424-433 (b3bfc0f,
  onair-live): the ada-and-bo `Roster` literal, in two test binaries
  (#137).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 5c38557..44b1cbc -- crates
apps web scripts ':!apps/on-air/page/vendor'` match
`(struct|enum|trait|type|class)\s+[A-Z]\w*` seven times
(/tmp/onairaudit1/new-types.out): `SignQuery` (surface.rs, c03b6db),
`App` and `NotAnApp` (app.rs, 57dbade), `Live` (rail.rs, b3bfc0f),
`Removal` (cli.rs, bd25915), and `Server` and `Quiet` (the harness's
public.py, 2a37770). The vendored code's 15 classes are upstream's.
Each was run through `converge noun <Name> --corpus-id prod-44b1cbc`
(nouns.out), with `git grep -nP` over every path beside it
(nouns-grep.out):

| noun | converge (production) | git grep (all, demo/ and vendor/ apart) | read |
|---|---|---|---|
| SignQuery | 1: surface.rs:163 | 1: surface.rs:164 | One. |
| App | **2**: svrngs-core app.rs:21; svrngs-node door/choices.rs:74 | 2: app.rs:22, choices.rs:75 | Two things of one name in the node's reach: the core's, an app's name as it is signed under, which surface.rs imports; and choices.rs's private body an app posts to stop. **Fixed in 6e278f3**: the private one is `AppBody`. |
| NotAnApp | 1: app.rs:27 | 1: app.rs:28 | One. |
| Live | 1: rail.rs:452 | 1: rail.rs:453 | One. |
| Removal | **2**: svrngs cli.rs:314; svrngs-core register.rs:139 | 2: cli.rs:315, register.rs:140 | The core's is the command asking a person out, `svrngs_node::Removal`, and a handover is not one (`HandOnTo`); the cli's is "remove or handover". **Fixed in e3c6a1e**: the cli's is `RemoveKind`, the kind of `Verb::Remove`. |
| Server | 0 | 2: apps/jellyfin/mw/src/page.rs:22, apps/on-air/harness/public.py:13 | Two apps, two languages, each its own; converge sees neither (I4 for Python, I19 for jellyfin's workspace). |
| Quiet | 0 | 1: public.py:17 | One; converge cannot see it (I4). |

After the two renames, the renamed files put into the indexed tree and
`refresh` run again (refresh-renamed.out), converge gives `App` 1
(app.rs), `Removal` 1 (register.rs), `AppBody` 1 and `RemoveKind` 1
(nouns-after.out). converge's lines are still one early (I1). **No noun
has more than one definition now.**

### Reading the range

Every hunk of product code the range changed in crates/ and web/ was read
(/tmp/onairaudit1/range-src.diff), and the app's own files: station.mjs,
listen.mjs, page/air.mjs, page/page.js, harness/stranger.mjs, public.py
and run.py's judging.

- **A move is its own commit** (AGENTS.md 2). 79b8301 moved every domain
  into sig.rs before 57dbade added the app's; 5058650's parse helper
  landed apart from bd25915's variant; each with its tests named.
- **The core is pure** (fence 3). app.rs signs and checks with the key
  and the bytes as arguments, no I/O; its domain, `APP_DOMAIN`, lives
  beside every other in sig.rs (:53-103), and `sig_tests.rs`'s
  `domains_live_here_and_none_begins_another` holds that no core file
  outside sig.rs spells one. The node only carries it (surface.rs
  `sign`).
- **One decider** (AGENTS.md 8). What a lent page reads and reaches is
  one function, surface.rs `read_by`, which `reads` and `reaches` ask
  (cd80ad1). The app's domain is spelled twice, across two languages:
  svrngs-core's sig.rs `APP_DOMAIN` with app.rs `message`, and the
  page's air.mjs `appDomain`; laws.test.mjs checks the core's frozen
  vector (app_tests.rs `a_frozen_vector_for_a_stranger`), byte for byte,
  the golden equivalence ARCH_PRINCIPLES 8 asks of two that must exist.
  One decision is spelled twice: whether a key stands in a group,
  `Standing::names` (svrngs-runtime register.rs:90) and ring_routes.rs
  `standing_asker`'s own match (:102), which b3bfc0f wrote to take the
  member's row with the answer (#146).
- **Never silently substitute** (AGENTS.md 6; fence 2). A silent lender
  is could-not-judge, `away`, on both paths (surface.rs `read_by`,
  `reaches`). The listener refused what it could not check, said why,
  but read a payload's text and sender under two field names no svrngs
  drain has given (`p.text`, `p.sender`), a guess that would be taken
  as the sender if ever met. Fixed in 5d8faef. `/v1/sign` with no `app`
  is read as the empty name and refused as not a name; nothing is signed.
  The harness says never-ran for a bar whose step did not run, and
  could-not-judge is never a pass (run.py `judge`).
- **Glassbox** (AGENTS.md 1). `sign`'s refusal and its signing trace
  (target `sign`), `read_by`'s three refusals and `reaches`' three
  outcomes trace (target `surface`), and the live route traces its
  buffering with the sender's two keys (ring_routes.rs `ring_live`).
- **Closed sets are enums** (AGENTS.md 9). `RemoveKind` (then `Removal`)
  is an enum. The flags that take a value are a space-joined `&str`
  split at each argument (cli.rs `VALUED`, :306-307, read at :350), a
  closed set kept as text (#147).
- **The nouns gate** (fence 6). `scripts/vocab_check.py` (check.sh
  :22-23) says "47 app nouns, 0 in infra code", exit 0. `git grep -niwE
  'station|show|radio|broadcast' 44b1cbc -- crates web scripts`, tests
  apart, finds `show` as a verb and choices.rs:5's "a private grant is
  not broadcast" (d9cb3c4, before the range); none names the app's.

### Fixed here

- 6e278f3: the door's private `App` (choices.rs:75) is `AppBody`;
  converge's `App` 2 → 1.
- e3c6a1e: the cli's `Removal` (cli.rs:315) is `RemoveKind`; converge's
  `Removal` 2 → 1.
- d7f0eef: tests/surface.rs's three `declare` closures and three origin
  requests are one fn each; red on its plant after the move.
- 44576a8: app_tests, pass_tests and sig_tests ask `tests_support::key`;
  the core's exact group with a side in the range, 3 copies → 0.
- 5d8faef: listen.mjs reads a drained payload under the fields svrngs
  gives, `payload`, `key`, `person_key`, and a bare string as before.

`scripts/check.sh` at c51450f, once, at the close: exit 0 in 661 s,
every step passed, LINT (rustfmt, clippy, web rustfmt and clippy) among
them (/tmp/onairaudit1/check.out).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 146 | 8 | crates/svrngs-node/src/ring_routes.rs:102 (`standing_asker`: `Ok(Standing::Members(mut standing)) if standing.contains_key(&asker)`); crates/svrngs-runtime/src/register.rs:90 (`Standing::names`, `matches!(self, Standing::Members(m) if m.contains_key(actor))`), which ring_sync.rs:366 asks | fixed: aff607b (events-principles (c)): `Standing::member`, which `names`, `standing_asker` and `own_person` ask; ring_routes/tests.rs `the_routes_serve_the_keys_the_register_names`, red on a plant in `names` | Whether a key stands in a group is decided in two places. b3bfc0f (onair-live) needed the member's row with the answer, so it wrote `names`'s rule again inline in place of asking it. They agree today, word for word; a change to who stands (a key standing while its person is being removed, say) made in one would leave the other answering. A fix gives `Standing` one accessor, `member(&self, actor) -> Option<&Member>`, which `names` asks and `standing_asker` takes the row from; about +4 counted lines, at 16014 / 16015. |
| 147 | 9 | crates/svrngs/src/cli.rs:306-307 (`const VALUED: &str = "--as --origin --name …"`) and :350 (`VALUED.split(' ').any(\|f\| f == a)`) | fixed: 13dbdbe (events-principles (d)): an or-pattern in `valued`'s arm, 0 lines net; tests/flags.rs, red on a doubled space and on a misspelling | 5058650 (onair-economize (b)) kept the flags that take a value as one space-joined string, split on every argument. A closed set kept as text, which no compiler or test reads as a set: a doubled space puts the empty string among the flags, so an empty argument would take the next as its value before it is refused; a flag misspelled there is a flag that takes no value, and its value is then read as the next argument. A `&[&str]`, or each flag's arm saying it takes a value, is the idiom; rustfmt puts the twelve on lines of their own. Waits for lines freed, or the operator's raise. |
| 137 | 8 | as recorded; and now crates/svrngs-node/tests/door/main.rs:289-298 (aa052a1, `Bos::new`) == crates/svrngs-node/tests/record_api.rs:424-433 (b3bfc0f, `a_payload_drained_names_its_sender_and_their_person`) | not fixed: two test binaries | Updated. onair-live's test writes the ada-and-bo `Roster` literal (ada by key 1, bo by keys 2 and 6) the door's tests already had. The two binaries share no support module for a seed; tests/common, which both include, is where a `two_people(…)` would go. |
| 80 | — | scripts/lean_check.py BUDGET | not fixed (operator's) | 1 line left in svrngs-core (3149 / 3150) and 1 in the repository (16014 / 16015), strict, after the range's +28 each. onair-blob raises from there, within the queue's 16,287 and 3,182. |
| 130, 132-145 | as recorded | as recorded | not fixed | Not touched by this range's decisions. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (app.rs:21 for :22, choices.rs:74 for :75, cli.rs:314 for :315, register.rs:139 for :140, rail.rs:452 for :453). The dry-report half reported the plant at its own `fn` line (rail.rs:639). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/'s and apps/on-air's exact tiers are could-not-judge; converge gives public.py's `Server` and `Quiet` 0 definitions. |
| I9 | 6, 7 | `code dry-report` near tier | Still present: `uses` ~ `choicesOf` (web/app.js :285-307, :288-303), a closure and the function it sits in. |
| I19 | 6 | dry-report, `tests/` scopes; converge in apps/jellyfin/mw | Still present, and said as could-not-judge. The line-window pass stood in for the tests scopes; converge gives `Server` 0 where page.rs:22 defines one. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: "prod-44b1cbc is not registered", then the in-process rebuild and exit 0. |
| I30 | 7 | the binary by path | Not met this time: 67e46f63… at this audit's start and close. |

## REVIEW-audit-onair-auto-2 (2026-10-14, range 88b2595..969d8c9)

The previous `REVIEW-audit-` row is REVIEW-audit-onair-auto-1, its hash
88b2595 (ralph/next/onair/STATE.md). The range ends at 969d8c9, the
commit that made this row. `git rev-list --count 88b2595..969d8c9` is 17,
none a merge (`--merges`: 0): onair-blob and onair-run, each with its
`done`, auto-1's `done` (6e55692), the events queue's rows (2843760) and
this row's own commit.

This audit's own commits before this section: cd0e89d (the row's two
paths, as d829cdc corrected auto-1's), 0213ebb (blob.rs's private `Asked`
named `Replies`), 5bf3db5 (a get's `unanswered` refusal written once) and
036a456 (a test of it, red first). It opens #148 and #149, updates #146
and #80, and closes none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
88b2595..969d8c9` (/tmp/onairaudit2/ledger.raw), summed by
/tmp/onairaudit2/scripts/ledger.py (ledger.out) to auto-1's rules:
product is crates/, web/ and bridge/; apps/ apart, which
`scripts/lean_check.py:14-16` does not count; a test as lean_check names
one; a commit that names no unit counts toward the next one that does
(6e55692 to onair-blob, 2843760 to onair-run).

| unit | commits | src + | src − | src net | test + | test − | test net | apps + | apps − |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| onair-blob | 8 | 335 | 26 | 309 | 132 | 0 | 132 | 8 | 3 |
| onair-run | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 22 | 4 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 17 | 335 | 26 | 309 | 132 | 0 | 132 | 30 | 7 |

The src lines, by file, all onair-blob's db6ba51: svrngs-node blob.rs
+276 (new), folder.rs +55 −23 (`carried`, `serves`/`keep`/`read_at` taught
`now`, the routes), node.rs +3 −2 (the members' route), ring_routes.rs
+1 −1 (`standing_asker` made `pub(crate)` for blob.rs's ask route).
apps/on-air: harness/run.py alone (429525c, 1b281c6, 6440fc3).

Counted apart: demo/runs/onair +11,923 (two runs and four plants);
docs/DESIGN.md +50 −5, ARCHITECTURE.md +9 −1; scripts/lean_check.py +3 −3
(d0347ae's raise); ralph/next +191 −3.

The size budget, by scripts/lean_check.py over a `git archive` tree of
each end (lean-88b2595.out, lean-969d8c9.out) and over the working tree
after this audit's commits:

| at | svrngs-core | repository |
|---|---:|---:|
| 88b2595, range start | 3149 / 3150 | 16014 / 16015 |
| 969d8c9, range end | 3149 / 3150 | 16236 / 16237 |
| 036a456, this audit's last change | **3149 / 3150** | **16236 / 16237** |

onair-blob's +222 counted lines under its raise to 16,237 (d0347ae).
lean_check fails at `n >= cap` (scripts/lean_check.py:99): **no line is
left in the repository, 1 in svrngs-core** (#80), within the queue's
16,287 and 3,182.

### The instrument

~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 67e46f63…
(`shasum -a 256`), the binary auto-1 ran, the same at this audit's start
and close.

The corpus is 969d8c9's crates/, apps/, web/, scripts/, Cargo.toml and
Cargo.lock by `git archive` into /tmp/onairaudit2/prod-969d8c9 (287
files), apps/on-air/page/vendor left out (upstream's) and demo/ left out
(recorded runs). The first `code index . --corpus-id prod-969d8c9` exited
1, "no embedder" (index.out's first run, overwritten): every endpoint
"unavailable", :9741 among them, though `curl localhost:9741/v1/models`
answered a moment after. Run again it gave 3402 chunks in 394 s. `refresh`
gave rust 15450 symbols, typescript and python "skipped (export failed)"
(I4), and "prod-969d8c9 is not registered" before its in-process rebuild
(I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node and apps/on-air (`git diff --stat
88b2595 969d8c9 -- crates apps web scripts`), each reported at 0.95 and
0.90, with crates/svrngs-node/tests (/tmp/onairaudit2/clean-*.md).

**The exact tier was checked against a plant.** blob.rs `ask` (:139-150,
db6ba51) was appended to the indexed tree's rail.rs and `refresh` run (+10
symbols, refresh-plant.out). The node's report at 0.95 gave `ask —
crates/svrngs-node/src/blob.rs:141 == crates/svrngs-node/src/rail.rs:641`,
2 copies × 10 lines, beside api.rs `addr` (plant.md); :641 is the planted
`fn`'s line (rail.rs was 637 lines, then a blank and two doc lines).
rail.rs was restored (`diff` against `git show 969d8c9:` empty) and
refreshed (−10, refresh-restored.out).

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 531 | 1 | 0 | 7 | none |
| apps/on-air/ | 38 | could-not-judge (I4) | 0 | 0 | none |
| crates/svrngs-node/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

The node's groups are api.rs `addr` ×2, and near pairs in door/sign_in.rs,
synced.rs, redeem.rs, door/choices.rs, rail.rs, ring_sync/tests.rs and
lend.rs: none in a file the range changed (blob.rs, folder.rs, node.rs,
ring_routes.rs), so no side was added in the range. The report counts 45
files where auto-1's counted 44: blob.rs is read.

The tests scope is could-not-judge (I19), so a line-window pass stood in:
/tmp/onairaudit2/scripts/windows.py, 8 whitespace-normalized lines,
blanks and lone braces dropped, a span new when `git blame` at the end
puts any of its lines in the range. Over crates/svrngs-node/tests at
969d8c9: **145 window hits, 50 spans, 0 new** (windows.out). Its plant: a
commit no ref names (9e455fc, `commit-tree` over 969d8c9) adding
tests/plant.rs, a copy of folders.rs :1362-1385 (`put_blob` and
`get_blob`): 167 hits, 51 spans, **1 new**, `folders.rs:1362-1385 ==
plant.rs:1-24` (windows-plant.out).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 88b2595..969d8c9 -- crates apps
web scripts ':!apps/on-air/page/vendor'` match
`(struct|enum|trait|type|class)\s+[A-Z]\w*` five times, all blob.rs
(new-types.out): `Holds`, `BlobQuery`, `Asked`, `Carried`, `People`. Each
through `converge noun <Name> --corpus-id prod-969d8c9` (nouns.out), with
`git grep -nP` beside it (nouns-grep.out):

| noun | converge (production) | git grep (demo/ and vendor/ apart) | read |
|---|---|---|---|
| Holds | 1: blob.rs:44 | 1: blob.rs:45 | One. |
| BlobQuery | 1: blob.rs:52 | 1: blob.rs:53 | One. |
| Asked | **2**: svrngs-core joining.rs:64; svrngs-node blob.rs:136 | 2: joining.rs:65, blob.rs:137 | The core's is a link redeemed, re-exported into the node (redeem.rs:65, `pub use svrngs_core::joining::{…, Asked, …}`); blob.rs's is a private alias for what the group's computers said of a hash. **Fixed in 0213ebb**: it is `Replies`. |
| Carried | 1: blob.rs:38 | 1: blob.rs:39 | One. |
| People | 1: blob.rs:133 | 2: blob.rs:134, apps/jellyfin/mw/src/people.rs:26 | One in svrngs; jellyfin's is its own app's (I19: converge does not see that workspace). |

After the rename, blob.rs put into the indexed tree and `refresh` run
(refresh-renamed.out), converge gives `Asked` 1 (joining.rs) and
`Replies` 1 (blob.rs) (nouns-after.out). converge's lines are still one
early (I1). **No noun has more than one definition now.**

### Reading the range

Every hunk of product code the range changed was read
(/tmp/onairaudit2/range-src.diff, blob.rs whole), and run.py's three
fixes.

- **One decider** (AGENTS.md 8; fence 4). Who receives a group's bytes is
  still folder.rs `serves` alone, taught `held` (folder.rs:463);
  nothing in blob.rs serves bytes beside it, and its ask route asks
  ring_routes.rs `standing_asker` rather than writing a dialer's standing
  again. The store, its sweep and the verified fetch are folder.rs's,
  reused (`keep`, `hold`, `lacks`). A get's refusal `unanswered` was
  written twice (:233, :250): **fixed in 5bf3db5**. "This computer stands
  in the group" is written once more, blob.rs `standing_in` (:89-97),
  beside synced.rs:166 and surface.rs:229-234 (#146).
- **Never silently substitute** (AGENTS.md 6; fence 2). Lapsed, never held
  and unanswered are three kinds, three statuses (410, 404, 502), and
  "never held" only when every computer asked said so (:233); 036a456
  holds that last, red on its plant. Two ways a computer's silence can
  still be lost (#148).
- **Glassbox** (AGENTS.md 1). `get`'s refusals trace through `refused`
  (:154-158), each computer's answer at debug (:127), the provider's
  blob branch (folder.rs:464). `put`'s two refusals do not (#149).
- **Closed sets are enums** (AGENTS.md 9). `Holds` is one, the wire form
  of the ask. A refused get's `kind` is a `&str` of three values, two of
  which are `Holds`'s serde names; one closed set kept partly as text,
  left with #149's lines.
- **The nouns gate** (fence 6). `scripts/vocab_check.py`: "47 app nouns,
  0 in infra code". `git grep -niwE 'station|show|radio|broadcast' HEAD
  -- blob.rs folder.rs`: nothing.
- **The harness** (apps/, not counted). 429525c reads "4.1, scoped"
  passed on Dee's own computer refusing as well as on Bo's provider, and
  says why in run.py (:357-360): the provider's group check is
  tests/folders.rs's (`bytes_an_app_put_in_one_group_are_refused_to_a_member_of_another`),
  red on the group-check-out plant (demo/runs/onair/2026-10-13/plants/
  group-check-out/red.txt). Said, not substituted.

### Fixed here

- 0213ebb: blob.rs's private `Asked` is `Replies`; converge's `Asked` 2 → 1.
- 5bf3db5: a get's `unanswered` refusal one `let`, beside `lapsed`;
  0 counted lines.
- 036a456: tests/folders.rs's carried test asks a hash only Ana's
  stopped computer holds, and is told `unanswered`, Ana named; red on
  the plant `(0, n) if n >= 1 => never`.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 148 | 6 | crates/svrngs-node/src/blob.rs:126 (`join_all(asks).await.into_iter().flatten()`) and :233; blob.rs:249 (`f.carry(hash, &group, None)`) | fixed: 0c55dc6 (events-principles (a)): `heard` says a failed task's ask under its key; `given` keeps a lapse this computer was given past a failed fetch; blob_tests.rs, both red on their plants | Two silences lost in a get. (a) An ask whose task panicked is a `JoinError`, which `.flatten()` drops: the computer is not in `answers`, so if every other one said never held, :233's `n == answers.len()` says "never held" with one computer unheard. Spawning with the actor kept outside the task and its `JoinError` said as its answer is about +3 lines. (b) When this computer carries a hash not lapsed but its store lacks the bytes (:211 falls through), and fetching from the holders fails, :249 removes its own entry, so asked again it says never held of a hash it was given a lapse for. Only a store that lost what the sweep keeps reaches it; forgetting only an entry `get` itself made is about +2. |
| 149 | 1 | crates/svrngs-node/src/blob.rs:175-177 (a put with no `keep`) and :180-181 (the store refusing the bytes); :216 (`_ => {}`: held here, the store lacking it); crates/svrngs-node/src/ring_routes.rs:48 (`err`, which traces nothing) | fixed: 4bd7fdf (events-principles (b)): `err` traces each refusal it writes; the get's fall-through traced with `given`; blob_tests.rs `a_refused_put_and_a_get_that_asks_the_group_are_traced`, red on each. The refused get's `kind` as an enum is left as the finding left it, a could | Two of `put`'s refusals and one of `get`'s branches leave no trace, so a person reading the trace of an app whose put was refused finds nothing (AGENTS.md 1, "each refusal included"). A `tracing::debug!` at each is +1 to +3 lines at 16236 / 16237. With it, a refused get's `kind` could become an enum beside `Holds` (AGENTS.md 9). |
| 146 | 8 | as recorded; and now crates/svrngs-node/src/blob.rs:89-97 (`standing_in`: `standing_of` then `people.contains_key(self.host.actor())`), beside synced.rs:166 and surface.rs:229-234 | fixed in part: aff607b (events-principles (c)): synced.rs asks `names`; blob.rs `standing_in` and surface.rs read `standing_of`'s map, not a `Standing`, and are left | Updated. Whether this computer's own key stands in a group is now written a third time in the node, each after `standing_of` or the register. The accessor #146 names, `Standing::member`, would serve all three. |
| 80 | — | scripts/lean_check.py:32 BUDGET, :99 `n >= cap` | not fixed (operator's) | 1 line left in svrngs-core (3149 / 3150) and **none** in the repository (16236 / 16237) after onair-blob's +222. The events queue adds nothing to svrngs (2843760); any row that does raises first, after an economizing pass. |
| 130, 132-145, 147 | as recorded | as recorded | not fixed | Not touched by this range. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (blob.rs:44 for :45, :52 for :53, :136 for :137, :38 for :39, :133 for :134; joining.rs:64 for :65). dry-report named the plant at its own `fn` line (rail.rs:641). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)"; apps/on-air's exact tier could-not-judge. |
| I19 | 6 | dry-report, `tests/` scopes; converge in apps/jellyfin/mw | Still present, said as could-not-judge; the line-window pass stood in. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: "prod-969d8c9 is not registered", then the in-process rebuild and exit 0. |
| I31 | 6, 7 | `code index`'s endpoint probe | New: its first run found no embedder, :9741 "unavailable" by a GET to /v1/models, while `curl` of the same URL answered moments after; run again, it indexed. A probe that fails once says "unavailable" with no retry and no time; one run, not a rate. |

## REVIEW-audit-events-auto-1 (2026-10-14, range 2843760..43c0552)

The events queue has no earlier `REVIEW-audit-` row (ralph/next/events/
STATE.md), so the range starts where the queue did, 2843760 (its three
rows), and ends at 43c0552, the commit that made this row. `git rev-list
--count 2843760..43c0552` is 29, none a merge (`--merges`: 0): onair-run's
close and `done`, REVIEW-audit-onair-auto-2 whole (its own fixes, 0213ebb,
5bf3db5 and 036a456, audited by no row before this one), events-app,
events-run and events-doc, each with its `done`, and this row's own commit.

This audit's own commits before this section: fb7db91 (the row's two
paths, as cd0e89d corrected onair-auto-2's) and 370eec8 (the page's word
limit read from the fold). It opens #150 and #151, and closes none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
2843760..43c0552` (/tmp/eventsaudit1/ledger.raw), summed by
/tmp/eventsaudit1/scripts/ledger.py (ledger.out) to the earlier audits'
rules: product is crates/, web/ and bridge/; apps/ apart, which
`scripts/lean_check.py:14-16` does not count, split the same way; a test as
lean_check names one, and `*.test.mjs`; a commit that names no unit counts
toward the next one that does (276fff3 to events-run).

| unit | commits | src + | src − | src net | test + | test − | test net | apps src + | apps src − | apps test + | apps test − |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| onair-run | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| REVIEW-audit-onair-auto-2 | 8 | 4 | 4 | 0 | 17 | 1 | 16 | 0 | 0 | 0 | 0 |
| events-app | 4 | 0 | 0 | 0 | 29 | 5 | 24 | 815 | 0 | 412 | 0 |
| events-run | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 29 | 10 | 9 | 7 |
| events-doc | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 29 | 4 | 4 | 0 | 46 | 6 | 40 | 844 | 10 | 421 | 7 |

The src lines, all auto-2's: svrngs-node blob.rs +4 −4 (0213ebb `Asked`
named `Replies`, 5bf3db5 `unanswered` one `let`). The tests: auto-2's
folders.rs +17 −1 (036a456), and events-app's canaries.rs +29 −5 (f1df40f:
`admit` given a group, and `events`). **The events queue added no line of
svrngs's product**, as its rows say (fence 3). apps/events: events.js 243,
server.mjs 316 (+3 −3 in a22edd0), store.mjs 103, canary.mjs 131,
nouns.txt 11, README.md, and its three test files.

Counted apart: demo/runs +17,360 −13 (events-run's two takes, 37 pictures
binary), demo/events.md +286; docs/DESIGN.md +14 −1; ralph/next +8 −6;
ralph/REVIEW_FINDINGS.md +203 −4 (auto-2's section).

The size budget, by scripts/lean_check.py over a `git archive` tree of
each end (lean-2843760.out, lean-43c0552.out) and over the working tree
after this audit's commits:

| at | svrngs-core | repository |
|---|---:|---:|
| 2843760, range start | 3149 / 3150 | 16236 / 16237 |
| 43c0552, range end | 3149 / 3150 | 16236 / 16237 |
| 370eec8, this audit's last change | **3149 / 3150** | **16236 / 16237** |

Unchanged across the range: auto-2's moves net 0, and apps/ uncounted.
Still **no line left in the repository, 1 in svrngs-core** (#80).

### The instrument

~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 67e46f63…
(`shasum -a 256`), the binary onair-auto-2 ran and planted.

The corpus is 43c0552's crates/, apps/, web/, scripts/, Cargo.toml and
Cargo.lock by `git archive` into /tmp/eventsaudit1/prod-43c0552 (296
files), apps/on-air/page/vendor left out (upstream's). `code index .
--corpus-id prod-43c0552` gave 3483 chunks in 332 s, its first run
(index.out; I31 not seen this time, one run). `refresh` gave rust 15467
symbols, typescript and python "skipped (export failed)" (I4), and
"prod-43c0552 is not registered" before its in-process rebuild (I29)
(refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node and apps/events (`git diff --stat
2843760 43c0552 -- crates apps web scripts bridge`), each reported at 0.95
and 0.90, with crates/svrngs-node/tests; and apps/ whole, since events was
made in expenses' shape and a clone across the two apps is outside either
app's scope (/tmp/eventsaudit1/clean-*.md).

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 531 | 1 | 0 | 7 | none |
| apps/events/ | 55 | could-not-judge (I4) | 0 | 2 | 2, both store.mjs |
| apps/ | 385 | could-not-judge (I4) | 9 | 22 | 7 at 0.95, 13 at 0.90 |
| crates/svrngs-node/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

The node's groups are auto-2's, the same eight (api.rs `addr` ×2; near
pairs in door/sign_in.rs, synced.rs, redeem.rs, door/choices.rs, rail.rs,
ring_sync/tests.rs, lend.rs): none in blob.rs, the one src file the range
changed, so no side was added in the range.

Every cluster with a side the range added, apps/ at 0.90 (the 0.95 ones
marked •):

| cosine | apps/events | its other side | read |
|---:|---|---|---|
| 1.000 • | laws.test.mjs:58–66 `constructor` | expenses/laws.test.mjs:48–56 | The five laws' scaffolding, copied (#150). |
| 1.000 • | laws.test.mjs:61–69 `now` | expenses/laws.test.mjs:51–59 | The same (#150). |
| 0.996 • | canary.mjs:52–66 `unheld` | expenses/canary.mjs:54–68 | The canary's wait, copied with WAIT_MS (#150). |
| 0.992 • | laws.test.mjs:53–82 `elsewhere` | expenses/laws.test.mjs:43–72 | #150. |
| 0.984 • | laws.test.mjs:80–90 `determinism` | expenses/laws.test.mjs:70–80 | #150. |
| 0.974 • | laws.test.mjs:46–55 `said` | expenses/laws.test.mjs:37–45 `answers` | #150; events' also folds `answers`. |
| 0.962 • | canary.mjs:42–52 `append` | expenses/canary.mjs:43–53 | #150; events' names the group. |
| 0.949 | canary.mjs:31–44 `call` | expenses/canary.mjs:32–45 | #150; events' names the group. |
| 0.927 | store.mjs:49–58 `log` | store.mjs:53–61 `append` | Not a clone: `record`'s two methods, four and three lines (:52-55, :56-58), padded to the window (I32). |
| 0.920 | laws.test.mjs:110–134 `totality` | expenses/laws.test.mjs:98–120 | Each its own app's hostile acts; the loop around them #150's. |
| 0.917 | store.mjs:81–89 `people` | store.mjs:84–93 `log` | Not a clone: `file`'s (:84-86, :87-90), as above (I32). |
| 0.904 | events.js:116–127 `actsOf` | expenses/expenses.js:125–136 | Not one decider twice: events keeps voided ops and decides whose correction counts (events.js:111-118), expenses drops them (expenses.js:120-127). Both read an op's person as `on_behalf_of ?? person` (:123, :132), which is the record's (DESIGN §3), said in each app. |
| 0.903 | laws.test.mjs:100–114 `idempotence` | expenses/laws.test.mjs:88–102 | #150; events skips an answer a correction names (:100-102). |

The line-window pass (/tmp/eventsaudit1/scripts/windows.py, onair-auto-2's
unchanged, its plant there) over apps/events, apps/expenses and
crates/svrngs-node/tests at 43c0552: **185 window hits, 52 spans, 2 new**
(windows.out): `apps/events/canary.mjs:54-67 == apps/expenses/canary.mjs:56-69`
and `apps/events/laws.test.mjs:53-81 == apps/expenses/laws.test.mjs:43-71`,
both #150. None in the node's tests: canaries.rs's `events` is its own.
(Its first run read 0 hits: zsh passed the 37 paths as one word; split by
`xargs`, run again.)

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 2843760..43c0552 -- crates apps
web scripts bridge ':!apps/on-air/page/vendor'` match
`(struct|enum|trait|type|class)\s+[A-Z]\w*` twice (new-types.out): `Said`
(server.mjs:55, `class Said extends Error`) and `Replies` (blob.rs:137).
Each through `converge noun <Name> --corpus-id prod-43c0552` (nouns.out),
with `git grep -nP` beside it (nouns-grep.out):

| noun | converge (production) | git grep (demo/ apart) | read |
|---|---|---|---|
| Said | **2**: svrngs cli.rs:1042; svrngs-node door/choices.rs:39 | 5: cli.rs:1043, choices.rs:40, tests/folders.rs:1051, web/app.js:83, apps/events/server.mjs:55 | The two Rust ones as :4459 above read them, three concepts, no change. events' is web/app.js's concept, a page's refusal in a person's words with its status, on the far side of the app's line: the app runs alone (EVENTS.md §5) and lends no svrngs code, so it is a fork across two systems, not two deciders in one (AGENTS.md 8). Its arguments run `(status, words)`, app.js's `(words, status)`; nothing passes one to the other. No change. converge does not see either JS class (I4). |
| Replies | 1: blob.rs:136 | 1: blob.rs:137 | One (auto-2's rename). |

converge's lines are still one early (I1).

### Reading the range

Every hunk of product code and of apps/events the range changed was read
(`git diff 2843760 43c0552 -- crates apps`, events.js and server.mjs
whole).

- **One decider** (AGENTS.md 8). The words an event may hold were limited
  in two homes: events.js `MAX_WORDS` (:46), which `validate` holds an
  event to (:95-99), and the page's `maxlength="200"`, three times
  (server.mjs:160-162). **Fixed in 370eec8**: the page reads `MAX_WORDS`.
  The five laws' judge is in two apps' tests, and the canary's wait in two
  canaries (#150). Who stands in a group is `store.people` alone, read by
  `standing` (server.mjs:82-93) and `myGroups` (:99-113).
- **Never silently substitute** (AGENTS.md 6; fence 2). A group that
  cannot be read is said as that on the home page (:143, :155), on its
  event page (:88) and in the host's view (:231). A group read in part is
  not: the record says `complete: false` when it knows of acts it does not
  hold (rail.rs:384, admit.rs:423), and the page says so only for the
  path's group (:214); the home page's list and the host's other groups
  drop it (#151).
- **Glassbox** (AGENTS.md 1). Every refusal goes through the one catch's
  `answered` trace (:310-312), with its status and words; standing, each
  write, each change of mind and each view trace at their branch. When
  `EVENTS_DEBUG` is unset nothing is traced; it is the app's switch.
- **Machines build, people speak** (fence 4). `write` (:123-135) writes
  every act as the person asking, `name` from `standing`; no route takes
  another person's name. A guest's `came` is keyed by its writer
  (events.js:180).
- **Closed sets** (AGENTS.md 9). The three kinds are `EVENT`, `COMING`,
  `CAME` and `FIELDS` (events.js:43-54); the routes' three verbs a list
  (server.mjs:296). JS, so constants, as expenses keeps them.
- **The nouns gate** (fence 3). `python3 scripts/vocab_check.py`: "53 app
  nouns, 0 in infra code".
- **The run's driver** (demo/runs/events, not counted) is kept byte for
  byte as it ran (#61) and was not reviewed as product.

### Fixed here

- 370eec8: the page's word limit is events.js `MAX_WORDS`; the page byte
  for byte the same, and moving with a plant of 150.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 150 | 8 | apps/events/laws.test.mjs:53-81 (`elsewhere`, `determinism`) and :159 (`broken`) == apps/expenses/laws.test.mjs:43-71 and :129; apps/events/canary.mjs:32 (`WAIT_MS = 10_000`) and :54-67 (`unheld`, `arrived`) == apps/expenses/canary.mjs:33 and :56-69 | not fixed: a move across two apps, not small | RING_APP_LIBRARY §4's five laws and the sixth are judged by two copies of one harness, events' citing expenses' as its source (events laws.test.mjs:1); a law tightened in one stays loose in the other with nothing red. The canary's wait is copied too, and its bound twice: events' cites expenses' WAIT_MS (:31) and spells 10_000 again, and only expenses' says why 10 s (:30-32). One home for both, e.g. `apps/laws.mjs` that each app's tests import with its own journal and plants, is a move each test re-proves red on its plants; apps/, so no counted line. A third app with laws makes it three. |
| 151 | 6 | apps/events/server.mjs:224 (`(await folded(g.id)).answers`, `complete` dropped) and :148-152 (the home list, "Nothing posted yet." without `complete`); beside :214, which says it for the path's group alone | fixed: 6919521 (events-principles (f)): `partly`, said per group on the home list and in each section of either view; apps/events/partial.test.mjs, red on the home list and on the host's view | A group whose record knows of acts it does not hold (rail.rs:384 `complete`, admit.rs:423) is shown in the host's view, or on the home page, as if it held them all: the home page can say "Nothing posted yet." of a group that has posts this computer has not received, and the host's other groups show their answers with no "Some answers may not have reached this computer yet." Fence 2 says an absence is said; carrying `complete` with each group's answers and saying it per section is about +4 lines in apps/, uncounted. |
| 80 | — | scripts/lean_check.py:32 BUDGET, :99 `n >= cap` | not fixed (operator's) | Unchanged: 1 line left in svrngs-core (3149 / 3150), **none** in the repository (16236 / 16237). The events queue added none. |
| 130, 132-149 | as recorded | as recorded | not fixed | Not touched by this range but for auto-2's own commits, read in its section. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (cli.rs:1042 for :1043, choices.rs:39 for :40, blob.rs:136 for :137). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)"; apps/events' and apps/'s exact tier could-not-judge, and converge blind to JS classes. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, said as could-not-judge; the line-window pass stood in. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: "prod-43c0552 is not registered", then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's JS near tier | New: a JS object's methods are named over spans padded three lines each side, so two short neighbours overlap and read as a clone: `log` "store.mjs:49–58" for :52-55 and `append` "53–61" for :56-58, cosine 0.927; `people` "81–89" for :84-86, `log` "84–93" for :87-90. A cluster whose two spans overlap in one file is the window, not the code. |

## REVIEW-audit-events-auto-2 (2026-10-14, range 28af323..b1d04bc)

The previous `REVIEW-audit-` row of this queue is REVIEW-audit-events-auto-1,
its hash 28af323 (ralph/next/events/STATE.md), so the range starts there
and ends at b1d04bc, the commit that made this row. `git rev-list --count
28af323..b1d04bc` is 45, none a merge (`--merges`: 0): auto-1's `done`, the
five rows' state commit, events-economize, events-asleep, events-links and
events-principles, each with its `done`, and this row's own commit.

This audit's own commits before this section: ba1124d (the row's two paths,
as fb7db91 corrected auto-1's), 389094a (`Joined` names one thing in the
node), 3844bc2 (the node's tests' trace sink, one copy for two) and 0e5818d
(the door asks `Standing::names`). It opens #152-#154, and closes none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
28af323..b1d04bc` (/tmp/eventsaudit2/ledger.raw), summed by
/tmp/eventsaudit2/scripts/ledger.py (ledger.out; each commit's unit in
ledger.commits) to the earlier audits' rules: product is crates/, web/ and
bridge/; apps/ apart, which `scripts/lean_check.py:14-16` does not count,
split the same way; scripts/ apart; a test as lean_check names one, and
`*.test.mjs`; a commit that names no unit counts toward the next one that
does (460898b to events-economize). Raw lines, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-events-auto-1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| events-economize | 5 | 13 | 13 | 0 | 0 | 0 | 0 | 0 | 0 | 0 (+4 −4) |
| events-asleep | 12 | 91 | 33 | +58 | 137 | 2 | +135 | 0 | 0 | +1 (+4 −3) |
| events-wait (its state commit, 089096c) | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| events-links | 7 | 70 | 12 | +58 | 257 | 7 | +250 | +31 (+35 −4) | +26 (+30 −4) | 0 (+6 −6) |
| events-principles | 18 | 66 | 37 | +29 | 364 | 0 | +364 | +8 (+16 −8) | +88 | 0 (+12 −12) |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 45 | 240 | 95 | **+145** | 758 | 9 | +749 | +39 | +114 | +1 |

Where the src lines went: dial.rs +24 (new, 883fca8, 5c097eb; mostly its
doc), guest.rs +30 −12 and web/src/lib.rs +25 −13 (the cap given to the
lend's dials), door.rs +42 −5 (the two headers, 12d6331), blob.rs +33 −13
(#148, #149), web/app.js +15 −6 and home/home.js +13 −1 (events-links (c)),
ring_routes.rs +12 −7, register.rs +10 −3, cli.rs +5 −7, synced.rs +5 −7
and declared.rs +6 −5 (the economizing cut), the rest a line or two each.
scripts/ is lean_check.py's caps and its comment.

What the budget counts, by scripts/lean_check.py over a `git archive` tree
of each end and of each unit's `done` (lean-<hash>.out; the caps printed
are HEAD's, so the cap column is each commit's own lean_check.py):

| at | svrngs-core | repository | its cap | counted net |
|---|---:|---:|---:|---:|
| 28af323, range start | 3149 | 16236 | 16237 | |
| 9ac4a4c, events-economize done | 3149 | 16234 | 16235 | −2 |
| 9c2d2cf, events-asleep done | 3149 | 16254 | 16255 | +20 |
| dab41b0, events-links done | 3149 | 16287 | 16288 | +33 |
| e0f0bae, events-principles done | 3149 | 16304 | 16305 | +17 |
| b1d04bc, range end | 3149 | 16304 | 16305 | **+68** |
| 0e5818d, this audit's last change | **3149 / 3150** | **16304 / 16305** | | 0 |

Each unit's raises ended where its lines did, the cap one past the count at its `done`, none left over (dbdf8fe,
2aeb961, b47f2dc, d54ad83, aa51c1b, 4250682, 611a0b3, 2d55aa2), and
svrngs-core's cap stayed at 3150. Of the +80 the operator gave over the
economizing pass's 16,235 (fence 7), **70 are spent, 10 left**: 16,315 is
the ceiling events-wait raises within. Still **no line left in the
repository, 1 in svrngs-core** (#80). This audit's changes counted 0: two
renames, a test move, and one line for one in door.rs.

### The instrument

~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 67e46f63…
(`shasum -a 256`), the binary auto-1 ran.

The corpus is b1d04bc's crates/, apps/, web/, scripts/, Cargo.toml and
Cargo.lock by `git archive` into /tmp/eventsaudit2/prod-b1d04bc (301
files), apps/on-air/page/vendor moved out (upstream's). `code index .
--corpus-id prod-b1d04bc` gave 3532 chunks in 538 s, its first run
(index.out). `refresh` gave rust 15742 symbols, typescript and python
"skipped (export failed)" (I4), after "prod-b1d04bc is not registered" and
its in-process rebuild (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node, crates/svrngs-runtime, crates/svrngs,
web and apps/events (`git diff --stat 28af323 b1d04bc -- crates apps web
scripts bridge`), each reported at 0.95 and 0.90, with
crates/svrngs-node/tests, and apps/ whole as auto-1 did
(/tmp/eventsaudit2/clean-*.md). Whether a side was added in the range is
`git blame` at b1d04bc over each side's lines against 28af323.

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 542 | 2 | 0 | 7 | 1: exact, `flush`, blob_tests.rs |
| crates/svrngs-runtime/ | 136 | 1 | 0 | 0 | none |
| crates/svrngs/ | 77 | 0 | 0 | 0 | — |
| web/ | 100 | could-not-judge (I4) | 0 | 2 | none |
| apps/events/ | 60 | could-not-judge (I4) | 0 | 2 | none |
| apps/ | 390 | could-not-judge (I4) | 9 | 22 | none |
| crates/svrngs-node/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

The one cluster with a side the range added:

| tier | its side | its other side | read |
|---|---|---|---|
| exact, 2 × 3 lines | `flush` — blob_tests.rs:161 (4bd7fdf, events-principles (b)) | `flush` — relay_tests.rs:160 | Test code counted as production (I8). The three lines are the tail of a test's trace sink, `Said`, copied whole from folders.rs; with home.rs's copy (0db97c8) the line-window pass below finds all three. Two of them **moved to one in 3844bc2**; the rest #152. |

The rest, none with a side in the range: the node's api.rs `addr` ×2 and
seven near pairs (sign_in.rs, synced.rs, redeem.rs, choices.rs, rail.rs,
ring_sync/tests.rs, lend.rs) are auto-1's, blame putting no line of theirs
in the range, synced.rs's `kept_files`/`kept_at` (:334-361) and choices.rs's
`load`/`read` (:79-99) included though their files changed; runtime's
tests.rs:19 == tests_write_index.rs:12 `open`, before the range; web's two
(app.js `uses`/`choicesOf` :285-307 over :288-303, lend.js
`cookieFor`/`keepJar` :174-181 over :177-185) and store.mjs's two are each
one span inside another (I32); apps/'s 13 with an events side are auto-1's
table again, none in server.mjs, page.test.mjs or partial.test.mjs, the
three the range changed (#150).

The line-window pass (/tmp/eventsaudit2/scripts/windows.py: 8
whitespace-normalized lines, blanks and lone closers dropped, overlapping
hits merged to spans, a span new when blame at b1d04bc puts a line of
either side in the range) over the node's and the command's tests,
svrngs-runtime's tests.rs, web/tests and apps/events and apps/expenses'
.mjs, 84 files at b1d04bc: **165 window hits, 53 spans, 6 new**
(windows.out). Both verdicts are seen: auto-1's #150 spans (events against
expenses) read old, and the range's copies new.

| new span | read |
|---|---|
| blob_tests.rs:153-162 == folders.rs:1050-1059; blob_tests.rs:152-162 == home.rs:742-752; folders.rs:1050-1059 == home.rs:743-752 | The trace sink `Said`, three copies. folders.rs's and home.rs's moved to tests/common/mod.rs in 3844bc2; blob_tests.rs's cannot import it (#152). |
| blob_tests.rs:179-186 == home.rs:768-775 (at b1d04bc) | The subscriber and the second dispatcher with tracing-core's `has_just_one` reason, written twice in the range; folders.rs's two subscribers (:392, :1193) have neither (#152). |
| browser/main.rs:539-550 == :1451-1462 | `linking_app` (12d6331) writes `kept_page`'s accept loop again, to choose its answer by the request's head (#154). |
| web/tests/links.mjs:19-26 == reads.mjs:21-28 | The drivers' preamble (serve, launch, a page, its log, `held`), as ask.mjs:22-32 == reads.mjs:14-24 and handover.mjs == runtime.mjs had before the range. Each driver is a script node runs alone; a shared preamble is page.mjs's to grow, not this audit's. No finding. |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 28af323..b1d04bc -- crates apps
web scripts bridge ':!apps/on-air/page/vendor'` match
`(struct|enum|trait|type|class)\s+[A-Z]\w*` four times (new-types.out):
`Said` twice (home.rs, blob_tests.rs), `Joined` (blob.rs) and `Asking`
(blob_tests.rs). Each through `converge noun <Name> --corpus-id
prod-b1d04bc` (nouns.out), with `git grep -nP` beside it (nouns-grep.out):

| noun | converge (production) | git grep (demo/ and vendor/ apart) | read |
|---|---|---|---|
| Said | **2**: svrngs cli.rs:1040; svrngs-node door/choices.rs:39 | 7: cli.rs:1041, choices.rs:40, blob_tests.rs:154, tests/folders.rs:1051, tests/home.rs:744, web/app.js:83, apps/events/server.mjs:56 | The two production ones as :4459 read them, no change. The range's two are the tests' trace sink, folders.rs's copied (#152); after 3844bc2 the node's tests hold it once in common/mod.rs:89, and blob_tests.rs's stays. |
| Joined | **2**: svrngs-core membership.rs:131; svrngs-node blob.rs:142 | 2: membership.rs:132, blob.rs:143 | The core's is when and how a person joined, read in the node through lib.rs:33 (`pub use svrngs_runtime::*`) at rail.rs:59 and removal.rs:34; blob.rs's a private alias for an ask's task as it ended (0c55dc6). Two things of one name in the node's reach, as 0213ebb found of `Asked`. **Fixed in 389094a**: it is `Ended`. |
| Asking | 1: node.rs:810 | 3: node.rs:811, blob_tests.rs:114, tests/folders.rs:1080 | node.rs's is the people route's state; blob_tests.rs's (0c55dc6) a spawned ask's handle, unseen by converge; folders.rs's a test harness (I18). blob_tests.rs's **renamed `Spawned` in 389094a**, beside `Ended`. folders.rs's is before the range. |

After the renames, blob.rs and blob_tests.rs put into the indexed tree and
`refresh` run (refresh-renamed.out, +0 symbols), converge gives `Joined` 1
(membership.rs), `Ended` 1 (blob.rs:143), `Asking` 1 (node.rs), `Spawned`
0, test code (nouns-after.out). converge's lines are still one early (I1).
**No noun the range added has more than one definition now.**

### Reading the range

Every hunk of product code and of apps/events the range changed was read
(`git diff 28af323 b1d04bc -- crates web apps/events`).

- **One decider** (AGENTS.md 8). Who stands in a group: aff607b gave it one
  home, `Standing::member`, but door.rs `pages` (12d6331, events-links,
  landed before it) wrote the rule a fifth time,
  `matches!(.., Ok(Standing::Members(m)) if m.contains_key(device))`.
  **Fixed in 0e5818d**, red on #146's own plant (a change in `member` the
  door did not follow), green after. #146's note on blob.rs `standing_in`
  and surface.rs stands: they read `standing_of`'s map, not a `Standing`.
  The dial cap is `dial::DIAL_WITHIN` alone, read by the bridges
  (iroh.rs:279) and the page (web/src/lib.rs, by `#path`), as events-wait
  will keep it. **A link's path is judged by two rules** that disagree
  (#153): on a phone open.js:55-56 takes `at` only when it resolves to the
  lend's own origin; on a computer's home page home.js:172 takes any `at`
  that starts with `/` and not `//`, so `/\evil.example/x` passes there and
  opens `http://evil.example/x` (`node -e`, new URL with the lend's origin
  as base: "home.js passes: true -> http://evil.example/x | open.js same
  origin: false").
- **Never silently substitute** (AGENTS.md 6; fence 2). A dial not answered
  within the cap is said as asleep on both sides (guest.rs, "the dial was
  not answered in time"; web/src/lib.rs, "not answered in …: asleep"), apart
  from a refused dial. A get's panicked ask is said, never dropped (blob.rs
  `heard`). The door's `x-svrngs-pages` says `none` and `unread` apart, and
  the app says each in a person's words (server.mjs `linkTo`). A group read
  in part is said per group (6919521, #151).
- **Glassbox** (AGENTS.md 1). Each new branch in the node traces at debug:
  the dial given up, the door's standing per group and the header told,
  the page's link refused (home.rs, invite.rs), every ring-route refusal
  (ring_routes.rs `err`), a get asking the group with `given`. The home
  page's new "Open it" refuses in words beside the box and traces nothing,
  as home.js traces nothing anywhere; #153 says so.
- **Closed sets** (AGENTS.md 9). The CLI's valued flags are a match arm the
  compiler holds (13dbdbe, #147), no longer a space-joined string.
- **Machines build, people speak** (fence 4). The app writes a link for the
  host to copy and send in the group's own chat; nothing is sent in anyone's
  name (server.mjs `linkTo`, a button that copies).
- **The nouns gate** (fence 3). `python3 scripts/vocab_check.py`: run in
  scripts/check.sh at the close.
- The runs' drivers (demo/runs/asleep) are kept as they ran (#61) and were
  not reviewed as product.

### Fixed here

- 389094a: blob.rs's alias `Joined` is `Ended`, blob_tests.rs's `Asking`
  `Spawned`; converge gives each one.
- 3844bc2: the node's tests' trace sink `Said` once, in tests/common/mod.rs,
  for folders.rs and home.rs; the three tests that read one pass.
- 0e5818d: the door's `pages` asks `Standing::names`; red on #146's plant,
  green without it, both engines.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 152 | 8 | crates/svrngs-node/src/blob_tests.rs:152-164 (`Said`) and :175-186 (the subscriber and `_two`, tracing-core's `has_just_one` reason) == crates/svrngs-node/tests/common/mod.rs:81-99 (`Said`, since 3844bc2) and tests/home.rs:750-761 (the subscriber and `_two`); beside crates/svrngs-node/src/relay_tests.rs:150-163, crates/svrngs/src/relay_tests.rs:12, crates/svrngs/tests/default_relay.rs:33 and crates/svrngs/tests/relay.rs:105 (`Kept`, a static sink each) | not fixed: a test-support home across a crate's library and its integration tests, not small | A test that reads its own trace builds its sink and subscriber by hand, six times over two crates, and the range added two. The reason a second dispatcher is kept alive is written twice (blob_tests.rs:183-186, home.rs:758-761), and folders.rs's two subscribers of the same shape (:387-394, :1188-1195) have neither the dispatcher nor the reason; whether they need it is not judged here. One home is a small test-support crate or a `#[cfg(test)]` module both the library's tests and tests/common reach (`traced(filter) -> (Said, guards)`); tests, so no counted line. |
| 153 | 8, 6 | crates/svrngs-node/src/home/home.js:170-173 (`at.startsWith('/') && !at.startsWith('//')`, then `new URL(at, <lend's origin>)`); beside web/open.js:55-56 (`new URL(back, location.href)`, taken only at its own origin) | not fixed: a change of behaviour, so red first and the operator's budget (1 line left) | What makes `at` a path within the lend is decided twice, and the two disagree: `/\evil.example/x` passes the home page's rule and opens `http://evil.example/x` in a new tab, where the phone's open.js refuses it to `/`. A pasted link from a group's chat can so send a person to any site from their own svrngs home page. The fix is open.js's rule on the home page — resolve, and refuse unless the URL's origin is the lend's — about +1 counted line with the refusal traced; home.js traces none of its own branches, the refusal said only in words beside the box. Red on a link whose `at` is `/\` and another host, through links.mjs's home-page step. |
| 154 | 8 | crates/svrngs-node/tests/browser/main.rs:538-564 (`kept_page`) and :1450-1492 (`linking_app`, 12d6331) | not fixed: test code, a change each browser test re-proves | `linking_app` writes `kept_page`'s listener, accept loop, head read and answer again, to choose its page by the request's head and keep only `GET /`'s. One server taking `fn(&str) -> Option<String>` (the page for a head, kept when `Some`) serves both and `page_origin`. |
| 80 | — | scripts/lean_check.py:33 BUDGET, :100 `n >= cap` | not fixed (operator's) | Unchanged: 1 line left in svrngs-core (3149 / 3150), **none** in the repository (16304 / 16305), 10 of the follow-up's +80 left (16,315). |
| 130, 132-145, 150 | as recorded | as recorded | not fixed | Not touched by this range. #146-#149 and #151 are events-principles', marked fixed with their commits; #140 likewise. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (membership.rs:131 for :132, blob.rs:142 for :143, node.rs:810 for :811, cli.rs:1040 for :1041). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)"; web/'s, apps/events' and apps/'s exact tier could-not-judge, and converge blind to JS classes. |
| I8 | 6 | dry-report scope | Still present: blob_tests.rs's `flush` reported as a production exact clone of relay_tests.rs's. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, said as could-not-judge; the line-window pass stood in. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: "prod-b1d04bc is not registered", then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's JS near tier | Still present, and seen in web/ now: app.js `uses` :285-307 over `choicesOf` :288-303, lend.js `cookieFor` :174-181 over `keepJar` :177-185. |

## REVIEW-audit-events-auto-3 (2026-10-14, range a6476cc..8e97b27)

The previous `REVIEW-audit-` row of this queue is REVIEW-audit-events-auto-2,
its hash a6476cc (ralph/next/events/STATE.md), so the range starts there
and ends at 8e97b27, the commit that made this row. `git rev-list --count
a6476cc..8e97b27` is 21, none a merge (`--merges`: 0): auto-2's `done`,
events-wait's eleven, events-doc-link's eight, and this row's own commit.

This audit's own commit before this section: 42b7c92 (the row's two paths,
as ba1124d corrected auto-2's). It opens #155-#156, and closes none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
a6476cc..8e97b27` (/tmp/eventsaudit3/ledger.raw), summed by
/tmp/eventsaudit3/scripts/ledger.py (auto-2's, its unit list this range's;
ledger.out, each commit's unit in ledger.commits) to the earlier audits'
rules: product is crates/, web/ and bridge/; apps/ apart, split the same
way; scripts/ apart; a test as lean_check names one, and `*.test.mjs`; a
commit that names no unit counts toward the next one that does (6d225a3
and a924a0e, docs, to events-doc-link). Raw lines, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-events-auto-2 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| events-wait | 11 | 53 | 22 | **+31** | 6 | 6 | 0 | 0 | 0 | 0 (+3 −3) |
| events-doc-link | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 21 | 53 | 22 | **+31** | 6 | 6 | 0 | 0 | 0 | 0 |

Where the src lines went: dial.rs +27 −8 (the cap's doc rewritten for 4 s,
cb71f8e; `held`, a07a9cc), web/src/lib.rs +23 −13 (`held` threaded through
`bind_for` and `bind`, and a held dial's silence said `asleep`, a07a9cc),
web/lend.js +3 −1 (`asleep` not dialled again, a07a9cc). The test lines are
lend_origin.rs's slow path, 175 → 750 ms, and its plant, 300 ms → 1 s
(cb71f8e); scripts/ is lean_check.py's cap and its comment (98e6d87).
events-doc-link changed demo/ and docs only.

What the budget counts, by scripts/lean_check.py over a `git archive` tree
of each end and of each unit's `done` (/tmp/eventsaudit3/lean-<hash>.out;
each tree's own lean_check.py, so the cap is its own):

| at | svrngs-core | repository | its cap | counted net |
|---|---:|---:|---:|---:|
| a6476cc, range start | 3149 | 16304 | 16305 | |
| cf6a0cf, events-wait done | 3149 | 16314 | 16315 | +10 |
| e251035, events-doc-link done | 3149 | 16314 | 16315 | 0 |
| 8e97b27, range end | **3149 / 3150** | **16314 / 16315** | | **+10** |

events-wait raised once, 16,305 → 16,315 (98e6d87), and its lines ended at
16,314, the cap one past them; svrngs-core's cap stayed at 3150. Of the +80
the operator gave over the economizing pass's 16,235 (fence 7), **all 80 are
now raised**: the ceiling, 16,315, is the cap. **No line left in the
repository or in svrngs-core** (#80): each cap is one past its count, and
lean_check.py:100 refuses `n >= cap`, so one more line breaks either. This audit changed no counted line.

### The instrument

~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 67e46f63…
(`shasum -a 256`), the binary auto-1 and auto-2 ran.

The corpus is 8e97b27's crates/, apps/, web/, scripts/, Cargo.toml and
Cargo.lock by `git archive` into /tmp/eventsaudit3/prod-8e97b27 (301
files; `diff -rq` against auto-2's prod-b1d04bc: dial.rs, door.rs, the
node's tests' common/mod.rs, folders.rs, home.rs and lend_origin.rs,
lean_check.py, web/lend.js and web/src/lib.rs, auto-2's own fixes and this
range's), apps/on-air/page/vendor moved out (upstream's). `code index .
--corpus-id prod-8e97b27` gave 3529 chunks in 790 s under a load of 48-60
(index.out). `refresh` gave rust 15734 symbols, typescript and python
"skipped (export failed)" (I4), after "prod-8e97b27 is not registered" and
its in-process rebuild (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node (src/dial.rs, tests/lend_origin.rs)
and web (src/lib.rs, lend.js) (`git diff --stat a6476cc 8e97b27 -- crates
apps web scripts bridge`), each reported at 0.95 and 0.90, with
crates/svrngs-node/tests (/tmp/eventsaudit3/clean-*.md). Whether a side was
added in the range is `git blame` at 8e97b27 over each side's lines against
a6476cc.

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 543 | 2 | 0 | 7 | none |
| web/ | 100 | could-not-judge (I4) | 0 | 2 | none |
| crates/svrngs-node/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

**No clone has a side the range added.** The node's are auto-2's table
again: blob_tests.rs `flush` == relay_tests.rs (exact, test code, I8,
#152), api.rs `addr` ×2, and the seven near pairs in sign_in.rs, synced.rs,
redeem.rs, choices.rs, rail.rs, ring_sync/tests.rs and lend.rs, none of
whose files the range changed (`git log a6476cc..8e97b27 --` them: 0
commits). web's two are each one span inside another (I32): app.js
`uses`/`choicesOf` :285-307 over :288-303, unchanged in the range, and
lend.js `cookieFor`/`keepJar` :174-181 over :177-185, which the range's
lend.js change (:343-345) does not reach (blame over :174-185: e502de9 alone,
before a6476cc).

The line-window pass (/tmp/eventsaudit3/scripts/windows.py, auto-2's) over
auto-2's 84 files with the node's and the page's src (crates/svrngs-node/src,
web/src, web/lend.js), 118 files at 8e97b27: **159 window hits, 50 spans,
0 new** (windows.out). auto-2's six new spans read old now, three fewer
after its 3844bc2.

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges a6476cc..8e97b27 -- crates apps
web scripts bridge ':!apps/on-air/page/vendor'` match
`(struct|enum|trait|type|class)\s+[A-Z]\w*` **zero times**
(new-types.out, empty), and demo/'s drivers add none either: the range
added no `struct`, `enum` or `trait`, so no noun to converge. converge
answers on this corpus as before (`converge noun Said --corpus-id
prod-8e97b27`: cli.rs:1040 and choices.rs:39, one line early, I1;
nouns.out).

### Reading the range

Every hunk of product code the range changed was read (`git diff a6476cc
8e97b27 -- crates web scripts`); events-doc-link's driver and data are
kept as they ran (#61) and were not reviewed as product.

- **One decider** (AGENTS.md 8). The dial cap is `dial::DIAL_WITHIN` alone,
  and the page's idle timeout and keep-alive are derived from it in
  `dial::held` (dial.rs:39-43), which the page reaches by
  web/src/lib.rs:36's `#[path]`, as events-wait asked. svrngs-node compiles
  `held` and calls it nowhere (`git grep -n "held()" -- crates web`: dial.rs
  and lib.rs:133 only); its doc says it is the page's alone, so no finding.
  **What "asleep" means crosses from the page's wasm to its worker as a word
  in an error's text** (#155): lib.rs:203 ends its message `: asleep`,
  lib.rs:249 begins its `asleep: `, and lend.js:345 decides not to dial
  again by `/asleep/.test(e)`.
- **Closed sets** (AGENTS.md 9). `bind_for` and `bind` take `held: bool`
  (lib.rs:83, :111), passed as a bare `false`/`true` at :308, :325 and :341
  (#156).
- **Never silently substitute** (AGENTS.md 6; fence 2). A held dial given
  up while asked is said asleep (lib.rs:247-249) and the worker answers 502
  `away` without a second dial; one given up before it was asked is dialled
  again, and both are traced (`HTTP: … REFUSED — …`, then lend.js's
  `RUNTIME: … failed (try n)`). Nothing is shown as no answer.
- **Glassbox** (AGENTS.md 1). The page traces whether its endpoint is held
  (lib.rs:136) and each refusal with its reason; lend.js's three reasons to
  stop (a body, a second try, asleep) share one `say` after one trace that
  carries the error, so which of them decided is read from that line, not
  said — part of #155.
- **Validate the instrument** (AGENTS.md 7). lend_origin.rs's cap test moved
  its slow path past the new slowest measured dial, 1,013 ms, to 1.5 s, and
  its plant to 1 s (cb71f8e), so it still separates the two; the wait runs'
  verdicts are their README's, not re-judged here.
- **The nouns gate** (fence 3). `python3 scripts/vocab_check.py`: run in
  scripts/check.sh at the close.

### Fixed here

Nothing: the range's two findings are in the page's wasm and worker, a
change each the page's tests in both engines re-prove, and #155's fix is
a change of behaviour the budget has no line left to pay for (#80).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 155 | 8, 9, 1 | web/src/lib.rs:203 (`refuse!(".. not answered in {within:?}: asleep")`) and :249 (`format!("asleep: {e}")`); web/lend.js:345 (`/asleep/.test(e)`) | not fixed: a change across the wasm boundary, red first in both engines, and no line left (#80) | Whether a dial's computer is asleep is decided in Rust and read in JS by matching a word in the error's prose, written two ways (a suffix at :203, a prefix at :249). A reworded message silently makes the worker dial again and wait the cap a second time; an error from iroh or a relay whose text holds "asleep" stops a dial that should be tried again. One decider is the verdict carried as data — a `JsError` whose `name` is `Asleep`, set in one helper both sites call, and lend.js reading `e.name` — with lend.js's `say` traced per reason (a body, a second try, asleep). Red on a refusal whose text says "asleep" but is not one. |
| 156 | 9 | web/src/lib.rs:83 and :111 (`held: bool`), :133 (`if held { dial::held() } else { Default::default() }`); callers :308 (`false`), :325 and :341 (`true`) | not fixed: small, but each form costs a counted line or changes the trace at :136 | A two-valued choice passed as a bare bool: `bind_for(key, &token, false)` does not say what `false` is. Passing the transport config itself (`dial::held()` or iroh's default) removes the bool and the branch, at the cost of the trace's `held: {held}`; a two-variant enum keeps the trace and costs its declaration. |
| 80 | — | scripts/lean_check.py:33 BUDGET, :100 `n >= cap` | not fixed (operator's) | No line left in svrngs-core (3149 / 3150) or the repository (16314 / 16315), :100 refusing `n >= cap`; the follow-up's +80 all raised (16,315). |
| 130, 132-145, 150, 152-154 | as recorded | as recorded | not fixed | Not touched by this range: blob_tests.rs, the node's tests, home.js and browser/main.rs are unchanged in it (`git log a6476cc..8e97b27 --` them: 0 commits). |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early (cli.rs:1040 for :1041, choices.rs:39 for :40). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)"; web/'s exact tier could-not-judge. |
| I8 | 6 | dry-report scope | Still present: blob_tests.rs's `flush` reported as a production exact clone of relay_tests.rs's. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, said as could-not-judge; the line-window pass stood in. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: "prod-8e97b27 is not registered", then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's JS near tier | Still present: app.js `uses` :285-307 over `choicesOf` :288-303, lend.js `cookieFor` :174-181 over `keepJar` :177-185. |

## REVIEW-audit-v1ui-auto-1 (2026-10-14, range 8d8beb2..8d62b0a)

This queue has no earlier `REVIEW-audit-` row. The previous one anywhere is
REVIEW-audit-events-auto-3, its hash 8d8beb2 (ralph/next/events/STATE.md:24),
so the range starts there rather than at this queue's first commit, 4a2e8a7,
and so the look merged between them (efadec2) is audited too. It ends at
8d62b0a, the commit that made this row. `git rev-list --count
8d8beb2..8d62b0a` is 43, one a merge (`--merges`: efadec2, the look). The
other 42 are: the look's two, events-auto-3's `done`, v1ui-economize,
v1ui-reach, v1ui-door and v1ui-lend, each with its `done`, v1ui-lend-ask's
state commit and its part (a), and this row's own commit.

This audit's own commits before this section:
- dce6747, the row's two paths, as 42b7c92 corrected events-auto-3's;
- 89224c8, the page's lend body named `ToLend`, apart from node.rs's
  `Lending`;
- 9d2d904, the door's four cells said once in the browser tests,
  `drew_the_door`.

It opens #157-#162 and closes none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
8d8beb2..8d62b0a` (/tmp/v1uiaudit1/ledger.raw) is summed by
/tmp/v1uiaudit1/scripts/ledger.py (ledger.out; each commit's unit is in
ledger.commits). The rules are the earlier audits':
- a commit's unit is the longest unit id its subject names;
- product is crates/, web/ and bridge/, in lean_check.py's code suffixes;
- .css and .html are counted apart as markup, which lean_check.py does not
  count (:39 `CODE`);
- apps/ and scripts/ are counted apart;
- a test is what lean_check names one, and `*.test.mjs`;
- a commit that names no unit counts toward the next one that does:
  4a2e8a7 to v1ui-economize, 1152254 to v1ui-reach.

e2f6b30 names both v1ui-lend-ask (a) and v1ui-lend, and it counts as
v1ui-lend-ask's: that is the row whose part it is. It was made to close
v1ui-lend, and v1ui-lend-ask's row is still `[ ]`
(ralph/next/v1ui/STATE.md:18). The lines are raw, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| look (merged efadec2) | 2 | 5 | 4 | **+1** | 0 | 0 | 0 | +62 (+86 −24) | 0 | 0 | 0 (+6 −6) |
| REVIEW-audit-events-auto-3 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| v1ui-economize | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | +1 (+3 −2) |
| v1ui-reach | 12 | 95 | 35 | **+60** | 210 | 15 | +195 | 0 | 0 | 0 | +3 (+10 −7) |
| v1ui-door | 11 | 317 | 116 | **+201** | 430 | 65 | +365 | +31 (+36 −5) | 0 | 0 | +4 (+17 −13) |
| v1ui-lend | 8 | 400 | 225 | **+175** | 180 | 3 | +177 | −1 (+2 −3) | 0 | 0 | 0 (+6 −6) |
| v1ui-lend-ask (its state commit and (a)) | 2 | 25 | 21 | **+4** | 29 | 18 | +11 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 42 | 842 | 401 | **+441** | 849 | 101 | +748 | +92 | 0 | 0 | +8 |

Where the src lines went (`git diff --stat 8d8beb2 8d62b0a -- crates web`):
- share.rs moved from the command into the node: +244 in the node, −223 in
  the command (586e32d, v1ui-lend (a));
- home.rs +139 −8: `/running` and `/group?name=` (719150d, 6acfdd6);
- home.js +125 −40 and question.js +49: the door, the index and "Already
  running here";
- web/app.js +85 −24: reaching whoever's awake, and the door on the phone;
- group.rs +73 −8: the one rule for a name, `Namesake` (#138);
- cli.rs +33 −2 and rail.rs +28 −8: `device` in the people answer;
- web/src/lib.rs +28 −8 and dial.rs +15: `LINK_WITHIN`;
- surface.rs +20 −8: each group's `made`.

The test lines are mostly tests/browser/main.rs, +572. scripts/ is
lean_check.py's caps, vocab_check.py and build-web.sh's look.css.

What the budget counts, by scripts/lean_check.py over a `git archive` tree
of each end and of each unit's `done` (/tmp/v1uiaudit1/lean-<hash>.out;
each tree's own lean_check.py, so the cap is its own):

| at | svrngs-core | repository | its cap | counted net |
|---|---:|---:|---:|---:|
| 8d8beb2, range start | 3149 | 16314 | 16315 | |
| efadec2, the look merged | 3149 | 16314 | 16315 | 0 |
| aae56f8, v1ui-economize done | 3149 | 16314 | 16315 | 0 |
| 860d247, v1ui-reach done | 3149 | 16338 | 16339 | +24 |
| 57872c6, v1ui-door done | 3149 | 16446 | 16447 | +108 |
| 55dc4d6, v1ui-lend done (with lend-ask (a), e2f6b30: 16567, 0 of its own) | 3149 | 16567 | 16568 | +121 |
| 8d62b0a, range end | **3149 / 3150** | **16567 / 16568** | | **+253** |

There were eight raises, each one past its part's count and each naming the
pass (`git show <h> -- scripts/lean_check.py`):

| raise | from | to | for |
|---|---:|---:|---:|
| fe5feb2 | 16,315 | 16,316 | +1 |
| b409b49 | 16,316 | 16,338 | +22 |
| 371c7c5 | 16,338 | 16,339 | +1 |
| e3b8d2d | 16,339 | 16,400 | +61 |
| 5ad6d96 | 16,400 | 16,435 | +35 |
| 738bbb3 | 16,435 | 16,447 | +12 |
| 6d536c4 | 16,447 | 16,451 | +4 |
| 8d62784 | 16,451 | 16,568 | +117 |

That is +253 in all. The pass cut nothing (f3abffa), so the ceiling is
16,315 + 650 = 16,965 (fence 7). **253 are raised and 397 are left** for
v1ui-shelf and v1ui-voice. svrngs-core's cap stayed at 3150, with 1 line
left. The repository has **no line left under its cap** (#80):
lean_check.py:100 refuses `n >= cap`. This audit's changes counted 0: a
rename in home.rs, and a move within test code.

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
67e46f63… (`shasum -a 256`), the one events-auto-3 ran.

The corpus is 8d62b0a's crates/, web/, scripts/, Cargo.toml and Cargo.lock,
by `git archive` into /tmp/v1uiaudit1/prod-8d62b0a (214 files). apps/ is
left out: the range changed nothing there (`git diff --stat 8d8beb2 8d62b0a
-- apps`: empty), and a scoped dry-report compares within its scope alone
(I12).
- `code index . --corpus-id prod-8d62b0a`: 3031 chunks in 528 s, under a
  load of 35-52 (index.out).
- `refresh`: rust, 16083 symbols and 93699 references; typescript and python
  "skipped (export failed)" (I4). It first said "prod-8d62b0a is not
  registered", then rebuilt in-process (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched three crate dirs: crates/svrngs-node, crates/svrngs and
web (`git diff --stat 8d8beb2 8d62b0a -- crates apps web scripts bridge`).
Each was reported at 0.95 and 0.90, with the two crates' tests/
(/tmp/v1uiaudit1/clean-*.md). Whether a side was added in the range is `git
blame` at 8d62b0a over each side's lines against 8d8beb2
(scripts/blame.sh; blame-clusters.out).

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 566 | 2 | 0 | 9 | 2, both one span inside another (I32) |
| crates/svrngs/ | 70 | 0 | 0 | 0 | — |
| web/ | 103 | could-not-judge (I4) | 0 | 3 | 1, two thin wrappers over one `dial` |
| crates/svrngs-node/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |
| crates/svrngs/tests/ | 0 | could-not-judge (I19) | could-not-judge | could-not-judge | — |

The clusters with a side the range added:

| tier | its side | its other side | read |
|---|---|---|---|
| near 0.950 | `running` — home/home.js:139-163 (e2f6b30, 719150d) | `look` — home.js:142-162 | `look` is the function nested in `running`, so the tool scored a span against the span it holds (I32). Not a clone. |
| near 0.944 | `door` — home/question.js:88-102 (8bf344c) | `judge` — question.js:90-100 | `judge` is the closure `door` returns, nested in it (I32). Not a clone. |
| near 0.906 | `connect` — web/src/lib.rs:293-301 (b1c330b, 6b40abb) | `connect_to_join` — lib.rs:301-309 | Two exported wrappers of one line each over `dial` (lib.rs:311), differing in protocol and cap. The spans overlap at :301 and take in each one's doc. The shared body is already one function. No finding. |

The other clusters, none with a side in the range, are events-auto-3's
again: blob_tests.rs `flush` == relay_tests.rs (test code, I8, #152);
api.rs `addr` ×2; the seven near pairs in sign_in.rs, synced.rs, redeem.rs,
choices.rs, rail.rs, ring_sync/tests.rs and lend.rs; and web's app.js
`uses`/`choicesOf` (:289-311, 2 lines of 6b40abb inside, the pair as before,
I32) and lend.js `cookieFor`/`keepJar` (blame: e502de9 alone).

**The line-window pass.** It is /tmp/v1uiaudit1/scripts/windows.py:
- 8 whitespace-normalized lines, with blanks, lone closers and `//` lines
  dropped;
- overlapping hits merged to spans;
- a span counts as new when blame at 8d62b0a puts a line of either side in
  the range (scripts/newspans.sh).

It ran over the node's and the command's src and tests, web/src, web/tests
and web/*.js: 126 files at 8d62b0a. The result: **136 window hits, 46
spans, 1 new** (windows.out). Both verdicts were seen: 45 spans read old.

| new span | read |
|---|---|
| tests/browser/main.rs:2619-2628 == :2761-2770 (9c2817d, 7835c25) | The door's four cells read, sorted and asserted twice, once for the home page and once for the group's page, which draw it in one wording (question.js `door`). **Moved to one, `drew_the_door` (main.rs:2600), in 9d2d904.** Both tests were green in both engines, and a planted wrong cell failed all four. |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 8d8beb2..8d62b0a -- crates apps
web scripts bridge` were matched against `(struct|enum|trait|type|class)\s+[A-Z]\w*`
(new-types.out). They give:
- `Lending`, `Namesake` and `Naming`, new;
- `Signs`, `Share` and `ShareRefusal`, moved from the command to the node in
  586e32d: their removed lines are in removed-types.out;
- the alias `Made`, given a third field.

`converge noun` was run for each (nouns.out). One noun has more than one
definition:

| noun | definitions | read |
|---|---|---|
| `Lending` | crates/svrngs-node/src/node.rs:798 (`pub struct Lending`, a lend as a member is told it: person, key, relay, the offer; before the range); home.rs:281 (private, the `POST /running` body: port, name, kind; 719150d). Both are a line early (I1): :799 and :282. | Two things under one name in one crate (AGENTS.md 8). **The page's one was renamed `ToLend` in 89224c8.** serde reads fields, so the body taken is the same. the_home_page_lends_what_runs_here was green in both engines. |

`Signs`, `Share`, `ShareRefusal`, `Namesake`, `Naming` and `Made` each have
one definition. The command keeps no copy of the three it moved, so the
move left no fork.

### Reading the range

Every hunk of product src the range changed was read (`git diff 8d8beb2
8d62b0a -- crates web`). The look's CSS and the browser tests were read only
for clones.

- **One decider** (AGENTS.md 8).
  - **A group sharing a name is written twice, in two languages** (#157):
    - the command, cli.rs:1103 and `namesakes_said` :1321;
    - the home page, home.js:46-47 over question.js `whose` :70 and `day`
      :66.

    Each cites the other as its source, and they already differ. The command
    prints "Sam's" with a straight apostrophe and the date by chrono's "%-d
    %b". The page prints "Sam’s" with a curly one and the date by the
    browser's locale.
  - **The index's "Now" and how a lend opens are written once per page**
    (#158): home.js:102 and :104, and app.js:382, each with `l.limit &&
    l.using >= l.limit ? 'full' : 'on'` and `signs_in ? ', signing you in
    once'`. 8bf344c moved the door's pieces into question.js so both pages
    draw it in one wording; the index rows stayed apart.
  - The rest is fine. The name rule is `group::called` alone (group.rs:306),
    which `named` and `namesakes` both ask, and the command asks
    `group::namesakes` as the page does (cli.rs:1449). The probe and the
    table are in the node only (share.rs), and the command asks them there.
    `LINK_WITHIN` has one home (dial.rs:44), which the page reaches.
- **Never silently substitute** (AGENTS.md 6; fence 2).
  - **`Offered::signs_in` reads an absence as no** (#159). lend.rs:271-274
    has `#[serde(default)]`, "Absent from a computer before, read as no", and
    door.rs:518 writes it. Two pages then say how a lend opens without its
    sign-in, though the lender's computer never said so. Twelve lines above,
    lend.rs:258-259 holds `upload` to the rule this breaks: "an answer that
    leaves it out is not one, never read as none".
  - **The phone draws no door, and says nothing, when its answer names
    nobody** (#162). This is web/app.js:375, `people ? door(people) :
    [el('p'), () => {}]`, for a computer before v1ui-door (d). The page
    neither lights nor draws anyone off, which is better than drawing an
    empty door. But who is on now is not known, and the page does not say
    so.
  - A computer that did not answer is drawn off by its person
    (question.js:95), and the phone says whose computer it reached the group
    through (app.js:370). Both are as fence 2 asks.
- **Glassbox** (AGENTS.md 1).
  - **`GET /running` refuses without saying which read failed, or why**
    (#160). home.rs:257-258 drops both errors, `lend::all`'s and
    `spawn_blocking`'s, behind `let (Ok(all), Ok(found)) = … else`, and
    warns "what this computer lends, or what runs on it, cannot be read".
  - `Namesake::of`, `namesakes`, `place`'s `taken` (group.rs) and
    `lend_one`'s two decisions are each traced.
- **Closed sets** (AGENTS.md 9). **`POST /running` says one wording for
  every failure of a closed set** (#161).
  - home.rs:297's `let Ok(Json(ToLend { … })) = asked else` tells every
    `JsonRejection` that "Its name and what it is are each one word": a body
    that is not JSON, a missing port, and a port of 0, which `NonZeroU16`
    refuses. None of these is traced.
  - home.rs:321's `let Ok(lend) = share::lend_of(…) else` says every
    `ShareRefusal` as `NothingListens`, by the invariant in the comment at
    :320, and drops the refusal itself. `lend_of` traces each one
    (share.rs), so the trace is right where the person's words might not be.
- **Validate the instrument** (AGENTS.md 7). v1ui-lend's two failed checks
  were kept as they ran (7007bb1, 0255842), and the next ran after the sweep
  was moved behind a button (51ef36e). Their verdicts are those commits',
  and are not judged again here. v1ui-lend-ask (b), its five runs, has not
  run yet.
- **The nouns gate** (fence 3). `python3 scripts/vocab_check.py` runs in
  scripts/check.sh at the close.

### Fixed here

- 89224c8: `Lending` (home.rs) renamed `ToLend`.
- 9d2d904: the doubled door assertion, now `drew_the_door`.

Both keep behaviour and cost no counted line. The rest are not fixed here.
Each of #157-#159 and #161-#162 changes what a person reads or what a
refusal says, red first, and #158 and #160 each cost at least one counted
line, and none is left (#80).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 157 | 8 | crates/svrngs/src/cli.rs:1103 and `namesakes_said` :1321; crates/svrngs-node/src/home/home.js:46-47 (`namesakes`) over question.js:66 (`day`) and :70 (`whose`) | **fixed in 44d5536** (v1ui-findings (e); the move bb318fd): one wording, pinned by crates/svrngs/tests/home_group.rs | One sentence ("You're already in Trip (Sam's, since 3 Oct). This is a second group called Trip.") has two deciders, one in Rust and one in JS, each citing the other. They already disagree: the apostrophe (' and ’), and the date (chrono "%-d %b" and the browser's locale). One home would be the node answering the sentence, or its parts, with `/group` and `namesakes`, and both printing that. Red on the two outputs compared for one `Namesake`. |
| 158 | 8 | crates/svrngs-node/src/home/home.js:102 and :104; web/app.js:382 | **fixed in 0668aa2** (v1ui-findings (c)): question.js `opens` | The index row's judgement of "full" or "on" (`l.limit && l.using >= l.limit`) and its "signing you in once" are written once per page. A change to when a lend is full lands on one page and not the other. One export from question.js, beside `door` and `row`, which both pages import already. |
| 159 | 6 | crates/svrngs-node/src/lend.rs:271-274 (`#[serde(default)] pub signs_in: bool`, "read as no"); written at door.rs:518; read at home.js:102 and app.js:382 | **fixed in 3aefde0** (v1ui-findings (a)): `Option<bool>`, said as an older svrngs | An absence is defaulted. A lender's computer from before v1ui-door that does not say whether its program signs people in is read as saying it does not, and both pages say how it opens without the sign-in. lend.rs:258-259 holds `upload` to the opposite rule. `Option<bool>`, with the pages saying "may ask you to sign in" for `null`, would report it. |
| 160 | 1, 6 | crates/svrngs-node/src/home.rs:257-258 | **fixed in 2d0c3f2** (v1ui-findings (d)) | `GET /running` refuses with a warn that says "what this computer lends, or what runs on it, cannot be read". It does not say which of the two failed, or the error: `lend::all`'s `io::Error` and the sweep's `JoinError` are both dropped by the let-else. |
| 161 | 9, 6 | crates/svrngs-node/src/home.rs:297 (`let Ok(Json(ToLend { … })) = asked else`) and :320-321 (`let Ok(lend) = share::lend_of(…) else`) | **fixed in 686a4c2** (v1ui-findings (b)) | Two closed sets, each said in one wording. Every `JsonRejection` (a body that is not JSON, no port, a port of 0) is told that a name and kind must each be one word, and none is traced. Every `ShareRefusal` is said as nothing answering, by an invariant held in a comment, and its own words are dropped. A `match` on each, its refusals in a person's words, would make the invariant structural (AGENTS.md 10). Red on a port of 0. |
| 162 | 6 | web/app.js:375 (`people ? door(people) : [el('p'), () => {}]`) | **fixed in 3aefde0** (v1ui-findings (a)) | A phone answered by a computer from before v1ui-door (d) gets no `people`. It draws no door and says nothing of who is on now, so the absence is not reported. One line in the door's place would say that this computer cannot say who is on now. |
| 80 | — | scripts/lean_check.py:33 BUDGET, :100 `n >= cap` | not fixed (operator's) | No line is left under the repository's cap (16567 / 16568), and 1 is left in svrngs-core (3149 / 3150). Of the +650, 253 are raised and 397 left: the ceiling is 16,965 (fence 7). |
| 130, 132-145, 150, 152-156 | as recorded | as recorded | not fixed | Not reached by this range's findings. #155-#156 (web/src/lib.rs, lend.js) stand: the range's lib.rs change is `LINK_WITHIN` and `issuer_of` (:290-364), not :83-:249. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early: node.rs:798 for :799, home.rs:281 for :282. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)", so web/'s exact tier could not judge. |
| I8 | 6 | dry-report scope | Still present: blob_tests.rs's `flush` reported as a production exact clone of relay_tests.rs's. |
| I19 | 6 | dry-report, `tests/` scopes | Still present, said as could-not-judge. The line-window pass stood in, and found the range's one new span there. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: "prod-8d62b0a is not registered", then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's near tier, JS and nested spans | Still present, and two more: home.js `running` over its nested `look`, and question.js `door` over its returned `judge`. |

## REVIEW-audit-v1ui-auto-2 (2026-10-15, range 7dbfc37..ed8b2c6)

The previous `REVIEW-audit-` row is REVIEW-audit-v1ui-auto-1, its hash
7dbfc37 (ralph/next/v1ui/STATE.md:17). The range ends at ed8b2c6, the
commit that made this row. `git rev-list --count 7dbfc37..ed8b2c6` is 43,
none a merge (`--merges`: 0). They are: auto-1's `done`; v1ui-lend-ask's
state commit, its (b) and its close; v1ui-shelf; v1ui-findings; v1ui-voice,
its failed check and its close; and v1ui-race's state commit 3bdbf29, made
by the operator between v1ui-voice's two checks. Each unit's `done` is in
the range, and so is this row's own commit.

This audit's own commits before this section:
- 835fc08, the row's two paths, as dce6747 corrected auto-1's;
- 6795efa, carried.rs's second `People` alias dropped for blob.rs's;
- 6592309, the test harness carried.rs copied from folders.rs made one
  (tests/common/computers.rs), and home.rs's trace setup made one
  (`heard`);
- 31b6a28, the cap lowered to where 6795efa's cut landed, 16,870;
- 47f80cb, carried.rs's other three aliases renamed apart from nouns owned
  elsewhere.

It opens #163-#168 and closes none.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
7dbfc37..ed8b2c6` (/tmp/v1uiaudit2/ledger.raw) is summed by
/tmp/v1uiaudit2/scripts/ledger.py (ledger.out; each commit's unit is in
ledger.commits). The rules are auto-1's:
- a commit's unit is the longest unit id its subject names;
- product is crates/, web/ and bridge/, in lean_check.py's code suffixes;
- .css and .html are counted apart as markup;
- apps/ and scripts/ are counted apart;
- a test is what lean_check names one, and `*.test.mjs`.

Every commit in the range names its unit. The lines are raw, comments and
blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-v1ui-auto-1 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| v1ui-lend-ask (its state commit, (b), its close) | 5 | 0 | 0 | **0** | 11 | 3 | +8 | 0 | 0 | 0 | 0 |
| v1ui-shelf | 11 | 542 | 112 | **+430** | 953 | 29 | +924 | +1 (+2 −1) | +141 | 0 | +0 (+9 −9) |
| v1ui-findings | 14 | 106 | 65 | **+41** | 326 | 14 | +312 | 0 | 0 | 0 | +0 (+15 −15) |
| v1ui-voice | 10 | 65 | 60 | **+5** | 27 | 23 | +4 | +0 (+4 −4) | +0 (+7 −7) | 0 | 0 |
| v1ui-race (its state commit, 3bdbf29) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 43 | 713 | 237 | **+476** | 1317 | 69 | +1248 | +1 (+6 −5) | +141 (+148 −7) | 0 | +0 (+24 −24) |

Where the src lines went (`git diff --numstat 7dbfc37 ed8b2c6 -- crates
web bridge`, tests apart):
- carried.rs +281, new: the carried path (c98635a's port, 87d68ab, 67ffbba,
  21bcbef);
- home.rs +66 −41: `/shelf`, and #160's and #161's refusals said apart;
- door.rs +60 −38: a lend's `Source`, carried apps in `offered` and `find`,
  `LENDS_NOTHING`, and the door's words in the house voice;
- lend.rs +59 −12: `Source`, `Lend::carried`, `signs_in: Option<bool>`
  (#159), `Offered::group`;
- question.js +45 −5: `once`, `table`, `opens` (#158) and `namesakes`
  (#157);
- build.rs +31, new: the shelf embedded from apps/;
- cli.rs +26 −19, web/app.js +25 −11, home.js +16 −18: the shelf on the
  phone, the findings and the house voice.

The tests are mostly tests/carried.rs (+491) and tests/browser/carried.rs
(+345). apps/ is Money's and Events' carried pages, index.js and
index.html each (5558b28), outside the budget.

What the budget counts, by scripts/lean_check.py over a `git archive` tree
of each end and of each unit's `done` (/tmp/v1uiaudit2/lean-<hash>.out;
each tree's own lean_check.py, so the cap is its own):

| at | svrngs-core | repository | its cap | counted net |
|---|---:|---:|---:|---:|
| 7dbfc37, range start | 3149 | 16567 | 16568 | |
| 3578b26, v1ui-lend-ask done | 3149 | 16567 | 16568 | 0 |
| dc86e9f, v1ui-shelf done | 3149 | 16853 | 16854 | +286 |
| 5bb4b6a, v1ui-findings done | 3149 | 16870 | 16871 | +17 |
| e34f094, v1ui-voice done | 3149 | 16870 | 16871 | 0 |
| ed8b2c6, range end | **3149 / 3150** | **16870 / 16871** | | **+303** |

There were eight raises, each one past its part's count and each naming the
pass (`git log 7dbfc37..ed8b2c6 -- scripts/lean_check.py`):

| raise | from | to | for |
|---|---:|---:|---:|
| d6c7ff1, v1ui-shelf (a) | 16,568 | 16,759 | +191 |
| 296693f, v1ui-shelf (b) | 16,759 | 16,764 | +5 |
| c97ea64, v1ui-shelf (c) | 16,764 | 16,854 | +90 |
| c2f5050, v1ui-findings (c) | 16,854 | 16,856 | +2 |
| a6e62be, v1ui-findings (a) | 16,856 | 16,858 | +2 |
| 16c5d60, v1ui-findings (b) | 16,858 | 16,868 | +10 |
| 6486c2b, v1ui-findings (d) | 16,868 | 16,870 | +2 |
| 678c82b, v1ui-findings (e) | 16,870 | 16,871 | +1 |

That is +303 in all. c97ea64's subject says "the 89 lines", and it raised
90: 16,764 + 89 is 16,853, its count, and the cap is one past it. With
auto-1's +253, **556 of the +650 were raised by ed8b2c6**, against the
ceiling of 16,965 (fence 7). This audit's cut (6795efa) took one counted
line and lowered the cap with it (31b6a28): **16869 / 16870 now, 555
raised and 95 left**. svrngs-core's cap stayed at 3150, with 1 line left.
v1ui-voice rewrote words and counted 0 net.

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
67e46f63… (`shasum -a 256`), the one auto-1 ran.

The corpus is ed8b2c6's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock, by `git archive` into /tmp/v1uiaudit2/prod-ed8b2c6 (361
files). apps/ is in it this time, because the range added Money's and
Events' pages there.
- `code index . --corpus-id prod-ed8b2c6`: 4767 chunks in 723 s
  (index.out).
- `refresh`: rust, 16674 symbols and 96890 references; typescript and
  python "skipped (export failed)" (I4). It fell back to the in-process
  rebuild (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node, crates/svrngs, web, apps/expenses
and apps/events (`git diff --stat 7dbfc37 ed8b2c6 -- crates apps web
scripts bridge`; apps' check.sh files changed a word each). Each was
reported at 0.95 and 0.90 (/tmp/v1uiaudit2/clean-*.md). Whether a side was
added in the range is `git blame` at ed8b2c6 over each side's lines against
7dbfc37 (scripts/blame.sh; blame-clusters.out).

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 579 | 2 | 1 | 9 | 1, one span inside another (I32) |
| crates/svrngs/ | 70 | 0 | 0 | 0 | — |
| web/ | 104 | could-not-judge (I4, I33) | 0 | 3 | — |
| apps/expenses/ | 28 | could-not-judge (I4) | 0 | 0 | — |
| apps/events/ | 63 | could-not-judge (I4) | 0 | 2 | — |

The one cluster with a side the range added:

| tier | its side | its other side | read |
|---|---|---|---|
| near 0.917 | `door` — home/question.js:102-116 (2 lines of 21bcbef) | `judge` — question.js:104-114 | `judge` is the closure `door` returns, nested in it, as auto-1 read it (I32). 21bcbef's two lines are the shift `table` made below it. Not a clone. |

The other clusters, none with a side in the range, are auto-1's again:
api.rs `addr` ×2; blob_tests.rs `flush` == relay_tests.rs (test code, I8);
home.js `running`/`look` (I32); the near pairs in sign_in.rs, synced.rs,
redeem.rs, choices.rs, rail.rs, ring_sync/tests.rs and lend.rs; web's
app.js `uses`/`choicesOf`, lib.rs `connect`/`connect_to_join` and lend.js
`cookieFor`/`keepJar`. Events' two are store.mjs's `log` against `append`
and `people`, each the span beside it (blame: before the range).
carried.rs, the range's largest new file, has no cluster at either
threshold.

**The line-window pass.** It is auto-1's /tmp/v1uiaudit2/scripts/windows.py,
over the node's and the command's src and tests, web/src, web/tests,
web/*.js, and apps/expenses and apps/events: 145 files at ed8b2c6
(windows.files). The result: **193 window hits, 56 spans, 9 new**
(windows.out, windows.verdicts). Both verdicts were seen: 47 spans read
old.

| new span | read |
|---|---|
| tests/carried.rs:18-49 == tests/folders.rs:13-44, and :61-79 == :70-88 (87d68ab) | carried.rs wrote out folders.rs's harness: `signing`, `seed_file`, `start`, `found`, the address book, `round`, `until_on` and `http`. **Made one in 6592309**: tests/common/computers.rs, loaded by those two alone. carried 3 and folders 23 passed, as at HEAD. |
| tests/carried.rs:36-44 == tests/home.rs:39-47 (87d68ab) | `start`'s `RelayConfig`. home.rs's `start` passes `no_seed`, not `seed_file`, so it is its own; it was there before the range. No finding. |
| tests/home.rs:754-766 == :806-816, and :802-813 == :872-883 (686a4c2, 2d0c3f2) | The trace sink and the second dispatcher, written out in three tests. **Made one in 6592309**, `heard(filter)`. home's 7 passed, the three reading their trace. |
| tests/browser/main.rs:2785-2793 == :2840-2848 (3aefde0) | Two tests' setup: ada, cy joined, cy lending a shelf. It is nine lines of fixture, not a decision, so no finding. |
| web/tests/shelf.mjs:11-22 == reads.mjs:17-28, and :15-22 == links.mjs:19-26 (21bcbef) | A driver's preamble written a third time, beside page.mjs's `held`, `launch` and `serve`. **#168.** |
| apps/events/index.js:13-21 == apps/expenses/index.js:12-20 (5558b28) | `el`, `button` and `field`, the two pages' DOM helpers. Each app is a folder of its own, carried by itself, its files listed in src/shelf.txt (build.rs). A helper shared across two apps would make one app's folder need another's. No finding. |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 7dbfc37..ed8b2c6 -- crates apps
web scripts bridge` were matched against
`(struct|enum|trait|type|class)\s+[A-Z]\w*` (new-types.out). They give:
- `Source` (lend.rs:181), new;
- `ConnectSrc`, ported in c98635a and dropped in 87d68ab (removed-types.out),
  so it is not at HEAD;
- carried.rs's private aliases `Refused`, `Answer`, `People`, `File` and
  `Asked`.

`converge noun <Name> --corpus-id prod-ed8b2c6` was run for each
(nouns.out). Four nouns have more than one definition. Lines are at
ed8b2c6, before this audit's commits, each one past the tool's (I1):

| noun | definitions | read |
|---|---|---|
| `People` | blob.rs:146 and carried.rs:101, both `type People = BTreeMap<String, Member>`, in two modules of folder.rs | One alias written twice. **carried.rs names blob.rs's, made `pub(super)`, in 6795efa.** |
| `Answer` | carried.rs:98 (an alias, 87d68ab); door/sign_in.rs:72 (`pub(crate) struct`); node.rs:753 (a fn's own struct) | Three things under one name in one crate. **carried.rs's is renamed `Served` in 47f80cb.** sign_in.rs's and node.rs's two were there before the range and stand: node.rs's is local to its function. |
| `Asked` | carried.rs:122 (an alias); svrngs-core joining.rs:65 (`pub struct`) | **Renamed `Ticks` in 47f80cb**, five letters, so `bring`'s signature stays one line. |
| `Refused` | carried.rs:77 (an alias); svrngs-core command.rs:67 (`pub enum Refused<R>`); and web/src/http.rs:47 (`pub enum Refused`), which converge does not list (I33) | **Renamed `Withheld` in 47f80cb.** core's and web's two are in different crates and were there before the range. |

`Source` and `File` each have one definition, and `ConnectSrc` none. All
five renames and the alias change no type: clippy is clean, carried 3
passed, and scripts/lean_check.py reads 16869 as before the renames.

### Reading the range

Every hunk of product src the range changed was read (`git diff 7dbfc37
ed8b2c6 -- crates web`). The tests and apps were read for clones only.
v1ui-voice's hunks are words: I read them for what each refusal says
being true still, and every one was. 0570d47's `LENDS_NOTHING` (door.rs:199) made
one sentence of four copies: two in door.rs, one in choices.rs, one in
node.rs.

- **Never silently substitute** (AGENTS.md 6).
  - **A carried app's file says "does not hold it yet" for any failure of
    the store** (#163). carried.rs:215 is `held.map_err(|_| NOT_YET)`, and
    :196 and :210 fold the rail's and the layout's errors into `UNREAD`.
    The trace at :198 carries the words, not the error. A store that
    cannot be read is not one that has not been reached yet ("answered: no"
    is not "did not answer").
  - **A write's absent `op` is read as `null`, which voids the act**
    (#165). carried.rs:246 is `body.remove("op").unwrap_or(Value::Null)`,
    and with `corrects` (:251) that is a void. The module's docs say a
    void is `op` null (carried.rs:20-21), not `op` absent. The rail refuses
    `{}` on its own: a null payload is not an object (svrngs-core
    payload.rs:81). A body over `WRITE_MOST` (:59) is told it is not an
    act (`NOT_AN_ACT`, :244).
  - `Folders::carried`'s `standings().unwrap_or_default()` (:168) is not
    one: `Host::standings` warns before its `None` (node.rs:571).
- **Glassbox** (AGENTS.md 1).
  - **Bringing from the shelf drops both its errors** (#164).
    carried.rs:139 has `dirs_of(…).map_err(|_| NO_FOLDER)` and :148 has
    `wrote.map_err(|_| UNWRITTEN)`. `bring`'s trace (:128) says the status
    alone.
  - **A folder under apps/ that is not an app's name is dropped untraced**
    (#167). carried.rs:178 runs `filter_map(|kind| Lend::carried(kind,
    group))`, and lend.rs:229 and :233 return `None` by `.ok()?` for a name
    that is not one. The trace at :181 lists what was found, never what was
    left out.
  - Every refusal the carried route gives is traced once, at
    carried.rs:198, and `bring`'s at :128. `lend_for`'s branch to a carried
    app is traced (door.rs:1065).
- **One decider** (AGENTS.md 8).
  - **"A carried app only to its own group" is decided twice** (#166).
    door.rs:482 keeps a carried app when the asker stands in its group by
    the rail (`standing(g).names(actor)`). node.rs:918 keeps it when the
    group is among the `groups` the caller passed. `Folders::open`
    (carried.rs:196, `NOT_ITS_GROUP`) is the gate. The two lists are its
    shadow, written two ways.
  - #157 and #158 are one decider now (question.js `namesakes`, `opens`),
    and cli.rs `namesakes_said` is pinned to the page's words by
    tests/home_group.rs.
- **Closed sets** (AGENTS.md 9). `Source` is an enum, and each `match` on
  it is total (door.rs:527, :1062), and each let-else on it says the
  other arm (lend.rs:487, cli.rs:1199). #161's
  `ShareRefusal` match (home.rs:339-343) names each arm.
- **Forked code** (AGENTS.md 11). carried.rs's two ports name their
  sources on lines 2-3, and c98635a landed them unchanged
  (`scripts/ports.py`: both "unchanged; here: changed in 5 commit(s)").
- **Validate the instrument** (AGENTS.md 7). v1ui-voice's failed check
  (25bd5f1: one browser bar missed by 347 and 705 ms) is kept as it ran.
  The operator made v1ui-race of it (3bdbf29), and the passing check
  (c7a5715) ran after. Those verdicts are those commits', and are not judged
  again here.
- Not a principle, but worth a line: `door.folders.carried()` reads each
  group's layout on every `find` by label (door.rs:858, the chain is built
  eagerly) and on every `/lends`. That is a cost per request, not a fault.

### Fixed here

- 6795efa: carried.rs's second `People` dropped for blob.rs's (−1 counted
  line).
- 6592309: tests/common/computers.rs, the harness carried.rs copied from
  folders.rs (−210 +118 test lines), and `heard` in home.rs.
- 31b6a28: the cap 16,871 → 16,870.
- 47f80cb: `Served`, `Ticks`, `Withheld`.

Each keeps behaviour, and none adds a counted line. The rest are not fixed
here. #163-#165 and #167 each change a refusal or a trace, and so are
behaviour, each red first. #166 needs one rule both lists ask, a move with
a test. #168 changes three browser drivers, which only the browser suite in
both engines can witness.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 163 | 6, 1 | crates/svrngs-node/src/carried.rs:215 (`held.map_err(\|_\| NOT_YET)`); :196 and :210 (`UNREAD`); the trace at :198 | fixed: 2841e97 (v1ui-polish (c)): `UNSTORED` apart from `NOT_YET` by the store's `has`, every unread cause traced (`traced`); tests/carried.rs red first in 542e683 | A carried app's file answers "this computer does not hold it yet" (503) for every error of the store's `get_bytes`, and "its group cannot be read" for every error of the rail or the layout. The trace carries the words alone. A blob absent and a store that fails are opposite responses, so it is answered:no said as did-not-answer. Red on a store whose read fails. |
| 164 | 1 | crates/svrngs-node/src/carried.rs:139 (`dirs_of(…).map_err(\|_\| NO_FOLDER)`) and :148 (`wrote.map_err(\|_\| UNWRITTEN)`); `bring`'s trace :128 | fixed: 495288a (v1ui-polish (c)): both `io::Error`s traced by `traced`; red first in 0b99bc0 | Bringing an app from the shelf drops the `io::Error` of finding the group's folder and of writing the app's files. The page is told the right words, but the trace says the status only, so why a write failed cannot be read at debug. |
| 165 | 6 | crates/svrngs-node/src/carried.rs:246 (`body.remove("op").unwrap_or(Value::Null)`), against its docs :20-21; :244 (`to_bytes(…, WRITE_MOST).map_err(\|_\| NOT_AN_ACT)`) | fixed: 7759231 (v1ui-polish (c)): an absent `op` is `NOT_AN_ACT`, a body past `WRITE_MOST` `TOO_LARGE` (413), the docs :20-21 saying so; red first in 7859c32 | A carried app's `POST {corrects: id}` with no `op` voids that act: the absence is read as `null`, which the docs give as the void. A body past `WRITE_MOST` (:59) is told "an act is JSON, {op} or {corrects, op}", which is not why it was refused. Red on `{corrects}` alone voiding an act. |
| 166 | 8 | crates/svrngs-node/src/door.rs:482 (`offered.retain(… is_none_or(stands))`, by the rail's standing) and crates/svrngs-node/src/node.rs:918 (`lent.retain(… is_none_or(carries))`, by the groups passed) | fixed: 9516673 (v1ui-polish (c)): lend.rs `Offered::listed`, asked by door.rs `lends` and node.rs's list, behaviour kept | One rule, a carried app listed only in its own group, written twice in two ways. `Folders::open` (carried.rs:196) is the gate. A change to who sees a carried app lands on one list. One function of the group and the asker, which both ask. |
| 167 | 1 | crates/svrngs-node/src/carried.rs:178 (`filter_map(\|kind\| Lend::carried(kind, group))`) over crates/svrngs-node/src/lend.rs:229 and :233 (`.ok()?`) | fixed: 8e78475 (v1ui-polish (c)): lend.rs `Lend::carried` traces the folder left out; red first in fae4b2b | A folder `apps/<x>/index.html` whose `x` is not a lend name or a kind is left out of what a computer carries, and no trace names it. carried.rs:181 lists only what was found. |
| 168 | 8 | web/tests/shelf.mjs:13-22; web/tests/reads.mjs:17-28; web/tests/links.mjs:19-26 | fixed: bbd99f1 (v1ui-polish (c)): page.mjs `driving`, the three drivers' preambles made one, tests only | A browser driver's preamble (the server, the context, the console log, the page, `held`) is written a third time beside page.mjs's `held`, `launch` and `serve`. Test code; one export from page.mjs would collapse it, witnessed by the browser suite in both engines. |
| 80 | — | scripts/lean_check.py:36 BUDGET, :100 `n >= cap` | not fixed (operator's) | 16869 / 16870 after this audit, and 3149 / 3150 in svrngs-core. Of the +650, 555 are raised and 95 left (ceiling 16,965, fence 7). |
| 130, 132-145, 150, 152-156 | as recorded | as recorded | not fixed | Not reached by this range's findings. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early: lend.rs:180 for :181, carried.rs:97 for :98, blob.rs:145 for :146. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". The JS exact tiers (web/, apps/) could not judge. |
| I8 | 6 | dry-report scope | Still present: blob_tests.rs's `flush` reported as a production exact clone of relay_tests.rs's. |
| I19 | 6 | dry-report, `tests/` scopes | Still present: the tool drops every path with a `tests` segment. The line-window pass stood in, and found the range's eight new test spans there. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: the daemon nudge refused, then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's near tier, nested spans | Still present: question.js `door` over its returned `judge`, home.js `running` over its nested `look`. |
| I33 | 6 | SCIP over web/src | New: web/ is a Cargo workspace of its own (the root's `members`, Cargo.toml:3, leaves it out), so `refresh` exports none of web/src's Rust. `converge noun Refused` lists core's and carried.rs's and not web/src/http.rs:47, and web/'s exact tier says "no SCIP function symbols in scope", which auto-1 read as I4 alone. |

## REVIEW-audit-v1ui-auto-3 (2026-10-15, range cbe994c..469394e)

The previous `REVIEW-audit-` row is REVIEW-audit-v1ui-auto-2, its hash
cbe994c (ralph/next/v1ui/STATE.md:22). The range ends at 469394e, the
commit that made this row. `git rev-list --count cbe994c..469394e` is 53,
none a merge (`--merges`: 0). They are: auto-2's `done`; v1ui-race;
v1ui-run; v1ui-polish; the operator's arch queue commit f8beba7, made
between v1ui-polish's (c) and (d); and this row's own commit.

This audit's own commits before this section:
- d90994a, the row's two paths, as 835fc08 corrected auto-2's;
- 28884ce, "a port nothing listens on" made once, tests/common `stopped`.

It closes #163-#168, which v1ui-polish (c) fixed (their rows above now
say with which commit), and opens #169-#173.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
cbe994c..469394e` (/tmp/v1uiaudit3/ledger.raw) is summed by auto-2's
ledger.py, its unit list this range's (/tmp/v1uiaudit3/ledger.out; each
commit's unit in ledger.commits). The rules are auto-1's. Every commit
names its unit; f8beba7 names `arch`. 30acc6a, v1ui-run's state commit,
names v1ui-polish and is counted there (0 lines). The lines are raw,
comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-v1ui-auto-2 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| v1ui-race | 8 | 41 | 24 | **+17** | 81 | 20 | +61 | 0 | 0 | 0 | +0 (+3 −3) |
| v1ui-run | 10 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| v1ui-polish | 32 | 162 | 59 | **+103** | 436 | 113 | +323 | 0 | 0 | 0 | +0 (+21 −21) |
| arch (f8beba7, the queue) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 53 | 203 | 83 | **+120** | 517 | 133 | +384 | 0 | 0 | 0 | +0 (+24 −24) |

v1ui-run's driver, judge and data are under demo/runs/, which no column
counts. Where the src lines went (`git diff --numstat cbe994c 469394e --
crates web`, tests apart):
- door.rs +49 −21: `paged` and `Words`, svrngs's refusal drawn as a page
  to a tab (v1ui-polish (a)); a stopped lend said by its lender; `lends`
  asking `Offered::listed` (#166);
- carried.rs +23 −10: `traced`, `UNSTORED`, `TOO_LARGE`, an absent `op`
  refused (#163-#165);
- share.rs +20 −5, build.rs +13 −1, home.rs +13 −2, well_known.txt +9:
  the `helper` row, `helper_of`, `HELPERS`, `/helper/<kind>` (v1ui-polish
  (d));
- web/app.js +20 −22, web/src/lib.rs +20 −3: the page dials every computer
  at once, `Endpoint::dial_link` (v1ui-race);
- lend.rs +16 −3: `Offered::listed`, `Lend::carried` traced (#166, #167);
- home.js +9 −5, question.js +8 −2: `owner`, `named` by kind, the helper's
  link (v1ui-polish (b), (d)).

The tests are mostly tests/carried.rs (+157), tests/browser/main.rs (+84
−20), tests/home.rs (+61 −19) and door/sign_in.rs (+59 −8). The web
drivers shrank by #168's `driving` (links, reads, shelf, reach: −73 +17).
apps/ changed only apps/jellyfin/README.md, the helper's steps (+8 −4).

What the budget counts, by scripts/lean_check.py over a `git archive` tree
of each end and of each unit's `done` (/tmp/v1uiaudit3/lean-<hash>.out;
each tree's own lean_check.py, so the cap is its own):

| at | svrngs-core | repository | its cap | counted net |
|---|---:|---:|---:|---:|
| cbe994c, range start | 3149 | 16869 | 16870 | |
| 4806ec6, v1ui-race done | 3149 | 16875 | 16876 | +6 |
| 5bf9941, v1ui-run done | 3149 | 16875 | 16876 | 0 |
| f6f8af1, v1ui-polish done | 3149 | 16933 | 16934 | +58 |
| 469394e, range end | **3149 / 3150** | **16933 / 16934** | | **+64** |

There were eight raises, each naming the pass (`git log cbe994c..469394e
-- scripts/lean_check.py`):

| raise | from | to | for |
|---|---:|---:|---:|
| f76cb20, v1ui-race (a) | 16,870 | 16,876 | +6 |
| 145672c, v1ui-polish (a) | 16,876 | 16,894 | +18 |
| e9ad1ca, v1ui-polish (b) | 16,894 | 16,895 | +1 |
| 026ce2b, v1ui-polish (c) #165 | 16,895 | 16,896 | +1 |
| 4461db3, v1ui-polish (c) #163 | 16,896 | 16,904 | +8 |
| e7d604d, v1ui-polish (c) #167 | 16,904 | 16,907 | +3 |
| 6ea0180, v1ui-polish (c) #166 | 16,907 | 16,910 | +3 |
| 20644ee, v1ui-polish (d) | 16,910 | 16,934 | +24 |

That is +64, the counted net. f76cb20 and 145672c each say "and one past";
4461db3 says "the 7 lines" and raised 8, and 20644ee "the 23 lines" and
raised 24, each landing one past its count as the cap is. With auto-1's
and auto-2's, **619 of the +650 are raised by 469394e, 31 left** under the
ceiling of 16,965 (fence 7). No line is left under the repository's cap
(16933 / 16934: lean_check fails at `n >= cap`). svrngs-core stayed at
3149 / 3150. This audit's one cut (28884ce) is test code and counts
nothing, so the cap stands.

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
c95165c5… (`shasum -a 256`): rebuilt since auto-2's 67e46f63…. It found
what auto-2's found where the code did not change (the clusters below), so
the rebuild moved nothing this audit reads.

The corpus is 469394e's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock, by `git archive` into /tmp/v1uiaudit3/prod-469394e (362 files).
- `code index . --corpus-id prod-469394e`: 4784 chunks in 658 s
  (index.out).
- `refresh`, run in the corpus: rust, 16834 symbols and 97833 references;
  typescript and python "skipped (export failed)" (I4); the daemon nudge
  refused and the in-process rebuild ran (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node, crates/svrngs (its share tests
alone) and web (`git diff --stat cbe994c 469394e -- crates apps web
scripts bridge`); apps/ only in a README. Each was reported at 0.95 and
0.90 (/tmp/v1uiaudit3/clean-*.md). Whether a side was added in the range
is `git blame` at 469394e over each side's lines against cbe994c
(scripts/blame.sh; blame-clusters.out).

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 584 | 2 | 0 | 9 | 1, one span inside another (I32) |
| crates/svrngs/ | 70 | 0 | 0 | 0 | — |
| web/ | 105 | could-not-judge (I4, I33) | 0 | 3 | 2 |

The clusters with a side the range added:

| tier | its side | its other side | read |
|---|---|---|---|
| near 0.946 | `running` — home/home.js:140-164 (2 lines of a754531) | `look` — home.js:143-163 | `look` is nested in `running` (I32), as auto-2 read it. a754531's two lines are the helper's link in each `li`. Not a clone. |
| near 0.920 | `uses` — web/app.js:290-312 (2 lines of 855caa3) | `choicesOf` — app.js:293-308 | 855caa3's two lines are :311-312, the doc of `reachGroup` below, which the span reaches by its line skew (I1). The pair is auto-1's, before the range. |
| near 0.902 | `dial_link` — web/src/lib.rs:417-426 (8 lines of 855caa3) | `dial` — lib.rs:409-418 | Two of `Endpoint`'s three two-line wrappers over `reach` (`dial`, `dial_link`, `dial_lend`, :412-431): each one wasm export, its arguments the whole of it; the decisions are in `reach`. No finding for the pair; that `dial_link` is a second way to dial what `connect` dials is #172. |

The other clusters, none with a side in the range, are auto-2's again:
api.rs `addr` ×2; blob_tests.rs `flush` == relay_tests.rs (test code,
I8); question.js `door`/`judge` (I32); sign_in.rs `of`/`forget`;
synced.rs `kept_files`/`kept_at`; redeem.rs `router`/`ask_route`;
choices.rs `load`/`read`; rail.rs `live_push`/`live_drain`;
ring_sync/tests.rs `admit`/`ingest`; lend.rs `from_mbit`/`bps`; web's
lend.js `cookieFor`/`keepJar`. Web's lib.rs `connect`/`connect_to_join`
no longer clusters at 0.90.

**The line-window pass.** auto-2's windows.py over the same set, at
469394e: 147 files (windows.files; build.rs and tests/common/computers.rs
added). **154 window hits, 48 spans, 0 new** (windows.out,
windows.verdicts: every span "old"). That newspans.sh can say NEW at all
was seen on a plant: lend_origin.rs:744-754, written in ea888b4, against
home.rs:1-5 gave "NEW … [ea888b4bf]" (plant.spans, plant.verdicts).

Reading the range's test hunks found what the 8-line window cannot: "a
port nothing listens on", five lines, written in door/sign_in.rs:392
(1addb0d) and lend_origin.rs:719 (ea888b4) beside door/sign_in.rs:347 and
home.rs:596, which were there before. **Made one in 28884ce**,
tests/common `stopped`; the four tests that use it passed, as at HEAD
(stopped-tests.out).

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges cbe994c..469394e -- crates apps
web scripts bridge`, matched against `(struct|enum|trait|type|class)\s+[A-Z]\w*`
(new-types.out), give one noun: `Words` (door.rs:276, ea888b4). None was
removed (removed-types.out).

`converge noun Words --corpus-id prod-469394e` (nouns.out): **one
definition**, crates/svrngs-node/src/door.rs:275 (:276, I1). No noun the
range added has more than one. That the tool lists more than one where
there are was seen on `Answer`: door/sign_in.rs:71 and node.rs:752
(nouns-plant.out).

### Reading the range

Every hunk of product src the range changed was read (`git diff cbe994c
469394e -- crates/svrngs-node/src crates/svrngs-node/build.rs web/app.js
web/src`). The tests were read for clones.

- **Never silently substitute** (AGENTS.md 6).
  - A lend now signs in by its kind's helper when the helper answers and
    its lender named none (share.rs:237-238). That is said: on the home
    page (home.js `needs`, "Its helper is running on this computer…"),
    by `svrngs share` (cli.rs:1221, "signed in to it as themselves by
    what answers on port …"), and traced (share.rs `lend_of`, `?helper,
    ?sign_in`). Not a substitution.
  - `shown`'s `helper` (home.rs:251) is three states, none, not answering
    and answering, each drawn apart (home.js `needs`, `helps`).
  - A carried file the store fails to read is `UNSTORED`, and one whose
    `has` itself fails is too, never "not held yet" (carried.rs:223-226).
  - A stopped lend's lender, when this computer stands in no group that
    names it, is "Its lender" (door.rs:1351): no one named, never a wrong
    one or a host, and pinned by lend_origin.rs's new test. It is not
    traced (#173).
- **Glassbox** (AGENTS.md 1).
  - **`GET /helper/<kind>` answers or refuses untraced** (#173), and so is
    door.rs:1351's fallback.
  - The door's refusal for a stopped lend traces `up` and the sign-in's
    `why` (door.rs:1365); the page's dials trace each computer reached and
    each closed late (app.js `reachGroup`).
- **One decider** (AGENTS.md 8).
  - **"A browser's tab asking for a page", by `Accept`, is decided twice**
    (#170): door.rs:282 (`paged`, ea888b4) and door/sign_in.rs:190.
  - **Whether a lend's port takes a connection is asked twice** (#171):
    door.rs:1363-1364 (ea888b4) beside `send_to`'s connect, :1203, with the
    same `CONNECT_WITHIN` and address; the probe's own error is dropped.
  - **The link's computer is dialled two ways** (#172): wasm `connect`
    (web/src/lib.rs:305), which 855caa3 left with web/tests/drive.mjs:35 its
    only caller, and `Endpoint::dial_link` (:420), the page's.
  - #166 and #168 are one decider now (`Offered::listed`, `driving`).
- **Closed sets** (AGENTS.md 9). **well_known.txt's first column is one
  of three things, a port, `-` or `helper`, and is read as a string in
  five places** (#169): share.rs:113, :123 and :166 and build.rs:38 (all
  a754531's), and home.rs:286 (before the range).
- **Forked code** (AGENTS.md 11). The range forked nothing (`git log
  --grep "ported from" cbe994c..469394e`: none); `scripts/ports.py` lists
  243 ports.
- **Validate the instrument** (AGENTS.md 7). v1ui-run kept both takes
  (take 1 three of five, Jellyfin stopped; take 2 five of five) and the
  planted one, each as it ran (18dd74f); v1ui-polish (d) ran its plant
  `nohelper` before its clean take (ad5e75b). Those verdicts are those
  commits', not judged again here.

### Fixed here

- d90994a: the row's two paths.
- 28884ce: tests/common `stopped`, four copies made one (−21 +17 test
  lines, none counted).

The rest are not fixed here. No line is left under the cap, and each of
#169-#171 and #173 adds counted lines (an enum, a function, a trace).
#172 changes the browser driver's dial, which binds unlike the page's
(`bind_for`'s `held`, lib.rs:83), and only the browser suite in both
engines can witness it.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 169 | 9, 8 | crates/svrngs-node/src/share.rs:113 (`*port != "helper"`), :123 (`*p != "helper"`), :166 (`*p == "helper"`); crates/svrngs-node/build.rs:38 (`["helper", kind, _, steps]`); crates/svrngs-node/src/home.rs:286 (`*port == "-"`) | 8a3c044 (arch-helper (c); build.rs stays the build's own reader, pinned by share/tests.rs:72, 6f56177 says why) | `share::rows()` (share.rs:62) yields a row's first column as a `&str`, and every reader decides for itself whether it is a port, `-` (a kind with no port) or `helper`. `known_kinds` and `says` must each remember to leave `helper` rows out: without it `known_kinds` names jellyfin twice, and `says` finds Jellyfin's sentence only because its port row comes before its helper row in the file. A fix parses the column once into an enum in `rows()`, the readers matching it; build.rs, the build's own reader of the table, pinned against `helper_of` by crates/svrngs/src/share/tests.rs:72 as it is now. |
| 170 | 8 | crates/svrngs-node/src/door.rs:282 (`paged`: `sign_in::said(…, ACCEPT).contains("text/html")`) and crates/svrngs-node/src/door/sign_in.rs:190 (the same, for the copy kept before a person's script) | 91f9839 (arch-helper (c), `sign_in::a_tab`) | One rule, a browser's tab asking for a page, written twice. A change (say, to `Sec-Fetch-Dest: document`) lands on one. One function in sign_in.rs both ask. |
| 171 | 8, 1 | crates/svrngs-node/src/door.rs:1363-1364 (`forward`'s probe, ea888b4) and :1203 (`send_to`'s connect) | 55fd7f5 (arch-helper (c), door.rs `connected`) | Whether a lend's port takes a connection within `CONNECT_WITHIN` is asked by two pieces of code, and the probe keeps only `up`, dropping its error, where `send_to` traces each. One `connected(to, lend)` both ask, its two warns kept. |
| 172 | 8 | web/src/lib.rs:305 (`connect`, its only caller web/tests/drive.mjs:35) and :420 (`Endpoint::dial_link`, web/app.js `reachGroup`) | 9543e52 (arch-helper (c); the browser test in both engines, c172-*.out) | Since 855caa3 the page dials the link's computer by `dial_link`, from the endpoint it dials lenders from (held, `bind_for(…, true)`), and the product's wasm keeps `connect` (its own endpoint, `held` false) for the test driver alone. The driver tests a dial the page no longer makes. A fix drives `endpoint(…).dial_link()` as the page does and drops `connect` (four counted lines), the browser suite in both engines before and after. |
| 173 | 1 | crates/svrngs-node/src/home.rs:257-261 (`steps`); crates/svrngs-node/src/door.rs:1350-1351 (the lender, or "Its lender") | 2018b7d (arch-helper (c)) | `GET /helper/<kind>` answers the steps or refuses with no trace, and `forward`'s refusal for a stopped lend traces neither the lender it found nor that it found none. Each a debug line, red first by `heard`. |
| 80 | — | scripts/lean_check.py:37 BUDGET, :104 `n >= cap` | not fixed (operator's) | 16933 / 16934 after this audit, and 3149 / 3150 in svrngs-core. Of the +650, 619 are raised and 31 left (ceiling 16,965, fence 7). |
| 130, 132-145, 150, 152-156 | as recorded | as recorded | not fixed | Not reached by this range's findings. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, dry-report spans | Still one line early: door.rs:275 for :276, sign_in.rs:71 for :72. A dry-report span (app.js `uses`, :290-312) reaches into the next function's doc. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". The JS exact tier (web/) could not judge. |
| I8 | 6 | dry-report scope | Still present: blob_tests.rs's `flush` reported as a production exact clone of relay_tests.rs's. |
| I19 | 6 | dry-report, `tests/` scopes | Still present. The line-window pass stood in, and reading found the one span under its window (28884ce). |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: the daemon nudge refused, then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's near tier, nested spans | Still present: home.js `running` over `look`, question.js `door` over `judge`. |
| I33 | 6 | SCIP over web/src | Still present: web/'s exact tier says "no SCIP function symbols in scope". |

## arch-helper (a) (2026-10-15): what lending the sign-in helper lets in

Opened by its row (`ralph/next/arch/STATE.md`), not a `REVIEW-audit-` row:
the run is `demo/runs/arch/2026-10-15-helper/` (README.md, a-red.out,
mw.out).

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 174 | 8, 6 | crates/svrngs-node/src/home.rs:279-282 (`taken`: a lend's sign-in left out, the table's helper not), :308-369 (`lend_one`); crates/svrngs-node/src/share.rs `lend_of`; crates/svrngs/src/cli.rs:768 | 1acbba2 (arch-helper (b), `share::signs_in`) | **Low.** The home page offers a kind's sign-in helper as something to lend until a lend names it, and lends it when asked; so does `svrngs share <its port>`. A member who opens it is told to it as themselves: their typed key and stand-in are displaced (door.rs:689), so no sign-in as another person. Jellyfin's own helper answers them 403 and 404, makes no account, and gives neither its API key nor a token (mw.out). A helper that did not ask for the door's stand-in would give a member their own token in plain (a-red.out), which the door keeps from every browser for a lend that signs in by it. |

## arch-peer (a) (2026-10-15): what the bridge lets reach a friend's door

Opened by its row (`ralph/next/arch/STATE.md`), not a `REVIEW-audit-` row:
the run is `demo/runs/arch/2026-10-15-peer/` (README.md, a-red.out,
chromium-public-unasked.out), the test `crates/svrngs-node/tests/peer.rs`.

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 175 | 6, 10 | crates/svrngs-node/src/guest.rs:245-480 (`HttpBridge`, a loopback listener that accepts anyone); crates/svrngs-node/src/iroh.rs:238-292 (`bridge_for`, at `preferred_bridge_port`, :363); crates/svrngs-node/src/door.rs:990-991 (`own_app`: no pass, no app), :1066-1076 (`Who::Member`, `app: None`); crates/svrngs-node/src/door/choices.rs:268-290 (`choose`) | fixed, a27e688 (arch-peer (d): the bridge cut; `tests/peer.rs`'s four green, `demo/runs/arch/2026-10-15-peer/d-green.out`) | **High.** Each computer listens on 127.0.0.1 for each member it dials, and carries whatever connects there to that member's door as this computer's key. A program holding no key reaches every lend that asks nobody as this computer's person, reads their choices, and keeps a yes as them for any app on a lend that asks; a page on loopback does the last two in Chromium and WebKit, and a page on the internet in Chromium once the person allows it to reach the local network (in WebKit, could-not-judge). The answers are kept from a page (CORS), not the act. |

## arch-stream (a) (2026-10-15): two endpoints of one key on one relay

Opened by its row (`ralph/next/arch/STATE.md`), not a `REVIEW-audit-` row:
the run is `demo/runs/stream/2026-10-15/` (README.md, finding 5).

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 176 | 6 | web/app.js `enter` (`dialler`, `bound`: one endpoint per page, never closed); iroh-relay-1.0.2/src/server/clients.rs:80-92 (one active client per key) | fixed (groups-tabs): run in 0bfc4ad0, red in both engines, the first tab's every ask "did not answer" for 60 s (demo/runs/tabs/2026-10-16/); e38afab8, one tab at a time holds the group's lock, another says it is open there and offers "Open it here"; tests/browser/tabs.rs | **Low, inferred, not run.** In the streaming test, a second endpoint of the page's own key lost its connection 16.0–23.9 s after it bound, when the page's first endpoint reconnected to the relay and took the key back, and its dial then failed at the idle timeout as "asleep" (the run's README, 1). A person who opens one group's page in two tabs of one browser holds two endpoints of one key, one per tab, and by the same mechanism a tab's dial may be cut and told "asleep" while the computer is awake. Not run with two tabs; a test opening two tabs would show it or not. |

## REVIEW-audit-arch-auto-1 (2026-10-16, range fafd42d5..0a2e6b3a)

The previous `REVIEW-audit-` row is REVIEW-audit-v1ui-auto-3, its hash
fafd42d5 (ralph/next/v1ui/STATE.md:26). The range ends at 0a2e6b3a, the
commit that made this row. `git rev-list --count fafd42d5..0a2e6b3a` is
94, none a merge (`--merges`: 0). They are: auto-3's `done`; arch-helper,
arch-peer, arch-stream, arch-seal, arch-core, arch-view,
arch-command-relay, arch-command and arch-version; the operator's and
the director's commits between them (dda32b22 `arch`, f5fe02fa
`grapevine`, d1c2a1c4 `groups`, 8ee132c0 and 75fd900d); and this row's
own commit. The row reads `docs/ARCH_PRINCIPLES.md`; the file is
`docs/principles/ARCH_PRINCIPLES.md` (AGENTS.md).

This audit's own commits before this section:
- 6ead9443, Alex's computer and Sam's made once, tests/common/pair.rs;
- 684dade8, a test's trace sink made once, iroh/tests.rs `Said`;
- 98fe181f, iroh.rs's `Answered` alias renamed `Streamed`.

It opens #177 and #178.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
fafd42d5..0a2e6b3a` (/tmp/archaudit1/ledger.raw) is summed by auto-3's
ledger.py, its unit list this range's (/tmp/archaudit1/ledger.out; each
commit's unit in ledger.commits). The rules are auto-1's: a commit's unit
is the longest unit id its subject names. So 75fd900d, "director(arch-command):
… splits out as arch-command-relay", counts under arch-command-relay,
and 98615583 and daa6444d, the state and charter commits naming
arch-stream, under arch-stream; each touched no code. The lines are raw,
comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-v1ui-auto-3 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| arch-helper | 13 | 123 | 76 | **+47** | 250 | 19 | +231 | 0 | 0 | 0 | +2 (+10 −8) |
| arch-peer | 19 | 395 | 910 | **−515** | 835 | 338 | +497 | 0 | 0 | 0 | +1 (+3 −2) |
| arch-stream | 9 | 0 | 0 | **0** | 40 | 10 | +30 | 0 | 0 | 0 | 0 |
| arch (dda32b22, the queue) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| grapevine (f5fe02fa, its queue) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| arch-seal | 6 | 94 | 874 | **−780** | 125 | 801 | −676 | 0 | 0 | 0 | +0 (+1 −1) |
| arch-core | 5 | 65 | 137 | **−72** | 29 | 40 | −11 | 0 | 0 | 0 | +0 (+1 −1) |
| arch-view | 11 | 22 | 22 | **+0** | 94 | 3 | +91 | 0 | 0 | 0 | +1 (+3 −2) |
| arch-command | 12 | 756 | 697 | **+59** | 110 | 20 | +90 | +0 (+1 −1) | 0 | 0 | +4 (+8 −4) |
| arch-command-relay | 6 | 99 | 123 | **−24** | 96 | 7 | +89 | 0 | 0 | 0 | +2 (+4 −2) |
| arch-version | 8 | 149 | 122 | **+27** | 142 | 69 | +73 | 0 | 0 | 0 | +2 (+4 −2) |
| groups (d1c2a1c4, its row) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 94 | 1703 | 2961 | **−1258** | 1721 | 1307 | +414 | +0 (+1 −1) | 0 | 0 | +12 (+34 −22) |

arch-command's src +756 −697 holds 0e4684ca (+189 −133, its first (a))
and its revert 0c5df4aa (+133 −189), which cancel (`git show --numstat`).
Where the src lines went (`git diff --numstat fafd42d5 0a2e6b3a -- crates
web`, tests apart), the largest:
- guest.rs +12 −370, iroh.rs +163 −168, ring_sync.rs +57 −180, live.rs
  +27 −106, redeem.rs +43 −55, origin.rs +23 −29: the loopback bridge
  cut, each caller's request carried on the stream (arch-peer);
- svrngs-core sync.rs +28 −141, lib.rs +33 −82, project.rs +17 −65,
  admit.rs +21 −47, svrngs-runtime journal.rs +1 −231, oplog.rs +3 −84
  and lib.rs +8 −43, rail.rs +34 −61, and core's process.rs gone: the
  seal and its compaction (arch-seal), the traits folded (arch-core);
- cli.rs +66 −209, join.rs +147 (new), home.rs +129 −58, invite.rs
  +39 −19, node.rs +101 −107, surface.rs +24 −55, svrngs's issuer.rs
  gone: the join and the link made by the node, the home page's join, one
  state for the routers (arch-command, arch-command-relay);
- version.rs +25 (new), web/app.js +18 −25, web/src/lib.rs +22 −28:
  one version check, wasm `connect` gone (arch-version; arch-helper
  #172);
- share.rs +44 −10, door.rs +28 −20, door/sign_in.rs +8 −1: a sign-in
  never offered, `share::At`, `a_tab`, `connected` (arch-helper).

The tests grew by tests/peer.rs (arch-peer (a)) and the browser suite's
streaming test (arch-stream), and lost the seal's (core tests_sealing.rs
+25 −128, svrngs-runtime's +40 −420). apps/ did not change.

What the budget counts, by scripts/lean_check.py over a `git archive` tree
of each end and of each unit's `done` (/tmp/archaudit1/lean-<hash>.out;
each tree's own lean_check.py, so the cap is its own):

| at | svrngs-core | repository | its cap | counted net |
|---|---:|---:|---:|---:|
| fafd42d5, range start | 3149 / 3150 | 16933 | 16934 | |
| 965b52b5, arch-helper done | 3149 / 3150 | 16963 | 16964 | +30 |
| 472e06e2, arch-peer done | 3149 / 3150 | 16575 | 16576 | −388 |
| c9e4c12a, arch-stream done | 3149 / 3150 | 16575 | 16576 | 0 |
| 06913a33, arch-seal done | 2992 / 2993 | 16198 | 16199 | −377 |
| c276c34f, arch-core done | 2954 / 2955 | 16162 | 16163 | −36 |
| 4a741668, arch-view done | 2952 / 2953 | 16160 | 16161 | −2 |
| 7bca97e6, arch-command-relay done | 2952 / 2953 | 16130 | 16131 | −30 |
| ecd0b28a, arch-command done | 2952 / 2953 | 16148 | 16149 | +18 |
| 23de662b, arch-version done | 2952 / 2953 | 16144 | 16145 | −4 |
| 0a2e6b3a, range end | **2952 / 2953** | **16144** | **16145** | **−789** |

The caps moved ten times (`git log fafd42d5..0a2e6b3a --
scripts/lean_check.py`):

| commit | the repository's cap | svrngs-core's | for |
|---|---|---|---|
| 8222c36d, arch-helper (b) | 16,934 → 16,955 | 3,150 | +21, a sign-in never offered nor lent |
| 6f561772, arch-helper (c) | 16,955 → 16,964 | 3,150 | +9, #169's enum (its subject says "the 10 lines") |
| 3a3f3fce, arch-peer | 16,964 → 16,576 | 3,150 | −388, the bridge cut |
| 50783fc9, arch-seal | 16,576 → 16,199 | 3,150 → 2,993 | −377, −157 |
| 5c6c8af9, arch-core | 16,199 → 16,163 | 2,993 → 2,955 | −36, −38 |
| cf08b1bf, arch-view | 16,163 → 16,161 | 2,955 → 2,953 | −2, −2 |
| 699e3d70, arch-command-relay | 16,161 → 16,131 | 2,953 | −30 |
| d556c768, arch-command (a) | 16,131 → 16,091 | 2,953 | −40, (a)'s cuts |
| e527da18, arch-command | 16,091 → 16,149 | 2,953 | +58, the page's join, within the row's 16,190 (the operator) |
| c5a09854, arch-version | 16,149 → 16,145 | 2,953 | −4 |

arch-helper raised +30 of the 31 v1ui left (fence 7's one exception, to
16,964 under the ceiling of 16,965). arch-command's raise is the one its
row gives (ralph/next/arch/STATE.md, "THE BUDGET (the operator) … at most
at 16,190"), and the row closed at 16,148, 18 over where it started.
Every other row closed below where it started and lowered both caps to
one past its close. Over the range: **−789 counted lines, −197 of them
svrngs-core's**. No line is left under either cap (16144 / 16145, 2952 /
2953: lean_check fails at `n >= cap`, lean_check.py:116). This audit's
three commits move no counted line (test code, and a rename), so the
caps stand.

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
c95165c5… (`shasum -a 256`): auto-3's, unchanged.

The corpus is 0a2e6b3a's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock, by `git archive` into /tmp/archaudit1/prod-0a2e6b3a (366
files).
- `code index . --corpus-id prod-0a2e6b3a`: 4748 chunks in 448 s
  (index.out).
- `refresh`, run in the corpus: rust, 16832 symbols and 97502 references;
  typescript and python "skipped (export failed)" (I4); the daemon nudge
  refused and the in-process rebuild ran (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node, crates/svrngs, crates/svrngs-core,
crates/svrngs-runtime and web (`git diff --numstat fafd42d5 0a2e6b3a --
crates apps web scripts bridge`); apps/ not at all. Each was reported at
0.95 and 0.90 (/tmp/archaudit1/clean-*.md). Whether a side was added in
the range is `git blame` at 0a2e6b3a over each side's lines against
fafd42d5 (scripts/blame.sh over the 63 sides, cluster-sides.txt;
blame-clusters.out, which saw both "before" and "IN RANGE").

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 588 | 3 | 2 | 11 | 5 |
| crates/svrngs/ | 65 | 0 | 0 | 0 | — |
| crates/svrngs-core/ | 441 | 4 | 0 | 10 | 0 |
| crates/svrngs-runtime/ | 122 | 1 | 0 | 0 | 0 |
| web/ | 104 | could-not-judge (I4, I33) | 0 | 2 | 0 |

The clusters with a side the range added:

| tier | its side | its other side | read |
|---|---|---|---|
| exact ×3, ×2; near 1.000 | `flush`, `write` — guest/tests.rs:51-60 (a88c4b34) | relay_tests.rs:153-162, blob_tests.rs:161 | A test's trace sink written out a third time (test code, I8). **Made one in 684dade8.** |
| near 0.928 | `signs_in` — share.rs:189-200 (1acbba28) | `helper_of` — share.rs:180-189 | `signs_in` asks `helper_of`; the spans overlap by a line (I1). Not a clone. |
| near 0.914 | `router` — redeem.rs:114-125 (3 lines of d8c871a2) | `ask_route` — redeem.rs:121-134 | d8c871a2's lines are the state's type, `Asking`. The spans overlap (I1), as auto-1 read the pair. Not a clone. |
| near 0.962 | `running` — home/home.js:142-168 (4 lines of 1acbba28) | `look` — home.js:145-167 | `look` is nested in `running` (I32), as auto-2 and auto-3 read it. 1acbba28's four lines say a sign-in left out. Not a clone. |

The other clusters, none with a side in the range, are auto-3's again
(api.rs `addr` ×2; sign_in.rs `of`/`forget`; synced.rs
`kept_files`/`kept_at`; question.js `door`/`judge`; choices.rs
`load`/`read`; rail.rs `live_push`/`live_drain`; ring_sync/tests.rs
`admit`/`ingest`; lend.rs `from_mbit`/`bps`; app.js `uses`/`choicesOf`;
lend.js `cookieFor`/`keepJar`), and svrngs-core's and svrngs-runtime's,
which no audit since lean-auto-1 reported: three-line accessors
(`is_empty`, `as_str`, `lines`, `signed_by_link`; runtime tests' `open`)
and test bodies alike in shape (removal_tests.rs, register_tests.rs,
membership_invite_tests.rs; I8). No side of them is the range's.

**The line-window pass.** auto-3's windows.py over the crates' src and
tests (core and runtime added, as the range touched them), web/src,
web/tests, web/*.js, apps/events and apps/expenses: 204 files at
0a2e6b3a (windows.files). **192 window hits, 64 spans, 13 new**
(windows.spans, windows.verdicts; 51 read old). All 13 are test code:

| new span | read |
|---|---|
| guest/tests.rs:48-69 == node's relay_tests.rs:150-170, == svrngs's src/relay_tests.rs:9-20, == svrngs's tests/relay.rs:103-113 (a88c4b34) | The trace sink. Node's three made one in 684dade8; svrngs's three are another crate's test binaries, each a static of its own. No finding. |
| peer.rs:97-115 == home.rs:132-150 == lend_origin.rs:133-151; peer.rs:52-65 == home.rs:39-52; peer.rs:52-60 == tests/common/computers.rs:30-38 (d08a4cf1) | peer.rs wrote out `alex_lets_sam_in`, `start`, `no_seed`, `signing` and `NS` a third time. **Made one in 6ead9443**, tests/common/pair.rs; computers.rs's `start` keeps `seed_file`, so is its own. |
| lend_origin.rs:624-633 == peer.rs:94-103 (d08a4cf1) | `a_lend_s_computer_asleep_is_said_away_within_the_cap` sets Alex and Sam up with a delay between, before the range. No finding. |
| peer.rs:314-321 == conformance.rs:521-528 == svrngs's home_group.rs:124-131 (d08a4cf1) | Running a browser half under node and failing with its stderr, eight lines in five test binaries (`git grep "browser half failed"`), each its own script. Fixture, not a decision. No finding. |
| svrngs's tests/common/mod.rs:28-36 == new_relay.rs:47-55 == phone_dry.rs:59-67 (a9897846) | Sam's computer started at a relay on 127.0.0.1. new_relay.rs starts its own relay because it loads neither the browser harness nor common (common/mod.rs:13, `use crate::harness`), as phone_dry.rs does. No finding. |
| svrngs's home_group.rs:182-190 == :296-303 (0c2ced28) | Two browser tests' setup. No finding. |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges fafd42d5..0a2e6b3a -- crates
apps web scripts bridge`, matched against `(struct|enum|trait|type|class)
+[A-Z]\w*` (new-types.out), give 15 nouns: `At`, `Browser`, `Entered`,
`NotJoined`, `Unsent`, `Answer`, `Bridge`, `Dialled`, `Joining`, `Kept`,
`Linked`, `Answered`, `Request`, `To`, `Whole`. `Entered`, `NotJoined`
and `Answer` were also removed (the join moved, reverted and moved again;
node.rs's `Answer` rewritten in place by 533c6442). The range removed 25
(removed-types.out), among them `Projection`, `RingSigner`,
`ProcessManager`, `Seal`, `Compaction`, `HttpBridge`, `Surface` and
`Issuer`.

`converge noun <Name> --corpus-id prod-0a2e6b3a` for each (nouns.out).
The nouns with more than one definition:

| noun | definitions | read |
|---|---|---|
| `At` | crates/svrngs/src/relay.rs:56 (:57, I1), where a relay is reached; crates/svrngs-node/src/share.rs:62 (:63), well_known.txt's first column (8a3c0448) | Two concepts in two crates; svrngs never names `share::At` (`git grep "share::At" crates/svrngs`: none). No finding. |
| `Answer` | door/sign_in.rs:71 (:72); node.rs:757 (:758, a struct inside `answered`) | Both before the range; auto-3 used this pair as its plant. Distinct, the second local to one function. No finding. |
| `Answered` | iroh.rs:260 (:261, 07ae5d19: an answer's head, its body still to read); node.rs:720 (:721, how a computer asked what it lends answered, there at fafd42d5's node.rs:741) | One crate, one noun, two meanings. **Renamed in 98fe181f**: iroh's alias is `Streamed`, beside its `Whole`. |

`Browser`, `Bridge` and `Dialled` (peer.rs, tests/common) are test code,
and the instrument counts none (it found none for `Dialled`, though
web/src/lib.rs:232 defines one, before the range: I33). The rest have one
definition each.

### Reading the range

Every hunk of product src in join.rs, version.rs, home.rs, iroh.rs's
request on the stream, node.rs's version check, and web/app.js and
web/src/lib.rs was read (`git diff fafd42d5 0a2e6b3a -- …`); the rest's
added lines were searched for a silent default (`unwrap_or`, `.ok()`,
`filter_map`, `??`), and each hit read.

- **One decider** (AGENTS.md 8).
  - One version rule: version.rs, which the phone's page compiles by
    `#[path]` (web/src/lib.rs `older`), and node.rs `answered`. A later
    version is read, and an answer this build cannot read is refused,
    traced (node.rs `answered`'s `Err` arm): never a substitute.
  - One join (join.rs `Host::join`) for `svrngs join` and the home page;
    one maker of a link (`Host::link`); one maker of a new group's relay
    (`group::kept_relay`).
  - **A connection kept to a computer is decided twice** (#177): the
    blob fetch's cache, folder.rs:113 and :565-597, and the members'
    protocol's, which 07ae5d19 added, iroh.rs:142 and :228-244. Each keeps
    one connection per key while it stays open and dials afresh else, with
    its own cap (`ANSWER_WITHIN`; `DIAL_WITHIN` through `dialled`'s held
    transport) and its own owner: folder.rs's a `watch::Sender`, iroh.rs's
    a `std::sync::Mutex` under `#[expect]`.
- **Glassbox** (AGENTS.md 1).
  - **Whether a request rode a kept connection is untraced** (#178,
    iroh.rs:230-236). A dial traces in `dialled`, and a failure in
    `on_stream`, but a request on a kept connection says nothing, and a
    poisoned lock is read as "none kept" (`.lock().ok()`), unsaid.
  - The join's refusals each trace where decided (join.rs `not_let_in`,
    `asked`'s "not believed", `keep_group`'s refusal); the home page's
    `link_of` and `join` trace theirs.
- **Never silently substitute** (AGENTS.md 6). The hits read were
  before the range or moved unchanged (`standings().unwrap_or_default()`,
  each `None` said at warn in node.rs `standings`; redeem.rs:366, whose
  empty name is refused as "a name this build cannot read"), or a cap
  said in its error (iroh.rs `whole`, `dialled`). The seal on disk is
  read as an act that does nothing (arch-seal (b), (c); f8ae1ee8's run).
- **Closed sets** (AGENTS.md 9). `share::At` (#169), `Unsent`, `Entered`,
  `NotJoined` and node.rs `Answered` are enums.
- **Forked code** (AGENTS.md 11). The range forked nothing: 69 files
  carry a `ported from` line at each end (`git grep -lE 'ported from
  (commonwealth-ai|sovereign)' <rev> | wc -l`), and no added line is one.
- **Validate the instrument** (AGENTS.md 7). arch-seal (c) and arch-view
  (b) each ran a build from the row's start beside one from its close in
  one group (f8ae1ee8, 53a35336); arch-stream ten runs in a row
  (9e568a62). Those verdicts are those commits', not judged again here.

### Fixed here

- 6ead9443: tests/common/pair.rs, Alex's computer and Sam's, three copies
  made one (−160 +12 in the three tests, +67 in pair.rs; none counted).
  home 8, lend_origin 5 and peer 4 passed, as at HEAD (pair-before.out,
  pair-after.out).
- 684dade8: iroh/tests.rs `Said`, three sinks made one (−55 +32, none
  counted); the three tests that read their trace passed, as at HEAD,
  and the crate's 95 unit tests (said-before.out, said-after.out).
- 98fe181f: `iroh::Streamed`, one noun one meaning in the node (0
  counted).

#177 and #178 are not fixed here. No line is left under the cap, and
#178's trace is a counted line; #177 changes how a blob fetch dials.

`scripts/check.sh` at 18e1d998, the audit's close: exit 0 in 10 min 59 s,
all fourteen gates passed (principles pinned through app laws;
/tmp/archaudit1/check.out).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 177 | 8, 10 | crates/svrngs-node/src/folder.rs:113 (`conns: watch::Sender<BTreeMap<String, Connection>>`), :565-597 (`fetch`: kept while `close_reason()` is none, else dialled within `ANSWER_WITHIN`); crates/svrngs-node/src/iroh.rs:142 (`conns: std::sync::Mutex<HashMap<…>>`, 07ae5d19), :228-244 (`send`: the same, dialled by `dialled` within `DIAL_WITHIN`) | fixed (groups-tabs (c)): one cache, `IrohTransport::conns`, a `watch::Sender` keyed by key and protocol; `kept` and `let_go`, which `send` and folder.rs `fetch` ask, each with its own cap; blob_tests.rs `a_fetch_rides_the_connection_the_transport_keeps`, red with folder.rs as at c43d2cc7 | One rule, a connection to a computer kept while it stays open and dialled afresh else, written twice, with two caps and two owners. The newer one holds a lock where the older is one owner and messages (clippy.toml's "one owner and messages (RUST_ARCH §3)", here under `#[expect]`). A change to how a kept connection is let go (say, on a member's restart at another address) lands on one. One cache on `IrohTransport`, keyed by key and protocol, both ask. |
| 178 | 1, 6 | crates/svrngs-node/src/iroh.rs:230 (`self.conns.lock().ok()`), :231-237 (kept or dialled) | fixed (groups-tabs (c)): `IrohTransport::kept` traces each request's connection at debug, kept or dialled, with its protocol; the lock is gone with #177's `watch::Sender`, so nothing is read through a poison; iroh/tests.rs `which_connection_a_request_went_on_is_traced` | `send` says nothing when a request rides a kept connection: a trace shows a dial, or a failure, never which connection a request went on. And a poisoned lock is read as no connection kept, unsaid. One debug line naming kept or dialled, and the poison said. |
| 80 | — | scripts/lean_check.py:49 BUDGET, :116 `n >= cap` | not fixed (operator's) | 16144 / 16145 after this audit, and 2952 / 2953 in svrngs-core: −789 and −197 over the range. arch-helper spent 30 of v1ui's 31. |
| 130, 132-145, 150, 152-156, 176 | as recorded | as recorded | not fixed | Not reached by this range's findings; #176 is groups-tabs' (ralph/next/groups). |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, dry-report spans | Still one line early: share.rs:62 for :63, iroh.rs:260 for :261, node.rs:720 for :721, relay.rs:56 for :57. Spans overlap the next symbol (share.rs `helper_of`/`signs_in`, redeem.rs `router`/`ask_route`). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". The JS exact tier (web/) could not judge. |
| I8 | 6 | dry-report scope | Still present: test files reported as production clones (blob_tests.rs, guest/tests.rs, relay_tests.rs; core's removal_tests.rs, register_tests.rs, membership_invite_tests.rs; runtime's tests.rs, tests_write_index.rs). |
| I19 | 6 | dry-report, `tests/` scopes | Still present. The line-window pass stood in, and found the two fixed here. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: the daemon nudge refused, then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's near tier, nested spans | Still present: home.js `running` over `look`, question.js `door` over `judge`. |
| I33 | 6 | SCIP over web/src | Still present: web/'s exact tier says "no SCIP function symbols in scope", and `converge noun Dialled` finds no definition though web/src/lib.rs:232 holds one. |

## REVIEW-audit-groups-auto-1 (2026-10-16, range f4bd2323..c86e0f17)

The previous `REVIEW-audit-` row is REVIEW-audit-arch-auto-1, its hash
f4bd2323 (ralph/next/arch/STATE.md:34). The range ends at c86e0f17, the
commit that made this row. `git rev-list --count f4bd2323..c86e0f17` is
33, none a merge (`--merges`: 0). They are: arch-auto-1's `done`;
groups-v1, groups-economize, groups-tabs and groups-leave; and this
row's own commit. The row read `docs/ARCH_PRINCIPLES.md` and
`target/debug/sovereign-cli`; both paths corrected in b869a523, as
18e1d998 corrected arch-auto-1's.

This audit's own commits before this section:
- 412c3275, a home page driven in a browser made once,
  crates/svrngs/tests/common/home_page.rs;
- b0284f81, iroh.rs's `Kept` renamed `Connections`.

It opens #179, #180 and #181.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
f4bd2323..c86e0f17` (/tmp/groupsaudit1/ledger.raw) is summed by
arch-auto-1's ledger.py, its unit list this range's
(/tmp/groupsaudit1/ledger.out; each commit's unit in ledger.commits). A
commit's unit is the longest unit id its subject names, so 8f637a0e,
groups-v1's one commit ("docs(design): §8 … arch, groups, grapevine"),
counts under `groups`; it touched docs/DESIGN.md alone. The lines are
raw, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-arch-auto-1 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| groups (8f637a0e, groups-v1's DESIGN §8) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| groups-v1 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| groups-economize | 9 | 5 | 22 | **−17** | 19 | 14 | +5 | 0 | 0 | 0 | +4 (+5 −1) |
| groups-tabs | 9 | 127 | 77 | **+50** | 317 | 12 | +305 | 0 | 0 | 0 | +4 (+7 −3) |
| groups-leave | 11 | 305 | 91 | **+214** | 504 | 18 | +486 | 0 | 0 | 0 | +8 (+16 −8) |
| this row's own commit | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 33 | 437 | 190 | **+247** | 840 | 44 | +796 | 0 | 0 | 0 | +16 (+28 −12) |

The scripts' lines are lean_check.py's comment on each cap (+17 −1).
Where the src lines went (`git diff --numstat f4bd2323 c86e0f17 -- crates
web`, tests apart), the largest:
- iroh.rs +64 −34, folder.rs +14 −39: one connection kept per computer
  and protocol, a folder's fetch riding it (#177, #178; groups-tabs (c));
- web/app.js +47 −7: one tab at a time reaches a group, `hold` and
  `elsewhere` (#176; groups-tabs (b));
- svrngs-core register.rs +60 −7, membership.rs +19 −7: `Leave`,
  `RedeemRefusal::Cut`, `RemovalRefusal::Own`, the fold's leave
  (groups-leave (a));
- removal.rs +57 −14, cli.rs +62 −22, join.rs +9 −1, svrngs-runtime
  register.rs +31 −7, journal.rs +7 −13: `POST /v1/leaves`, `svrngs leave`,
  `Standing::Left`, `RemovalSaid::Left`, one read of the kept fold
  (groups-leave (b));
- surface.rs +28 −9, home/home.js +19 −2, question.js +4 −4: `left` and
  `keeps` on `/v1/groups`, "Leave this group", "Groups you left", "Ana
  left on …" (groups-leave (c));
- node.rs −5, svrngs's lib.rs +3 −5, share.rs −3: what only tests
  reached (groups-economize (b)).

The tests grew by crates/svrngs/tests/leave_page.rs (+175),
crates/svrngs-node/tests/browser/tabs.rs (+120), web/tests/tabs.mjs
(+91), tests/folders.rs (+101), core's register_tests.rs (+96) and
removal_tests.rs (+52), and blob_tests.rs and iroh/tests.rs (#177, #178).
apps/ did not change.

What the budget counts, by scripts/lean_check.py over a `git archive` tree
of each end and of each unit's `done` (/tmp/groupsaudit1/lean-<hash>.out;
each tree's own lean_check.py, so the cap is its own):

| at | svrngs-core | repository | its cap | counted net |
|---|---:|---:|---:|---:|
| f4bd2323, range start | 2952 / 2953 | 16144 | 16145 | |
| 02e7e75c, groups-v1 done | 2952 / 2953 | 16144 | 16145 | 0 |
| c43d2cc7, groups-economize done | 2952 / 2953 | 16135 | 16136 | −9 |
| c50dad17, groups-tabs done | 2952 / 2953 | 16157 | 16158 | +22 |
| c6c5957f, groups-leave done | 2992 / 2993 | 16295 | 16296 | +138 |
| c86e0f17, range end | **2992 / 2993** | **16295** | **16296** | **+151** |

The caps moved seven times (`git log f4bd2323..c86e0f17 --
scripts/lean_check.py`):

| commit | the repository's cap | svrngs-core's | for |
|---|---|---|---|
| df996e3b, groups-economize (c) | 16,145 → 16,136 | 2,953 | −9, five items only tests reached |
| eb050a26, groups-tabs (b) | 16,136 → 16,159 | 2,953 | +23, one tab at a time (#176) |
| a4c70ef9, groups-tabs (c) | 16,159 → 16,158 | 2,953 | −1, one connection cache (#177) |
| eb0248c4, groups-leave (a) | 16,158 → 16,195 | 2,953 → 2,987 | +37, +34, the fold's leave and `Leave` |
| f3a5a519, groups-leave (b) | 16,195 → 16,252 | 2,987 | +57, `svrngs leave`, `Standing::Left` |
| 00864058, groups-leave (c) | 16,252 → 16,301 | 2,987 → 2,993 | +49, +6, the page's leave, `RedeemRefusal::Cut` |
| 05d46b23, groups-leave | 16,301 → 16,296 | 2,993 | −5, one read of the kept fold |

The queue's ceilings are +200 over the cap groups-economize left, 16,336,
and +40 over svrngs-core's, 2,993 (ralph/next/groups/PROMPT.addendum.md,
fence 7). The range closes at 16,296, 160 of the 200 spent, and at 2,993,
all 40 of svrngs-core's spent. Over the range: **+151 counted lines, +40
of them svrngs-core's**. No line is left under either cap (16295 / 16296,
2992 / 2993: lean_check fails at `n >= cap`, lean_check.py:132). This
audit's two commits move no counted line (test code, and a rename), so the
caps stand.

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
c95165c5… (`shasum -a 256`): arch-auto-1's, unchanged.

The corpus is c86e0f17's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock, by `git archive` into /tmp/groupsaudit1/prod-c86e0f17 (370
files).
- `code index . --corpus-id prod-c86e0f17`: 4773 chunks in 445 s
  (index.out).
- `refresh`, run in the corpus: rust, 17099 symbols and 99238 references;
  typescript and python "skipped (export failed)" (I4); the daemon nudge
  refused and the in-process rebuild ran (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node, crates/svrngs, crates/svrngs-core,
crates/svrngs-runtime and web (`git diff --numstat f4bd2323 c86e0f17 --
crates apps web scripts bridge`); apps/ not at all. Each was reported at
0.95 and 0.90 (/tmp/groupsaudit1/clean-*.md). Whether a side was added in
the range is `git blame` at c86e0f17 over each side's lines against
f4bd2323 (the 56 sides, cluster-sides.txt; blame-clusters.out). Its
plant: crates/svrngs-core/src/register.rs blamed the same way says 1b19d80f,
groups-leave (a), "IN-RANGE", so the blame tells the range's lines apart.

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 588 | 1 | 1 | 10 | 0 |
| crates/svrngs/ | 66 | 0 | 0 | 0 | — |
| crates/svrngs-core/ | 450 | 4 | 0 | 10 | 0 |
| crates/svrngs-runtime/ | 121 | 1 | 0 | 0 | 0 |
| web/ | 107 | could-not-judge (I4, I33) | 0 | 2 | 0 |

**No cluster has a side the range added**: every one of the 56 sides
blames to commits before f4bd2323. They are arch-auto-1's again: api.rs
`addr` ×2; home.js `running`/`look` (nested, I32); sign_in.rs
`of`/`forget`; synced.rs `kept_files`/`kept_at`; question.js
`door`/`judge`; redeem.rs `router`/`ask_route` and share.rs
`helper_of`/`signs_in` (overlapping spans, I1); choices.rs `load`/`read`;
rail.rs `live_push`/`live_drain`; ring_sync/tests.rs `admit`/`ingest`;
lend.rs `from_mbit`/`bps`; app.js `uses`/`choicesOf`; lend.js
`cookieFor`/`keepJar`; and svrngs-core's three-line accessors and test
bodies alike in shape (removal_tests.rs, register_tests.rs,
membership_invite_tests.rs; I8). The node's exact tier fell from 3 to 1
with 684dade8's trace sink.

**The line-window pass.** arch-auto-1's windows.py over the same file
set at c86e0f17, its four new files added (tabs.rs, pair.rs,
leave_page.rs, web/tests/tabs.mjs): 208 files (windows.files). **141
window hits, 54 spans, 3 new** (windows.spans, windows.verdicts; 51 read
old). All three are test code, each a side in leave_page.rs (e7bd7ad3):

| new span | read |
|---|---|
| leave_page.rs:34-47 == home_group.rs:121-134 | `home_page`, which drives web/tests/home.mjs, written out a second time in one crate's tests. **Made one in 412c3275**, tests/common/home_page.rs, by `#[path]` into the two binaries that drive a page. |
| leave_page.rs:36-46 == node's conformance.rs:520-530; leave_page.rs:37-44 == peer.rs:267-274 | Running a browser half and failing with its stderr, as arch-auto-1 read it in five binaries: fixture, not a decision. No finding. |

After 412c3275, over the working tree (windows-after.spans): 132 hits,
52 spans; leave_page.rs's only span left is its header, the harness
included as home_group.rs includes it.

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges f4bd2323..c86e0f17 -- crates
apps web scripts bridge`, matched against `(struct|enum|trait|type|class)
+[A-Z]\w*` (new-types.out), give `Leave`, `LeaveAsk`, `Kept` and
`Refusal`. `Refusal` is `type Refusal = RemovalRefusal;`, `Leave`'s
`Command` binding (register.rs:212), not a definition. The range removed
none (removed-types.out).

`converge noun <Name> --corpus-id prod-c86e0f17` for each (nouns.out).
The nouns with more than one definition:

| noun | definitions | read |
|---|---|---|
| `Kept` | crates/svrngs-node/src/door/choices.rs:62 (:63, I1), the choices a door keeps by person and lend, before the range; crates/svrngs-node/src/iroh.rs:145 (:146), the connections kept by key and protocol (5d5cb1f6, #177) | One crate, one noun, two meanings, as `Answered` was. **Renamed in b0284f81**: iroh's is `Connections`. |
| `Refusal` | crates/svrngs/src/cli.rs:537 (:538); crates/svrngs-node/src/door/sign_in.rs:132 (:133) | Both before the range, two crates, two concepts; the range only bound `Leave`'s associated type. No finding. |

`Leave` (register.rs:208) and `LeaveAsk` (removal.rs:49) have one
definition each.

### Reading the range

Every hunk of product src in svrngs-core's register.rs and membership.rs,
svrngs-runtime's register.rs and journal.rs, the node's removal.rs,
surface.rs, join.rs, rail.rs, home.js and question.js, cli.rs and
web/app.js was read (`git diff f4bd2323 c86e0f17 -- …`); iroh.rs and
folder.rs were read for a silent default, groups-tabs (c) having fixed
#177 and #178 there. The added lines were searched for one (`unwrap_or`,
`.ok()`, `filter_map`, `??`), and each hit read.

- **A leave never reaches the leaver's other computers** (#179;
  principles 6, 2). removal.rs:210 tells a leave to each computer
  standing that the leave did not cut (`left != cut.contains(k)`), so not
  to the leaver's own others, which `Leave` cut with this one
  (register.rs `Leave::decide`, every key standing as the person). DESIGN
  §6 says "a computer off as it lands hears it from the others' rounds";
  those computers cannot: every other computer's round offers only keys
  standing (ring_sync.rs:359), and its sync route refuses a dialer not
  standing (ring_routes.rs:105-120), as does the leaver's, whose
  `Standing::Left` lists them cut. So Ana's second computer would go on
  showing the group as hers, every lender refusing it, never told why.
  **Read, not reproduced**: groups-leave's tests leave from a person with
  one computer (tests/folders.rs, leave_page.rs).
- **Whether a cut was a leave is decided twice** (#180; principle 8, fence
  3). svrngs-core's fold decides it (membership.rs:584-588, `leaves` by
  the `devices` map) but keeps only the cut's act (`Cut::by`,
  membership.rs:208); svrngs-runtime's `removals_in` decides it again to
  say `RemovalSaid::Left` (register.rs:182-186, `own` by `person_of`).
  They disagree on a keeper's `Remove` of another key of their own: the
  fold's cut by the keeper, `removals_in`'s "left". No command writes it
  (`Removal` refuses the keeper's person, `Keeps`; `Leave` cuts every
  key), so no page shows it at HEAD. surface.rs:139-144's `keeps`, the
  keeper's person this computer's, is a third reading of "stands as the
  keeper's person", by `person_key`.
- **Glassbox** (#181; principles 1, 6). join.rs:71 reads this computer's
  standing in the group to refuse a computer that left, and an `Err` there
  goes on untraced; the issuer then refuses the cut key with the one
  shared refusal (redeem.rs:171-174), said as "the link was refused", not
  as the leave. Every other branch the range added traces where it
  decides: the fold's `leaves` and `alone`, `removes`, `Leave::confirm`,
  `Removal::confirm`, `Standing::Left`, `removals_in`'s `said`, join.rs's
  refusal, `cut`'s `left`, and the page's `hold` and `elsewhere`.
- **Never silently substitute** (AGENTS.md 6). The hits read:
  home.js `asked` says an outcome it does not name as "cannot be changed
  on this computer now"; `g.name ?? 'this group'` and `?? 'A group'` say
  a group with no name as one; iroh.rs's `unwrap_or(Err(Unsent::Asleep))`
  is the dial's cap, said. `Removing::key` is `#[serde(skip)]`, so no
  page is sent a key.
- **One decider** (AGENTS.md 8). One connection cache (`IrohTransport::kept`,
  `let_go`); one read of the kept fold (`RingJournal::folded`, 802519a1);
  the leave's refusals mapped in one `cut` with the removal's
  (removal.rs); one `said` in cli.rs for a removal, hand-on or leave;
  `question.js`'s one wording for a removal and a leave.
- **Closed sets** (AGENTS.md 9). `Standing::Left`, `RemovalSaid::Left`,
  `RedeemRefusal::Cut`, `RemovalRefusal::Own`, `Outcome::Left`/`Own`,
  `NotJoined::Left`, `Said::Left` are enum arms.
- **The core is pure** (fence 3). `Leave` and the fold's leave are
  svrngs-core's; the node commits the `Leave` and carries its verdict.
- **Forked code** (AGENTS.md 11). The range forked nothing: 69 files
  carry a `ported from` line at each end (`git grep -lE 'ported from
  (commonwealth-ai|sovereign)' <rev> | wc -l`), and no added line is one.
- **Validate the instrument** (AGENTS.md 7). groups-tabs (a) reproduced
  #176 in both engines before its fix (0bfc4ad0); groups-run, the queue's
  run, is still to come. Those verdicts are those commits', not judged
  again here.

### Fixed here

- 412c3275: tests/common/home_page.rs, `home_page` two copies made one
  (−41 +13 in the two tests, +27 in home_page.rs; none counted).
  home_group 5 and leave_page 2 passed, in Chromium and WebKit, as at
  HEAD (/tmp/groupsaudit1/home-before.out, home-after.out).
- b0284f81: `iroh::Connections`, one noun one meaning in the node (0
  counted).

#179, #180 and #181 are not fixed here. No line is left under either cap:
#181's trace is a counted line, #180's home is svrngs-core (all 40 of
its lines spent), and #179 changes who a leave is told to, a change
in behaviour wanting its red run first.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 179 | 6, 2 | crates/svrngs-node/src/removal.rs:195 (`left`), :210 (`left != cut.contains(k)`); crates/svrngs-node/src/ring_routes.rs:105-120 (a dialer not standing refused); crates/svrngs-node/src/ring_sync.rs:359 (the round offers keys standing); docs/DESIGN.md:1737 ("hears it from the others' rounds") | fixed (groups-leave-others): reproduced in 041e35f0, both engines, all three cases red; 3d3ad2d6, a leave told to every other computer, the leaver's own too, and `ring_routes::told_cut` answering a cut computer back on the `Remove` naming it; tests/leave_page.rs `a_leave_reaches_the_leavers_other_computer`, its three plants red (demo/runs/leave-others/2026-10-16/) | A leave cuts every key of the person, but the leaving computer tells only the computers it did not cut, and no other computer's round or sync route serves a cut key. A person who leaves from one of two computers leaves the other showing the group, every lender refusing it, never told it left. Red first: two computers of Ana, one leaving; the other's page within 30 s. Then the leaving computer tells its person's other computers too. |
| 180 | 8 | crates/svrngs-core/src/membership.rs:584-588 (`leaves`), :208 (`Cut` keeps `by` only); crates/svrngs-runtime/src/register.rs:182-186 (`own`, `RemovalSaid::Left`); crates/svrngs-node/src/surface.rs:139-144 (`keeps`) | fixed (groups-leave-others (c)): f53e06f6, `Cut::left` decided by the fold's `leaves` and read by `removals_in`; svrngs-runtime register_tests.rs `a_keepers_remove_of_another_key_of_their_own_is_read_as_the_fold_reads_it` | Whether a cut is a person's leave is decided in the fold and again in `removals_in`, which disagree on a keeper's `Remove` of another key of theirs (the fold's keeper's cut; "left" on every page). No command writes one at HEAD. The fold says it once: `Cut` carries whether it was a leave, and `removals_in` reads it. |
| 181 | 1, 6 | crates/svrngs-node/src/join.rs:71 (`if let Ok(Standing::Left(_))`); crates/svrngs-node/src/redeem.rs:171-174 (the shared refusal) | fixed (groups-leave-others (d)): 0c3b464d, join.rs's `Err` traced and the issuer's `RedeemRefusal::Cut` answered 410, said as `NotJoined::Left`; tests/join_by_link.rs `a_computer_the_group_cut_is_told_so_by_the_issuer`, red and its two trace plants (demo/runs/leave-others/2026-10-16/ d-*.out) | This computer's standing, unreadable, is read as "not left" and untraced; the issuer then refuses the cut key as any refused link, so a computer that left is told its link was refused. One debug line for the `Err`. |
| 80 | — | scripts/lean_check.py:65 BUDGET, :132 `n >= cap` | not fixed (operator's) | 16295 / 16296 after this audit, and 2992 / 2993 in svrngs-core: +151 and +40 over the range, 160 of the queue's 200 and all 40 of svrngs-core's spent. groups-keys and groups-run remain. |
| 130, 132-145, 150, 152-156 | as recorded | as recorded | not fixed | Not reached by this range's findings. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, dry-report spans | Still one line early: choices.rs:62 for :63, iroh.rs:145 for :146, register.rs:208 for :209, removal.rs:49 for :50, cli.rs:537 for :538, sign_in.rs:132 for :133. Spans overlap the next symbol (share.rs, redeem.rs). |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". The JS exact tier (web/) could not judge. |
| I8 | 6 | dry-report scope | Still present: test files reported as production clones (core's removal_tests.rs, register_tests.rs, membership_invite_tests.rs, tests_support.rs; ring_sync/tests.rs; runtime's tests.rs, tests_write_index.rs). |
| I19 | 6 | dry-report, `tests/` scopes | Still present. The line-window pass stood in, and found the one fixed here. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present: the daemon nudge refused, then the in-process rebuild and exit 0. |
| I32 | 7 | dry-report's near tier, nested spans | Still present: home.js `running` over `look`, question.js `door` over `judge`. |
| I33 | 6 | SCIP over web/src | Still present: web/'s exact tier says "no SCIP function symbols in scope". |

## groups-run (2026-10-16): leaving a group on one host

Opened by its row (`ralph/next/groups/STATE.md`), not a `REVIEW-audit-` row:
the run is `demo/runs/groups/2026-10-16/` (README.md, "What it found").

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 182 | 6 | crates/svrngs-node/src/origin.rs:240-246 (`dialled`, `Unlent`); crates/svrngs-node/src/door.rs:297-302 (`gone`, the lenders' words for a cut computer) | fixed in 1eab7c0e (gv-leave (a); origin.rs `refused`; leave_page.rs, both engines) | **Run.** After Ana left, her computer answered every lend she had opened 404 `unlent`, "nothing is lent here" (the run's `summary.json`, `after`): the refusal for a lender standing in no group held here, not her leave. Nothing reached her, so the bar held; the words substitute for "you left". `dialled` could say the leave where this computer's register cut its own key in the group the label's lender stands in. |
| 183 | 1 | crates/svrngs-node/src/node.rs:939 (the trace), :371-374 (the `gone` arm, untraced) | fixed in 7f2a8d1d (gv-leave (b); door/main.rs `a_removed_person_is_refused_as_removed_by_pass_and_by_dial` hears it) | **Run.** A lender's acceptor, dialled by a computer it holds cut, traces "node: refused — the dialer stands in no group held here" and then serves it the `gone` router, which tells it its cut in words; that branch traces nothing (the run's `data/sam.log`:12-14, answered 403 `removed`). One debug line in the arm. |

## REVIEW-audit-groups-auto-2 (2026-10-16, range 439319ec..572b3872)

The previous `REVIEW-audit-` row is REVIEW-audit-groups-auto-1, its hash
439319ec (ralph/next/groups/STATE.md, its row). The range ends at
572b3872, the commit that made this row. `git rev-list --count
439319ec..572b3872` is 24, none a merge (`--merges`: 0). They are:
groups-auto-1's `done`; groups-keys, groups-leave-others and groups-run;
and this row's own commit. The row read `docs/ARCH_PRINCIPLES.md` and
`target/debug/sovereign-cli`; both paths corrected in 8ac9ff95, as
b869a523 corrected groups-auto-1's.

This audit's own commits before this section:
- 7e1ced4c, join_by_link.rs's `heard` and `Kept` read from svrngs-node's
  tests/common.

This section also marks #179, #180 and #181 fixed in their rows above,
which groups-leave-others fixed and left "not fixed". It opens #184.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
439319ec..572b3872` (/tmp/groupsaudit2/ledger.raw) is summed by
groups-auto-1's ledger.py, its unit list this range's
(/tmp/groupsaudit2/ledger.out; each commit's unit in ledger.commits).
a14f2dfb, the state commit that answered groups-keys, names
groups-leave-others too and counts there; it touched STATE.md alone. The
lines are raw, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-groups-auto-1 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| groups-keys | 4 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| groups-leave-others | 10 | 79 | 27 | **+52** | 359 | 10 | +349 | 0 | 0 | 0 | +5 (+8 −3) |
| groups-run | 8 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| this row's own commit | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 24 | 79 | 27 | **+52** | 359 | 10 | +349 | 0 | 0 | 0 | +5 (+8 −3) |

groups-keys was a survey and DESIGN alone (64ed421b, 6c185b15, 345db9dd);
groups-run's driver, judge and data are under demo/runs/, which the
ledger and the budget leave apart. Where groups-leave-others's src lines
went (`git diff --numstat 439319ec 572b3872 -- crates web`, tests apart):
- ring_routes.rs +27: `told_cut`, a cut computer back on told the
  `Remove` naming it (#179; (b));
- node.rs +9 −7, removal.rs +9 −6: the cut computer's router the ring
  routes before `gone`; a leave told to every other computer (#179; (b));
- svrngs-core membership.rs +7 −4, svrngs-runtime register.rs +6 −7:
  `Cut::left`, decided by the fold and read by `removals_in` (#180; (c));
- join.rs +11 −3, redeem.rs +10: the unread standing traced, the issuer's
  `RedeemRefusal::Cut` answered 410 and said as the leave (#181; (d)).

The tests grew by crates/svrngs/tests/leave_page.rs (+236),
join_by_link.rs (+86) and svrngs-runtime's register_tests.rs (+27). The
scripts' lines are lean_check.py's comment on each cap. apps/ and web/ did
not change.

What the budget counts, by HEAD's scripts/lean_check.py over a `git
archive` tree of each end and of each unit's `done`
(/tmp/groupsaudit2/lean-<hash>.out):

| at | svrngs-core | repository | counted net |
|---|---:|---:|---:|
| 439319ec, range start | 2992 | 16295 | |
| 1956f9bf, groups-keys done | 2992 | 16295 | 0 |
| b522a8b0, groups-leave-others done | 2992 | 16328 | +33 |
| 62170a32, groups-run done | 2992 | 16328 | 0 |
| 572b3872, range end | **2992** | **16328** | **+33** |

The caps moved twice (`git log 439319ec..572b3872 -- scripts/lean_check.py`):

| commit | the repository's cap | svrngs-core's | for |
|---|---|---|---|
| d0d6ee01, groups-leave-others (b) | 16,296 → 16,316 | 2,993 | +20, a leave told to the leaver's other computers (#179) |
| 6c7e7e5f, groups-leave-others (d) | 16,316 → 16,329 | 2,993 | +13, the unread standing traced and the issuer's cut said (#181) |

The queue's ceilings are 16,336 and 2,993 (the groups-leave-others row).
The range closes at 16,328 / 16,329, 193 of the 200 spent and 7 under the
ceiling unraised, and at 2,992 / 2,993, svrngs-core's 40 all spent, as
the range began. lean_check fails at `n >= cap` (lean_check.py:137), so
no counted line is left under either cap. This audit's commits move no
counted line (a test helper, and this file).

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
c95165c5… (`shasum -a 256`): groups-auto-1's, unchanged.

The corpus is 572b3872's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock, by `git archive` into /tmp/groupsaudit2/prod-572b3872 (371
files).
- `code index . --corpus-id prod-572b3872`: 4789 chunks in 545 s
  (index.out).
- `refresh`, run in the corpus: rust, 17243 symbols and 100100
  references; typescript and python "skipped (export failed)" (I4); the
  daemon nudge refused and the in-process rebuild ran (I29) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node, crates/svrngs-core,
crates/svrngs-runtime, crates/svrngs (its tests alone) and scripts
(`git diff --numstat 439319ec 572b3872 -- crates apps web scripts
bridge`). Each was reported at 0.95 and 0.90 (/tmp/groupsaudit2/clean-*.md).
Whether a side was added in the range is `git blame` at 572b3872 over
each side's lines against 439319ec (the 52 sides, cluster-sides.txt;
blame-clusters.out). Its plant: ring_routes.rs:120-140 blamed the same
way says 3d3ad2d6, groups-leave-others (b), "IN-RANGE", so the blame
tells the range's lines apart.

| scope | locations | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---:|---:|---:|---:|---|
| crates/svrngs-node/ | 589 | 1 | 1 | 10 | 0 |
| crates/svrngs-core/ | 450 | 4 | 0 | 10 | 0 |
| crates/svrngs-runtime/ | 122 | 1 | 0 | 0 | 0 |
| crates/svrngs/ | 66 | 0 | 0 | 0 | — |
| scripts/ | 15 | could-not-judge (I4) | 0 | 0 | 0 |

**No cluster has a side the range added**: every one of the 52 sides
blames to commits before 439319ec. They are groups-auto-1's, less web/,
which the range did not touch.

**The line-window pass.** groups-auto-1's windows.py over its file set at
572b3872, the 209 files that still stand (windows.files). **133 window
hits, 53 spans, 1 new** (windows.spans, windows.verdicts; 52 read old):

| new span | read |
|---|---|
| svrngs-node tests/common/mod.rs:142-149 == crates/svrngs/tests/join_by_link.rs:954-961 | 0c3b464d (groups-leave-others (d)) wrote out the node's tests' `heard` and its `Said` sink a second time, the sink as `Kept`, to see #181's traces. **Made one in 7e1ced4c**: join_by_link.rs includes the node's tests/common by `#[path]`, as the command's browser tests include the node's harness. |

After 7e1ced4c, over the working tree (windows-after.spans): 132 hits, 52
spans; join_by_link.rs's only span left is its header beside new.rs's,
old.

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 439319ec..572b3872 -- crates
apps web scripts bridge`, matched against `(struct|enum|trait|type|class)
+[A-Z]\w*` (new-types.out), give one: `Kept`, the test sink above. The
range removed none (removed-types.out).

`converge noun Kept --corpus-id prod-572b3872` (nouns.out): one
first-party production definition, crates/svrngs-node/src/door/choices.rs:62
(:63, I1); the noun leaves `tests` out (I8 is its dry-report twin), so
the range's test `Kept` was not counted. It is gone with 7e1ced4c, so
`Kept` names one thing in the repository again. No noun has more than one
definition.

### Reading the range

Every hunk of product src the range changed was read (`git diff
439319ec 572b3872 -- crates/svrngs-core/src crates/svrngs-node/src
crates/svrngs-runtime/src/register.rs`). The added lines were searched for
a silent default (`unwrap_or`, `.ok()`, `filter_map`, `??`), and each
hit read: ring_routes.rs:139's `lacks.ok()?` (#184) and leave_page.rs's
`unwrap_or_default` over a test's JSON.

- **A cut computer is told only the signer's acts** (#184; principles 6,
  2). `told_cut` (ring_routes.rs:136-151) answers a computer that does not
  stand the first `Remove` naming it that its digest lacks, with the
  acts before it of that `Remove`'s signer alone (:142). The asker's fold
  counts the `Remove` only if it reads the signer as the keeper or as the
  person cut (membership.rs:551 `keeps`, :590 `leaves`). A keeper made by
  a `HandOn` another computer signed (:616) is not the keeper in a copy
  that lacks that `HandOn`. So Ana's second computer, off while Sam hands
  on to Bo and Bo removes Ana, is told Bo's `Remove`, reads it as an ask
  that cuts nobody, and, asking again, lacks no `Remove` naming it: it is
  refused 403 (:171) from then on, shows the group as hers, and is never
  told. groups-leave-others's test leaves by the person's own `Remove`,
  which the signer's own acts carry, and removes by the founder, whose
  keeping is in the seed. **Read, not reproduced.** And a journal that
  cannot answer what the asker lacks (:139, `lacks.ok()?`) is read as
  nothing to tell, untraced, then refused as any asker not standing.
- **Glassbox** (principle 1). Every other branch the range added traces
  where it decides: `told_cut`'s answer, the fold's one Remove trace with
  its `kept`, join.rs's unread standing, redeem.rs's `Cut`, `removals_in`'s
  `said`. join.rs:160 maps a `Cut` with no group named to `IssuerAway`;
  only the redemption, which names the group, can answer one
  (svrngs-core register.rs:78; join.rs:66 is the ask), and `not_let_in`
  traces the error first (:147). #183's untraced `gone` arm stands
  (node.rs:363 now).
- **Never silently substitute** (AGENTS.md 6). `Cut::left` is
  `#[serde(default)]` (membership.rs:216); the fold is held in memory and
  rebuilt from the journal (svrngs-runtime journal.rs:323-333), never read
  back from disk, so no stored cut reads a leave as a removal.
- **One decider** (AGENTS.md 8). Whether a cut was a leave is the fold's
  alone (f53e06f6); one `heard` in the tests (7e1ced4c).
- **Closed sets** (AGENTS.md 9). `RedeemError::Cut` is an enum arm,
  answered by one status (410, redeem.rs:169, :382).
- **The core is pure** (fence 3). The fold decides the leave; the node
  carries `Cut::left` and the runtime reads it.
- **Forked code** (AGENTS.md 11). The range forked nothing: 69 files carry
  a `ported from` line at each end (`git grep -lE 'ported from
  (commonwealth-ai|sovereign)' <rev> | wc -l`), and no added line is one.
- **Validate the instrument** (AGENTS.md 7). groups-leave-others (a)
  reproduced #179 in both engines before its fix (041e35f0); groups-run
  predicted before its data (f407b7a4 before 1b6e4c85) and planted each
  bar red (fd0a5179). Those verdicts are those commits', not judged again
  here.
- **Write for the next reader** (AGENTS.md 3). groups-leave-others fixed
  #179-#181 and left their rows "not fixed"; corrected above, here.

### Fixed here

- 7e1ced4c: join_by_link.rs's `heard` and `Kept`, a copy of svrngs-node's
  tests/common `heard` and `Said`, read from there (−33 +5; none
  counted). join_by_link 11 passed before and after, the cut's two traces
  asserted (/tmp/groupsaudit2/join-before.out, join-after.out).
- #179, #180, #181 marked fixed, with their commits and tests.

#184 is not fixed here: it changes what a cut computer is told, a change
in behaviour wanting its red run first, and no counted line is left under
the repository's cap.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 184 | 6, 2, 1 | crates/svrngs-node/src/ring_routes.rs:142 (`told_cut` keeps the signer's acts alone), :139 (`lacks.ok()?`, untraced); crates/svrngs-core/src/membership.rs:551 (`keeps`), :616 (a `HandOn` moves the keeper) | fixed in 8098efbf (gv-leave (c): reproduced, leave_page.rs `a_cut_by_a_handed_on_keeper_reaches_a_computer_off_as_it_landed`; the hand-ons told with the cut, the journal's `Err` traced; a keeper's own `Admit` the asker lacks not told, not reproduced) | A computer off while its person is removed by a keeper made by a `HandOn` it never heard is told the `Remove` alone, reads it as an ask, and is refused from then on, never told, the group shown as its person's. Red first: Sam hands on to Bo while Ana's second computer is off, Bo removes Ana, the second comes back; its page within 90 s. Then `told_cut` tells the acts the fold reads the `Remove` by (the keeper's `HandOn` chain, the signer's `Admit`), or says plainly that this computer cannot tell. And the journal's `Err` traced. |
| 80 | — | scripts/lean_check.py:70 BUDGET, :137 `n >= cap` | not fixed (operator's) | 16328 / 16329 after this audit, and 2992 / 2993 in svrngs-core: +33 and 0 over the range. 193 of the queue's 200 spent, 7 left under the ceiling unraised; all 40 of svrngs-core's spent. |
| 182, 183 | as recorded | as recorded | not fixed | From groups-run; this range's code is theirs, not touched since. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, dry-report spans | Still one line early: choices.rs:62 for :63. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". scripts/'s exact tier could not judge. |
| I8 | 6 | dry-report scope | Still present: core's removal_tests.rs, register_tests.rs, membership_invite_tests.rs, tests_support.rs, ring_sync/tests.rs and runtime's tests.rs reported as production clones. `converge noun` leaves `tests` out, so a test's noun is not seen. |
| I19 | 6 | dry-report, `tests/` scopes | Still present. The line-window pass stood in, and found the one fixed here. |
| I29 | 6 | `refresh` over a corpus `code index` made | Still present. |
| I32 | 7 | dry-report's near tier, nested spans | Still present: home.js `running` over `look`, question.js `door` over `judge`. |

## REVIEW-audit-grapevine-auto-1 (2026-10-16, range 01b1cf97..95b24ac8)

The previous `REVIEW-audit-` row is REVIEW-audit-groups-auto-2, its hash
01b1cf97 (ralph/next/groups/STATE.md, its row). The range ends at
95b24ac8, the commit that made this row. `git rev-list --count
01b1cf97..95b24ac8` is 50, none a merge (`--merges`: 0). They are:
groups-auto-2's `done`; gv-leave, gv-fold, gv-via, gv-page and gv-run;
the mid-queue audit's row and its removal (5aec7726, a163f48e); and this
row's own commit. The row read `docs/ARCH_PRINCIPLES.md` and
`target/debug/sovereign-cli`; both paths corrected in e040ffed, as
8ac9ff95 corrected groups-auto-2's.

This audit's own commits before this section:
- bc6098ed, relay.mjs `heldBy`'s unread people traced.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
01b1cf97..95b24ac8` (/tmp/gvaudit/ledger.raw) is summed by groups-auto-2's
ledger.py, its unit list this range's and `apps/*/vendor/` its own column
(/tmp/gvaudit/ledger.out; each commit's unit in ledger.commits). A commit
naming no unit counts toward the next that does: dbbf4ab4, the
`on_behalf_of` finding, counts to gv-via, and a163f48e, the queue file, to
gv-run. The lines are raw, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | apps vendor | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-groups-auto-2 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gv-leave | 9 | 69 | 34 | **+35** | 137 | 3 | +134 | 0 | 0 | 0 | 0 | +5 (+10 −5) |
| gv-fold | 8 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | +897 (+911 −14) | +503 (+505 −2) | +2391 | 0 |
| gv-via | 8 | 0 | 0 | **0** | 59 | 0 | +59 | 0 | +44 (+76 −32) | +107 (+112 −5) | 0 | 0 |
| gv-page | 6 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | +810 (+816 −6) | +495 (+502 −7) | 0 | 0 |
| REVIEW-audit-grapevine-auto-1 (its rows) | 2 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| gv-run | 16 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | +46 (+70 −24) | +40 (+42 −2) | 0 | 0 |
| total | 50 | 69 | 34 | **+35** | 196 | 3 | +193 | 0 | +1797 (+1873 −76) | +1145 (+1161 −16) | +2391 | +5 (+10 −5) |

Where the src lines went (`git diff --numstat 01b1cf97 95b24ac8 -- crates
web bridge scripts`, tests apart), gv-leave's alone, as the CHARTER allows:
- origin.rs +34 −8: `refused`, the leaver's old lend address saying "You
  left Trip on 16 Oct" (#182; (a));
- node.rs +15 −14: the `gone` arm's trace, rustfmt reflowing the closure
  around it (#183; (b));
- ring_routes.rs +16 −9, removal.rs +3 −2: `told_cut` telling the
  hand-ons with the cut, and its unread journal traced at warn (#184; (c));
- surface.rs +1 −1: `seed_of` made `pub(crate)` for (a);
- Cargo.toml +5 −3: chrono moved from dev-dependencies to dependencies,
  with `clock`, for (a).

svrngs-node's tests grew by tests/browser/carried.rs (+59, gv-via (b): the
home page's carried line under svrngs, a check's own test, which the
CHARTER's fence allows) and door/main.rs (+7 −1, #183's trace);
crates/svrngs/tests/leave_page.rs by +130 −2 (#182, #184). The scripts'
lines are lean_check.py's comment on each cap. web/, bridge/ and
svrngs-core did not change.

apps/: grapevine is new, 1,756 lines of source (grapevine.js 581,
relay.mjs 367, server.mjs 330, canary.mjs 125, scene.mjs 121, store.mjs
103, seal.mjs 70, kinds 59) and 1,066 of tests; Events' and Expenses'
folds, pages and tests grew by gv-via's `via`. tweetnacl 1.0.3 is 2,391
lines vendored whole (vendor/, pinned). demo/runs/grapevine/'s driver
(run.py, people.mjs, judge.py) and demo/grapevine.md are what ran, which
the ledger and the budget leave apart.

What the budget counts, by HEAD's scripts/lean_check.py over a `git
archive` tree of each end and of each unit's `done`
(/tmp/gvaudit/lean-<hash>.out). apps/ is apart, so only gv-leave moves it:

| at | svrngs-core | repository | counted net |
|---|---:|---:|---:|
| 01b1cf97, range start | 2992 | 16328 | |
| a68d7fc7, gv-leave done | 2992 | 16350 | +22 |
| 15b507f0, gv-fold done | 2992 | 16350 | 0 |
| c247f415, gv-via done | 2992 | 16350 | 0 |
| 40544fd5, gv-page done | 2992 | 16350 | 0 |
| c70e0ba3, gv-run done | 2992 | 16350 | 0 |
| 95b24ac8, range end | **2992** | **16350** | **+22** |

The caps moved three times, gv-leave's alone (`git log 01b1cf97..95b24ac8
-- scripts/lean_check.py`):

| commit | the repository's cap | svrngs-core's | for |
|---|---|---|---|
| a5b386fc, gv-leave (a) | 16,329 → 16,347 | 2,993 | +18, the leaver told she left (#182) |
| 6557eb42, gv-leave (b) | 16,347 → 16,348 | 2,993 | +1, the `gone` arm's trace (#183) |
| 05bc8e2e, gv-leave (c) | 16,348 → 16,351 | 2,993 | +3, the hand-ons told with a cut (#184) |

The CHARTER's ceiling was +30 over groups' close, at most 16,358, and
svrngs-core unchanged. The range closes at 16,350 / 16,351, 22 of the 30
spent and 8 under the ceiling unraised, and at 2,992 / 2,993, svrngs-core
unchanged. lean_check fails at `n >= cap` (lean_check.py:142), so no
counted line is left under either cap. This audit's commits move no
counted line (apps/, and this file).

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
7230170e… (`shasum -a 256`), rebuilt since groups-auto-2's c95165c5….

The corpus is 95b24ac8's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock, every `vendor/` left out, by `git archive` into
/tmp/gvaudit/gv-95b24ac8 (345 files).
- `code index . --corpus-id gv-95b24ac8`: 3855 chunks in 493 s
  (index.out).
- `refresh`, run in the corpus: rust, 17324 symbols and 100636 edges;
  typescript and python "skipped (export failed)" (I4) (refresh.out).
- A first corpus of svrngs-node, apps/ and scripts/ alone could not be
  refreshed: with svrngs-core and svrngs-runtime absent, rust's export
  failed too, and the refresh named the directory, not the corpus. It was
  not used.

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-node, crates/svrngs (its tests alone),
scripts, apps/grapevine, apps/events and apps/expenses (`git diff
--numstat 01b1cf97 95b24ac8 -- crates apps web scripts bridge`). Each was
reported at 0.95 and 0.90, and apps/ whole, so a clone across two apps is
seen (/tmp/gvaudit/clean-*.md). Whether a side was added in the range is
`git blame` at 95b24ac8 over each side's lines against 01b1cf97 (the 94
sides, cluster-sides.txt; blame-clusters.out). Its plant: origin.rs:252-262
blamed the same way says 1eab7c0e, gv-leave (a), and api.rs's old `addr`
pair says before (plant.verdicts), so the blame tells the range's lines
apart.

| scope | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---|---:|---:|---|
| crates/svrngs-node/ | 1 | 1 | 10 | 0 |
| crates/svrngs/ | 0 | 0 | 0 | — |
| apps/ (grapevine, events, expenses within it) | could-not-judge (I4) | 23 | 33 | 19 sides, 15 clusters |
| scripts/ | could-not-judge (I4) | 0 | 0 | 0 |

The clusters with a side the range added:

| cosine | range side | other side | read |
|---|---|---|---|
| 1.000 (×10 clusters) | apps/grapevine/store.mjs:23-103 (`record`, `call`, `groups`, `people`, `log`, `append`, `file`, `group`) | apps/events/store.mjs, the same lines | The copy, byte for byte (6d4ee6ff; `cmp` exits 0). GRAPEVINE_DEMO §2 chose it: "Events' store, copied unchanged; the second copy becomes a shared module if a third app needs it", and README.md:84 says so. Held to that, not a finding. Its header still reads "Where Events' acts are kept … (events.js)" (:1-4), left so the two stay identical. |
| 1.000 (×2) | apps/grapevine/laws.test.mjs:31-42 (`constructor`, `now`) | apps/events/laws.test.mjs:58-69, apps/expenses/laws.test.mjs:48-59 | #150's `elsewhere`, now in a third app. See #150 below. |
| 0.967 | relay.mjs:346-367 `loop` | relay.mjs:349-364 `turn` | Not a clone: `turn` is `loop`'s inner function (I32). |
| 0.907 | grapevine.test.mjs:19-28 `pins` | grapevine.test.mjs:25-36 `unpinned` | Not a clone: two neighbours, padded to overlap (I32). |
| 0.902 | apps/grapevine/page.test.mjs:52-63 `as` | apps/events/page.test.mjs:49-60 `as` | Not a clone: Events' sets the person's header, grapevine's opens `?as=`; one `goto` and one `innerText` each. |

Every other cluster's sides blame to commits before 01b1cf97.

**The line-window pass.** groups-auto-2's windows.py over its file set,
with grapevine's files and lean_check.py added, at 95b24ac8 (226 files,
windows.files). **222 window hits, 55 spans, 3 new** (windows.spans;
windows.verdicts, by groups-auto-2's blame against 01b1cf97; 52 old):

| new span | read |
|---|---|
| apps/events/store.mjs:17-100 == apps/grapevine/store.mjs:17-100 | The copy above. |
| apps/events/laws.test.mjs:53-81 == apps/grapevine/laws.test.mjs:26-54 | `elsewhere`, #150. |
| apps/expenses/laws.test.mjs:43-71 == apps/grapevine/laws.test.mjs:26-54 | The same. |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 01b1cf97..95b24ac8 -- crates
apps web scripts bridge` (vendor/ apart), matched against
`(struct|enum|trait|type|class) +[A-Z]\w*` (new-types.out), give one:
`Refused`, the JS class at apps/grapevine/relay.mjs:49. The range removed
none (removed-types.out).

`converge noun Refused --corpus-id gv-95b24ac8` (nouns.out): one
first-party production definition, crates/svrngs-core/src/command.rs:66
(`enum Refused<R>`, :67, I1). It does not see grapevine's: rust alone was
exported (I4). Its plant: `converge noun Said` lists cli.rs:1073 and
choices.rs:42 (:1074, :43) and not Events' `class Said`
(apps/events/server.mjs:56), which exists. By `git grep`, **`Refused`
has two definitions**: svrngs-core's, why a commit wrote nothing; and
grapevine's, a refusal said to the person in the page's words, which
Events, the app grapevine takes its shape from, names `Said`. #186.

### Reading the range

Every hunk of gv-leave's src was read (`git diff 01b1cf97 95b24ac8 --
crates/svrngs-node/src`), and grapevine's source whole for silent defaults
(`catch`, `?? `, `|| []`), each hit read.

- **An unread group's people, untraced** (principles 1, 6). relay.mjs
  `heldBy` checked svrngs's `person` against the group's people, and where
  those could not be read (`.catch(() => null)`, :109) it kept svrngs's
  answer with nothing traced. **Traced in bc6098ed**, red first.
- **The day said in two homes** (principle 8). gv-leave (a) says the day a
  person left as `" on %-d %b"` in local time (origin.rs:268-270), citing
  cli.rs `namesakes_said` (:1354-1357) as the form, which spells it again;
  question.js:79 cites the same. #185.
- **Never silently substitute** (AGENTS.md 6). origin.rs `refused` reads
  groups that cannot be listed and an unreadable register as no cut
  (:258-259); `Host::contact` has just asked `standing_anywhere`, which
  warns on both (svrngs-runtime register.rs:298-320), as its comment says.
  `told_cut`'s unread journal now warns (ring_routes.rs:144-145, #184's
  untraced `ok()?`). grapevine's store failures, writes and loop turns each
  trace and say (relay.mjs:153, :184, :249, :359, at HEAD); an
  unopenable seal is a gap the fold names.
- **Glassbox** (principle 1). #183's `gone` arm traces (node.rs:374).
- **Sealed means sealed.** No trace bc6098ed adds holds a message: the
  group's id and the store's error. relay.test.mjs's "no plaintext or
  secret in any trace" law stays green on it.
- **Forked code** (AGENTS.md 11). The range forked nothing from
  commonwealth-ai: 69 files carry a `ported from` line at each end (`git
  grep -lE 'ported from (commonwealth-ai|sovereign)' <rev> | wc -l`).
  tweetnacl is vendored whole and pinned (grapevine.test.mjs, red on a
  changed byte), not forked.
- **Zero svrngs lines** (the CHARTER). Outside gv-leave the range's crates/
  lines are gv-via's carried.rs, a check's own test. gv-leave spent 22 of
  its 30, svrngs-core unchanged.
- **Validate the instrument** (AGENTS.md 7). Each row's laws were planted
  red as they landed, and gv-run predicted before its data (69d3171e before
  485427fa). Those verdicts are those commits', not judged again here.

### Fixed here

- bc6098ed: relay.mjs `heldBy` traces a group whose people cannot be read.
  relay.test.mjs's new test red on HEAD's line (`[{"what":"me",
  "group":"page","standing":true}]`, no trace of the error), green after;
  relay.test.mjs 3 pass, who.test.mjs 8 pass.

Not fixed here: #150 and #186 are moves across apps, each test re-proved
on its plants; #185 is svrngs lines, which this queue keeps at zero
outside gv-leave, with no counted line left under the cap.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 185 | 8 | crates/svrngs-node/src/origin.rs:268-270 (`" on %-d %b"`, local); crates/svrngs/src/cli.rs:1354-1357 (`namesakes_said`, `", since %-d %b"`) | not fixed (svrngs lines; no counted line under the cap) | How a day is said to a person is spelled twice, in two crates, the second citing the first as its form. A change to one (a year when it is not this one, say) leaves the other. One function in svrngs-node, `day_said(unix) -> Option<String>`, which cli.rs reads; question.js:79's citation follows it. |
| 186 | 8 | apps/grapevine/relay.mjs:49 (`class Refused`); apps/events/server.mjs:56 (`class Said`); crates/svrngs-core/src/command.rs:67 (`enum Refused<R>`) | not fixed (a rename across apps, with #150) | grapevine's refusal said to the person is Events' `Said` by another name, and `Refused` already names svrngs-core's commit refusal. Where #150's shared module lands, one class and one name for both apps. |
| 150 | 8 | as recorded, and apps/grapevine/laws.test.mjs:26-54 (`elsewhere`) | not fixed: a move across three apps | **A third app with laws made it three**, as the row said: grapevine's `elsewhere` is Events' and Expenses' (line-window spans above; dry-report 1.000 on `constructor` and `now`). With grapevine's store.mjs (GRAPEVINE_DEMO §2: "a shared module if a third app needs it") and #186, one `apps/` module the apps' tests and servers import. |
| 80 | — | scripts/lean_check.py:75 BUDGET, :142 `n >= cap` | not fixed (operator's) | 16350 / 16351 after this audit, and 2992 / 2993 in svrngs-core: +22 and 0 over the range. gv-leave spent 22 of its 30; no counted line left under either cap. |
| 182, 183, 184 | as recorded | as recorded | fixed (gv-leave; d9052f4a) | Not reached by this range's findings. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, dry-report spans | Still one line early: command.rs:66 for :67, choices.rs:42 for :43, cli.rs:1073 for :1074. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". apps/'s and scripts/'s exact tiers could not judge, and `converge noun` sees no JS class (`Refused`, `Said` above). |
| I8 | 6 | dry-report scope | Still present: svrngs-node's ring_sync/tests.rs reported as production; `*.test.mjs` files too (grapevine.test.mjs `pins`). |
| I32 | 7 | dry-report's near tier, nested spans | Still present: home.js `running` over `look`, question.js `door` over `judge`, and now relay.mjs `loop` over `turn`. |
| I33 | 6 | `refresh` over a corpus whose Cargo workspace names absent crates | New: rust's export failed, the refresh used the directory's name, not `--corpus-id`'s, and said only "export produced 0 symbols". A corpus is the whole workspace or none. |

## northstar (2026-10-16, at 1edc5fbd): ENG_NORTH_STAR.md against the code

Opened by the operator ("Has svrngs maintained integrity with its eng north
star?"), not a `REVIEW-audit-` row. Six readings of the code at 1edc5fbd,
one per clause of `ENG_NORTH_STAR.md` and one of the process that keeps it,
each cited below; `scripts/check.sh` run once at 1edc5fbd, exit 1 (#193).
Two holes reproduced outside the repo, against svrngs-core at 1edc5fbd with
its `test-support` fixtures, before any fix:

- #187: one bound signer's act at seq 10,000,000 beside its seq 0 —
  `admitted()` returned 9,999,999 gaps in 0.33 s at 1.70 GB maximum
  resident (`/usr/bin/time -l`); at 1,000,000, 999,999 gaps in 32 ms.
- #188: a link minted at `NOW` with `until` `NOW + 86,400` and redeemed at
  `NOW + 60` through `commit` against `Memory`: joined with nothing held
  ahead; refused after one act of bo's timed `NOW + 2 days`, validly
  signed; refused after one line nobody signed, timed the same.

What holds is said in the operator's reply, not here: the core no_std and
its eight `Command`s decided and confirmed under the write lock, one
appending writer with `sync_data`, admission a function of the set over
720 orderings, every act-authorized effect written first.

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 187 | 10 | crates/svrngs-core/src/admit.rs:676-690 (`for n in 0..=highest`), :497-517 (a parent not held is not judged) | fixed in 2fd52865 (tests.rs `an_act_far_past_its_chain_is_one_hole`, red 999,999 gaps) | **Reproduced.** The hole audit pushes one `SequenceHole` per missing seq from zero, so its cost is the seq a line claims, not the lines held. One act a member signs at a seq far above its chain — its parent not held, so not judged — replicates to every copy (ingest takes it, the digest offers it) and every `admit` on it allocates ~170 bytes a missing seq: a seq near `u64::MAX` never returns. Removing the signer does not help: the audit counts "removed alike". The log is append-only, so nothing takes the line back. |
| 188 | 8, 6 | crates/svrngs-core/src/command.rs:229-234 (`after` over every line held); crates/svrngs-node/src/invite.rs:132 (`until: now + LINK_LIFE_SECS`); crates/svrngs-core/src/membership.rs:717 (`Lapsed`) | fixed in 82aac96e (register_tests.rs `a_link_redeems_after_*`, both red; `AHEAD_SECS`) | **Reproduced.** `commit` times each act after every line held, verified or not and however far ahead, so one act timed ahead — a computer whose clock runs fast, or a line no one signed pushed by any member — carries every later act past it. A link lapses by the wall clock and a day, the `Admit` redeeming it is timed past that, and every join by link in the group is refused `Lapsed` until real time passes the line's: forever, for a line timed in 2100. A computer writes its `Relay` act as it starts, so a wrong clock alone does it. Whether a line is authentic is decided twice: by admission, and not at all by the timing. |
| 189 | 8 | crates/svrngs-runtime/src/journal.rs:184 (`self.ids.insert(id)`, verified or not), :411 (`ingest_all` skips any id held) | fixed in 026451a9 (runtime tests.rs `a_forged_copy_first_does_not_shadow_the_act`, red) | **Read, not run.** The journal's held set is keyed by the derived id, which the signature is not part of, and holds ids whose line failed verification. A copy of an act with one signature byte flipped, ingested first, makes every later copy of the genuine act "already held": dropped every round, its digest never naming it, the copy's admission reporting `BadSignature` and a hole where its peers count the act. Arrival order decides the state. A member cut by the keeper who gets such a copy of their `Remove` to a computer first keeps standing there. |
| 190 | 2, 10 | crates/svrngs-node/src/removal.rs:218-233 (`tell`, "traced, never retried"); crates/svrngs-node/src/node.rs:553 (the round passes over a group whose `standing` is not `Members`); docs/DESIGN.md:1752 | fixed in 1716275e, 5753b234, da90742e (leave_page.rs `a_leave_reaches_a_computer_off_as_it_landed`, red after its first run passed by the first telling's dial; `tell_leaves`) | **Read, not run.** A leave is committed and told once to each computer standing; from then on the leaver's computer stands in the group for nobody, so its round never offers it and its acceptor refuses the others. A computer off as the leave lands hears it only from a computer that was told it. In a group of two, the other asleep, nobody ever does: its copy keeps Ana standing and its round is refused every minute. DESIGN:1752's "hears it from the others' rounds" holds only when another was on. The log is written ahead; nothing drives the effect from it after. |
| 191 | 6 | crates/svrngs-runtime/src/oplog.rs:161-181 (`append_all`, no look at the file's last byte); crates/svrngs-runtime/src/tests.rs:620 | fixed in 4a22ab43 (runtime tests.rs `an_act_after_a_torn_line_is_whole`, red) | **Read, not run.** A torn last line — power lost mid-write, a full disk — is read as a `MalformedLine` (tested), and the next append is glued onto it: the act `commit` returned `Recorded`, its `sync_data` done, is unreadable from then on, never offered, and its seq is reused by the next commit, forking the signer's chain. The test tears the tail and never appends after. |
| 192 | 3 | crates/svrngs-node/src/home/index.html:26; crates/svrngs-node/tests/browser/main.rs:2490 (asserts it) | fixed in ecfbb870 (the pinning test turned, red both engines) | The home page offers "Leave this group" (groups-leave (c)) and says, under "Done in a terminal", "You cannot leave a group yet: svrngs has no way to do it." A test pins the stale sentence. Shipped in V1 (de8923f7). |
| 193 | 7 | web/tests/home.mjs:84 (`drawn` read once `#home h2` is); crates/svrngs-node/tests/browser/main.rs:2706 (the load guard over the timing alone) | fixed in 76dda350 (red on a plant, /v1/lends held 1.5 s; green with and without it) | `the_door_draws_who_is_on_now` failed in both engines in check.sh's `tests` step at 1edc5fbd and passed in its `browser` step; 3 more runs of the binary passed at load 47-56: 1 of 5. Its run shows every cell "asking…" while `li.off` was drawn 107 ms in: `drawn` is read as soon as the heading is, before the door is judged from `/v1/lends`, and on a loaded host the read wins. |
| 194 | 5 | scripts/vocab_check.py:34-35 (the first `#[cfg(test)]` ends the file); crates/svrngs-node/src/guest.rs:24-25; scripts/vocab_check.py:47-48 (the files read) | fixed in 4836ea12 (three plants in the self-test, red; web/src's "relay-hop" renamed) | The gate stops reading a file at its first `#[cfg(test)]`, so guest.rs's 93 code lines after `#[cfg(test)] mod tests;` (`RelayConfig` and the endpoint builder) are never read: a noun planted there passes. It reads no `web/src/*.rs` (the page's wasm crate, infra by DESIGN §5) and of the page's scripts only app.js. Its self-test plants in neither. |
| 195 | 5 | scripts/lean_check.py:107 (`Arc<\s*Mutex<\|Rc<\s*RefCell<`) | fixed in 10b23294 (a planted `Arc<std::sync::Mutex<…>>` passed, then refused; test files apart) | The RUST_ARCH regex matches only an unqualified `Arc<Mutex<`: `Arc<std::sync::Mutex<…>>`, `Arc<RwLock<…>>` and `Arc<tokio::sync::Mutex<…>>` pass it. clippy.toml's ban holds the std and tokio ones by path; the regex claims more than it reads. |
| 196 | 6, 3 | docs/DESIGN.md:1478-1479; ARCHITECTURE.md:691-693 | fixed in 2c6a6f9c | "the canaries of §8, Expenses and Tap, run against the infra in `scripts/check.sh`" and "Tap is a canary": apps/tap holds only nouns.txt, and canaries.rs runs Expenses and Events. A canary that never ran is described as running. ARCHITECTURE.md also says apps/jellyfin holds only its nouns. |
| 197 | 8 | scripts/lean_check.py:13-14 (the docstring: 3,150 and 16,964), :75 (`BUDGET`: 2,993 and 16,351); docs/DESIGN.md:1482-1483 (3,150 and 16,870) | fixed in e18ec26a (BUDGET the one home) | One cap, three homes, three answers. DESIGN's stopped at events-economize (31b6a289); the docstring at 3a3f3fce. |
| 198 | 3 | ENG_NORTH_STAR.md; docs/DESIGN.md:1468 (the invariants' State column) | fixed in bacbee2d (the operator's north star edited to name the exemption; theirs to keep) | The north star names process managers, and none exists since 9db9dd2d deleted process.rs: what follows an act is code order (removal.rs:218). It says "every piece of state is a projection folded from it" with no exemption, where DESIGN §5 keeps outside the record a person's yes and stop to an app (door/choices.rs:3-5), what a computer lends, folder-sync state and app blobs. DESIGN's "every write goes through `commit`, and no effect is left" is not so: `ingest_all` writes a peer's lines (journal.rs:397), the cut is told after it (removal.rs:218), the blob sweep deletes (folder.rs:160-176). |
| 199 | 3 | ARCHITECTURE.md:16, :131-141, :120, :440-442 | fixed in 7752e646 | Stale: "joining by that link is next (U2)"; `HttpBridge`, `pump`, `preferred_bridge_port` and `base_url`, cut in a27e688b; `RingJournal::standing`, now `folded`; `Node::lends` answered through "the dialler's bridge". Leaving a group (`svrngs leave`, `POST /v1/leaves`) is not on the map. |
| 200 | 11 | crates/svrngs-core/src/lib.rs:150-177 (`LOCAL_ONLY_NAMESPACES`, `is_local_only`); crates/svrngs-runtime/src/{lib.rs:119-129, journal.rs:359-366, register.rs:270} | fixed in befedc5b (behaviour kept for every group svrngs can hold; caps ratcheted, 71be570f) | Upstream's private namespaces ("notes-private", "work-atlas-private", "wikipedia-newsworthy-portal" …), kept and asked on every round, sync answer and standing: a group's namespace is its seed's hash, so none is ever one. Code no svrngs group reaches, and upstream's app names in the core. |
| 201 | 12 | crates/svrngs-runtime/src/lib.rs:59-63 (`open: Mutex<BTreeMap<…>>`); docs/DESIGN.md:1462-1463 | not fixed (a unit of its own, the operator's) | "an actor per journal replaces it" has no row in any queue and no finding until this one. |
| 202 | 8 | crates/svrngs-node/build.rs:20-41; crates/svrngs-node/src/well_known.txt:24, :33; docs/DESIGN.md:2439 (U5's bar) | not fixed (a choice of direction, the operator's) | The infra's build reads apps/expenses, apps/events and apps/jellyfin/README.md into the binary, and well_known.txt holds Jellyfin 12's own menus; U5's bar was "nothing that knows Jellyfin outside apps/jellyfin/". The vocab gate passes because the words are data. |
| 203 | 6 | crates/svrngs-node/src/door.rs:905-909 (`unix_now`, `map_or(0, …)`); crates/svrngs-core/src/pass.rs:208 | fixed in 6371d263 (door/tests.rs `a_clock_before_1970_is_no_time`, red on a plant) | A clock that cannot be read is read as 0, and `0 >= until` is false, so a lapsed pass opens a lend. The runtime's `now_unix` (journal.rs:463) refuses the same absence. |
| 204 | 8 | crates/svrngs-node/src/group.rs:146-150, :187; crates/svrngs-runtime/src/journal.rs:453-457 | not fixed (with #107) | A group's seed is kept twice, `seed.json` and `roster.json`, each by a plain `fs::write`; the register reads the second, written only as the group is made or joined, and nothing writes it again. |
| 153 | 8, 6 | as recorded; and crates/svrngs-node/src/home/home.js:212 (`ask`'s `back`, the same rule) | fixed in 4cb7d408 (tests/browser/main.rs `the_home_page_opens_a_link_only_within_its_lend`, red both engines; open.js keeps its own line of the same rule) | Still present at 1edc5fbd, at home.js:245-246 now, and a second site: the ask's way back to an app, `location.assign(new URL(back, app.origin))` after the same two `startsWith`. |
| 80 | — | scripts/lean_check.py:75 | not fixed (operator's) | 16,350 / 16,351 and 2,992 / 2,993 at 1edc5fbd: #153 waited on one line. After northstar-economize's cut (−28, −12) and within the operator's +100 and +15 (2026-10-16): 16,419 / 16,420 and 2,991 / 2,992 at c2db36c3, the caps 3 and 4 below the grant's 16,423 and 2,996; `scripts/check.sh` exit 0 there, every step passed, the ceiling canary 0.0336 s. |

## canon-run (2026-10-17): a house's rules on one host

Opened by its row (`ralph/next/canon/STATE.md`), not a `REVIEW-audit-` row:
the run is `demo/runs/canon/2026-10-17-house/` (README.md, "What it found").

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 205 | 6 | web/app.js:322-334 (`reachGroup`, the first computer to answer), :392 (`enter`, one computer's `/people/lends`), :97 (`failed`'s words) | fixed in 3666d2a4 (browser carried.rs `the_phone_goes_on_to_a_computer_that_answers`, red at HEAD in both engines, 2606e527, with cy's computer made the first to answer; red again on a plant, the first computer's failure ending the reach) | **Run.** After Bo left Elm, Dee's phone reached Elm through Bo's computer, which still runs svrngs and answers the dial. Its `/people/lends` failed ("the computer's answer is not HTTP: connection error"), and the page said "This browser could not open the page." without trying Ana's computer, which was on. House rules was listed only when the page was opened again, 10,937 ms later (`data/takes/2/summary.json`, step 7-dee, its console). The trial takes met it after both leaves, for 52.7 s and 134.1 s (2b2107aa's body). It is a race: takes 1 and 3 never met it. `enter` could go on to the next computer that answers when one fails, or drop a computer whose person the group's last answer said left. |
| 206 | 8 | apps/canon/index.js:53 (`fold` asked at `Date.now()`); crates/svrngs-core/src/command.rs:243-250 (a new act timed after every act held) | fixed in 5eb3d563 (laws.test.mjs `one answer has one clock`, red at HEAD, 5af576d4; red again on a plant, `fold` asked at `Date.now()` alone) | **Run, and reproduced.** Right after the Agree that passed a change (the clean take's steps 8 and 10), Ana's page said the old rule with nothing proposed; opened again, the new rule. An agreement written in the second of the proposal is timed a second ahead of the browser's clock, and canon asked before that second calls the change ratified and keeps the old rule in force (`same_second.mjs` there: "asked at 2099: the change ratified since 2100; the rule in force majority"). The page has two clocks for one answer. Asking canon at no earlier than the newest act's time would give it one. |
| 207 | 6 | apps/canon/house.js `fold` (canon asked at the newest act it is handed, #206's fix); crates/svrngs-core/src/command.rs:238-250 (`reach`, AHEAD_SECS) | closed, not fixed (guests-fixes (d), with why: the row below in REVIEW-audit-canon-auto-1's findings) | **Read, and run under node.** Since 5eb3d563 an act dated ahead moves the page's clock with it. svrngs signs none more than AHEAD_SECS past the newest act it follows, so only a changed svrngs writes one. With one, a house deciding "unless someone objects" sees a proposal as holding before its two days are out: a proposal at 2000 and an act dated three days on, asked at 2200, read `active`; with bo's objection at 2100, `refused`, as canon judges by the acts' own times (a scratch script over `fold` and the vendored build, 2026-10-17). Bounding the clock at AHEAD_SECS in the page would be that threshold's second home; svrngs telling the page the time it would give a new act would keep one. |

## REVIEW-audit-canon-auto-1 (2026-10-17, range ac5d2d0f..59ddb5b2)

The previous `REVIEW-audit-` row is REVIEW-audit-grapevine-auto-1, its hash
ac5d2d0f (ralph/next/grapevine/STATE.md, its row); no audit row ran in
v1patch or before this one in canon. The range ends at 59ddb5b2, the
commit that made this row. `git rev-list --count ac5d2d0f..59ddb5b2` is
122, none a merge (`--merges`: 0). They are: grapevine-auto-1's `done`;
the north-star audit (6369790c) and its fixes, #187-#204 and #153;
v1patch-loopback; canon-economize, canon-floor, canon-pin, canon-spike,
canon-spans, canon-house, canon-share, canon-run, canon-docs and canon-fix;
V1.1's release notes (3637ccaf); and this row's own commit. The row reads
`docs/ARCH_PRINCIPLES.md` and `target/debug/sovereign-cli`, as
grapevine-auto-1's did (e040ffed): the principles are
docs/principles/ARCH_PRINCIPLES.md, the instrument
~/dev/commonwealth-ai/target/debug/sovereign-cli. The state file is left
as it is; the loop owns the row.

This audit's own commits before this section:
- c949a678, the runtime's three torn-tail tests set up through one helper;
- f9c7aeef, ring_sync's `Told` and carried.rs's `Answer` renamed.

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
ac5d2d0f..59ddb5b2` (/tmp/canonaudit/ledger.raw) is summed by
grapevine-auto-1's ledger.py, its unit list this range's
(/tmp/canonaudit/ledger.out; each commit's unit in ledger.commits). A
commit naming no unit counts toward the next that does: 5bc0db8e, 1edc5fbd
and 6369790c (docs, and the north-star audit) to northstar-economize; the
north-star fixes, each named by its finding, and the queue files 4e0c6cdd
and 606fea37 to northstar, whose budget commits name it; 1a4f8961 to
v1patch-loopback; bcaec482 (DESIGN §8) to canon-floor; 59348697 (DESIGN
§3) to canon-pin; c522d907 to canon-house; ddf0e902, 8d24369f and 3637ccaf
(LICENSE, the queue, V1.1's notes) to canon-share. The lines are raw,
comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | apps vendor | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-grapevine-auto-1 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| northstar-economize | 6 | 10 | 56 | **−46** | 0 | 44 | −44 | 0 | 0 | 0 | 0 | +2 (+3 −1) |
| northstar | 30 | 256 | 124 | **+132** | 317 | 38 | +279 | −1 | 0 | 0 | 0 | +30 (+70 −40) |
| v1patch-loopback | 4 | 0 | 0 | **0** | 43 | 0 | +43 | +2 (+7 −5) | 0 | 0 | 0 | 0 |
| canon-economize | 5 | 0 | 9 | **−9** | 14 | 14 | 0 | 0 | 0 | 0 | 0 | +3 (+4 −1) |
| canon-floor | 6 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | +102 (+112 −10) |
| canon-pin | 7 | 233 | 80 | **+153** | 383 | 24 | +359 | 0 | 0 | 0 | 0 | +3 (+7 −4) |
| canon-spike | 5 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | +593 (+594 −1) | +60 | +147062 | 0 |
| canon-spans | 8 | 59 | 41 | **+18** | 156 | 18 | +138 | 0 | 0 | 0 | 0 | +5 (+9 −4) |
| canon-house | 11 | 24 | 18 | **+6** | 35 | 3 | +32 | 0 | +344 | +312 (+315 −3) | 0 | +2 (+5 −3) |
| canon-share | 10 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | +18 (+57 −39) | +27 (+58 −31) | +4994 (+40519 −35525) | 0 |
| canon-run | 9 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| canon-docs | 10 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| canon-fix | 9 | 15 | 7 | **+8** | 153 | 5 | +148 | 0 | +6 (+10 −4) | +15 | 0 | +3 (+4 −1) |
| REVIEW-audit-canon-auto-1 (its row) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 122 | 597 | 335 | **+262** | 1101 | 146 | +955 | +1 (+7 −6) | +961 (+1005 −44) | +414 (+448 −34) | +152056 (+187581 −35525) | +150 (+214 −64) |

Where the src lines went (`git diff --stat ac5d2d0f 59ddb5b2`, tests
apart): the north-star fixes across svrngs-core (admit.rs, command.rs),
svrngs-runtime (journal.rs, oplog.rs) and svrngs-node (removal.rs,
ring_sync.rs, door.rs, home.js), and their cut of upstream's local-only
namespaces (lib.rs, tests_local_only.rs gone, #200); canon-pin's
carried.rs (+102 −19 over the range: the files an app was brought with,
`POST /runs`, `changed`) and synced.rs's `Runs` in svrngs-core (+52 −7);
canon-spans' `Span` and `Membership::spans` (membership.rs), the four
traces it put on one line each (3141a979) netting it low; canon-fix's
reach in web/app.js (+21 −13 over the range with canon-pin's yes).
scripts/: canon-floor's trust_check.py (95) and trust_floor.txt, and
northstar's vocab_check.py and lean_check.py.

apps/: canon is new, 961 lines of source (build.sh 125, build/src/lib.rs
224, house.js 210, index.js 153, build's house.mjs 88, prove.mjs 104,
call.mjs 19, examples/time.rs 38) and 414 of tests (canon.test.mjs 60,
laws.test.mjs 234, page.test.mjs 120). canon's build is vendored whole and pinned by sha256: 147,062
lines at 0.3.0 (canon-spike), 152,056 at 0.5.0 (canon-share, faf50750).
demo/runs/canon/'s drivers are what ran, which the ledger and the budget
leave apart.

What the budget counts, by HEAD's scripts/lean_check.py over a `git
archive` tree of each end and of each unit's `done`
(/tmp/canonaudit/lean-<hash>.out). Under HEAD's caps a555a225 and 455398af
read BROKEN on svrngs-core's 3,016 / 3,009; their own cap was 3,017
(bdf99942), which canon-spans' cut ratcheted:

| at | svrngs-core | repository | counted net |
|---|---:|---:|---:|
| ac5d2d0f, range start | 2992 | 16350 | |
| c2db36c3, northstar's last budget commit | 2991 | 16419 | +69 |
| a0129152, v1patch-loopback done | 2991 | 16419 | 0 |
| 96c19ed0, canon-economize done | 2991 | 16413 | −6 |
| 8d86ed29, canon-floor done | 2991 | 16472 | +59 |
| a555a225, canon-pin done | 3016 | 16557 | +85 |
| 455398af, canon-spike's needs-human | 3016 | 16557 | 0 |
| b718b316, canon-spans done | 3008 | 16556 | −1 |
| 8c52cf5c, canon-house done | 3008 | 16560 | +4 |
| 5bb317e5, canon-share done | 3008 | 16560 | 0 |
| 0c52b4c7, canon-run done | 3008 | 16560 | 0 |
| 9bd35aed, canon-docs done | 3008 | 16560 | 0 |
| 923805f3, canon-fix done | 3008 | 16563 | +3 |
| 59ddb5b2, range end | **3008** | **16563** | **+213** |

The caps (`git log ac5d2d0f..59ddb5b2 -- scripts/lean_check.py`, BUDGET
at each):

| commit | the repository's cap | svrngs-core's | for |
|---|---|---|---|
| ac5d2d0f, range start | 16,351 | 2,993 | |
| 71be570f, northstar-economize | → 16,323 | → 2,981 | the cut, #200 |
| 02893e39, c3c795b8, 7a3a77f3, fc2ad0ec, 0d0f4f9b, d62c393e, 23dcc2ef, bab14a6e, 10b23294, c2db36c3 | → 16,420 | → 2,992 | #187, #188, #189, #191, #190, #203, #153, #194, #195, #190, within the operator's +100 and +15 (2026-10-16) |
| 8d6f6cdf, canon-economize (c) | → 16,414 | 2,992 | the cut, ratcheted |
| 2d6d5edb, 9d2e7256, canon-floor | → 16,473 | 2,992 | the trust floor's gate, +59 |
| 42377ed5, bdf99942, canon-pin | → 16,558 | → 3,017 | a carried app pinned, each new version each person's yes, +85 and +25 |
| 26f94fe9, canon-spans' cut | → 16,526 | → 2,985 | ratcheted, one past where the cut landed |
| 4e267e48, canon-spans (b) | → 16,557 | → 3,009 | each person's time in the group |
| ceb999f9, canon-house (a) | → 16,561 | 3,009 | House rules on the shelf |
| 9e62c074, canon-fix (d) | → 16,564 | 3,009 | the page going on to a computer that answers (#205) |

The CHARTER's budget is "+150 repo, +40 core" over the caps the economizing
pass left, 16,414 and 2,992: ceilings 16,564 and 3,032. The range closes at
16,563 / 16,564, all 150 spent, and at 3,008 / 3,009, 17 of 40 spent and 23
under the ceiling unraised. lean_check fails at `n >= cap`
(lean_check.py:173), so no counted line is left under the repository's
cap. This audit's commits move no counted line: c949a678 is test code,
f9c7aeef renames on the lines it touches (16563 / 3008 after each).

### The instrument

The binary is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
7230170e… (`shasum -a 256`), the one grapevine-auto-1 ran.

The corpus is 59ddb5b2's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock, every `vendor/` and `node_modules/` left out, by `git archive`
into /tmp/canonaudit/canon-59ddb5b2 (365 files).
- `code index . --corpus-id canon-59ddb5b2`: 3950 chunks in 601 s
  (index.out).
- `refresh`, run in the corpus: rust, 17787 symbols and 103324 edges;
  typescript and python "skipped (export failed)" (I4) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-core, svrngs-runtime, svrngs-node and
svrngs, web, scripts and apps/canon (`git diff --stat ac5d2d0f 59ddb5b2 --
crates apps web scripts`, vendor/ apart). Each was reported at 0.95 and
0.90, and apps/ whole, so a clone across two apps is seen
(/tmp/canonaudit/clean-*.md). Whether a side was added in the range is
`git blame` at 59ddb5b2 over each side's lines against ac5d2d0f (132
sides, cluster-sides.txt; blame-clusters.out). Its plant: house.js:1-6
says 38cbfa58, in range, and api.rs's old `addr` pair (:212-214) says
5d6ddd5c, before (plant.verdicts).

| scope | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---|---:|---:|---|
| crates/svrngs-core/ | 4 | 0 | 11 | 1 cluster, 2 sides |
| crates/svrngs-runtime/ | 1 | 0 | 0 | 0 |
| crates/svrngs-node/ | 1 | 1 | 10 | 0 |
| crates/svrngs/ | 0 | 0 | 0 | — |
| web/ | could-not-judge (I4) | 0 | 2 | 0 |
| scripts/ | could-not-judge (I4) | 0 | 0 | 0 |
| apps/canon/ | could-not-judge (I4) | 0 | 1 | 1 cluster, 2 sides |
| apps/ whole | could-not-judge (I4) | 24 | 33 | 1 cluster, 2 sides (the 0.95 `pins` pair; at 0.90 the same four as one) |

The clusters with a side the range added:

| cosine | range side | other side | read |
|---|---|---|---|
| 1.000 | apps/canon/canon.test.mjs:12-21 `pins` (e0996d7f) | apps/grapevine/grapevine.test.mjs:19-28 `pins` | A copy, byte for byte, and `unpinned` under it (:18-29 and :25-36): how a vendored file's sha256 is read from vendor/README.md's table and checked, two apps' tests reading one format. #208. |
| 0.928 | canon.test.mjs:12-21 `pins` | canon.test.mjs:18-29 `unpinned` | Not a clone: two neighbours, padded to overlap (I32). |
| 0.902 | crates/svrngs-core/src/register_tests.rs:950-960 `a_link_redeems_after_an_act_timed_days_ahead` (82aac96e) | :959-970 `a_link_redeems_after_a_forged_line_timed_ahead` (82aac96e) | Not a clone: #188's two plants, a signed act and a line nobody signed, each its own test. A test file reported as production (I8). |

Every other cluster's sides blame to commits before ac5d2d0f.

**The line-window pass.** grapevine-auto-1's windows.py over its file set,
with every code file the range added or changed (vendor/ apart), at
59ddb5b2 (241 files, windows.files). **220 window hits, 56 spans, 3 new**
(windows.spans; windows.verdicts, by newspans3.sh's blame against
ac5d2d0f; 53 old). Its positive control: the old span
apps/events/store.mjs:17-100 == apps/grapevine/store.mjs:17-100 is among
the 53.

| new span | read |
|---|---|
| apps/canon/canon.test.mjs:12-25 == apps/grapevine/grapevine.test.mjs:19-32 | `pins` and `unpinned`, #208. |
| crates/svrngs-runtime/src/tests.rs:167-174 == :858-865 | 4a22ab43's #191 test, the third copy of a nine-line setup. **Fixed in c949a678.** |
| crates/svrngs-runtime/src/tests.rs:622-631 == :858-867 | The same. |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges ac5d2d0f..59ddb5b2 -- crates
apps web scripts` (vendor/ apart), matched against `(struct|enum|trait|
type|class) +[A-Z]\w*` (new-types.out), give ten: `Request`, `Film`,
`Runs` (twice: `type`, then `struct`), `Span`, `Answer`, `Files`, `Told`,
`Written`, `Yes`. Removed (removed-types.out): `type Runs` and `type
Written`, each rewritten in place (synced.rs). `Film` is vocab_check.py's
planted text (:84, :86), not a type.

`converge noun <Name> --corpus-id canon-59ddb5b2` over each (nouns.out):

| noun | definitions | read |
|---|---|---|
| `Answer` | 3: crates/svrngs-node/src/carried.rs:172 (`type Answer = Result<Response, Response>`, 40295803, in range); door/sign_in.rs:71 (`struct Answer`, a sign-in's answer, 8572711e); node.rs:758 (fn-local, a computer's lends answer, 533c6442) | The range's is an alias for one signature. **Renamed `Handled` in f9c7aeef**; the older two stay. |
| `Told` | 2: crates/svrngs-node/src/door.rs:1182 (`struct Told`, what the door decided for a request let in, 345bfdfd); ring_sync.rs:413 (`pub type Told`, the leaves told, 1716275e, in range) | One name, two things, in one crate (principle 8). **Renamed `LeavesTold` in f9c7aeef.** |
| `Request` | 1 seen: crates/svrngs-node/src/iroh.rs:283 (axum's request, 07ae5d19) | By `git grep`, a second: apps/canon/build/src/lib.rs:84 (`enum Request`, a question to canon's build, afd379c9). The build is its own Cargo project under apps/ (AGENTS.md 12), not in the workspace (Cargo.toml:3), so the instrument cannot see it (I34). Two systems, not two deciders; not a finding. |
| `Runs`, `Span`, `Files`, `Written`, `Yes` | 1 each | synced.rs:146, membership.rs:151, synced.rs:141, :173; `Yes` carried.rs:170 by `git grep` (`converge` names it once). |

(The instrument's lines are one early, I1.)

### Reading the range

The canon rows' src was read hunk by hunk (`git diff ac5d2d0f 59ddb5b2 --
crates/svrngs-node/src/carried.rs crates/svrngs-core/src/synced.rs
crates/svrngs-core/src/membership.rs crates/svrngs-core/src/record.rs
web/app.js scripts/trust_check.py`), and apps/canon's house.js and
index.js whole for silent defaults (`catch`, `?? `, `|| []`), each hit
read. The north-star fixes were each read and planted by their own
commits against #187-#204 (6369790c's section above), not judged again
here.

- **An unreadable answer said as an unsettled house** (principle 6).
  index.js:49 reads the house's acts as `r.json().catch(() => ({}))`, and
  when the answer was OK, `fold` takes `{}` as `spans` absent: `{"unsettled":
  true}` (house.js:171; run under node, 2026-10-17), so the page says
  "This computer can't yet say who is in your house … Open this page again
  in a minute." for a body it could not read. The pattern is Expenses'
  (index.js:26) and Events' (:31). #209.
- **canon's page traces nothing** (principle 1). No `console` or trace in
  apps/canon/*.js (`grep -n 'console\.\|trace' apps/canon/*.js`: none);
  house.js:181's bare `catch` drops an act canon could not build with "this
  version of House rules can't read it" and canon's own error lost. The
  page says each refusal to the person; nothing says it to whoever reads
  why. #210.
- **One decider** (fence 3, principle 8). house.js decides no rule of
  canon's: what passes, holds and who has a seat are canon's answers
  (`derive`, `rule`, `seat`, house.js:191, :200, :203); its own refusals
  (house.js:90-100) are what House rules carries — acts speaking for
  someone else, or moving membership, which svrngs alone writes. "Most"
  and "everyone" are canon's shares, cited at ratify.rs (house.js:126).
  #207 stays open.
- **Never silently substitute** (AGENTS.md 6). carried.rs `changed` says a
  group whose record cannot be read at warn and answers `null`, never
  "nothing changed" (:246-247); `acts` answers `spans` null, never empty,
  while who stands is unsettled (:331-333). `changed[kind]` is indexed
  safely: every placed `apps/<kind>/index.html` came from a write that set
  it (synced.rs:195-198).
- **Glassbox** (principle 1). Each new branch in carried.rs, synced.rs
  `Runs::of` and `Membership::spans` traces at debug.
- **The trust floor's gate** reads every line of the floor or says it is
  not one (trust_check.py:61-72), and judges what it could not read as
  BROKEN, never passed (:76, :88).
- **Forked code** (AGENTS.md 11). The range forked nothing from
  commonwealth-ai: 69 files carry a `ported from` line at ac5d2d0f and 68
  at 59ddb5b2, the one gone tests_local_only.rs, cut with #200 (befedc5b).
  canon's build is vendored whole and pinned (canon.test.mjs, red on a
  changed byte), not forked.

### Fixed here

- c949a678: the runtime's three torn-tail tests set up through
  `whole_then`, copies 3 → 1. `cargo test -p svrngs-runtime --lib` 46
  passed; red on a plant (the helper writing no tail), 3 failed at
  tests.rs:190, :634, :865.
- f9c7aeef: `Told` → `LeavesTold` (ring_sync.rs:414), `Answer` →
  `Handled` (carried.rs:173). `cargo check -p svrngs-node --tests` and
  `cargo fmt --check` clean; vocab_check.py "73 app nouns, 0 in infra
  code"; counted lines unchanged.

Not fixed here: #208 is a move across apps, with #150; #209 and #210
change what a page does, so each wants its red run first.

`scripts/check.sh` at f9c7aeef, this audit's last code commit, exit 0:
all 15 steps passed, principles pinned to app laws
(/tmp/canonaudit/check.out).

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 208 | 8 | apps/canon/canon.test.mjs:12-29 (`pins`, `unpinned`); apps/grapevine/grapevine.test.mjs:19-36 | fixed in aa70a97c (moved to apps/pinned.mjs, behaviour kept, # pass 24 before and after; red in both apps on a plant, `unpinned` filtering nothing out) | How a vendored file is pinned — vendor/README.md's `\| \`file\` \| \`sha256\` \|` table, and each file's hash against it — is read by two copies, byte for byte (dry-report 1.000; line-window span). A change to the table's form in one app's README passes the other's test unread. With #150's shared module: one reader both apps' tests import. |
| 209 | 6 | apps/canon/index.js:49 (`r.json().catch(() => ({}))`), apps/canon/house.js:171 (`spans` absent is unsettled); apps/expenses/index.js:26, apps/events/index.js:31 (the pattern) | fixed in a9543849 for House rules (laws.test.mjs `an answer the page could not read is said as unread …`, red at HEAD, 0bb7a0cb; red again on a plant, the old `!Array.isArray(read.spans)`); Expenses' and Events' pages keep the pattern, not this row's | An OK answer whose body is not JSON is read as `{}`, and House rules says the house's membership is unsettled, "Open this page again in a minute", where it could not read the answer at all (`fold(call, {}, …)` is `{"unsettled":true}`, run under node). In Expenses and Events the same `{}` folds as a group with nothing in it. A body that cannot be read is said as that. |
| 210 | 1 | apps/canon/house.js:181 (bare `catch`), apps/canon/index.js (no trace anywhere) | fixed in 278c7cbb (laws.test.mjs `every decision the glue makes is traced …`, red at HEAD, e001cb06; red again on two plants: the catch without canon's error, and index.js handing `fold` no trace, page.test.mjs red in both engines) | canon's page makes every decision a person can see — an act dropped and why, the rule asked of canon at which time — and traces none. An act canon refuses to build is said "this version of House rules can't read it", and canon's own error is lost. |
| 150 | 8 | as recorded, and apps/canon/canon.test.mjs:12-29 | not fixed: a move across apps | A fourth app's tests now copy a third helper (#208). **The third helper is one since aa70a97c** (apps/pinned.mjs); the five laws' harness and the canary's wait stay copied in events, expenses and grapevine, a move each app's laws re-prove red, not small. |
| 80 | — | scripts/lean_check.py:105 BUDGET, :173 `n >= cap` | not fixed (operator's) | 16,563 / 16,564 and 3,008 / 3,009 after this audit: +213 and +16 over the range, the CHARTER's +150 spent to the line, 23 of core's +40 left unraised. |
| 205, 206 | as recorded | as recorded | fixed (canon-fix; 3666d2a4, 5eb3d563) | Not reached by this range's findings. |
| 207 | 6 | as recorded | closed, not fixed (guests-fixes (d)) | **Closed with why.** Re-run under node at 278c7cbb (/tmp/gf207/ahead.mjs, the finding's case over `fold`): a consent house, ana proposing at 2000, asked at 2200 — `proposed`, canon asked at 2200; with kim's agreement dated three days on, `active`, canon asked at 261200; with bo's objection at 2100 as well, `refused`. (1) svrngs signs no act past AHEAD_SECS on its clock (command.rs:238-250), so only a changed svrngs writes one. (2) canon judges each act at its own time, so the act moves when the page says a proposal holds, never what the house decided: an objection inside the proposal's days still refuses it. (3) Since 278c7cbb the move is traced: `canon asked` gives the page's clock beside the act that set it (`{asked: 2200, at: 261200, newest: 261200}`). Said and never followed needs the bound in the page, AHEAD_SECS's second home (principle 8), or svrngs telling every carried app the time it would give a new act, a new field in each app's `svrngs/acts` answer (carried.rs:336-337) and counted lines in svrngs-node at 16,582 / 16,583 (`scripts/lean_check.py`). Reopen it when a carried page is handed svrngs's clock for its own sake, as a guest's visit ending on time may want. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun`, dry-report spans | Still one line early: carried.rs:172 for :173, ring_sync.rs:413 for :414, door.rs:1182 for :1183. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/'s, scripts/' and apps/' exact tiers could not judge, and `converge noun` sees no JS class. |
| I8 | 6 | dry-report scope | Still present: register_tests.rs and `*.test.mjs` reported as production. |
| I32 | 7 | dry-report's near tier, nested spans | Still present: canon.test.mjs `pins` over `unpinned`. |
| I34 | 6 | `converge noun`, `refresh` | New: a Rust crate outside the Cargo workspace is not exported, so its nouns are absent with nothing said: apps/canon/build/src/lib.rs:84's `enum Request`, which `git grep` finds, is not among `Request`'s definitions. |

## guests-run (2026-10-20): a visit on one host

Opened by its row (`ralph/next/guests/STATE.md`), not a `REVIEW-audit-` row:
the run is `demo/runs/guests/2026-10-20-run/` (README.md, "What it found").

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 211 | 6 | crates/svrngs-node/src/removal.rs:211-222 (a leave told as the Remove alone); crates/svrngs-core/src/membership.rs:515-556 (`covered`, a `prev` chain the copy does not hold), :590 (an uncovered act of a cut key counts nowhere), :626-633 (a Visit judged against the visits then counted) | fixed in c599547e (leave-told): a cut told with what it covers, `ring_routes::told_with` the one selection the at-once tell and `told_cut` make; reproduced by the survey (demo/runs/leave/2026-10-21/SURVEY.md, 503b891c), its test red at 153ce69d in 4220c342 (tests/removal.rs `a_leave_reaches_every_copy_with_the_acts_it_covers`, Ana's copy `[]` against Kim's `["kim-paid", "kim-last"]`), green with four plants each red (demo/runs/leave/2026-10-21/told/): the tell's acts the lines alone, `told_with` keeping only the `Remove`s, the tell keeping its first answer, the signer's first `Remove` for its highest. A copy that already holds a lone leave is not healed (the survey's A+, not built: nobody has left a real group, the operator, 2026-10-21) | **Run, five takes.** Kim's computer wrote Lee's visit 2.7 to 4.0 s before Kim left. The leave reached Ana's and Bo's computers in 3 to 15 ms; Lee's visit, the leave's `prev`, never did on Ana's in any take, nor on Bo's in three. On such a copy the leave covers only itself, so four of Kim's acts, Sam's and Jo's visits among them, are said to come "after they were removed from the group", with an `unheld_cut` naming Lee's visit (each take's `summary.json`, `end_record`). With Kim's visits gone from that fold, Max's visit, refused at Ana's link as past the house's bound (`counted=false`), counts, and the home page says "Max, Ana’s guest" — on the copies without Lee's visit alone, so the copies disagree. Any act a leaver makes within a round of leaving does the same to their earlier acts elsewhere; a Visit is where the run met it. |
| 212 | 10 | crates/svrngs-node/src/redeem.rs:157-166 (a join wakes the round), :186-226 (a Visit does not); crates/svrngs-node/src/home.rs:437-451 (a Stop does not); crates/svrngs-node/src/ring_sync.rs:107 (every 60 s) | open | **Run, five takes.** Bo's computer held Sam's visit 49.3 to 50.9 s after Sam got in, and Kim's Stop reached Ana's computer after 51.8 to 59.5 s, against GUESTS §3.2's "everyone in the house can see who's visiting". Each writer of an act others wait for remembers to wake the round, and these two do not. The doors are not late: each asks its own copy (door.rs:979-981), and Kim's computer dropped Sam in 2 to 4 ms. |
| 213 | 6 | crates/svrngs-core/src/admit.rs:134 (`NotAMember`); crates/svrngs-core/src/membership.rs:633-637 (a Visit past a bound `continue`s, so is never `counted`) | open | **Run, five takes.** Max's visit is Ana's act, a member's. From the moment it is written, every computer's sync warns "this answer covers a subset — the rail could not account for every op … NotAMember" of it, 7 to 19 times in each computer's kept log in each take, and 0 to 2 in the `count` plant's, where the visit counts. Bo's `/v1/rail/log` names it a `not_a_member` gap (`data/plants/host/summary.json`, `end_record`). GUESTS §3.7: a visit past a bound is "simply not a visit"; the record says its signer is no member. |

## guests-first (2026-10-20): a guest's first request

Opened by its row (`ralph/next/guests/STATE.md`), not a `REVIEW-audit-` row:
svrngs-bb's V1.1.3 check runs met it
(`demo/runs/v1.1.3/2026-10-20/main-check-1.out:148-221`, `main-check-3.out:142`).

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 214 | 8 | crates/svrngs-core/src/membership.rs:252-263 (`Guest::live`, a visit begun only at its own time); crates/svrngs-core/src/command.rs:238-250 (a new act timed after every line held up to AHEAD_SECS ahead); crates/svrngs-node/src/door.rs:1030 (the door asks at now) | fixed in 86a3bd08 (door `guests::sam_is_let_in_at_once_though_kims_visit_is_timed_ahead_of_the_clock`, red at bb0f1a1f five of five, b37bc81c; red again on a plant, the visit begun only at its own time; core `a_visit_is_on_from_when_its_act_is_written_and_no_earlier`, red on two plants, begun only at its own time and begun at any time) | **Run, and reproduced.** In 2 of 5 of the V1.1.3 checks' runs, Sam's `POST /visit` answered 200 and Sam's first `GET /lend/<label>/` 403, the page waiting on a film that never came (`crates/svrngs-node/tests/browser/guests.rs:100`). Those runs kept the page's trace, not the door's. With an act of Bo's held 60 s ahead of Kim's clock, Kim's `Visit` is timed after it, and the door refused Sam on every run, its trace `live=false own=true named=true`, `ended`, "its time is up": the act's clock and the door's, two clocks for one answer, as #206 met them in House rules. A visit is now on from its writer's clock as it was written, at most AHEAD_SECS + 1 before its own time, and never earlier, so no clock far ahead lengthens it. The door, the pages and the fold still ask `Guest::live` alone, and the fold's walk sorts by time (membership.rs:411), so its counting is unchanged. After the fix the browser film test passed 5 of 5 alone, in Chromium and in WebKit (`cargo test -q -p svrngs-node --test browser guests::sam_opens_the_link_kim_sent_and_watches_kims_film_as_kims_guest`, 2 passed each run). Those runs show the film test still passing; the door test, red before and green after, is what shows the fix. |

## REVIEW-audit-guests-auto-1 (2026-10-20, range 34d6e287..beb20b68)

The previous `REVIEW-audit-` row is REVIEW-audit-canon-auto-1, its hash
34d6e287 (ralph/next/canon/STATE.md:72); none ran in this queue before
this one. The range ends at beb20b68, the commit that made this row:
`git rev-list --count 34d6e287..beb20b68` is 117, none a merge
(`--merges`: 0). Beside the guests rows it holds what main took between
them: DESIGN distilled and §8-§10 moved to PLAN (f7508c19, adee4194),
the 25-computer run and Linux's allocator fix (V1.1.1), a group on a
screen (V1.1.2), apps/doc, V1.1.3, and PLAN §2's notes. The row reads
`docs/ARCH_PRINCIPLES.md`, which is docs/principles/ARCH_PRINCIPLES.md,
and `target/debug/sovereign-cli`, which is
~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256 7b86f7bf…
(`shasum -a 256`), rebuilt since canon-auto-1's 7230170e…. The state
file is left as it is; the loop owns the row. Every file named /tmp/
below is under /tmp/guestsaudit/.

This audit's own commit before this section: c91069af, the browser
tests' two node starters made one (below, "Fixed here").

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
34d6e287..beb20b68` (ledger.raw), summed by canon-auto-1's ledger.py
with this range's units (scripts/ledger.py; each commit's unit in
ledger.commits). A subject's scope, `kind(scope)`, names its unit; the
work main took between rows is grouped by its scope: `docs`, `design`,
`citations` and `deploy-page` as design-to-plan; `twentyfive`, `leak`,
`door` (23c182ae) and `svrngs` (61a06614) as twentyfive-leak; `home` and
`screen` as screen (V1.1.2); `releases` with v1.1.3; `§2` as plan §2. A
commit naming none counts toward the next that does: the guests queue's
own drafts (203ac18b, 4f93ff76, cc8a2ebe) and V1.1.1's notes (dc1781ef)
fall to screen, with no product line; eae4c386 to guests-pages,
f3b2faa9 to guests-first. The lines are raw, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps src net | apps test net | apps vendor | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-canon-auto-1 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| design-to-plan | 4 | 86 | 86 | **+0** | 35 | 35 | +0 | 0 | +0 (+9 −9) | 0 | 0 | +0 (+11 −11) |
| twentyfive-leak | 7 | 23 | 14 | **+9** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | +4 (+9 −5) |
| screen (V1.1.2) | 10 | 68 | 20 | **+48** | 128 | 2 | +126 | +31 | 0 | 0 | 0 | +5 (+11 −6) |
| guests-fixes | 8 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | +47 (+63 −16) | +12 (+50 −38) | 0 | 0 |
| guests-economize | 5 | 1 | 6 | **−5** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | +5 (+6 −1) |
| guests-visit | 7 | 127 | 17 | **+110** | 267 | 5 | +262 | 0 | 0 | 0 | 0 | +3 (+4 −1) |
| guests-door | 6 | 114 | 15 | **+99** | 325 | 0 | +325 | 0 | 0 | 0 | 0 | +2 (+4 −2) |
| guests-link | 7 | 250 | 35 | **+215** | 363 | 25 | +338 | 0 | 0 | 0 | 0 | +5 (+6 −1) |
| guests-pages | 11 | 89 | 18 | **+71** | 176 | 8 | +168 | +2 (+4 −2) | 0 | 0 | 0 | +4 (+7 −3) |
| guests-run | 11 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| doc | 15 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | +1075 (+1898 −823) | +402 (+1036 −634) | +117 (+34815 −34698) | 0 |
| plan §2 | 3 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| v1.1.3 | 14 | 50 | 14 | **+36** | 453 | 5 | +448 | +28 (+33 −5) | 0 | 0 | 0 | +12 (+18 −6) |
| guests-first | 7 | 8 | 3 | **+5** | 60 | 0 | +60 | 0 | 0 | 0 | 0 | 0 |
| REVIEW-audit-guests-auto-1 (its row) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 117 | 816 | 228 | **+588** | 1807 | 80 | +1727 | +61 (+68 −7) | +1122 (+1970 −848) | +414 (+1086 −672) | +117 (+34815 −34698) | +40 (+76 −36) |

The guests rows' src is +495 raw of the range's +588: guests-link's
redemption, the visit link and the guest's page (redeem.rs +102 −20,
invite.rs +55 −4, web/app.js, web/store.js), guests-visit's fold
(membership.rs, lib.rs `Visit`), guests-door's `guest_of` (door.rs) and
the runtime's `visit_anywhere`, guests-pages' **Your guests** and
**Visiting now** (home.js, question.js, surface.rs, rail.rs `visiting`).
apps/doc is commonwealth-ai's ring-doc forked (bfe715a6), its expense
book dropped (909eff3b) and its own server added (server.mjs, ws.mjs);
its vendored bundle went from ring-doc-bundle.js (34,698 lines) to
doc-bundle.js, built by bundle/build.sh.

What the budget counts, by HEAD's scripts/lean_check.py over a `git
archive` tree of each end and each unit's last commit (lean-<hash>.out).
Main's work and the guests rows interleave, so a row's `done` holds
what main took before it:

| at | svrngs-core | repository | counted net |
|---|---:|---:|---:|
| 34d6e287, range start | 3008 | 16563 | |
| 0cea24ff, twentyfive-leak | 3008 | 16561 | −2 |
| d64c19d6, screen (V1.1.2) | 3008 | 16582 | +21 |
| 1eeb092b, guests-fixes done | 3008 | 16582 | 0 |
| e36dee5b, guests-economize done | 3008 | 16577 | −5 |
| bc4b588d, guests-visit done | 3068 | 16637 | +60 |
| 90f8dcdd, guests-door done | 3068 | 16703 | +66 |
| 7ee8b565, guests-link done | 3075 | 16846 | +143 |
| 91b71e28, guests-pages done | 3075 | 16886 | +40 |
| 05504007, doc's last | 3075 | 16886 | 0 |
| 793a6e88, v1.1.3's last | 3075 | 16906 | +20 |
| 1ad20b9d, guests-run done | 3075 | 16906 | 0 |
| f13e7d06, guests-first done | 3075 | 16906 | 0 |
| beb20b68, range end | **3075** | **16906** | **+343** |

The guests rows counted +304 in the repository and +67 in svrngs-core
(16,582 → 16,886 and 3,008 → 3,075 over guests-fixes..guests-pages,
guests-first's +5 raw all comment); main's work +39 (twentyfive-leak
−2, screen +21, V1.1.3 +20).

The caps (`git log 34d6e287..beb20b68 -- scripts/lean_check.py`, BUDGET
at each):

| commit | the repository's cap | svrngs-core's | for |
|---|---|---|---|
| 34d6e287, range start | 16,564 | 3,009 | |
| 61a06614, b7be71c3 | → 16,559 | 3,009 | the door's one dialer guard (23c182ae, −3) less mimalloc's one line, then the home page's one reader, each ratcheted |
| 7a385472, 2bebba22 | → 16,583 | 3,009 | a group on a screen, its founder first |
| 50071696, guests-economize (c) | → 16,578 | 3,009 | the cut, ratcheted |
| d31f39e1, guests-visit (b) | → 16,638 | → 3,069 | a visit folded beside membership |
| efd32fba, guests-door (b) | → 16,704 | 3,069 | a visit's key let in at its host's own lends |
| ad4eac42, guests-link (b) | → 16,847 | → 3,076 | a visit made by a link, the operator's raise (eae4c386) |
| 0b884151, 5732cc5c, guests-pages | → 16,887 | 3,076 | who is visiting, on the home page and the phone's |
| f0849b8c, 32398138, 423d28c5, v1.1.3 | → 16,911 | 3,076 | V1.1.3's own +24 |
| a620548e, v1.1.3's economizing | → 16,907 | 3,076 | its cut, ratcheted |

The guests ceiling is 16,925 and 3,076 (CHARTER.md, "2026-10-20,
later"). The range closes at 16,906 / 16,907 and 3,075 / 3,076: no
counted line left under either cap (lean_check.py:173, `n >= cap`), and
18 of the repository's ceiling unraised. c91069af is test code and moves
no counted line.

### The instrument

The corpus is beb20b68's crates/, web/, scripts/, apps/, Cargo.toml and
Cargo.lock by `git archive`, every `vendor/` and `node_modules/` left
out (388 files).
- `code index . --corpus-id guests-beb20b68`: 4147 chunks in 527 s
  (index.out). Its embedder is the daemon on 127.0.0.1:9741, which
  commonwealth-ai's session held its restart for.
- `refresh`, in the corpus: rust 18452 symbols and 107366 edges;
  typescript and python "skipped (export failed)" (I4) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-core, svrngs-runtime, svrngs-node and
svrngs, web, scripts, and apps/canon, doc, events, expenses, grapevine,
jellyfin, taste and web-conformance (`git diff --stat 34d6e287 beb20b68
-- crates web scripts apps`, vendor/ apart). Each crate, web/, scripts/,
apps/canon and apps/doc was reported at 0.95 and 0.90, and apps/ whole,
so a clone across apps is seen (clean-*.md). Whether a side was added
in the range is `git blame` at beb20b68 over each side's lines against
34d6e287 (130 sides, cluster-sides.txt; blame-clusters.out). Its
controls are in the same run: apps/pinned.mjs:9-18 blames to aa70a97c,
in the range, and apps/events/canary.mjs:42-52 to f1df40f1, before.

| scope | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---|---:|---:|---|
| crates/svrngs-core/ | 4 | 0 | 11 | 0 |
| crates/svrngs-runtime/ | 1 | 0 | 0 | 0 |
| crates/svrngs-node/ | 1 | 1 | 8 | 0 |
| crates/svrngs/ | 0 | 0 | 0 | — |
| web/ | could-not-judge (I4) | 0 | 2 | 0 |
| scripts/ | could-not-judge (I4) | 0 | 0 | — |
| apps/canon/ | could-not-judge (I4) | 0 | 0 | — |
| apps/doc/ | could-not-judge (I4) | 1 | 2 | 2 clusters, 4 sides |
| apps/ whole | could-not-judge (I4) | 24 | 35 | 3 clusters, 6 sides |

The clusters with a side the range added:

| cosine | range side | other side | read |
|---|---|---|---|
| 0.965 | apps/doc/doc.test.mjs:66-74 `send` (e6da8ce6) | :69-79 `close` (e6da8ce6) | Not a clone: two neighbouring methods of the test's socket, padded to overlap (I32). |
| 0.929 | apps/doc/ws.mjs:108-116 `send` (e6da8ce6) | :112-122 `close` (e6da8ce6) | The same, in the server's socket. |
| 0.925 | apps/pinned.mjs:9-18 `pins` (aa70a97c) | :15-23 `unpinned` (aa70a97c) | The same; #208's one reader, now one file. |

Every other cluster's sides blame to commits before 34d6e287. No side
of guests-visit's, guests-door's, guests-link's or guests-pages' Rust
is in any cluster.

**The line-window pass.** canon-auto-1's windows.py over every code
file at beb20b68, vendor/ and node_modules/ apart (323 files,
windows.files): **263 window hits, 58 spans, 6 new** (windows.spans;
windows.verdicts, newspans.sh's blame against 34d6e287; 52 old). Its
positive control, the old span apps/events/store.mjs:17-100 ==
apps/grapevine/store.mjs:17-100, is among the 52.

| new span | read |
|---|---|
| crates/svrngs-node/tests/browser/main.rs:115-130 == :196-211 | `computer_on` and `standing_on` (c488f080, f7bf1a79), one node start and seed but the founder's key. **Fixed in c91069af.** |
| crates/svrngs-node/tests/browser/main.rs:3244-3255 == :3749-3760, and :3263-3271 == :3761-3769 | V1.1.3's phone test (9de18ec0) copies `with_stopped`'s house: ada, cy and cy's shelf, and the driver's args. #217. |
| crates/svrngs-node/tests/browser/main.rs:3670-3679 == :3810-3819 | V1.1.3's two stand-in servers, `gzip_only` (fa2e9078) and `socket_page` (f7bf1a79), read a request's head by the same hand. #217. |
| crates/svrngs-node/tests/browser/guests.rs:83-92 == crates/svrngs/tests/join_page.rs:292-301 | guests-link's test (65c7fbb3) tells a browser driver its JSON line and reads its last line as join_page.rs does, in another crate's tests. #217. |
| web/tests/carried.mjs:17-25 == web/tests/screen.mjs:13-21 | The screen's driver (e522374c) opens a context and keeps its console as carried.mjs's does; page.mjs's `launch` is the shared part. #217. |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges 34d6e287..beb20b68 -- crates
apps web scripts` (vendor/ and node_modules/ apart), matched against
`(struct|enum|trait|type|class) +[A-Z]\w*` (new-types.out), give nine:
`Visit`, `Guest`, `Visiting`, `VisitLink`, `VisitRequest`, `Dark`,
`House`, `Refusal`, `Peer`. Removed (removed-types.out): none.

`converge noun <Name> --corpus-id guests-beb20b68` over each
(nouns.out):

| noun | definitions | read |
|---|---|---|
| `Refusal` | 2: crates/svrngs/src/cli.rs:537 (b08defef), door/sign_in.rs:132 (8572711e) | Neither is the range's. The range's is `type Refusal = Infallible` on `impl Command for Visit` (svrngs-core register.rs:145, 7754eb7a), the trait's associated type, no noun (as judged at the U7, v1 and canon audits). |
| `House` | 0 seen | Tests are out of the instrument's scope. By `git grep`, two test fixtures in two test binaries: crates/svrngs-node/tests/door/guests.rs:18 (Kim's house of three, 795ae01d, in range) and tests/surface.rs:108 (Alex and Sam, 1c61b377). Two houses, two binaries; not a finding. |
| `Peer` | 0 seen | apps/doc/ws.mjs:85, `class Peer`, by `git grep` the only one; JavaScript is not exported (I4). |
| `Visit`, `Guest`, `Visiting`, `VisitLink`, `VisitRequest`, `Dark` | 1 each | lib.rs:440, membership.rs:243, svrngs-runtime register.rs:86, invite.rs:38, redeem.rs:129, door.rs:286 (the instrument says each one line early, I1). |

### Reading the range

The guests rows' src was read hunk by hunk (`git diff 34d6e287 beb20b68
-- crates/svrngs-core/src/{membership,lib,register}.rs
crates/svrngs-node/src/{door,redeem,invite,home,surface,rail}.rs
crates/svrngs-runtime/src/{register,journal,lib}.rs web/app.js
web/store.js crates/svrngs-node/src/home/{home,question}.js`), and the
range's web/lend.js, svrngs main.rs and home.js for silent defaults
(`catch`, `?? `, `|| []`), each hit read. #211-#214 were opened and
watched by their own rows; they are not judged again here.

- **A visit's key admitted after it stands as a member while the fold
  still says it is visiting** (GUESTS §3.8, §3.9). The walk refuses a
  Visit whose key was bound (membership.rs:635-636, `used`), but an
  Admit binds a key with no word of the visits (:654-678). Run in a
  scratch test beside visit_tests.rs (then removed): Kim's visit of
  key 9 at Friday, alex's Admit of key 9 as "sam" an hour later; the
  fold says "key 9 stands: true; visiting at FRI+2h: [("sam", <kim's
  key>, 1172800)]; gaps []" (scratch-admit.out). The door asks standing
  first (door.rs:989-990), so from the Admit Sam is carried as a
  member. #215.
- **A guest's link this computer cannot read said as one it never
  made** (principle 6). `claim_visit` reads the kept link with
  `std::fs::read(&at).ok()` and parses it with `.ok()`
  (invite.rs:232-233), so an unreadable or unparseable link, already
  removed, is traced `held = false` (:235) as a link never made or
  claimed already, and the guest is told the refusal every link shares
  (redeem.rs:83). #216.
- **One decider** (fence 4, principle 8). The two bounds are each one
  constant (membership.rs `VISIT_LONGEST`, `GUESTS_PER_HOST`); the
  door, the pages and the fold ask `Guest::live` alone (door.rs
  `guest_of`, rail.rs `visiting`, membership.rs `visiting`). The page's
  "Your visit … ended" (web/app.js `visit`) reads `until` from the
  host's answer and only words it; the door's refusal stays the
  decider.
- **Never silently substitute** (AGENTS.md 6). `visiting` answers
  `null` when this computer's clock cannot be read, and both pages say
  so ("This computer cannot say who is visiting …", home.js:110; "…
  cannot be read just now.", web/app.js). `visit_anywhere` says an
  unreadable register at warn. lend.js's `away` traces an away page it
  could not fetch before saying svrngs's words.
- **Glassbox** (principle 1). Each new branch in the fold, the door's
  `guest_of`, the redemption, the link's claim and the host's Stop
  traces at debug, each refusal with the fact that refused it.
- **Forked code** (AGENTS.md 11). apps/doc is the range's one fork:
  four files carry their `ported from` line (adapter.js,
  adapter.test.mjs, app.js, index.html), `scripts/ports.py` lists them;
  the two the port brought and 909eff3b dropped (expenses.js and its
  test) went with their marks. 68 files carried a mark at 34d6e287 and
  71 at beb20b68; docs/DESIGN.md's mention went with adee4194's rewrite.

### Fixed here

- c91069af: the browser tests' `computer_on` and `standing_on` start
  and seed a node through one `seeded`, copies 2 → 1. `cargo test -p
  svrngs-node --test browser --
  a_phone_is_shown_a_lend_s_computer_that_did_not_answer_as_its_window_dark`,
  which starts nodes through both, 2 passed; red on a plant (the
  founder given `signing(me)` whatever is asked), 2 failed at
  main.rs:292 and :3768.

Not fixed here: #215 changes what the fold counts, and #216 what a
trace says, each in counted code with no line left under its cap; #217
is V1.1.3's tests and two crates' drivers, a move larger than this row.

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 215 | GUESTS §3.8, §3.9; 6 | crates/svrngs-core/src/membership.rs:635-636 (a Visit's key never bound before), :654-678 (an `Admit` binds with no word of the visits); crates/svrngs-node/src/door.rs:989-990 (standing asked before a visit) | not fixed (changes the fold; red first, and svrngs-core has no line left under 3,076) | **Read, and run in a scratch test.** A visit's key admitted after its visit stands, and the visit stays live: "key 9 stands: true; visiting at FRI+2h: [("sam", …)]". The door then carries the guest as a member. svrngs's own pages never offer a visit's key to a join (web/app.js makes it fresh and keeps it apart, store.js `keepVisit`), so only a changed client or an `Admit` by hand writes one; the fold, not the page, is where "a visit never becomes membership" holds. An `Admit` of a key some counting Visit holds could count nowhere, as a Visit of a bound key does. |
| 216 | 6, 1 | crates/svrngs-node/src/invite.rs:232-235 (`claim_visit`: `read(..).ok()`, `from_slice(..).ok()`, `held = kept.is_some()`); crates/svrngs-node/src/redeem.rs:196 (`NoInvite`) | not fixed (a trace's words in counted code, no line left under 16,907) | **Read.** A guest's link this computer made, whose kept file it cannot read or parse, is removed and traced as one never made or claimed already, `held=false`; whoever reads the log cannot tell a broken file from a second redemption. The person's words are rightly the one refusal every link shares. The trace could say which of the three it was. |
| 217 | 8 | crates/svrngs-node/tests/browser/main.rs:3244-3271 (`with_stopped`) and :3749-3769 (`a_phone_is_shown_…`); :3670-3679 (`gzip_only`) and :3810-3819 (`socket_page`); crates/svrngs-node/tests/browser/guests.rs:83-92 and crates/svrngs/tests/join_page.rs:292-301; web/tests/carried.mjs:17-25 and web/tests/screen.mjs:13-21 | not fixed (test code; a move across V1.1.3's tests and two crates) | The line-window pass's new spans in tests: V1.1.3's phone test sets up `with_stopped`'s house by hand, its two stand-in servers read a head by the same hand, a browser driver is told and read the same way in two crates' tests, and the screen's driver opens its context as carried.mjs's does. A change to one, such as a driver's line format, passes the other unread. |
| 80 | — | scripts/lean_check.py:105 BUDGET, :173 `n >= cap` | not fixed (operator's) | 16,906 / 16,907 and 3,075 / 3,076 after this audit: +343 and +67 over the range, the guests rows' +304 and +67 among them. 18 of the repository's ceiling (16,925) unraised, none of svrngs-core's. |
| 211-213 | as recorded | as recorded | not fixed | Open since guests-run; this range adds nothing to them. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early: lib.rs:439 for :440, membership.rs:242 for :243, door.rs:285 for :286, cli.rs:537 for :538. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)". web/'s, scripts/' and apps/' exact tiers could not judge, and `converge noun` sees no JS class (`Peer`). |
| I32 | 7 | dry-report's near tier, nested spans | Still present: three of the three clusters with a side in this range are neighbours padded to overlap. |

## REVIEW-audit-leave-auto-1 (2026-10-21, range e5fdc353..123ba07b)

The previous `REVIEW-audit-` row is REVIEW-audit-guests-auto-1, its hash
e5fdc353 (ralph/next/guests/STATE.md:72); none ran in this queue before
this one. The range ends at 123ba07b, the commit that made this row:
`git rev-list --count e5fdc353..123ba07b` is 16, none a merge
(`--merges`: 0). Its first commit, db1ab12e, is the loop's `done` for
that row and touches only its state file; blame and windows below are
taken against db1ab12e, which holds the same code. The range is the leave
queue alone: the survey of #211 (503b891c), leave-economize and
leave-told. The row reads `docs/ARCH_PRINCIPLES.md`, which is
docs/principles/ARCH_PRINCIPLES.md, and `target/debug/sovereign-cli`,
which is ~/dev/commonwealth-ai/target/debug/sovereign-cli, sha256
22c48619… (`shasum -a 256`), rebuilt since guests-auto-1's 7b86f7bf….
The state file is left as it is; the loop owns the row. Every file named
/tmp/ below is under /tmp/leaveaudit/.

This audit's own commit before this section: d1ab28b9, the removal
tests' two three-node starters made one (below, "Fixed here").

### (1) Net lines per unit, product code

`git log --reverse -M --numstat --format='@@%h %s' --no-merges
e5fdc353..123ba07b` (ledger.raw), summed by guests-auto-1's ledger.py
with this range's units (scripts/ledger.py; each commit's unit in
ledger.commits). The survey's scope `leave` is its own row; the queue's
draft (39f39a59), naming no unit, counts toward leave-economize. Raw
lines, comments and blanks in:

| unit | commits | src + | src − | src net | test + | test − | test net | markup net | apps | scripts net |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| REVIEW-audit-guests-auto-1 (its `done`) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 |
| survey (leave) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 |
| leave-economize | 6 | 4 | 9 | **−5** | 3 | 2 | +1 | 0 | 0 | +5 (+6 −1) |
| leave-told | 7 | 99 | 56 | **+43** | 355 | 19 | +336 | 0 | 0 | +2 (+3 −1) |
| REVIEW-audit-leave-auto-1 (its row) | 1 | 0 | 0 | **0** | 0 | 0 | 0 | 0 | 0 | 0 |
| total | 16 | 103 | 65 | **+38** | 358 | 21 | +337 | 0 | 0 | +7 (+9 −2) |

leave-economize's src is `Membership::visiting` cut (membership.rs +3
−8, lib.rs's doc link); leave-told's is removal.rs +61 −36 (`tell`,
`told`), ring_routes.rs +32 −18 (`told_with`, `told_cut`) and
ring_sync.rs +6 −2 (`leave_told`'s doc and call) (`git diff --numstat
153ce69d..747744ee -- crates`). The survey's .rs files are under
demo/runs/, outside product code.

What the budget counts, by HEAD's scripts/lean_check.py over a `git
archive` tree of each end and each unit's last commit (lean-<hash>.out):

| at | svrngs-core | repository | counted net |
|---|---:|---:|---:|
| db1ab12e, range start | 3075 | 16906 | |
| 153ce69d, leave-economize done | 3072 | 16903 | −3 |
| 747744ee, leave-told done | 3072 | 16927 | +24 |
| 123ba07b, range end | **3072** | **16927** | **+21** |

The caps (`git log e5fdc353..123ba07b -- scripts/lean_check.py`, BUDGET
at each):

| commit | the repository's cap | svrngs-core's | for |
|---|---|---|---|
| e5fdc353, range start | 16,907 | 3,076 | |
| 00f8a82d, leave-economize (c) | → 16,904 | → 3,073 | the cut, ratcheted |
| 62945f12, leave-told (b) | → 16,928 | 3,073 | #211's fix, +24 over the pass's cap |

The raise is +24 of the charter's +25 (CHARTER.md, "Pass first, then
raise") and svrngs-core's cap unchanged, as fence 7 asks. The range
closes at 16,927 / 16,928 and 3,072 / 3,073: no counted line left under
either cap (lean_check.py:217, `n >= cap`). d1ab28b9 is test code and
moves no counted line (`lean_check.py` after it: 16927 and 3072).

### The instrument

The corpus is 123ba07b's crates/, scripts/, Cargo.toml and Cargo.lock by
`git archive` (200 files). The range touched nothing in web/ or apps/
(`git diff --stat e5fdc353 123ba07b -- web apps`: empty), so they are
left out.
- `code index . --corpus-id leave-123ba07b`: 3090 chunks in 271 s
  (index.out), the embedder the daemon on 127.0.0.1:9741.
- `refresh`, in the corpus: rust 18575 symbols and 108114 edges;
  typescript and python "skipped (export failed)" (I4) (refresh.out).

### (2) Clones (`code dry-report`)

The range touched crates/svrngs-core, crates/svrngs-node and scripts/
(`git diff --stat e5fdc353 123ba07b -- crates web scripts apps`). Each
was reported at 0.95 and 0.90 (clean-*.md). Whether a side was added in
the range is `git blame -s db1ab12e..123ba07b` over each side's lines,
a line before the range carrying `^` (38 sides, cluster-sides.txt;
blame-clusters.out). Its controls in the same run: removal.rs:267-272
(`told`, c599547e) 6 of 6 lines new; api.rs:1-6 0 of 6.

| scope | exact | near 0.95 | near 0.90 | with a side added in the range |
|---|---|---:|---:|---|
| crates/svrngs-core/ | 4 | 0 | 11 | 0 |
| crates/svrngs-node/ | 1 | 1 | 8 | 0 |
| scripts/ | could-not-judge (I4) | 0 | 0 | — |

No side of `tell`, `told`, `told_with`, `told_cut` or `leave_told` is in
any cluster; every side blames to a commit before the range.

**The line-window pass.** canon-auto-1's windows.py over every code
file at 123ba07b, vendor/ and node_modules/ apart (321 files,
windows.files): **228 window hits, 57 spans, 1 new** (windows.spans;
windows.verdicts, newspans.sh's blame against db1ab12e; 56 old). Its
positive control, the old span apps/events/store.mjs:17-100 ==
apps/grapevine/store.mjs:17-100, is among the 56.

| new span | read |
|---|---|
| crates/svrngs-node/tests/removal.rs:117-133 == :415-431 | leave-told (a)'s `oak` (4220c342, the survey's test) seeds, admits, wires and rounds three nodes as the file's `three` does. **Fixed in d1ab28b9.** |

### (3) Duplicate type names (`code converge noun`)

The added lines of `git log -p --no-merges e5fdc353..123ba07b -- crates
apps web scripts`, matched against `(struct|enum|trait|type|class)
+[A-Z]\w*` (types-diff.out): **none**. Over the whole range, demo/
included (types-all.out), every hit is a line of
demo/runs/economize/2026-10-21-leave/data/demoted.diff, the pass's
record of `pub` demoted and rebuilt, not a type the range defines. So no
noun is run. The instrument still answers (nouns-control.out): `Said`, 2
definitions, crates/svrngs/src/cli.rs:1073 and door/choices.rs:42, both
older (u2-auto-2's list, :484 above), and `Digest`, 1, view.rs:108;
each one line early (I1: :1074, :43, :109).

### Reading the range

leave-told's src was read hunk by hunk (`git diff e5fdc353 123ba07b --
crates/svrngs-node/src crates/svrngs-core/src`), against the charter's
fences and principles 1, 6 and 8.

- **A cut told late to a removed computer counts through the keeper's
  later `Remove`s** (fence 4; CHARTER.md "anything beyond the record";
  DESIGN §6). Ran a scratch test (below), with its control. #218.
- **One decider** (principle 8, fence 3). `told_with` (ring_routes.rs:137)
  is the one selection; `told` (removal.rs:285) and `told_cut`
  (ring_routes.rs:163) call it, and nothing else retains a cut's chain
  (`git grep -n "ops.retain" -- crates/svrngs-node/src`: ring_routes.rs:141
  alone).
- **Never silently substitute** (principle 6, fence 5). `told` returns
  an `Err` for an unreadable digest, an unread lack and a digest that
  did not move, and `tell` says each at warn, "asked again, never sent the
  cut alone" (removal.rs:251); `leave_told` keeps only the computers that
  answered `Ok` (ring_sync.rs:443-448). A refusal is `Ok("it refused this
  computer")`, an answer, as #190 asked.
- **Glassbox** (principle 1). Each pass of `told`'s loop traces at debug
  (`sent`), its two ends by `tell`'s `how`; `told_with` traces its `seq`
  and `more`; `told_cut` its asker and signer.
- `told_with` is `pub` (ring_routes.rs:137) though only svrngs-node calls
  it (`git grep -n told_with -- '*.rs'`: removal.rs and ring_routes.rs,
  tests in comments only), where leave-economize's survey demoted such
  items. As `pub(crate)` rustfmt wraps its signature to 5 lines: tried,
  `lean_check.py` 16931 / 16928, so reverted. Not a finding; it costs
  lines the cap does not have.

### Fixed here

- d1ab28b9: the removal tests' `three` and leave_race's `oak` seed, admit,
  wire and round through one `knit`, copies 2 → 1. `cargo test -q -p
  svrngs-node --test removal`, 9 passed (knit-green.out); red on a plant
  (`knit` admitting only the first of the two), 5 failed, two through
  `three` (removal.rs:154, :259) and three through `oak` (:422, :527,
  :576) (knit-plant.out). The window pass over the file after: 0 spans.

Not fixed here: #218 changes what a removed computer is sent, counted
code with one line left under its cap, and its fix is a choice the
charter leaves to the operator ("anything beyond the record").

### Findings

| # | principle | path:line | fixed in | finding |
|---|---|---|---|---|
| 218 | 6; fence 4 | crates/svrngs-node/src/ring_routes.rs:139 (`told_with`: the signer's **highest** `Remove`), :161-163 (`told_cut` finds the `Remove` naming the asker, then passes only its signer); at e5fdc353 ring_routes.rs:146-149 (`told_cut` counted through the `Remove` naming the asker) | not fixed (what a removed computer is sent; the charter's "anything beyond the record" is the operator's) | **Run, with its control.** A scratch test beside leave_race (then removed): Oak, Ana keeping it; Ana's computer cannot reach Kim's, so Kim's is off as far as Ana can tell. Ana removes Kim, writes `ana-after-kims-cut`, then removes Bo. Kim's round asks; Kim's copy holds "key 2 cut: true, key 3 cut: true, entries ["ana-after-kims-cut"]". Without Bo's removal (the control): "key 2 cut: true, key 3 cut: false, entries []" (scratch-later-remove.out). So a computer told its cut late takes the keeper's acts after its cut, through the keeper's latest `Remove` of anyone, and the record of who was removed after it, DESIGN §6 says that selection as the code makes it, "the signer's acts through its highest `Remove`" (docs/DESIGN.md:999-1000), so the doc and the code agree; what they disagree with is the charter, which sends a removal "with only the keeper's acts before it and the hand-ons" (fence 4) and leaves "any act sent that is not the cut's signer's before it, or a hand-on" to the operator, and `told_cut` before c599547e, which counted through the `Remove` naming the asker. The leave case `told_with`'s `.max()` was chosen for (a person's other computer told the signing key's leave, which `Leave` writes last in one batch, tests/removal.rs:614-625) does not need a later batch's `Remove`; the bound that serves both is the operator's to choose. The at-once tell meets it only if the keeper writes a later `Remove` while `told` is still sending (removal.rs:267-290, `ops_missing_from_within` read again each pass, :284); a leave never, since the leaver signs nothing after it. |
| 80 | — | scripts/lean_check.py:149 BUDGET, :217 `n >= cap` | not fixed (operator's) | 16,927 / 16,928 and 3,072 / 3,073 after this audit: +21 and −3 over the range, leave-told's +24 the raise of 62945f12. |
| 211 | 6 | as recorded | fixed in c599547e | Not judged again; #218 is a reach of its fix this range adds. |
| 212-217 | as recorded | as recorded | as recorded | Open since guests-auto-1; this range adds nothing to them. |

### Problems in the commonwealth-ai instrument (reported, not fixed here)

| # | principle | site | finding |
|---|---|---|---|
| I1 | 7 | `code converge noun` | Still one line early: cli.rs:1073 for :1074, choices.rs:42 for :43, view.rs:108 for :109. |
| I4 | 6 | `refresh` | Still present: typescript and python "skipped (export failed)"; scripts/' exact tier could not judge. |
