# The doc, polished — 2026-10-20

This is the operator's polish pass ("Yes, polish now"), taken once. It is
the story of `../README.md` again:
- two phones and a computer;
- Alex's lid closed with SIGSTOP for 45 s, then opened;
- `apps/doc` at 0f9df88b, with the same machines, binaries and page.

The setup and the bars are in `PREDICTIONS.md`, committed with the driver
(56c7272d) before any data. Every person is a program (AGENTS.md 7), and
one take is a spread, not a rate.

```
PLAYWRIGHT=<web/node_modules/playwright/index.mjs> node run.mjs <data dir>   # from the Mac; agent.mjs is ../'s
node judge.mjs <data dir>
```

The Mac's 1-minute load was 7 at the start and 5 at the end. Fedora's was
1.8. Fedora's clock read 4 ms ahead of the Mac's over a 5 ms round trip.
The page was the one served then (`index.html` `ac15e0ef…`, V1.1.2's).

## Against the predictions

| # | Bar | Read | Verdict |
|---|---|---|---|
| 1 | Opens on each phone from the published page | Bo 2,303 ms and Di 2,252 ms from pressing "doc"; Cy 167 ms | passed |
| 2 | Named as svrngs named them | each page's "You’re …" and "Here now: Bo, Cy, Di" within 55 ms of being looked for | passed |
| 3 | Live | after Bo's first line was on every page, three edits typed at once; on each page the other two were there 108, 68 and 110 ms after their last keystroke | passed |
| 4 | Each paragraph names its last editor | every page's notes, paired with their paragraphs, name the writer, within 2 ms | passed |
| 5 | Goodnight | "☾ Alex’s window is dark … Goodnight from Alex’s." 4,451 ms (Bo), 4,294 ms (Cy) and 4,701 ms (Di) after the lid | passed |
| 6 | Writing goes on | each away edit in its page's editor, and "What you’ve written since is kept here." | passed |
| 7 | It joins | goodnight gone 3,570, 2,299 and 374 ms after the lid opened; all seven lines on all three pages, one doc, by 3,623 ms | passed |
| 8 | Nothing lost | a new page on Cy's computer opened in 262 ms and held all seven lines 8 ms later; 80 acts, by Bo, Cy and Di | passed |
| 9 | No blank paragraph | the seven lines on every page, and no paragraph without words | passed |
| 10 | Each note beside its paragraph on a phone | on Bo's and Di's pages, seven notes, each inside its own paragraph's box and naming its writer | passed |

## What it found

- **The blank paragraphs of `../`** came from how a page starts a doc
  nobody has written in. Each page shows a line of its own, and its first
  edit writes that line. The first take's first key was Enter, so each
  page wrote its line empty.
  - `found/replay.out` replays the first take's own acts.
  - `found/blank-cases.out` holds three ways of starting, run twice.
  - 0f9df88b makes each page take typing only once it holds the server's
    doc. This take starts the doc with one line, and no blank paragraph
    came.
- **A hard break in one paragraph.** `data/shots/bo-joined.png` shows an
  empty line inside "Di brings a speaker", and Yjs holds
  `<paragraph>Di brings a speaker<hardbreak></hardbreak></paragraph>`.
  - Act 42, Di's Enter in Chromium as a Pixel 7, wrote a paragraph that
    already held a hard break.
  - Pressed one at a time, Enter never makes one, on any engine
    (`found/hb-sequential.out`).
  - Pressed at once on pages that are all at the doc's end, it does:
    `found/hb-race.out` finds 10 in 6 rounds with HardBreak in the page's
    schema, and none in 6 without it (`found/hb-race-desktop.out`: none in
    6 more, three desktop pages).
  - Bar 9 reads paragraphs, not lines, so it did not catch this. The next
    commit takes HardBreak out of the page's schema.
- **The driver can type at the wrong place when everyone types at once.**
  It puts the caret through the page's own selection, and an edit
  arriving in the next few milliseconds can put the editor back where it
  last was. In `found/hb-race*.out`, lines are mixed into others in some
  rounds, three desktop pages included. A person's tap is read by the
  editor as it lands. The takes' own lines came out whole, but this is
  the driver's limit, not something the evening would see.
- **The driver writes Bo's screenshots into `events.jsonl`**, as in `../`.
  Their bytes are cut and marked `bytes_cut`.

## The pictures

`data/shots/`:
- `bo-open.png`
- `bo-written.png`, `di-written.png`, `cy-written.png`
- `bo-goodnight.png` and `di-goodnight.png`: V1.1.3's goodnight, each note
  under its own paragraph. The newest line has no note yet, since the
  server it would come from is not answering.
- `bo-joined.png`: back, with the others' carets.

## Take 2, at 9302090a

The same story, driver and bars, rerun once HardBreak was out of the
page's schema (9302090a), so the pictures are of the page as it is now.
Its data is in `take2/data/`. The Mac's load was 4.6 and Fedora's 0.7,
with Fedora's clock 3 ms ahead of the Mac's over a 6 ms round trip.

| # | Read | Verdict |
|---|---|---|
| 1 | Bo 2,243 ms and Di 2,328 ms; Cy 167 ms | passed |
| 2 | named, and everyone here | passed |
| 3 | 106, 126 and 110 ms | passed |
| 4 | every note names its paragraph's writer | passed |
| 5 | "☾ Alex’s window is dark" 4,308, 4,455 and 4,611 ms after the lid | passed |
| 6 | each away edit kept | passed |
| 7 | goodnight gone 3,393, 2,936 and 5,347 ms after the lid opened; all seven lines on all three pages by 5,408 ms | passed |
| 8 | a new page held all seven lines; 80 acts | passed |
| 9 | the seven lines, no paragraph without words | passed |
| 10 | seven notes on each phone, each inside its own paragraph | passed |

The server's doc after the take: seven paragraphs of words and no hard
break. The pictures are in `take2/data/shots/`. `di-goodnight.png` and
`bo-goodnight.png` show the goodnight, and `bo-joined.png` shows the page
after the lid opened.

## What this does not show

The same as `../README.md`: no real phone, no real lid, no venue network,
and three pages rather than twenty.
