# svrngs js — the pages' JavaScript formatted, typed in JSDoc, linted for structure, and read (operator, 2026-11-02)

`CHARTER.md` holds why, what was measured, what the operator decided, the
shape the code takes, and the open choices with their defaults. Read it
whole before a row.

**Not launched.** The loop starts after HUMAN-js-agreed is ticked, once the
security work stashed on 2026-11-02 has landed on main.

**The demo.** A reader who knows JS/TS opens `web/page/shelf.js`,
`ui/machine.js` and the home page's flows:
- each machine names its states and messages as types;
- `step` is a `switch` the compiler checks is exhaustive;
- each view is a named component with typed props;
- no line is packed.

`scripts/check.sh` holds all of it: `format`, `types` and the lint's
structural rules each fail on a planted file. Every browser test passes
unchanged, so no person sees a change.

The rows run in order, since all of them touch the same files.

- [ ] HUMAN-js-agreed — depends [] — THE OPERATOR (CHARTER's choices answered, 2026-11-02: oxfmt; a `js` rung with no cap until a jubilee at the queue's end; the rest as proposed):
  - confirms the security work is on main.

  The watching session then:
  - lands the `js` rung from the operator seat (CHARTER, "Answered" 2), with its plant: one counted line in a planted `web/page/x.js` moves `js` and leaves `.` unchanged;
  - ticks this row and starts the loop.
- [ ] js-format — depends [HUMAN-js-agreed] — ONE ROW, each part its own commit, and `scripts/check.sh` once, at the close — oxfmt in the gate, then one commit that only formats:
  - **(a) The tool.**
    - oxfmt is pinned exactly in `web/package.json`, and `web/package-lock.json` is updated by `npm install --prefix web`.
    - Its config sits at the root, at width 100 with single quotes, and its ignores match `.oxlintrc.json`'s.
    - Shown on a scratch copy: the htm templates' text comes out byte for byte (CHARTER, "Answered" 1). Otherwise, stop.
    - No step is wired yet, so every commit stays green.
    - oxfmt's check on today's tree is the plant run: it fails, and the commit cites its count of unformatted files.
  - **(b) One commit that only formats, and the step.**
    - oxfmt over every file (a) covers.
    - `check.sh` gains a `format` step, right after `lint`, checking every file it covers and naming each unformatted one.
    - Nothing else goes in the commit.
    - **Its proof of no behavior change:** the pages' laws (`node --test web/tests/*.test.mjs`), `parts.test.mjs`, and the browser tests run under the gate lock. Every one passes unchanged.
    - **Lines:** `js` and `.` from `lean_check` before and after, by directory, as in CHARTER's table. No cap stops it (CHARTER, "Answered" 2).
  - **(c) `.git-blame-ignore-revs`** names (b)'s commit, so blame skips it, and CONTRIBUTING or the README says how to use it.
  - **Plant:** a planted unformatted line in `web/page/shelf.js` fails `format`, naming the file.
  - **Moves:** nothing a person sees. It moves the line count to the code's real size.
- [ ] js-types — depends [js-format] — ONE ROW, each part its own commit, red first, and `scripts/check.sh` once, at the close — the pages' code typed in JSDoc, checked by `tsc --checkJs --strict`:
  - **(a) The checker.**
    - `typescript` is pinned exactly in `web/package.json`.
    - `jsconfig.json` sits at the root and holds:
      - `checkJs`, `strict` and `noEmit`;
      - `noImplicitReturns`, `noFallthroughCasesInSwitch` and `noUncheckedIndexedAccess`;
      - `include` set to the pages' code (CHARTER, "Scope").
    - `check.sh` gains a `types` step.
    - Preact 11.1.1's and htm 3.1.1's `.d.ts` are vendored from the same tarballs (`web/vendor/README.md` gives the commands), with `lean_check`'s skip, per CHARTER "Answered" 3.
    - Red first: the step's first run lists the errors at HEAD, and the commit cites the count.
    - Files that aren't typed yet carry `// @ts-nocheck`, which (b) to (d) remove. A file still carrying one when the row closes fails the row.
  - **(b) The core.** `ui/machine.js` takes a generic `Machine<State, Message>`:
    - `go`, `no`, `ask` and `late` are typed;
    - `assertNever` takes the `never` an exhaustive `switch` leaves;
    - `ui/view.js`'s `useStore` and `useMachine` are typed;
    - `ui/trace.js` is typed.
  - **(c) The phone page,** one commit per machine:
    - its states and messages as discriminated-union `@typedef`s;
    - `step`'s `switch` exhaustive;
    - `io` typed;
    - each component's props a `@typedef`.
  - **(d) The home page,** the same way, one commit per machine.
  - **Plant:** a planted message `{ type: 'nope' }` that no case takes fails `types` at the `assertNever`. A planted wrong prop type fails it too.
  - **Kept:** every runtime byte but `assertNever` and the guards narrowing needs. `git diff` on a typed file shows comments only, apart from those.
- [ ] js-lint — depends [js-types] — ONE ROW, each part its own commit, `scripts/check.sh` once, at the close — the structural rules, each on with its hits fixed in the same commit:
  - **The scope.** `.oxlintrc.json` gains an `overrides` entry for the pages' code (CHARTER, "Scope"). The rest keeps today's rules.
  - **(a) Names:**
    - `id-length` (min 2; `i` and `_` excepted);
    - `no-shadow`.

    The 813 one-letter names become names a reader follows. A rename only: the words a person reads are untouched.
  - **(b) Control flow:**
    - `no-sequences`;
    - `no-nested-ternary`;
    - `no-unused-expressions` (no `a && b()` as a statement);
    - `one-var`;
    - `complexity` at 10;
    - `max-params` at 4;
    - `max-depth` at 3.
  - **(c) Modules:**
    - `import/no-cycle`;
    - `oxc/no-barrel-file`: `ui/kit.js` goes, and each flow imports what it uses, from its module;
    - `import/no-restricted-paths` for CHARTER's layers;
    - `web/tests/specifiers.test.mjs` still passes.
  - **(d) Type-aware,** per CHARTER "Answered" 4, or each rule named never-ran with why.
  - **Plant:** each rule watched failing on a planted file before its fixes. The commit names the rule and the plant.
- [ ] js-read — depends [js-lint] — ONE ROW, each part its own commit, `scripts/check.sh` once, at the close — the pages read as code meant for people, behavior kept:
  - **(a) Survey.** `demo/runs/economize/<date>-js-read/SURVEY.md` lists:
    - each view packed into an expression;
    - each machine not yet in its own module;
    - each comment with no why;
    - each duplicate across the two pages (`sovereign-cli code dry-report`).

    With the cut or move each one takes.
  - **(b) The phone page, then (c) the home page,** one commit per flow:
    - views become named components, one per file;
    - templates are broken up by hand where `embeddedLanguageFormatting: "off"` left them long;
    - the machine goes to its own module;
    - `io` sits at the edge;
    - comments are rewritten to say why.
  - **(d) The rules written down** (AGENTS.md 3): CHARTER's "The shape the code takes" goes into svrn-docs `rings/current/PAGE_STATE.md` §3 and DESIGN §5, rewritten in place.
  - **Kept:** every browser test, the pages' laws and `parts.test.mjs`, unchanged and passing. A change a person would see is NEEDS_HUMAN, not this queue's.
  - **Lines:** `js` and `.` before and after each part, with no cap to stop it.
  - **(e) The jubilee** (CHARTER, "Answered" 2). After the gate passes at the close:
    - a ready diff in `NEEDS_HUMAN.md` setting `js`'s cap to one past where it lands, and `lean_check`'s "no cap" mode removed;
    - the queue's total named: `js` at HUMAN-js-agreed against `js` now, by row;
    - then stop. The watching session lands it.
  - **Moves:** the operator's reading of the pages, the demo above.
