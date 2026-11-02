# svrngs js — the pages' JavaScript, written to be read

## Why

The operator, 2026-11-01, after the page switchover's code landed:

> I find the js files to be hideous. I've worked on JS / TS for a long time
> and those files look like they aren't meant to be read by, or understood
> by humans at all. They lack proper abstractions and there's so much
> inline hackery, and it overall just looks like someone trying to squeeze
> as much into one line as possible

**The cause.** `scripts/lean_check.py` counts lines, `:403`, and nothing
formatted the JavaScript. A row up against its cap found that it could join
lines instead of cutting code.
- page-switchover (a) "cut" 65 lines by importing everything in one line
  from `ui/kit.js` (48cabcea).
- page-parity folded `check.sh`'s `v2=` onto the line before it (4d19c945).
- Both were recorded as cuts, and the watching session let them through.

**Measured, 2026-11-02, on main at a625a891.** The probes are in the
watching session's scratchpad, and each row measures again before it acts.
- **Width.** 325 lines in the pages' flows run past 140 characters. The
  average line is 94 characters, and `web/page/enter.js` reaches 310.
- **Prettier 3.6.2 at width 100** over every counted JavaScript file, with
  `lean_check`'s exclusions:

  | | Before | After |
  |---|---|---|
  | `crates/svrngs-node/src/home/` | 575 | 2,505 |
  | `web/page/` | 429 | 1,702 |
  | `web/*.js` | 863 | 1,243 |
  | **Total** | **1,867** | **5,450 (+3,583)** |

  That probe formatted inside the `html` templates too. With
  `embeddedLanguageFormatting: "off"`, js-format (b) measures the real delta.
- **oxlint 1.89.0, with the structural rules below** over the pages' code:
  - 813 `id-length` (`g`, `s`, `m`, `q`);
  - 23 `no-sequences`;
  - 16 `no-nested-ternary`;
  - 10 `no-shadow`;
  - 5 `complexity` over 10;
  - 4 `max-params` over 4;
  - 2 `import/no-cycle`;
  - 1 `oxc/no-barrel-file`, which is `ui/kit.js`.

  `max-lines-per-function` found nothing, because packed code has few lines.
  Only rules about structure, names and types catch it.

## What the operator decided, 2026-11-02

The operator: "Let's write that up for the next queue". That was in answer
to the watching session's recommendation, recorded here whole.

1. **Types in JSDoc, checked by `tsc --checkJs --strict --noEmit`, and no
   build step.**
   - The bytes in the repo stay the bytes the browser runs.
   - `tsc`, the formatter and oxlint are dev tools in `web/package.json`,
     run only by `scripts/check.sh`.
   - The page holds the person's key (DESIGN §3), so no compiler joins
     what has to be trusted for it.
   - The 2026-10-31 decision stands: Preact with htm, vendored, with no
     build step (`ralph/next/page/CHARTER.md:12`).
2. **Lint that judges structure.** Each rule's hits are fixed in the commit
   that turns the rule on.
3. **A formatter in the gate.** Line width stops being the author's choice,
   and `lean_check` counts formatted lines.
4. **A readability pass,** each commit keeping behavior. This queue runs
   after the security work lands on main and before any new page feature.

**Why not TypeScript or TSX** (the watching session to the operator,
2026-11-02). Each would add a transform, and each has these costs:
- `include_str!` builds the home page into the binary, so the transform
  would have to run inside `cargo build`, either through a `build.rs` with
  Node or through oxc's crates. The other choice is committed output, which
  means two copies of every file.
- Node here is v20, so the page tests cannot import `.ts`.
- TSX ends "what you read is what runs", since its output needs source maps.
- The release, built from a detached worktree, gains one more step that can
  fail.

The JSDoc route leaves one gap: what is passed through an htm template is
untyped. The browser tests pin every word, id, class and element kind the
templates draw. Revisit when a bug traces to an untyped template:
TypeScript with its types blanked to spaces (as Node strips them) is the
smallest step that would type them.

## The shape the code takes

These are the rules the rows hold the code to. js-read writes them into
PAGE_STATE §3 and DESIGN §5 (AGENTS.md 3).

- **A page's machine is its own module:**
  - its states and messages, as discriminated-union `@typedef`s;
  - `step`, whose `switch (m.type)` the compiler checks is exhaustive
    (`assertNever` in `ui/machine.js`);
  - the effects it describes, as data.
- **I/O only at the edge module** (`io`), never in a view or a step.
- **One component per file,** named for what it draws, with its props a
  `@typedef`. A view is never an expression packed into an object literal.
- **Layers:**
  - `page/` may import `ui/`;
  - `ui/` never imports `page/`;
  - only `ui/view.js` imports `vendor/`;
  - no re-export-only files.
- **Names a reader follows:** `group`, `state` and `message`, not `g`,
  `s` and `m`. A loop's `i` and a destructured `_` are the exceptions.
- **Comments say why,** in a way someone can check. A row's letter or a
  `file@commit` alone is not a why.
- **The formatter decides how a line looks.** No `prettier-ignore` packs
  a line.

## Scope

- **Typed, linted for structure and read:** the pages' code, which is
  `web/*.js`, `web/page/`, and `crates/svrngs-node/src/home/` with its
  `page/` and `ui/`.
- **Formatted only, with today's lint:** every other JavaScript file oxlint
  reads, such as `web/tests/`, `apps/` and `scripts/`.
- **Untouched:** `web/vendor/`, `demo/runs/` and
  `apps/on-air/vendored.mjs`, as `.oxlintrc.json`'s ignores have it.

## Open, each with the session's default

The operator answers these at HUMAN-js-agreed.
1. **The formatter.** Default: Prettier, pinned exactly in
   `web/package.json`, at width 100, with single quotes and
   `embeddedLanguageFormatting: "off"`. "Off" keeps the htm templates'
   text byte for byte: formatting inside them can move the whitespace a
   test pins (" Copied."). js-read breaks the templates up by hand. The
   alternative is oxfmt, one toolchain with oxlint.
2. **The format's raise.** js-format (b) only reformats, so it adds no
   code, and AGENTS.md "Done" asks for an economizing pass before a raise.
   Default: waive the pass for that one commit, and raise `.` by exactly
   its measured delta, landed from the operator seat beside it. Later rows
   keep the usual rule.
3. **The vendored type declarations.** These are Preact 11.1.1's and htm
   3.1.1's `.d.ts`, from the same tarballs as the vendored code.
   `lean_check` counts `.ts`, so they would count. Default: they sit in
   `web/vendor/` and `lean_check` skips `*.d.ts` there, since they are not
   ours and run nothing. build-web.sh and home.rs serve only named `.js`,
   so they are never served.
4. **Type-aware lint.** Default: oxlint's type-aware mode, if it reads
   `checkJs` JavaScript:
   - `switch-exhaustiveness-check`;
   - `no-floating-promises`;
   - `strict-boolean-expressions`;
   - `no-unnecessary-condition`;
   - `prefer-nullish-coalescing`.

   If it can't read that JavaScript, these rules wait as never-ran, named
   in js-lint's commit, rather than adding ESLint.
5. **Pool or serial.** Default: `pool` with 1 lane, so the main checkout
   stays free for other sessions, as the page queue ran.

## Estimates

Lines are `.`'s count by `lean_check`, against `.` 19,435 of 19,436 at
a625a891. The security work lands first, so each row measures again from
where main is.

| Row | Expected change to `.` |
|---|---|
| js-format | +1,500 to +3,583. The top is the probe with templates formatted; "off" brings it down. Re-measured in (b). |
| js-types | +5 to +20. JSDoc is comments and uncounted (`lean_check.py:403`); only `assertNever` and the narrowing guards count. |
| js-lint | +50 to +150. `ui/kit.js` goes, and its 65 lines of imports by name come back; nested ternaries become statements. |
| js-read | −100 to +300. Components out of expressions add their signatures; folding duplicates cuts. |

Every raise after js-format follows AGENTS.md "Done": an economizing pass,
then a ready diff in `NEEDS_HUMAN.md`, landed from the operator seat.
