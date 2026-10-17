# canon to plain JavaScript — predictions, before any data

Row canon-spike (a) (`ralph/next/canon/STATE.md`; svrn-docs
`rings/current/CANON_IN_SVRNGS.md` §9 step 2, `WHAT_RUNS_AS_YOU.md` §6).
Written and committed before anything was built. Nothing here runs a
person; no people bar is judged (AGENTS.md 7). One host, one run of each:
not a rate.

## The setup

- **canon** is release 0.3.0 at `49eaf31b1e2c6d2f2ca31bee6d220da79e264bda`
  ("contract: adopt 3e5828b… (format v4, release 0.3.0)"), read from the
  checkout at `~/dev/canon` by `git archive` of that commit, so the build
  never sees the checkout's working tree.
- **The wrapper** is `apps/canon/build/`, a crate of its own (`[workspace]`,
  outside the svrngs workspace), depending on canon-core by path into that
  archive. One entry, `call(request) -> response`, JSON in and JSON out:
  `derive`, the ratification rule for a scope, a person's seat in a scope,
  and `replay`. `replay` is canon-cli's own `crates/canon-cli/src/replay.rs`,
  compiled unchanged by `#[path]`, so the governance bar runs through canon's
  own scenario runner and not a second one; the four `crate::` modules it
  names (`cmds`, `store`, `wrap`, `govern`) are shims, reached only from its
  `run`, which the wrapper never calls.
- **Tools.** Rust 1.95.0's `wasm32-unknown-unknown`; binaryen 133's
  `wasm2js` (Homebrew); node 20.20.2.
- **The fixtures.** Every canon fixture with a `scenario.jsonl`: 17
  directories (`fernwood-commons`, `eleven-principles`,
  `maple-house/accepted`, 14 under `cpr/`), and `fernwood-commons`'s second
  scenario `kitchen.jsonl`: 18 replays. canon-cli's answer to each is
  `canon replay <dir> --json`, built from the same archive.
- **The house of a few hundred acts** is generated in canon's seed dialect
  (12 people, grants, rules proposed, positions, supersedes, retracts) at
  300 acts, minted by canon's own loader, then folded by `derive`.

## Predictions

1. **canon-core builds for `wasm32-unknown-unknown` unchanged.** It is
   `no_std + alloc` (`crates/canon-core/src/lib.rs:12`), and its four
   dependencies (serde, serde_json, sha2, ed25519-compact) are pure Rust
   with no `getrandom`, no C and no `wasm-bindgen`. The wrapper, with
   replay.rs and std, builds too, with zero edits to canon's files.
2. **`wasm2js` carries it, after a lowering.** Rust 1.95 emits bulk-memory,
   sign-extension and non-trapping float-to-int for this target by default.
   Predicted: `wasm2js` on the raw `.wasm` either refuses
   `memory.copy`/`memory.fill` or needs `wasm-opt`'s
   `--llvm-memory-copy-fill-lowering`, `--signext-lowering` and
   `--llvm-nontrapping-fptoint-lowering` first; after them it emits one ES
   module that node imports with no `WebAssembly` object touched.
3. **The folds are the same.** All 18 replays give, step by step, the JSON
   canon-cli gives, in the wasm and in the JavaScript: 18 of 18, every step.
   wasm2js lowers i64 to i32 pairs; canon's timestamps are i64 and
   ed25519-compact's field arithmetic is u64, so a mistranslation would show
   as a wrong id or a wrong verdict — predicted none. A planted mismatch
   (one step's answer altered) fails the comparison.
4. **Size.** The release `.wasm` (opt-level `z`, LTO, one codegen unit,
   stripped) at 350–700 KB. The JavaScript 2–4 times that, 1–2.5 MB, and
   200–500 KB gzipped. replay.rs adds under 15% to either.
5. **Time over 300 acts** (median of 20 `derive` calls, warm, this host):
   native canon-core under 5 ms; the wasm under 3 times native; the
   JavaScript 3–10 times the wasm, under 150 ms. A page that folds a house
   on open and after each act stays under a second with room.
6. **Standing from the roster: canon-core 0.3.0 has no input for it.** A
   seat and a scope's holders are read from `Canon::grants` alone
   (`ratify.rs:543-600` `seat`, `:608-627` `holders_at`), which the fold
   fills only from `grant` acts. `derive` takes `&[Act]` and nothing else
   (`fold.rs:621`). So "membership is standing" (CANON_IN_SVRNGS §3, §8)
   cannot be handed to canon as a roster without either a change to
   canon-core or grant acts written in someone's name that nobody wrote —
   the second breaks "who said it, truly". Predicted: row (d)'s second
   case, a NEEDS_HUMAN.md for the operator, canon's repository's.

## What would falsify each

1. a compile error naming a canon-core file, or a canon file edited to build.
2. `wasm2js` failing after the lowerings, or the module needing
   `WebAssembly` at run time.
3. any step's JSON differing between canon-cli and either build.
4. sizes outside the ranges.
5. timings outside the ranges.
6. an input in canon-core 0.3.0 that seats an actor with no `grant` act.
