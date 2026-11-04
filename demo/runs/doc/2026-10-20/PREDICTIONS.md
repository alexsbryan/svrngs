# The doc, lent: predictions — 2026-10-20

Written before the driver and before any data. The evening (the operator,
2026-10-20): Alex lends the doc from their laptop to a room mostly on
phones, and closes the lid at the end; people opening the doc then see
a goodnight, keep writing, and what they wrote joins when Alex's computer
answers again. This run asks that of `apps/doc` at e6da8ce6, lent with
the published V1.1.2, through the published page. Every person is a
program, so no people bar is judged (AGENTS.md 7); one take is a spread,
not a rate.

## 1. Setup

- **Fedora** (`alexbryan@fedora`, x86_64, 32 cores): Alex's computer,
  `svrngs new House --as Alex --origin https://group.svrnme.sh/` with the
  published V1.1.2 Linux download (its binary `850005d5…`,
  `demo/runs/v1.1.2/2026-10-18/README.md`), and the doc's server lent
  from it: `node apps/doc/server.mjs --port <p> --keep <dir> --svrngs <Alex's
  api file>`, `svrngs share <p> --name doc --kind doc < /dev/null`. Cy's
  computer joins by Alex's link and opens the doc in Chromium at its own
  origin. Di is a phone: Chromium as a Pixel 7, opening Alex's link on
  https://group.svrnme.sh, joining as Di and pressing "doc".
- **The Mac**: Bo is a phone, WebKit as an iPhone 13 (iOS Safari's engine),
  the same way. Playwright's WebKit cannot start on Fedora without system
  packages (`npx playwright install webkit`: "Host system is missing
  dependencies"), so Bo's browser is on the Mac, reaching Alex on Fedora
  through n0's relay, as a phone in the room reaches a laptop. The Mac's
  1-minute load is recorded at the start; other work had it near 120 on
  12 cores an hour before.
- **The lid**: SIGSTOP to Alex's svrngs and the doc's server together,
  45 s, then SIGCONT.
- **The clock**: each wait is timed on the machine whose browser waits,
  from the command that starts it; a driver on the Mac sends Fedora's
  commands over SSH.

## 2. The bars

| # | Bar | Pass |
|---|---|---|
| 1 | The doc opens on each phone from the published page | each phone's editor ready within 15 s of pressing "doc" (the bundle is 449,565 bytes over the relay); Cy's within 5 s of opening its origin |
| 2 | Each person is named as svrngs named them | within 5 s of its editor being ready, each page says "You’re <the name it joined as>", and its "Here now" lists Bo, Cy and Di |
| 3 | Writing is live | three edits typed at once, one each by Bo, Cy and Di, each on the other two pages within 2 s of its last keystroke |
| 4 | Each paragraph names its last editor | within 3 s, each page's notes say Bo, Cy and Di against the paragraphs they wrote |
| 5 | Goodnight | with the lid closed, each page says "Goodnight from Alex’s doc" within 7 s (QUIET_MS 5 s, the last HERE at most 1 s before, a tick of 0.5 s) |
| 6 | Writing goes on | on each page, an edit typed under the goodnight is in its editor, and the page says "What you’ve written since is kept here." |
| 7 | It joins | after the lid opens, each page's goodnight is gone and the three pages read one doc holding all six edits within 30 s |
| 8 | Nothing is lost | a new page on Cy's computer after the run reads all six edits, and the server's `acts.jsonl` names Bo, Cy and Di as their writers |

Each bar gets one of passed, failed, could-not-judge or never-ran, in
`data/verdicts.json`, with the numbers it read.

## 3. What this cannot show

- **A real phone.** Playwright's WebKit is iOS Safari's engine, not iOS:
  a locked phone, a backgrounded tab, iOS's own service-worker limits and
  Slack's in-app browser are not in it.
- **A real lid.** SIGSTOP freezes the processes; a sleeping laptop also
  drops its network, and its connections may be reset rather than left
  silent. The page reads silence either way (`apps/doc/sync.js`).
- **The venue's network.** Both machines are on one home network, and
  every phone's way to Alex is n0's relay, as on the evening.
- **Twenty people.** Three pages and one server.
