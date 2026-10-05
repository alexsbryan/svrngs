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
