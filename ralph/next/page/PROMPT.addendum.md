<!-- section: page after=intro -->
This queue builds a second UI for both pages, v2, beside the one that is
live. v2 is Preact 11.1.1 and htm 3.1.1, vendored with no build step, with
one hand-written machine per page. The browser tests run against it by one
switch. Once the whole gate passes on v2, one commit makes v2 the pages and
deletes v1.

Before a row, read:
- this queue's `CHARTER.md`, whole: what is settled, the parts, and who owns
  each file;
- `STATE.md`'s opening;
- svrn-docs `rings/current/PAGE_STATE.md` §3, the rules.

Fences the rows keep:

1. **Red first** (AGENTS.md 2, 5). Each row's first test fails at HEAD.
   - For a move, that means a v1 test run with `SVRNGS_UI=v2` against the
     stub, which fails.
   - For a fix, it means a test in `crates/svrngs/tests/page_v2/`, watched
     failing with its `UI` flipped to `Ui::V1`. The commit names that run.
   - Every new check names its plant in the commit.
2. **v1 is frozen until page-switchover.** That covers `web/index.html`,
   `app.js`, `sw.js`, `store.js`, `open.html`, `open.js`, `lend.js` and
   `shim.js`, and `home/index.html`, `home.js`, `question.js`, `look.css`
   and `away.html`.
   - `web/tests/v1.test.mjs` pins each one's sha256.
   - The only exception is a fix the gate needs on v1. Its pin is updated
     in the same commit, which says why, and the same fix lands in v2 in
     the same row.
   - Any other edit to v1 is NEEDS_HUMAN.
   - `scripts/build-web.sh` with no `--ui` builds the same bytes as before.
3. **What the tests pin stays byte for byte.** That is words, ids, classes
   and element kinds (CHARTER, "The parts"; components.md §0).
   - A move keeps v1's behavior, even its faults, such as a second
     " Copied.".
   - Each change a person can see is its own commit after the move, with
     a v2-only test (DESIGN §4).
   - No v1 test file changes, except `web/tests/home.mjs` and `tabs.mjs`
     in their own rows.
4. **PAGE_STATE §3 from the first line.**
   - One store per page. Views read it and `send`, and never write state or
     the DOM.
   - `step(s, m)` is pure and synchronous. It reads no clock, DOM or
     storage.
   - Every effect is data. It is run at the edge and answers one message,
     done or failed.
   - Each region is a closed set, and a message a state does not take is
     refused with why.
   - Each read is counted with `ask`, and `late` refuses an older answer by
     name.
   - A page reads again after its own write and on `visibilitychange`.
     Nothing reloads to redraw.
   - The key never enters state: it is held by the io (phone.md).
5. **One URL per module.** Every import into `vendor/`, `ui/`, `v2/` or
   `question.js` carries `?v=__VERSION__`. Only `ui/view.js` imports
   `vendor/`. `web/tests/specifiers.test.mjs` holds this.
6. **The policy and the trust floor do not move.**
   - No `style=`, no inline script, no importmap, no `eval` and no
     `Function(`. A rule in look.css stands in for a style object.
   - `web/v2/body.html` holds no `<head>` of its own.
   - A diff to `scripts/trust_floor.txt` is NEEDS_HUMAN.
7. **The code names no app** (`scripts/vocab_check.py:62` reads
   `crates/*/src/**/*.js`).
   - A machine's input is `m`, with `m.type`.
   - Check the list before naming anything. Never `event`, `listing`,
     `tap`, `piece`, `guests` or `hop`.
   - States are named such as `reading`, `read` and `unread`.
8. **Touch only the files your row owns** (CHARTER, "Who owns each file").
   - A primitive you need that `ui/parts.js` lacks goes in your own flow
     file. page-components folds it in.
   - A message to another flow goes through the frame (`{run: 'send', m}`)
     or GROUP's `again`, never by writing its region.
   - A stub you depend on answers `no(s, 'not built yet: <flow>')`, so you
     write your sender against it.
9. **Glassbox** (AGENTS.md 1). Every step and every refusal goes through
   `ui/machine.js`'s one trace format to `window.__lines`. A failure a
   person sees is said in words they can act on (svrn-docs `VOICE.md`,
   American spelling), never a quiet idle.
10. **Four verdicts** (AGENTS.md 6). `scripts/parity.sh` writes passed,
    failed, could-not-judge and never-ran per test. A test that passes on
    the empty skeleton is listed apart in the baseline, and is never
    counted as parity.
11. **The budget.**
    - Each row reports two numbers, the `v2` rung's and `.`'s, against its
      range. A row past its range stops.
    - A worker's edit of `BUDGET` is refused as a CI bypass (give-tcp-1.out).
      So commit the row's work, write the raise as a ready diff in
      `NEEDS_HUMAN.md`, and stop. The operator's session lands it.
12. **Browsers and the gate lock** (c4a294fc; give's fence 13).
    - The browser tests share one Chromium and one WebKit: a computer's
      page through `in_chromium!`, a phone's through `in_webkit!`
      (`harness.rs:44`, `:59`). Any new spawn of a driver passes
      `.env("SVRNGS_BROWSERS", harness::browsers())`.
    - Every run that starts a browser takes the host's one lock. That
      includes `scripts/check.sh`, `scripts/parity.sh`, and a `cargo test`
      of `browser`, `join_page`, `scenes`, `conformance`, `page_v2`,
      `home_group` or `leave_page`.
    - Run each as `lockf -k /tmp/svrngs-gate.lock <cmd>`. Close with
      `ralph-result await <budget> -- lockf -k /tmp/svrngs-gate.lock bash
      scripts/check.sh`.
    - In a lane, the lock is the same file, so lanes wait on each other.
      Two browsers at once crashed this Mac's pasteboard daemon.
13. **In a lane** (if the queue runs as a pool), you work in
    `~/dev/svrngs-lanes/<row>` on `ralph/<row>`.
    - Commit there. The loop merges into `main`.
    - Never merge, rebase or push yourself.
    - A merge the loop left in progress is resolved first, each side's
      behavior kept.
14. **Scratch stays under `target/`.** A gate's result needs no commit of
    its own: the await log keeps it.
15. **Nothing outward-facing.** Deploying the page, a release, and pushing
    are the operator's.
