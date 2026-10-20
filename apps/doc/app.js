// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai cmnwlth/apps/ring-doc/app.js@47b8acfd0
// The page: the editor, the doc kept in this browser, and the socket to the
// server (sync.js), drawn. Everything it knows about who is who came from
// the server, which read it off svrngs's door (protocol.js `HELLO`).
import {
  Bold,
  BulletList,
  Collaboration,
  CollaborationCaret,
  Document,
  Editor,
  HardBreak,
  Heading,
  IndexeddbPersistence,
  Italic,
  ListItem,
  Paragraph,
  Text,
  Y,
  awarenessProtocol,
} from "./vendor/doc-bundle.js";
import { attributionLine, colorFor } from "./adapter.js";
import { sync } from "./sync.js";

const el = (id) => document.getElementById(id);
const WALL = new URLSearchParams(location.search).has("wall");
document.body.classList.toggle("wall", WALL);

const ydoc = new Y.Doc();
// Presence for this document. The caret extension draws it; the socket,
// not Tiptap, carries it.
const awareness = new awarenessProtocol.Awareness(ydoc);

// The lender's name, kept in this browser from the last time the server
// said it, so a page that opens while the lender's computer is not
// answering can still say whose doc this is.
const LENDER = "doc:lender";
const remembered = () => {
  try {
    return localStorage.getItem(LENDER);
  } catch {
    return null;
  }
};
let lender = remembered();
let you;

// The doc in this browser, so what is written while the server is not
// answering stays, even across a reload, and joins when it answers.
try {
  new IndexeddbPersistence("doc", ydoc);
} catch (e) {
  el("err").textContent = `This browser can’t keep the doc, so what you write while it’s away is lost if you close this page: ${e.message ?? e}`;
}

function caret(user) {
  const c = document.createElement("span");
  c.classList.add("collaboration-carets__caret");
  c.setAttribute("style", `border-color: ${user.color}`);
  // A caret svrngs named nobody for is drawn without a name, never with one
  // made up.
  if (user.name) {
    const label = document.createElement("div");
    label.classList.add("collaboration-carets__label");
    label.setAttribute("style", `background-color: ${user.color}`);
    label.textContent = user.name;
    c.append(label);
  }
  return c;
}

const editor = new Editor({
  element: el("editor"),
  extensions: [
    Document,
    Paragraph,
    Text,
    Bold,
    Italic,
    Heading.configure({ levels: [1, 2] }),
    BulletList,
    ListItem,
    HardBreak,
    // Yjs owns undo: the built-in history stack would undo a peer's paragraph
    // on a local ctrl-Z, because it has no idea the document has other authors.
    Collaboration.configure({ document: ydoc }),
    // A renderer over the awareness map, nothing more — it takes a `provider`
    // only to read `.awareness` off it.
    // Unnamed is "", not null: y-tiptap writes `User: <its number>` over a
    // null name (y-tiptap.js, `createDecorations`), a name nobody gave.
    CollaborationCaret.configure({ provider: { awareness }, user: { name: "", color: "#808080" }, render: caret }),
  ],
});

const possessive = (name) => `${name}’s`;
function named() {
  el("title").textContent = lender ? `${possessive(lender)} doc` : "Doc";
  document.title = el("title").textContent;
  el("night-title").textContent = lender ? `Goodnight from ${possessive(lender)} doc` : "Goodnight from this doc";
  const who = lender ? `${possessive(lender)} computer` : "The computer lending it";
  el("night-words").textContent = `${who} isn’t answering right now. You can keep writing: what you write stays in this browser, and joins everyone’s writing when ${lender ? who : "it"} answers again.`;
  if (you === undefined) return;
  el("you").textContent = you
    ? `You’re ${you}. Everyone here writes in it at once.`
    : "svrngs didn’t name you, so what you write here is unnamed: open this doc from your svrngs home page, or from your group’s page.";
}
named();

// Who last edited each paragraph, as the server last said; the "ago" is
// redrawn every few seconds from it.
let entries = [];
// Wide enough for a margin beside the doc, as index.html's media rule says.
const NARROW = matchMedia("(max-width: 62rem)");
function drawLines() {
  const prose = el("editor").querySelector(".tiptap");
  const gutter = el("attribution");
  if (!prose) return;
  const blocks = Array.from(prose.children);
  const wrap = gutter.parentElement.getBoundingClientRect();
  gutter.replaceChildren();
  entries.forEach((entry, i) => {
    const line = attributionLine(entry, Date.now());
    const block = blocks[i];
    if (line === null || block === undefined) return;
    const note = document.createElement("div");
    note.className = "note";
    note.textContent = line;
    gutter.appendChild(note);
    // Measured as drawn, against the doc's box, so a note follows its
    // paragraph wherever the editor puts it.
    const at = block.getBoundingClientRect();
    if (NARROW.matches) {
      note.style.left = `${at.left - wrap.left}px`;
      note.style.width = `${at.width}px`;
      note.style.top = `${at.bottom - wrap.top - note.offsetHeight}px`;
    } else {
      note.style.left = `calc(100% + 0.9rem)`;
      note.style.width = "10rem";
      note.style.top = `${at.top - wrap.top}px`;
    }
  });
}
// Redrawn when the paragraphs move under the notes, as well as when the
// server says, so a note never sits beside another paragraph for long.
let drawing = false;
const drawSoon = () => {
  if (drawing) return;
  drawing = true;
  requestAnimationFrame(() => ((drawing = false), drawLines()));
};
editor.on("transaction", drawSoon);
addEventListener("resize", drawSoon);
NARROW.addEventListener("change", drawSoon);
setInterval(drawLines, 5000);

function drawHere() {
  const names = [...new Set([...awareness.getStates().values()].map((s) => s.user?.name).filter(Boolean))].sort();
  el("here").textContent = names.length ? `Here now: ${names.join(", ")}` : "";
}
awareness.on("change", drawHere);

// Said without a number: each keystroke is a change to Yjs, and "40
// changes" would read as forty things.
const keptWords = (n) => (n ? "What you’ve written since is kept here." : "");
let backTimer = null;
sync({
  url: `${location.origin.replace(/^http/, "ws")}/sync`,
  doc: ydoc,
  awareness,
  WebSocket,
  // On a phone, a socket after the lender stopped answering goes through
  // the lend's worker, which keeps its dead connection to the lender for
  // sockets and drops it only when a plain request fails on it (svrngs
  // web/lend.js: `fetch` deletes the dial on a failure, `socket` does not).
  // So each new socket is preceded by one plain request, which the worker
  // answers by dialling again once the old connection fails it. Seen in
  // demo/runs/doc/2026-10-20/: without it, a phone never rejoined after a
  // 45 s lid; with one request, it rejoined 3 s later.
  poke: () => fetch("./here", { cache: "no-store", signal: AbortSignal.timeout(5000) }),
  on: {
    hello(h) {
      you = h.you;
      if (h.lender) {
        lender = h.lender;
        try {
          localStorage.setItem(LENDER, lender);
        } catch {
          // A browser that keeps nothing still shows the name this time.
        }
      }
      awareness.setLocalStateField("user", { name: you ?? "", color: colorFor(you) });
      named();
    },
    lines(e) {
      entries = e;
      drawLines();
    },
    away({ kept }) {
      el("night").hidden = false;
      el("back").hidden = true;
      el("kept").textContent = keptWords(kept);
      drawHere();
    },
    kept(n) {
      el("kept").textContent = keptWords(n);
    },
    here() {
      el("night").hidden = true;
    },
    joined({ kept }) {
      if (!kept) return;
      const who = lender ? `${possessive(lender)} computer` : "The computer lending it";
      el("back").textContent = `${who} answered, and what you wrote meanwhile joined everyone’s writing.`;
      el("back").hidden = false;
      clearTimeout(backTimer);
      backTimer = setTimeout(() => (el("back").hidden = true), 10_000);
    },
  },
}).start();

