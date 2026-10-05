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
| 38 | 5 (budget) | scripts/lean_check.py:25 (`crates/svrngs-core: 2500`); crates/svrngs-core/src/command.rs, process.rs, project.rs | not fixed: a raise is the operator's | svrngs-core grew from 1796 to 2359 of 2500 lines in this range. 563 lines are the operator's commits (6a56575 command.rs +218, 0c2952e process.rs +177, 75cbf14 project.rs +114). 141 lines are left, so the next decider or projection added to the core may hit the ceiling. Recorded so that is not a surprise. Nothing is duplicated there: the core is 0 exact and 0 near at 0.95. |
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
