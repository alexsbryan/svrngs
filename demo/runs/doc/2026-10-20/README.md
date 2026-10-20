# The doc, lent — 2026-10-20

`apps/doc` lent from Alex's computer with the published V1.1.2, opened by
two phones through the published page and by a second computer. Then
Alex's lid, as SIGSTOP, for 45 s. The setup and the bars are in
`PREDICTIONS.md`, committed (d7044f9e) before the driver (3fee02f4) and
before any data. Every person is a program, so no people bar is judged
(AGENTS.md 7). This is one take, a spread rather than a rate.

## How it ran

```
PLAYWRIGHT=<web/node_modules/playwright/index.mjs> node run.mjs <data dir>   # from the Mac
node judge.mjs <data dir>                                                   # its verdicts.json
```

- **Fedora** (32 cores, 1-minute load 2.7):
  - Alex's computer, with the published Linux download (binary
    `850005d5…`, the tarball `50b5f0b6…` as V1.1.2's sums list it);
  - the doc's server, at aa4b0093, `--svrngs` naming Alex's api file;
  - Cy's computer, in Chromium at its own origin;
  - Di, in Chromium as a Pixel 7.
- **The Mac:** Bo, in WebKit as an iPhone 13. The Mac's 1-minute load was
  118 at the start and 60 at the end, from other work on its 12 cores.
- **The page:** https://group.svrnme.sh as served then (`index.html`
  sha256 `ac15e0ef…`, `app.js?v=1be454a5db58`), which is V1.1.2's.
  V1.1.3 was not out.
- **The clocks:** Fedora's clock read 7 ms ahead of the Mac's over a 5 ms
  round trip. Waits on Fedora are moved by that much.
- **The app was at aa4b0093, not e6da8ce6** as the predictions name. A
  trial take at e6da8ce6 failed bars 7 and 8: after the lid, neither
  phone came back. `found/` holds the reproduction and its trace.
  aa4b0093 is the change it led to; the bars did not change.

## Against the predictions

The verdicts are in `data/verdicts.json`, and what each bar read is in
`data/summary.json`.

| # | Bar | Read | Verdict |
|---|---|---|---|
| 1 | Opens on each phone from the published page | Bo 2,247 ms and Di 2,461 ms from pressing "doc" to an editor; Cy 334 ms at its origin | passed |
| 2 | Named as svrngs named them | "You’re Bo." / "You’re Cy." / "You’re Di." and "Here now: Bo, Cy, Di" on all three, each within 46 ms of being looked for once all three editors were ready | passed |
| 3 | Live | three edits typed at once; on each page the other two were there 96, 117 and 116 ms after their last keystroke | passed |
| 4 | Each paragraph names its last editor | on all three pages each note sits beside its paragraph and names Bo, Cy or Di as its writer, within 70 ms | passed |
| 5 | Goodnight | "Goodnight from Alex’s doc" 4,579 ms (Bo), 4,645 ms (Cy) and 4,645 ms (Di) after the lid closed | passed |
| 6 | Writing goes on | each page's edit under the goodnight in its editor, and "What you’ve written since is kept here." | passed |
| 7 | It joins | goodnight gone 3,679 ms (Bo), 72 ms (Cy) and 332 ms (Di) after the lid opened; all six edits on all three pages, one doc, by 3,731 ms | passed |
| 8 | Nothing lost | a new page on Cy's computer opened in 180 ms and held all six edits 7 ms later; `acts.jsonl` holds 56 acts, by Bo, Cy and Di | passed |

The goodnight comes before 5 s (`QUIET_MS`) because the 5 s counts from
the last HERE each page heard, up to a second before the lid.

## What it found

- **On a phone, the lend's worker keeps a dead connection for sockets**
  (`found/`). After the lid, every WebSocket the doc's page opened failed
  "no stream could be opened to the computer: timed out", before and
  after the lid opened, and the worker never dialled again. svrngs's
  `web/lend.js` `socket` keeps a dial that failed it (:572-578, at
  v1.1.2 and on main), where `fetch` drops one (:337-342). One plain
  request from the page made the worker dial again, and the next socket
  was answered. The doc now asks once before each new socket (aa4b0093).
  The fix belongs in `web/lend.js`, and it is the operator's to
  schedule.
- **The driver wrote each of Bo's screenshots into `events.jsonl`, byte by
  byte** (people.mjs `shot` answers the image). Those four lines'
  bytes are cut from the file kept here, each marked `bytes_cut`. The
  pictures are in `data/shots/`.

## The pictures

- `bo-open.png`: Bo's phone with the doc open.
- `bo-written.png`, `di-written.png`, `cy-written.png`: after the three edits.
- `bo-goodnight.png`, `di-goodnight.png`: the goodnight, with an edit made
  under it.
- `bo-joined.png`: after the lid opened.

## What this does not show

- **A real phone.** Playwright's WebKit is iOS Safari's engine, not iOS.
  A locked phone, a tab in the background, iOS's limits on service
  workers and Slack's in-app browser are not covered.
- **A real lid.** SIGSTOP freezes the processes, but a sleeping laptop
  also drops its network. The page reads silence either way.
- **The venue's network, and twenty people.** There were three pages and
  one server, on one home network, every phone through n0's relay.
