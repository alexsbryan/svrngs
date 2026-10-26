# What the product carries that it does not need — give-economize's survey, 2026-10-25

Read at main `1d355c3d`, before any cut. Every count is
`scripts/lean_check.py`'s rule, per file by `scripts/count.py` here
(capability-economize's, unchanged), agreeing with its total: **18,240 /
18,241**, and svrngs-core **3,238 / 3,239** (`data/lean-before.txt`). One
counted line was left under each cap. The ceiling for the svrngs rows is
the operator's to give after this pass (CHARTER.md, "Awaiting the
operator"; AGENTS.md "Done"); §6 gives the number to ask with.

The instruments are capability-economize's (`../2026-10-23-capability/
SURVEY.md`): the compiler with every `pub` demoted, the clone report, the
line-window pass, the page's name count and the test-cfg count. Its
scripts are copied here. Two changes, each said where it is used:

- the scratch paths are under `target/give-econ/` (`scripts/loop.sh`,
  `scripts/refix.py`), and `loop.sh` runs `cargo check` in place of
  `cargo build`: the same privacy and dead-code diagnostics, with no
  codegen, on a disk at 30 GB free against its 15 GB floor;
- the clone report's corpus holds product sources only. The report reads
  nothing under `tests/` (its own exclusion list, `data/dry-*.md` line 7),
  so leaving tests out changes no finding and took the index from 522 s
  to 267 s.

## What changed since capability-economize's survey

`data/delta-5dd53719.txt` (`scripts/delta.py` over that survey's
`count-HEAD.txt` at `5dd53719` and this one): 17,634 → 18,240, **+606**;
svrngs-core 3,149 → 3,238, **+89**. It is the capability queue's rows and
keys-names: the door's navigation and the pass a page reaches with
(origin.rs +107), the phone's holder key and signing (web/lend.js +76,
web/src/lib.rs +68), authorship by signature (grant.rs +55, rail.rs +41,
pass.rs +22, sig.rs +14), the open's yes and what acts as you (node.rs
+41, app.js +40, question.js +32, home.rs +30, choices.rs +20), and the
links that carry a pass (invite.rs +30). Each of those rows ran its own
close, and the queue its audit (`ralph/REVIEW_FINDINGS.md`,
REVIEW-audit-capability-auto-1).

## 1. The compiler: one new item, kept

In a scratch copy of `1d355c3d` (`git archive` of crates, web, scripts,
apps and the manifests into `target/give-econ/scratch`; apps/ because
svrngs-node includes its pages), `scripts/demote.py` made every `pub` in
every non-test source of the four crates and `web/src` `pub(crate)` (77
files, `data/demoted.diff`). `scripts/loop.sh` checked the product and
`scripts/repub.py` made `pub` again what a downstream crate was refused,
round by round. `scripts/refix.py` was not needed. The hand edits were
the three the last survey names: `standing_reads` (membership.rs:798),
`NotASyncPath` (synced.rs:37) and `CliError::exit_code` (cli.rs:640). The
product ended `build-finished success: True`, and `web/` for
`wasm32-unknown-unknown` the same in four rounds. What still warns is
`data/warnings-prod.txt` and `data/warnings-web.txt`.

Against capability-economize's list, the lines that say an item is
unreachable:

- the four it kept, each read and kept again: `LINK_WITHIN` (dial.rs:46),
  `iroh::ALPN`'s re-export (iroh.rs:48), `MemoryLookup` (iroh.rs:64) and
  `At::Here` (relay.rs:63);
- `Key::kid`, gone (cut by that survey, d2596b56);
- **new: `version::links` (version.rs:36), "function `links` is never
  used".** The page calls it (web/src/lib.rs:371), compiled apart from
  the node, so the node's build cannot see the use. Kept, as
  `LINK_WITHIN` is. The same holds for `sig::unsigned_act`, unused in the
  node's build and called by the page (web/src/lib.rs:494);
- **new: `Capability::Read` (grant.rs:26), "variant `Read` is never
  constructed",** in both builds. Nothing in the product or its tests
  asks it. It is not cut, because the design it came from names it a job
  that was never wired: "**Read**: the carried route's filter, `own`
  (`carried.rs:314-315`), becomes `may(Read(kind))`. One scope then
  decides what an app reads and writes"
  (`demo/runs/authorship/2026-10-24/DESIGN.md:172-175`). At HEAD the
  carried route still filters with its own closure (carried.rs:317-318,
  `strip_prefix(&prefix)`), while `Grant::may` judges the same `<kind>/`
  scope for appends (grant.rs:69-70). Cutting the variant would decide
  against the design, and wiring it changes which code decides a read.
  Neither is a pass's to do, so it is a finding (below, "Open").

The svrngs-core re-exports that warn unused (`Reads`, `PayloadError`,
`act_id`, `redemption_message`, `ring_op_message`, `sign_ring_op`,
`NotASyncPath`, `DIGEST_V`) are the list start-economize read and kept
(`../2026-10-13/SURVEY.md`:59). The "more private than" warnings are the
pipe's own, an item demoted under a public one.

`scripts/cfgtest.py` finds the same three lines under a test cfg in
counted files as the last two surveys, now at `node.rs:214-215`,
`:419-420` and `:1023-1024` (`data/cfgtest.txt`): 3 lines, nothing new.

`scripts/jsnames.py` over the page's JavaScript (13 files,
`data/jsnames.txt`): every top-level name is said again. Its plant, a
function appended to web/app.js and never called, went red,
`web/app.js: plantedNeverCalled said 1`; with the file restored, green
(`data/jsnames-plant.txt`).

## 2. What no check, test or person runs: nothing

`git diff --name-status 5dd53719 1d355c3d -- scripts` adds no script that
`check.sh` does not run: the changes are `check.sh` (the lint step, the
browser engines), `lean_check.py` (caps and the budget comment),
`trust_floor.txt` and `build-web.sh --lint`'s oxlint, each run by
`check.sh`.

## 3. The clone report

`git archive 1d355c3d crates web scripts Cargo.toml Cargo.lock`, with
every `tests/` file removed (175 files), indexed by `sovereign-cli code
index . --corpus-id give-econ-corpus2` (2,483 chunks in 267 s,
`data/index.out`), `sovereign-cli refresh` (rust 11,926 symbols, 63,974
edges, `data/refresh.out`), then `code dry-report --scope <dir>
--threshold <t>` per crate, `web` and `scripts`, at 0.95 and 0.90
(`data/dry-<scope>-<t>.md`).

The corpus first sat in `target/give-econ/corpus`, inside this checkout.
There `refresh` resolved to the checkout's own registered project,
"svrngs", re-exported its SCIP graph and began a first build of its
LanceDB index, which `timeout 400` stopped
(`data/refresh-in-checkout.out`). Its `documents.lance` (13 Oct) was left
as it was, and the `.rebuild.lock` there dates from 3 Oct. That run's
`index.out` was not kept. So the corpus was copied to
`/tmp/give-econ-corpus` and run again from there, which is how the last
survey ran it. It is the one scratch outside `target/` (the capability
queue's rule, 43a7bace), and it is left there, since the session's
settings refuse `rm -rf` outside the tree.

| Scope | Exact | Near 0.95 | Near 0.90 |
|---|---:|---:|---:|
| crates/svrngs-core | 4 | 0 | 12 |
| crates/svrngs-runtime | 1 | 0 | 0 |
| crates/svrngs-node | 1 | 0 | 8 |
| crates/svrngs | 0 | 0 | 0 |
| web | could-not-judge (no SCIP for JavaScript) | 0 | 3 |
| scripts | could-not-judge (no SCIP for Python) | 0 | 0 |

Against the last survey, the same table but for two clusters:

- svrngs-core's `decide` cluster gains `record.rs:26-38` and
  `register.rs:257-269`, `:310-320` (`Append`, `Cutover` and `HandOnTo`'s
  `Command::decide`). Each decides a different act. They are alike
  because the trait gives them one shape, so there is nothing to fold.
- svrngs-node's `load`/`read` cluster gains `door/choices.rs:98-106`,
  `file`. The three spans overlap, one function inside its neighbour's
  span (I32), and `file` has three callers (choices.rs:107, :244, :334).
  Kept.

The exact clones are the three-line accessors the earlier surveys read
(`as_str`, `is_empty`, `lines`, `signed_by_link`, api.rs `addr`) and the
runtime test helper `open`. None is a cut.

## 4. Said twice: the line-window pass

`scripts/windows.py 3` over every non-test product source of the four
crates, `web/src` and the page's JavaScript (`data/windows-3.txt`, 122
runs). As before, most are a struct's fields beside the code that builds
it, a handler's extractors, `use` lines and argument lists. One is a
thing written twice:

| Cut | Where | Lines | Commit |
|---|---|---:|---|
| the links' sequencer taken in one line each, as rail.rs:537 and :557 take the live buffer's lock | invite.rs:179 and :196, `link` and `keep_linked` | 6 | 3acd44aa |

Read and not cut:

- `invite.rs:174` and `:209`: `link` and `link_held` share an argument
  list because one holds the lock and calls the other.
- `node.rs:259` and `:273`: `start` and `start_every` share an argument
  list for the same reason.
- `api.rs:159` and `room/owner.rs:167`: both read the caller's
  `ConnectInfo`, but one asks "this computer" (`check_loopback`) and the
  other "this network" (`guess`). They are two deciders of two things.
  give-token's check goes beside the first.
- `cli.rs:1520` and `:1554`: kept for the reason the last survey gives.

## 5. What the queue's rows will touch

Each file was read at 1d355c3d. Nothing a row will rewrite was cut.

- **`api.rs` (162 counted).** give-token's check goes beside
  `from_this_computer` (:158-178). The module's "Loopback IS the auth"
  (:12-14) and `NotLoopback`'s words (:33) are what the row says again.
  Nothing to cut: every refusal is said once, each with its `kind`.
- **`origin.rs` (323 counted, +107 since the last survey).** give-session
  changes `app_of`'s missing-`Sec-Fetch-Site` branch (:85-88 in GIVE.md's
  numbering, `app_of`'s first `if` today). Two cuts:

  | Cut | Where | Lines | Commit |
  |---|---|---:|---|
  | an own pass's reads, `Vec::from_iter((app != label).then(..))`, as register.rs's `Cutover::decide` writes an act it may not write | `own_pass` | 4 | 842f9da3 |
  | whether an answer is the door's "unasked", one statement with its receiver in parentheses, as `app_of` writes its own | `lent` | 3 | 36f58a12 |

  85a1817b's body names the first of these "5fe0a8c1". That hash is
  wrong: the commit is 842f9da3.
- **`home.rs` (511), `home/home.js` (222).** give-session's code per
  origin and give-play's row. `only_itself` (:172) and the WiFi cell
  (home.js:158-163) were read. The 5-line `if`s at home.rs:162 and :185
  each choose between two different calls, so neither is a cut.
- **`door.rs` (1,197), `door/limit.rs` (78).** give-udp's admission reuses
  `Using` and the standing check. The near clones `lends` (door.rs beside
  surface.rs) and the extractor lists at :986 and :1211 are read and kept
  as before.
- **`lend.rs` (377), `share.rs` (140), `crates/svrngs/src/cli.rs` share
  and unshare (:501-521, :770-, :827-).** The raw `Source` goes beside
  `Port` and `Carried` (lend.rs:181-188). Nothing dead there.
- **`node.rs` (683).** The fifth ALPN is dispatched beside `Alpn::Lend`
  (:403, :1004). Nothing to cut.
- **`room.rs`, `room/` (299 counted, as GIVE.md §6 says).** give-play
  follows its owner task. Its tests stay unchanged by name, so it is
  left as it is.
- **`web/install.sh` (125 counted).** give-token's curl (:174) and
  give-session's open (:179-182). Nothing to cut.

## 6. Each later row's lines, so the ceiling has a number

This is an estimate, not a measurement. Each range is grounded in the
counted code at the sites the row names, at 1d355c3d, and set beside
GIVE.md §9's sizings. The four ledgers are core (`crates/svrngs-core`),
library (the rest of `.`), `apps/` (not counted, reported) and surface
(items an app's author learns, DESIGN §3).

| Row | Its sites, and what each costs | Core | Library | `apps/` | Surface |
|---|---|---:|---:|---:|---:|
| give-token | a token minted and kept as `save_key` keeps a key (identity.rs:58-75, 17 counted): +15 to +25; one check beside `from_this_computer` (21 counted), naming the file in its refusal: +20 to +35; `api.rs`'s words said again: 0 to +5; `ask_running` sends it (cli.rs:916-937): +2 to +5; `install.sh`'s curl (:174): +1 to +3; the test helpers: not counted | 0 | +45 to +90 | +15 to +45: ten callers (events, pantry, doc, on-air, expenses' canary, movie-night, jellyfin's folders, taste's check), one to four lines each to read `<dir>/token` beside `<dir>/api` | +1: one header, read from one file |
| give-session | a code per open and a session cookie on the pattern of `oidc/provider.rs`'s `Codes` (:198-240, 42 counted), shared or a second: +40 to +70; the check beside `only_itself` and in `origin.rs` `app_of`: +20 to +40; "Open svrngs from your computer" and its command: +10 to +20; `svrngs open`: +20 to +40, the size of a verb that reads the API file and opens a browser (cli.rs's `link` arm, `opener` in install.sh); `install.sh`'s open with a code: +2 to +5; home.js links out with a code per origin (:54, :351-353): +15 to +30; a bookmark that keeps working: +3 to +15 | 0 | +110 to +220 | 0 | +1: `svrngs open` |
| give-udp | the fifth `Alpn`, one variant, one `bytes` arm, `ALL` widened (alpn.rs, 21 counted): +2 to +5 core; a raw `Source` and its declaration (lend.rs:181-188, share.rs's fields): +10 to +20; `--udp` in `share`'s parse (cli.rs:501-513): +5 to +10; dispatch beside `Alpn::Lend` (node.rs:403, :1004): +3 to +6; admission by standing and the limit across flows, reusing `Using` (door.rs:431-510, limit.rs): +15 to +30; the frame, fragmentation and reassembly: +40 to +70 (GIVE.md §6's own figure); flows with an idle expiry: +30 to +55; the lender's socket per flow on 127.0.0.1:0: +15 to +30; the dialling side the tests drive: +15 to +30 | +2 to +5 | +135 to +250 | 0 | +1: a lend may be udp |
| give-tcp | `--tcp` and its `Source` arm: +5 to +12; one bi-stream per connection, both sides, `copy_bidirectional` with half-close: +35 to +70; admission and limit through give-udp's: +10 to +25; traces and refusals: +10 to +20 | 0 | +60 to +130 | 0 | 0 (the same item, a lend may be tcp) |
| give-play | an owner task on the room's model (room/owner.rs, 125 counted; the raw one binds, unbinds and watches standing): +60 to +100; its on-state kept as `room.json` is (room/kept.rs, 16 counted): +15 to +25; the member's side, UDP flows and TCP accepts into the dialling side give-udp and give-tcp land: +40 to +80; the home page's route to turn Play on and answer the port (home.rs): +20 to +40; the row, its words and the copied address (home.js:158-163): +25 to +50; standing lost drops the listener: +5 to +15; `port_of` reused (room.rs:63): 0 | 0 | +165 to +310 | 0 | 0 |
| give-play-scene | the scene in `crates/svrngs/tests/scenes/` (tests, not counted), `demo/factorio.md` and `demo/minecraft.md` (docs) | 0 | 0 | 0 | 0 |
| **Total** | | **+2 to +5** | **+515 to +1,000** | **+15 to +45** | **3 items** |

Beside GIVE.md §9:

| Row | GIVE.md §9 | Here |
|---|---|---|
| give-token plus give-session | library +160 to +310 net, core 0 | library +155 to +310, core 0 |
| give-udp | library +160 to +290, core +13 to +25 | library +135 to +250, core +2 to +5 |
| give-tcp | library +115 to +195 | library +60 to +130 |
| give-play | library +215 to +350 | library +165 to +310 |
| **Sum** | **library +650 to +1,145, core +13 to +25** | **library +515 to +1,000, core +2 to +5** |

Two differences have reasons:

- **Core.** GIVE.md §9 counts "the `Alpn` variant and the raw `Source`"
  in the core. `Source` lives in svrngs-node (`lend.rs:181`), so the core
  grows by the variant alone, which is three lines in `alpn.rs`'s shape.
- **give-tcp.** GIVE.md sized the TCP stream before the UDP row would
  land the admission, the limit across flows and the dialling side.
  With those in place, give-tcp adds one stream per connection, and the
  range drops by give-udp's share.

The rest of GIVE.md's figures sit within 50 lines of this estimate at
either end. Its tops are the safer bound for a ceiling: library +1,145
against +1,000 here, and core +25 against +5.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §4, the line-window pass: the links' sequencer | 6 | 0 |
| §5, origin.rs: an own pass's reads, the door's "unasked" | 7 | 0 |
| §1, §2, §3 | 0 | 0 |
| **Total cut here** | **13** | **0** |

`python3 scripts/lean_check.py` before (`data/lean-before.txt`, at
1d355c3d): 18,240 / 18,241 and 3,238 / 3,239. After the cuts: **18,227**
and 3,238 (`data/lean-after.txt`, `data/count-after.txt`). The
repository's cap is lowered to one past where the cuts land, **18,228**
(85a1817b). svrngs-core's stays at 3,239, already one past. A plant at
the lowered caps, one counted line in scripts/ and one in svrngs-core,
was refused by both caps with exit 1, and the tree passed again once the
plants were removed (`data/ratchet-plant.txt`). The three cuts change no
test file: `git diff --stat 1d355c3d 85a1817b` names invite.rs,
origin.rs and lean_check.py alone.

### Open, for the director or the operator

- **`Capability::Read` is unwired (principle 8).** The carried route
  decides what an app reads with its own closure (carried.rs:317-318),
  and `Grant::may` decides the same `<kind>/` scope for appends
  (grant.rs:69-70), so one scope has two deciders. The authorship design
  meant one decider (`demo/runs/authorship/2026-10-24/DESIGN.md:172-175`).
  Wiring it is a change of decider, not of behaviour, and none of this
  queue's rows touches it. A row of its own, or PLAN §2's "capability
  core" (PLAN.md:159), which already names Read among the capabilities
  one `may` would decide.

The number to ask with: **+1,005 repository and +5 svrngs-core over the
caps this pass leaves, at most 19,233 and 3,244**, each row raising only
what it lands. That is this estimate's top. GIVE.md §9's top, +1,170 and
+25 (at most 19,398 and 3,264), is the larger bound if the operator would
rather leave room for its figures (NEEDS_HUMAN.md).
