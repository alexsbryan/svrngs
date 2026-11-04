# What the product carries that it does not need — oidc-economize's survey, 2026-10-21

Read-only until the cut commits, at main `a167c203`. Every count is
`scripts/lean_check.py`'s rule, per file by `scripts/count.py` here
(guests-economize's, unchanged), agreeing with its total: **16,927 /
16,928**, and svrngs-core **3,072 / 3,073** (`python3 scripts/lean_check.py`
at a167c203). One counted line is left under each cap. The oidc queue's
ceiling is the operator's to give after this pass (CHARTER.md, "Leave for
the operator"; AGENTS.md "Done"); §6 gives the number to ask with.

The instruments are guests-economize's (`../2026-10-19-guests/SURVEY.md`):
the compiler with every `pub` demoted, the clone report, and the open
findings that name a thing said twice. The scripts here are that survey's,
copied unchanged but for loop.sh's scratch paths (/tmp/oidc-econ-*). The
builds ran with `TD=/Users/alexsbryan/dev/svrngs/target` (and `web/target`
for the page), so the scratch clone reused the checkout's built
dependencies: the disk had 147 GB free at the start.

## What changed since leave-economize's survey

`data/delta-39f39a59.txt` (`scripts/delta.py` over `count.py` at
`39f39a59`, the commit leave-economize's survey read, and at `a167c203`):
16,906 → 16,927, **+21**, svrngs-core 3,075 → 3,072, **−3**. Every line of
it is the leave queue's own rows and REVIEW-audit-leave-auto-1 read them
already (`ralph/REVIEW_FINDINGS.md`, "(1) Net lines per unit"):

- +15 `crates/svrngs-node/src/removal.rs` (`told`, `tell`'s one selection);
- +9 `crates/svrngs-node/src/ring_routes.rs` (`told_with`, `told_cut`);
- −3 `crates/svrngs-core/src/membership.rs` (leave-economize's cut,
  `Membership::visiting`).

The other changed files since `39f39a59` count nothing: `ring_sync.rs`'s
4 lines are a doc and a call, `scripts/lean_check.py`'s 7 the budget
comment, and `crates/svrngs-node/tests/removal.rs` +316 is test code.

So this survey's delta is the one REVIEW-audit-leave-auto-1 read, and its
finding #218 is a behaviour the charter leaves to the operator ("anything
beyond the record"), not a pass's cut. Nothing else is new.

## 1. The compiler: nothing new

In a scratch clone of `a167c203` (/tmp/oidc-econ-scratch),
`scripts/demote.py` made every `pub` in every non-test source of the four
crates and `web/src` `pub(crate)` (69 files). `scripts/loop.sh` then built
the product (`cargo build --workspace --offline`) and `scripts/repub.py`
made `pub` again each item a downstream crate was refused, round after
round. Repub.py did not parse the E0451/E0616 fields and the "type X is
private"/"method X is private" messages, so `scripts/refix.py` (this
survey's, kept so the pub set is reproducible) made those `pub` from
cargo's errors. Hand edits covered the re-export chains repub.py cannot
follow:

- `standing_reads` (membership.rs:789; E0603 through `pub use
  membership::*`, which repub.py does not follow, as in earlier surveys);
- `NotASyncPath` (synced.rs; E0446, as before);
- `Author`'s five fields (command.rs:58-62; E0451, whose "defined here"
  child made repub.py treat it as already handled);
- `RelayConfig.relay_urls` (guest.rs:44) and the `pub(crate) use` lines
  naming `RelayConfig`/`RelayUrl`/`Name`/`Seed` (iroh.rs:53, :60,
  group.rs:39, :41), whose definitions then had to follow;
- `Invites`/`Invited`/`Visit`/`Wire`'s definitions, which E0365 refused
  once their re-exports were `pub`;
- `pub(crate) mod cli` (svrngs/src/lib.rs:7), which the binary's `main.rs`
  imports across the crate boundary.

The product ended `build-finished success: True`, and `web/` for
`wasm32-unknown-unknown` on its first round (0 errors, 0 fixed). The
scratch diff is `data/demoted.diff`; what still warns is
`data/warnings-prod.txt` and `data/warnings-web.txt`.

Against leave-economize's lists, the lines that say an item is unreachable
are **the same four, and nothing new**:

- `LINK_WITHIN` (dial.rs:46), which the page compiles;
- `iroh::ALPN`'s re-export (iroh.rs:48) and `MemoryLookup` (iroh.rs:64),
  each read and kept by earlier surveys;
- `At::Here` (relay.rs:63), how a test runs `svrngs relay`.

`data/warnings-prod.txt` differs from leave-economize's file in four
`unused import` lines (`project::Invites`, `project::Wire`, `synced::Entry`
and, one fewer, `sync::NO_BUDGET`) and one `more private` line. Those are
this pipe's own over-republications: the generic fixer made a re-export
`pub` when an error named its item, though the product reaches the item
through its module. They are not product findings, and the items are used
(`git grep -n "invites()\|Wire\|Entry\|NO_BUDGET" -- crates`).

`scripts/cfgtest.py` finds the same three lines in counted files as
leave-economize did — `node.rs:202-203` and `:388-389` (`Node`'s `routes`)
and `:958-959` (`pub use tests_support::splice`, #7) — 3 lines, and nothing
new (`data/cfgtest.txt`).

`scripts/jsnames.py` over the page's JavaScript (13 files,
`data/jsnames.txt`): every top-level name is said again. Its plant, a
function appended to web/app.js and never called, went red:
`web/app.js: plantedNeverCalled said 1`; with the file restored, green
(`data/jsnames-plant.txt`).

## 2. What no check, test or person runs: nothing

`git diff --name-status 39f39a59 a167c203 -- scripts` changes one file,
scripts/lean_check.py, whose 7 lines are the budget comment. No script was
added.

## 3. The clone report: nothing new

REVIEW-audit-leave-auto-1 ran this instrument over `123ba07b`'s crates/,
web/ and scripts/ (`ralph/REVIEW_FINDINGS.md`, its "(2) Clones"): `code
index` over a `git archive` (3090 chunks), `refresh` (rust 18575 symbols;
typescript and python "skipped (export failed)", I4), `code dry-report`
per crate and scripts/ at 0.95 and 0.90. Every side blames to a commit
before its range. The counted product at `a167c203` is `123ba07b`'s, line
for line: `git diff --name-status 123ba07b a167c203 -- crates web scripts
Cargo.toml Cargo.lock` names one test file (tests/removal.rs, d1ab28b9),
and `scripts/lean_check.py`'s only change after the audit is the budget
comment; the range touched nothing in web/ or apps/ (empty). A second
index would read the same bytes, so it was not run. Its product clusters,
which guests-economize read and kept: accessors on different types, test
code, and one function inside its neighbour (I32). None is a cut.

## 4. Said twice: the open findings

Open since leave-economize's survey: #212, #213, #215, #216, #217, #218
(`ralph/REVIEW_FINDINGS.md`). Only #217 is principle 8, and it names test
code alone (tests/browser/main.rs, tests/browser/guests.rs,
tests/join_page.rs, web/tests/), which the budget does not count: not
product, so not a cut here. #212 (a round not woken), #213 (a visit past a
bound said `not_a_member`), #215 (a visit's key admitted), #216 (a trace's
words) and #218 (what a removed computer is sent) change behaviour, so are
no pass's. The older ones stand as the last survey read them: #185 (one
helper for how a day is said costs 2 more than it saves), #186 and #150
(apps/, not counted), #202 and #204 (with #107), the operator's.

**#107 is closed in the code since that list:** `dfa0b31b` (lean-4 (a))
made one `declared::write_whole`, and api.rs:76, lend.rs:516,
choices.rs:182, synced.rs:130 and declared.rs:134 all call it. That is the
function oidc-token's key file will write through, with `harden`
(declared.rs:142) for 0600: the key row adds no fourth home, and this pass
cuts nothing there because nothing is duplicated now.

## 5. What the next rows will touch

So no cut is written twice. Each was read at a167c203.

- **`door.rs` `lend_for` (:1114-1190) and `forward`'s identity (:1395-1440).**
  oidc-authorize puts the reserved `/.svrngs/oidc/authorize` branch after
  the asker is known and before `choices::check`, `lend.refusing` and the
  limit, as the carried apps' `Source::Carried` branch is (:1171-1178);
  oidc-token's claims are the four headers `forward` already builds. Nothing
  here is dead or duplicated: `Asker` already carries `person`, `host`
  (guest) and `device`, and `user_name` (:155-161) and the guest's `told`
  (:1414-1416) are the one place each is said. Cutting either ahead of the
  rows would decide the claims' shape for them.
- **`api.rs` `router` (:87-108), `not_from_a_page` (:133-150),
  `from_this_computer` (:158-186).** oidc-provider merges four routes into
  this one router, behind the two layers it already has; the four routes
  need no new listener and no policy change. Nothing to cut: the router is
  22 lines and every route is used.
- **`lend.rs`'s declaration (`LendFile` :260-292, `declare` :449-480,
  `SecretAndSignIn` :358).** oidc-turn-on adds the sign-in alternative and
  its one-of check. `Lend.sign_in` (:214) and the secret are the two
  existing alternatives; a cut now would be rewritten by that row.
- **`home.rs` `/lent` (:224-228), `lend_one` (:320-380), `running`
  (:282-303), `shown` (:258-262).** oidc-turn-on adds the button's JSON and
  the words. `lend_one`'s `Share { .. }` with `sign_in: None` and
  `asks: None` is the declaration's shape, not duplication; the row adds a
  field to it.
- **`well_known.txt`'s `helper jellyfin 8097` (:33) and `share.rs`
  `helper_of`/`HELPERS`.** oidc-retire removes that one line; a kind with
  no helper answers as one with none. Cutting the helper mechanism now
  would change behaviour (today Jellyfin is offered its helper) and the
  design keeps the hook (SIGN_IN_WITH_SVRNGS §5, "The hook is general"), so
  it is the retirement's, not this pass's.
- **`door/sign_in.rs` (420 counted).** The hook the Jellyfin program sits
  behind. After oidc-retire the only helper in the tree is gone, so the
  product would carry a hook with no user; whether the hook itself retires
  is a change of direction (DESIGN §3, "The room", call 6 keeps sign-in
  with svrngs *beside* the hook) and changes behaviour, so it is the
  operator's and the retirement row's, not this pass's. Named here so the
  ceiling conversation sees it: retiring it would cut up to 420 counted
  lines, and it is the one large cut this queue could make later.
- **`apps/jellyfin/mw` (1,571 lines of Rust).** Not
  counted (`apps/` is outside the budget) and oidc-retire's to remove.

## 6. The provider's part, so the ceiling has a number

An estimate, not a measurement: each part's range is grounded on the
nearest counted code that does the same kind of thing at a167c203.

| row | part | lines | grounded on |
|---|---|---:|---|
| oidc-token | core: claims, their JSON, base64url, the signing input | 70–90 | `link.rs`'s `Token::encode`/`decode` (34 counted, base64url already in the core) + a claims struct |
| oidc-token | node: key at 0600, ES256 through `ring`, JWKS, `kid` | 110–130 | `declared.rs`'s write path + `harden` (~45); `sign_in.rs`'s answer parse (~60) |
| oidc-provider | code store, four routes, client auth, PKCE, the one decider | 290–330 | `redeem.rs` 297 counted for two routes and their refusal set; `api.rs`'s router 22 |
| oidc-authorize | the door's reserved path and its eight checks | 140–170 | `door.rs`'s carried branch + `choices::check` (~90); `lend_one`'s parse-and-refuse (~50) |
| oidc-turn-on | declaration field, client id/secret, page, `share` flag | 140–170 | `home.rs` `lend_one` 70; `share.rs`'s flags ~40; the page ~45 |
| **total** | | **750–890**, svrngs-core **70–90** | |

The number to ask with: **+800 counted lines, of which +80 in
svrngs-core**, each row raising only what it lands (CHARTER.md's "smaller
reversible step"). The range's top is generous on purpose: the two big
rows are routes and a page, and the lower end assumes the access token is
the signed ID token too rather than a second shape.

Two facts for the ask. The tree is at 16,927 / 16,928 and svrngs-core at
3,072 / 3,073, so one counted line is left under each cap and oidc-token
(b) cannot land any product line without the raise (its (a) test is test
code and can). And the retirement owes no counted cut as the design
stands: `apps/` is outside the budget, `well_known.txt` is not counted,
and the hook stays (`sign_in.rs`, §5 above, up to 420 lines) unless the
operator decides otherwise.

## What the survey finds

| Found | Lines | svrngs-core |
|---|---:|---:|
| §1, the compiler: the same four items, all read and kept | 0 | 0 |
| §2, §3, §4 | 0 | 0 |
| §5, the sites the queue touches | 0, left to their rows | 0 |
| **Total cut here** | **0** | **0** |

No cut. The caps stay at **16,928** and **3,073** (no ratchet commit: the
row's (c) lowers them only where a cut lands). The plant at those caps
refuses one counted line in scripts/ and one in svrngs-core, exit 1, and
the tree passes again when they are removed (`data/ratchet-plant.txt`).
