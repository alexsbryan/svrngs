# canon to plain JavaScript — the measure

Row canon-spike (`ralph/next/canon/STATE.md`), against `PREDICTIONS.md`
beside this file, which was committed first (`14bbb4a9`). One host (this
Mac, arm64), one run of each: not a rate (AGENTS.md 7). The build is
`apps/canon/build.sh --measure` at `afd379c9`; its output, `Compiling` lines
cut, is `build.log`. canon is `49eaf31` (release 0.3.0), by `git archive`.

## Each prediction

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| 1 | canon-core builds for `wasm32-unknown-unknown` unchanged | built, no canon file edited; replay.rs compiled is byte-identical to `git show 49eaf31:crates/canon-cli/src/replay.rs` (`cmp`, no output); `build-wasm.log` | held |
| 2 | `wasm2js` refuses the raw module, carries it after wasm-opt's lowerings, no `WebAssembly` at run time | refused: "Fatal: error in validating input", 755 validator errors, every one bulk memory (`wasm2js-raw.log`); after the lowerings the module validates as MVP and `wasm2js -O` exits 0; the JavaScript names no `WebAssembly`, `eval(` or `new Function` (build.sh's grep) | held |
| 3 | 18 of 18 replays equal, every step, in both builds; a planted mismatch caught | 18 of 18 judged, all equal, 541 steps; canon-cli itself "as expected" on the 17 with an expected file; canon's governance bar at the pin, 3 passed, 6 of 8 mechanisms (`build.log`). The plant (one step of canon-cli's answer altered) fails both builds at that step, exit 1 (`plant.log`) | held |
| 4 | wasm 350–700 KB; JavaScript 1–2.5 MB, 200–500 KB gzipped; replay.rs under 15% | wasm 884,288 bytes; JavaScript 5,499,606 bytes, 579,922 gzipped (`gzip -9`); without replay 765,084 and 4,654,949 (503,188 gzipped), so replay.rs adds 15.6% to the wasm and 18.1% to the JavaScript | missed: every size above its range |
| 5 | over 300 acts, median of 20: native < 5 ms, wasm < 3× native, JavaScript 3–10× wasm and < 150 ms | native 2.14 ms, wasm 3.59 ms (1.7×), JavaScript 9.94 ms (2.8× the wasm); the wasm's and the JavaScript's whole answer equal (42,958 bytes) | held, the JavaScript a little faster than the range |
| 6 | canon-core 0.3.0 has no input for an outside roster as standing | read, not run: `derive(acts: &[Act])` (`fold.rs:621`); `seat` (`ratify.rs:543-600`) and `holders_at` (`:608-627`) read `Canon::grants` alone, filled only from `grant` acts | held; see below |

The house of 300 acts (`apps/canon/build/house.mjs`): 12 people, each
granted `house` by the first, `consent:2d` for `house`, then rounds of a
rule proposed and scoped, three agreeing, every fourth objected to, every
fifth superseding an older one, every seventh retracting one. Folded three
days past its last act: 37 in force, 8 refused, 10 superseded, 7 retracted.

Loading the module: `import()` of the 5.5 MB JavaScript under node 20.20.2
took 70–73 ms, five fresh processes (`import.log`).

## Not measured, said

- **WebKit and Chromium.** Every number above is node's (V8). The page's
  own load and fold in both engines is canon-house's page test.
- **The raw module with wasm2js's feature flags.** `wasm2js` with
  `--enable-bulk-memory` and the rest also exits 0 on the unlowered module;
  that route was never compared against canon-cli, so the build does not
  take it.
- **Which lowerings were needed.** The validator stopped at bulk memory;
  whether sign extension and float-to-int were in the module is unread. All
  three run, and the comparison is what vouches for the result.

## What it means for House rules

- **wasm2js carries canon.** Carried pages keep their policy as pinned
  (`scripts/trust_floor.txt`), with no `'wasm-unsafe-eval'`; row (d)'s
  first case does not arise.
- **It is big.** 5.5 MB is one carried file every member's computer holds,
  for each version. From the computer's own page it is a local read; a
  phone fetches it through its computer. 4.65 MB without replay.rs. Neither
  is a decision made here: canon-house ships the build this row vendored,
  the one the proof ran.
- **Standing from the roster is not expressible in canon-core 0.3.0.**
  CANON_IN_SVRNGS §3 and §8 decide that the house's members are its root
  scope's standing, handed to canon as the roster. canon folds standing from
  `grant` acts alone. Writing grants that no member wrote, in someone's
  name, would break "who said it, truly" (the queue's fence 4). It is a
  change to canon-core, canon's repository's: row (d)'s second case.
