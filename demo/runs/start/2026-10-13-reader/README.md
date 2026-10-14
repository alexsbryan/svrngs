# A reader with only the document — 2026-10-13

Row start-doc (`ralph/next/start/STATE.md`), its check: a session with no
context but `demo/start.md` follows it on this host, from a clean data dir
to an answer from the friend's model; every place it stalled fixed in the
document or written as a follow-up. One reader, one host: not a rate
(AGENTS.md 7), and a program, not a person: no people bar is judged.

## 1. Setup

- **The page** built by `W=/tmp/start-reader python3
  ../2026-10-13-story/run.py build` at d7cf438 (the code at start-doc's
  commits is the same: they add documents and this harness), served by
  `run.sh` with `web/tests/serve.mjs`, as the document's "Before the page is
  published" says.
- **The reader**: `claude -p` started in a scratch dir outside the
  repository, so no project instructions reach it, given `PROMPT.md` and
  nothing else: the document's path, the page's address, Playwright's
  Chromium for a screen, a stub `open` that writes down the address and
  opens no window (so nothing opens on this host's screen), a Python with
  `openai` for step 9, and notes to keep with each stall marked **STALL**.
  It plays both friends, as the document's "Trying both friends on one
  computer" says, starting llama-server itself by step 1's line.
- **Run**: `W=/tmp/start-reader sh run.sh`; data in `data/`: `start.json`,
  `end.json`, `NOTES.md` (the reader's), `transcript.jsonl.gz` (its every
  call and answer), the group's token redacted in both.

## 2. Predictions

1. The reader reaches an answer from Sam's model in Bo's chat page, and
   Bo's program is answered: `REACHED: yes`.
2. It stalls at the one-computer section at least once: that section is
   the document's least traced to a person's path (the run's driver did
   it, not a person), and takes three terminal windows a session has not.
3. It does not stall at the page's red words and hidden boxes (steps 2 and
   6): the document says to leave them.
4. It meets the password question at step 5 and presses Enter.

## 3. What the run gave

`W=/tmp/start-reader PY=/tmp/sm-py/bin/python sh run.sh` at f000fcb,
exit 0 in 241 s (`data/end.json`). The reader's last line: `REACHED: yes`
(`data/NOTES.md`). From llama-server started (19:08:08) to Bo's program
answered (19:10:45), about 2.6 minutes at a program's pace; Sam's part,
from the page opened to the lend listed, about a minute; Bo's, from the
link opened to the chat's answer, about 40 s.

**Predictions.**

1. **Reached: right.** "Three rivers in Spain are the Guadalquivir, …" in
   llama-server's chat page on Bo's home page's tab, first words 0.08 s
   after Enter; the program streamed an answer at the copied address in
   2.3 s with its Python's start (`NOTES.md`, steps 8 and 9).
2. **A stall at the one-computer section: wrong.** The reader did its
   three windows as separate shells, each line starting with its own
   `export HOME=…`, the bootout and Sam's svrngs in window 1, and the
   clean-up, without a stall there. The one stall it did meet there is
   step 7's name (below), which that section caused.
3. **No stall at the red words and hidden boxes: right.** It left them,
   as told, at steps 2 and 6; at step 6 the red words did not show.
4. **The password question met, Enter pressed: right**, and a stall: the
   question came before "Lent llama-server …", not last as the document
   showed.

**Stalls, each fixed in the document** (099221d):

1. *Step 5, the password question first.* That is svrngs's order:
   `crates/svrngs/src/cli.rs:798` asks it before `:808` prints what it
   lent. start-run's data kept stdout and stderr apart, and the document
   was written from them in that order. Step 5 now shows the question,
   Enter, then the lines.
2. *Step 7, "as Alex Home", not "as Bo".* With no `--as`, svrngs names a
   joiner by the account's full name, else `$USER` (`cli.rs:1626`); on one
   computer both friends are one account. start-run set `USER=Bo`
   (`../2026-10-13-story/run.py:78`); the one-computer section now exports
   `USER=Sam` and `USER=Bo`, and says why.

**Doubts the reader noted, not stalls; follow-ups, not fixed here:**

- Sam's home page, **Done in a terminal**, gives `svrngs share <the
  number …>` with no `--kind`
  (`crates/svrngs-node/src/home/index.html:24`), which svrngs refuses for
  llama-server's 18089; the document is what adds it. start-run met the
  same (its README, Sam lends). Fence 3 asks the page for the one line to
  type: the page's line should be one that works.
- Sam's svrngs, run in a terminal, says "Apps on this computer reach your
  groups at http://127.0.0.1:64333", an address a newcomer might give a
  program, which the document's troubleshooting says is refused. Not
  tried.
- The address **Copy its address for a program** copies has no `/v1`;
  Python's `openai` answered with it as it is. Whether programs that want
  a base URL ending in `/v1` do is unread.
- Bo's page reached an outside relay (`usw1-1.relay.n0.iroh.link`) to ask
  Sam's computer what the link is for; the document says no account is
  made, and says nothing of what leaves the computer. The own relay is
  built and parked until the operator flips it (memory: own relay,
  parked); the document should say what a link's page reaches once that
  is decided.

One reader, one host, a program and not a person: no people bar is
judged, and the document's bars stay the operator's to set (DESIGN §8).
