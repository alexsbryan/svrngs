You are trying svrngs for the first time. You know nothing about it but one
document: {{DOC}} (and the pictures it links, beside it). Follow it on this
computer, from nothing to an answer from Sam's model, playing both friends
as its section "Trying both friends on one computer" says.

- Read {{DOC}} and the pictures it links. Read no other file of the svrngs
  repository, and look nothing up about svrngs anywhere else: where the
  document leaves you stuck, say so in your notes and try what a careful
  newcomer would.
- The group's page is served already at {{PAGE}}. Use it wherever the document
  says `https://group.svrnme.sh/`, and skip "Before the page is published".
- You have no screen. Where the document says to open a page, look at it or
  press a button, do it in Chromium by Playwright: a Node script of your
  own under {{ME}}, importing `{{NODE_MODULES}}/playwright/index.mjs` by that path.
  Read what the page says and press what a person would press. Keep the
  browser's own state for each friend in a folder of its own under {{ME}}.
- Before you paste a line from a page in a terminal, run
  `export PATH={{STUB}}:$PATH`: there `open` writes the address it was asked
  to open in `$HOME/opened` and opens no window. Open that address
  yourself in Chromium.
- Run what must keep running (llama-server, Sam's svrngs) in the
  background, its output to a file under {{ME}}.
- For step 9's program: a Python with the `openai` package is at {{PY}}.
- Nothing outside this computer: no sudo, nothing published.
- Write your notes as you go to {{ME}}/NOTES.md: each step, what you typed or
  pressed, what you saw, how long it took, and every place you stopped,
  were unsure, or did something the document did not say. Mark those
  places **STALL**.
- Stop when Sam's model has answered Bo in its chat page and Bo's program
  has an answer (step 9), or when you cannot go on. Then clean up as the
  one-computer section's last step says, and end your notes with one line:
  `REACHED: yes` or `REACHED: no — <where it stopped>`.
