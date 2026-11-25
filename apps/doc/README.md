# Doc

A document a room writes together, lent from one computer: everyone's
caret named, each paragraph saying who last edited it, and, when the
lender's computer stops answering, a goodnight on every page while each
browser keeps what its person writes, joined with everyone's when the
computer answers again.

- **Shape:** lent. A program on your computer that keeps the document.
- **Run it:** [below](#run-it), from svrngs's source
  ([run one](../../docs/guide/examples.md#3-run-one)).
- **How it's made:** [How it works](#how-it-works).
- **Change it:** `server.mjs` is the server, `app.js` the page. Restart it
  after a change.

## Run it

1. In a terminal, from svrngs's source, with svrngs running. Leave it open:
   ```
   node apps/doc/server.mjs --port 4318 --keep ~/doc
   ```
2. Lend it from your home page: under **Already running here**, press
   **Look for programs running on this computer**, and open the line at
   `localhost:4318`. Fill in **Its name:** `doc` and **Which program:**
   `doc`, tick the group, and press **Lend it to** your group. Or in a
   second terminal:
   `svrngs share 4318 --name doc --program doc --group <group>`, pressing
   Enter where it asks for a password: the doc takes none.

`--keep` is where the doc is kept, as one JSON line per edit
(`acts.jsonl`), read back whole on a restart; the doc is yours to move or
delete. Everyone in your groups then finds "doc" among what you lend, on their phone or on their svrngs home page.
`?wall` after its address draws it for a projector.

## How it works

- `server.mjs`: the page, and a WebSocket at `/sync` per open page,
  through svrngs's door, which names the person on the upgrade
  (`x-svrngs-person`; svrngs `crates/svrngs-node/src/door.rs`). Each edit
  is written down as that person's, sent to every other page, and the
  paragraphs' last editors sent to all. It binds loopback alone; svrngs
  carries everyone else to it.
- `ws.mjs`: the WebSocket's server half, on Node's own `http`.
- `protocol.js`: the messages: Yjs's sync and awareness as y-websocket
  frames them, and the doc's own `HELLO`, `LINES` and `HERE`.
- `sync.js`: the page's half. The server says `HERE` each second; a page
  that hears nothing for 5 seconds says goodnight, keeps writing into its
  own copy, and opens a new socket until one answers. On a phone the
  socket is svrngs's (`web/shim.js`, `web/lend.js`), which does not close
  when the lender's computer stops, so silence is what the page reads; and
  the lend's worker keeps a dead connection for sockets until a plain
  request fails on it, so each new socket is preceded by one, `GET /here`
  (`demo/runs/doc/2026-10-20/found/`).
- `app.js`, `index.html`: the editor, the doc kept in the browser
  (IndexedDB), the notes, the goodnight. svrngs's look: the browser's own
  fonts, plain rules, named colours.
- `adapter.js`: who last edited each paragraph, from the acts the server
  wrote, never from the Yjs client number an update carries.
- `vendor/doc-bundle.js`: Yjs and the editor, built by `bundle/build.sh`,
  pinned in `vendor/README.md`.

Ported from commonwealth-ai `cmnwlth/apps/ring-doc/` at 47b8acfd0 (each
ported file says so), where edits were signed acts on commonwealth-ai's
rail and carets rode its live lane. Here the lender's server is both, and
upstream's expense book is not kept (svrngs's is `apps/expenses/`).

## What it does not do

- **A caret's name is only what its page says.** The server tells each
  page the name svrngs gave it, and the page puts it on its caret; a page
  edited to say another name would be drawn so. The paragraph notes are
  the server's, from the door's name, and say so on the page.
- **What a browser keeps stays in that browser.** A page closed while the
  lender's computer is away keeps its writing in IndexedDB and sends it
  the next time it opens the doc. A doc whose `--keep` was emptied meets
  that writing again from every browser that kept a copy.
- **Opened at its own port**, not through svrngs, a page is named nobody,
  and says so.
- **A doc nobody has written in yet** starts as a line of each page's own.
  Pages that edit it at once each write theirs, as blank lines between
  the words or as words mixed into one line
  (`demo/runs/doc/2026-10-20/polish/found/`). Write the doc's first line
  before the room opens it. Each page takes typing only once it holds the
  server's doc, so a page that already has a first line never writes its
  own.

## Tests

`adapter.test.mjs` (the attribution and the pins) and `doc.test.mjs` (the
server as its own process, pages as `sync.js` runs them, the lid as
SIGSTOP) run in `scripts/check.sh`'s "app laws". `apps/` is outside the
size budget (`scripts/lean_check.py`).
