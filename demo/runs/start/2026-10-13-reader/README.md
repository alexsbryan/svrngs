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

(Written from `data/` after the run.)
