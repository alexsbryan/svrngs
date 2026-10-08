<!-- APPEND ONLY. One section per REVIEW-audit- row; a finding is cited by row
     and number, so entries are never renumbered or reordered. -->

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
| 16 | 6, 10 | crates/svrngs-node/src/journal.rs:76-89 (`Held::catch_up` folds every line's seq, verified or not), crates/svrngs-node/src/journal.rs:297-310 (the door follows the newest seq, and names an unverified line when none at that seq verifies) | not fixed | One forged line claiming a signer's key, above everything that signer wrote, moves the door's next seq past it, which leaves holes that never fill. Reproduced with a scratch test, not committed: alex appends `one` (seq 0), the journal ingests `forged_as_alex(5, &one)`, and alex appends `two`, which the door signs at **seq 6** naming the forgery. Admission then reports BadSignature and **SequenceHole 1, 2, 3, 4 and 5**, so the fold is never complete; `one` and `two` still apply. Upstream's `next_seq` counted every line the same way, so this is inherited. u2-2 kept it and wrote it down ("when none verifies … names the first"). The fix (seq from verified lines only, as the door already does for `prev`) changes which seq the door signs. That is a behaviour change, for the operator or U7 (forks) to place. |
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
  what reaches Jellyfin or a browser.
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
| 82 | 8 | apps/jellyfin/mw/src/forward.rs:39-48, :119-137 and crates/svrngs-node/src/door.rs:142-151, :387-409, :415-423, at e9ed277 | not fixed | The middleware's and the door's hop-by-hop lists disagree (`trailer`, `proxy-authenticate`, and headers a `Connection` names), with no equivalence test between the two programs. |
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
