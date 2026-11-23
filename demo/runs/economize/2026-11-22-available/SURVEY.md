# Economizing pass before the available-by-default release, 2026-11-22

`ralph/next/keeper/STATE.md` row avail-economize (svrn-docs
`rings/current/KEEP_IT_RUNNING_RELEASE.md` at 4f77680, §5 row R3): cuts
with behaviour kept, before the release's envelope (CHARTER, "The
release: available by default") is measured from the counts they leave.

## Scope

R3 names `surface.rs`, `home.rs`, `cli.rs`, `lend.rs`, `uid.rs`,
`door/bound.rs`, `job.rs`, `lent-row.js` and `running-here.js`.
`git fetch origin app-state && git diff --name-only main...origin/app-state`
at app-state's tip 966c0c35 names `surface.rs`, `home.rs`, `cli.rs`,
`lend.rs` and `home/page/running-here.js`, so those five stay out (CHARTER,
"Alongside app-state"). In: `uid.rs`, `door/bound.rs`, `job.rs` and
`home/page/lent-row.js`, under `crates/svrngs-node/src/`.

macOS-only code in `uid.rs` (`mac`, the `cfg(target_os = "macos")`
readers) is left alone, as the last pass left it: this computer has no
Apple target (`rustup target list --installed`: wasm32, x86_64 linux gnu
and musl). `record` is compiled for the tests too, so it is in.

## Baseline

`python3 scripts/lean_check.py` at 4e6ce6da:

```
  the core (svrngs-core)     3,471   most   3,539
  all code but JavaScript   24,655   most  24,655
  JavaScript                 8,668   most   8,688
```

Counted lines per file, by lean_check's rule: bound.rs 237, uid.rs 931,
job.rs 67, lent-row.js 161.

## The rule for a cut

As the last pass's (`2026-11-keeper/SURVEY.md`): a rewrite the formatter
lays out shorter, never a join of lines it would split, counted after
`cargo fmt` (stock rustfmt for svrngs-node, width 100, chains 60) and
`oxfmt`. oxfmt leaves an `html` template's text alone
(`embeddedLanguageFormatting: off`), so a tag laid out by hand there is
joined only where its line stays within oxfmt's `printWidth`, 100, as the
row's other one-line tags are.

## Candidates

| File | Where | What | Lines |
|---|---|---|---:|
| bound.rs | `decided` | its read typed `PortRead` (iroh.rs:317), so its signature fits a line | 4 |
| bound.rs | `udp_port` | its answer typed `PortRead` | 4 |
| bound.rs | `mutated` | the retry as a `while` over busy and tries, 20 at most as before; `LendError` imported | 3 |
| bound.rs | `judged`, `here` | the cgroup taken by `if let`; the stopped read inline (not cut: below) | 4 |
| job.rs | `show` | the answer read as one chain (not cut: below) | 1 |
| uid.rs | `record::cwd` | the path made by `Into` | 2 |
| uid.rs | `record::described` | each address made by `Into` from its tuple | 6 |
| uid.rs | `diag::socket`, `record::socket` | each address by `.ok().map(IpAddr::from)` (not cut: below) | 4 |
| uid.rs | `record::messages`, `linux::procs` | the length widened in `msg`; the pids read in the last chain (not cut: below) | 3 |
| lent-row.js | `Lent` | three acts' messages named, so each act is one line: "Let everyone in", "Sign people in with svrngs", "Save who can open this" | 18 |

Not cut, each tried and laid out no shorter by the formatter:
- bound.rs `judged`'s `if let … else` passes `single_line_if_else_max_width`
  (50), and `here`'s `read.await.unwrap_or(Err(…))` passes the chain width.
- job.rs `show`: `.ok().filter(…)` puts the command's chain on five lines,
  the match's lines saved.
- uid.rs `.ok().map(IpAddr::from)` past the chain width, each arm on three
  lines; `record::socket`'s closure unannotated is wrapped in a block all
  the same; `messages` and `procs` the same.
- job.rs otherwise: `Keep { unit, files }` and `Answer::Job { unit,
  description }` are named first so each literal fits rustfmt's
  `struct_lit_width` (18).

## The cuts

Each its own commit, named by its file (`git log --oneline
4f4cc72c..c3097364`: 0afc0575, 6d862b2c, fbef6af0), its caps lowered to
one past where it lands.

| File | Cut | All but JavaScript | JavaScript |
|---|---:|---:|---:|
| door/bound.rs | 11 (as thought) | 24,644 | 8,668 |
| uid.rs | 8 (`cwd` 2, `described` 6) | 24,636 | 8,668 |
| lent-row.js | 18 (as thought) | 24,636 | 8,650 |

job.rs: none.

## Close

`python3 scripts/lean_check.py` at the last cut, exit 0:

```
  the core (svrngs-core)     3,471   most   3,539
  all code but JavaScript   24,636   most  24,636
  JavaScript                 8,650   most   8,650
```

All but JavaScript −19 (bound.rs 11, uid.rs 8), JavaScript −18. The
release's envelope (at most about +435 and +30 JavaScript, CHARTER "The
release: available by default") is measured from these: 25,071 and 8,680
at most.
