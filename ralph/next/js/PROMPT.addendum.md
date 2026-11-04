<!-- section: js after=intro -->
This queue makes the pages' JavaScript code that people can read. The
pages' behavior stays as it is. Before a row, read:
- this queue's `CHARTER.md`, whole;
- `STATE.md`'s opening;
- svrn-docs `rings/current/PAGE_STATE.md` §3, the machine's rules, which
  stand.

A row carries its parts as (a), (b), and so on, each its own commit. A row
runs over several sessions:
- commit whole parts;
- end a session with `ralph-result continue`, naming the next part;
- run `scripts/check.sh` once, at the row's close.

Fences the rows keep:

1. **Never meet a budget by density.** The operator, 2026-11-01: the code
   "looks like someone trying to squeeze as much into one line as
   possible". These are refused:
   - joining lines;
   - packing statements with `,` or `&&`;
   - shortening names;
   - a re-export file to save imports;
   - a formatter-ignore comment to pack a line.

   This queue's `js` rung has no cap until the jubilee at its end
   (CHARTER, "Answered" 2). That is no license to pad or to pack: write
   the code the shape asks for, at the size it takes.
2. **No build step** (CHARTER, decision 1).
   - Nothing transforms the bytes that ship.
   - Types live in JSDoc comments.
   - `tsc`, the formatter and oxlint run only in `scripts/check.sh` and by
     hand. None of them is in `build-web.sh`, `deploy-page.sh` or a
     `build.rs`.
   - A `.d.ts` is never served. `build-web.sh` copies named `.js`, and
     `home.rs` serves named `.js`.
3. **Behavior kept, every commit.** These pass unchanged at each commit's
   tests:
   - the browser tests (`crates/svrngs-node/tests/browser`, `join_page`,
     the scenes);
   - the pages' laws (`web/tests/*.test.mjs`);
   - `parts.test.mjs`.

   Those tests pin words, ids, classes and element kinds. No test changes
   to make a commit pass. A rename that a test reads, such as
   `window.__lines` or an id, is not a rename. A change a person would see
   is NEEDS_HUMAN: this queue doesn't make one.
4. **Red first** (AGENTS.md 5). Each new check is watched failing on a
   planted file before it passes, and the commit names the plant.
5. **PAGE_STATE §3 holds:**
   - one store per page;
   - `step(state, message)` is pure and synchronous;
   - effects are data, run at the edge;
   - each region is a closed set;
   - a refusal says why;
   - every step is traced to `window.__lines`.
6. **One URL per module.** Every import into `vendor/`, `ui/`, `page/` or
   `question.js` carries `?v=__VERSION__`
   (`web/tests/specifiers.test.mjs`).
7. **The code names no app** (`scripts/vocab_check.py`). Check its list
   before naming anything. A message is `message`, with `message.type`.
8. **The policy and the trust floor don't move.** Never use:
   - a `style=` attribute;
   - an inline script;
   - an importmap;
   - `eval` or `Function(`.

   A diff to `scripts/trust_floor.txt` is NEEDS_HUMAN.
9. **The budget.** Each part reports `lean_check`'s `js` and `.`, before
   and after.
   - `js` has no cap until js-read (e), the jubilee.
   - `.` keeps its cap. This queue should not move it: a part that does
     says why.
   - A worker's edit of `BUDGET` is refused as a CI bypass. The jubilee's
     cap is a ready diff in `NEEDS_HUMAN.md`, which the watching session
     lands from the operator seat.
10. **Browsers and the gate lock.** Every run that starts a browser, and
    `scripts/check.sh`, runs as `lockf -k /tmp/svrngs-gate.lock <cmd>`.
    Close a row with `ralph-result await <budget> -- lockf -k
    /tmp/svrngs-gate.lock bash scripts/check.sh`. One known flake:
    `leave_race` at `crates/svrngs-node/tests/removal.rs:614`. Rerun that
    one test before calling it a failure.
11. **In a lane,** you work in `~/dev/svrngs-lanes/<row>` on `ralph/<row>`.
    - Commit there, and never merge, rebase or push.
    - The loop merges into `main`.
    - The disk is tight: build only in the lane's `target/`, which the loop
      cloned.
12. **Nothing outward-facing.** Deploying the page, a release, and pushing
    are the operator's.
