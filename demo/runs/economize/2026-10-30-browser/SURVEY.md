# The economizing pass before the browser rows, 2026-10-30

The browser rows (svrn-docs `rings/current/IN_THE_BROWSER.md` §4, §7 and
"Decided": the cold bookmark's way to the group's page, a lend's readable
alias, the address to keep and "How this reaches you") add to the page's
JavaScript (`web/app.js`), the pages' one wording
(`crates/svrngs-node/src/home/question.js`), the home page
(`crates/svrngs-node/src/home/home.js`) and the page's wasm
(`web/src/lib.rs`). At `4909b3a9` the repository stood at **19,509 of
19,510** counted lines (`python3 scripts/lean_check.py`), one line left. So
the pass came first (AGENTS.md "Done"), over the files those rows touch and
the store they call.

The instruments are give-economize's (`../2026-10-25-give/SURVEY.md`),
five days old, run again from there and not copied: `scripts/count.py 40`
(its total agreed with lean_check's, 19,509), `scripts/jsnames.py` ("13
files, every top-level name is said again") and `scripts/windows.py 2` and
`3` over `web/*.js`, `crates/svrngs-node/src/home/*.js`, `web/src/*.rs` and
`scripts/*`. The compiler's dead-code pass and the clone report were not run
again: the rows since give-economize touched JavaScript and shell, which
neither reads (give-economize §1, §3: "could-not-judge (no SCIP for
JavaScript)").

## What the instruments found

- `windows.py 2`: the runs said twice are argument lists, `use` lines, a
  script's `import` block and `if __name__` tail, each kept for the reasons
  give-economize §4 gives. `hex` is defined in `web/app.js` and in
  `web/lend.js`, one line each: a page and a worker, which share
  `store.js`; moving it there saves one line and was left, as below.
- `windows.py 3`: nothing.
- `jsnames.py`: nothing defined and never said.

So the cuts are of a kind the instruments do not see: statements the
formatter of an earlier day wrapped over several lines, or a function body
of one binding and one return, each said in one statement as the
surrounding code already says its own. Behaviour is kept in each: the same
calls, in the same order, answering the same values to their callers.

## Cut, behaviour kept

| Where | What | Lines |
|---|---|---:|
| `web/store.js` | `keep`, `keepVisit`, `visitOf`, `groups`, `opened`, `sealed` and `lendDb`, each one statement in place of a binding and its use, as `holder`, `lendKept` and `lendKeep` beside them are written | -21 |
| `web/app.js` | `reach` as one `Promise.race(..).catch(..)`, which traces and refuses in the person's words as the `try` did; `joining` and `taken` one conditional each; `post`'s fetch one call; the copy button's two answers one call; the worker's registration one statement | -32 |
| `crates/svrngs-node/src/home/home.js` | the index's `.catch` that judges the door and refuses again, one expression, as app.js's `lends` writes its own | -3 |

**-56 in all**, 19,509 to 19,453 (`python3 scripts/lean_check.py`).
`oxlint` passed each file (exit 0). One fold was tried and undone: `Said`'s
constructor as one sequence, which oxlint refuses (`constructor-super`).

## Cut later, for the operator's iPhone findings

The deep path's 404.html and the join's Home Screen line (+4) came after
the cap was lowered to 19,489, one past 19,488. Two more cuts, behaviour
kept (78b8669f): `hex` said once, in `web/store.js`, which the page and
the worker both import (-1), and `cold`'s guard traced and refused in one
statement (-3). The rows closed at 19,488 again.

## Not cut

- `hex`, twice (above): at first, as -1 net; cut later (above).
- `web/lend.js`: the four `catch` blocks that drop a lender's dial each
  trace and answer differently (`windows.py 2`'s one run there is their
  first line alike).
- `web/open.js`, `web/sw.js`: each statement already one line, or a
  promise chain whose steps each say a step.
- `web/src/lib.rs`: rustfmt sets its lines (`cargo fmt --manifest-path
  web/Cargo.toml`), so nothing there folds by hand.

## What the rows then spend

Said in each row's commit, counted by lean_check after it; the cap comes
down to one past where the last lands, if it lands under 19,510.

## Cut for the follow-ups, 2026-10-30

Three gaps after the rows (the alias's day the browser's time zone, a
computer's `<label>.localhost` bookmark refused cold, a person in two
groups sent to the last opened) add to `question.js`, `web/app.js` and
`crates/svrngs-node/src/origin.rs`, and the repository stood at 19,488 of
19,489. The instruments again, from give-economize's scripts:
`jsnames.py` ("13 files, every top-level name is said again") and
`windows.py 3` over `crates/svrngs-node/src/**/*.rs`, whose runs said twice
are `use` lines, argument lists and closing braces, as before. So the cuts
are again of a kind they do not see: a function body of a binding or two
and a return, said as one expression, as `cookieFor`, `live` and
`button` beside them are. Behaviour kept in each: the same calls in the
same order, answering the same values.

| Where | What | Lines |
|---|---|---:|
| `crates/svrngs-node/src/home/question.js` | `uses`: the `try` around `choicesOf`, whose functions are async on both pages (app.js `choicesOf`, home.js's), its `.catch` saying the line and answering no choices; the Stop by `el`, as `table`'s buttons are made; `el` one expression | -11 |
| `web/lend.js` | `passed`, `jarOf` (its store asked only when the jar is not held, by `\|\|`), `shim` (its nonce a default parameter) and `opening` each one expression; each called only once the worker runs, never while its module loads | -14 |
| `web/open.js` | `decode` one expression | -3 |

**-28**, 19,488 to 19,460 (`python3 scripts/lean_check.py`); `oxlint`
exit 0 over the tree.
