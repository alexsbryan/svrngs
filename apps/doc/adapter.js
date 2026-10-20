// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai cmnwlth/apps/ring-doc/adapter.js@47b8acfd0
// The doc's rules that need no socket: who last edited each paragraph, an
// update as text, and a person's colour. Pure functions over values, so
// they run under `node --test` with no browser and no server; the server
// (server.mjs) and the page (app.js) are the halves that do I/O.
//
// Upstream read edits as signed acts off commonwealth-ai's rail. Here the
// server is where every edit arrives, on a socket svrngs's door opened as a
// named person, so the server writes each edit down as an act stamped with
// that person (server.mjs `wrote`) and the rules below read those acts as
// they read the rail's.
import { Y } from "./vendor/doc-bundle.js";

// base64 in chunks. `String.fromCharCode(...bytes)` spreads the whole array
// onto the call stack and throws on a document of any size; 0x8000 is the
// conventional safe stride.
const CHUNK = 0x8000;

/// A Yjs update as a JSON-safe string, as the server keeps each act.
export function bytesToB64(bytes) {
  let s = "";
  for (let i = 0; i < bytes.length; i += CHUNK) {
    s += String.fromCharCode.apply(null, bytes.subarray(i, i + CHUNK));
  }
  return btoa(s);
}

/// The inverse. Throws on anything that is not base64, which is what we want:
/// a malformed act is a gap to report, never an update to guess at.
export function b64ToBytes(b64) {
  const s = atob(b64);
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i += 1) out[i] = s.charCodeAt(i);
  return out;
}

// ---------------------------------------------------------------------------
// Attribution — who last edited each paragraph, and when.
//
// Yjs stamps every character with the clientID of the client that produced it,
// and a clientID is self-asserted: a page edited to announce another person's
// number writes that number into every update it makes, and nothing in Yjs
// checks it. An act's `person` is the other kind of field — the name svrngs's
// door set on the socket the edit came in on (`x-svrngs-person`), which a page
// cannot set for somebody else: the door strips a client's copy and sets its
// own (svrngs crates/svrngs-node/src/door.rs, "Who is asking").
//
// So nothing below reads a name out of an update. It replays the acts in the
// server's order and records, per paragraph, WHICH ACT last changed it; the
// name is that act's `person`. A forged clientID travels inside an act like
// any other bytes and renders under the person whose socket carried it.
//
// This is also why per-character colouring is not shown, and why the page says
// so out loud: telling two authors apart INSIDE one paragraph could only read
// the per-character clientIDs, which is the field that cannot be checked.

/// The fragment Tiptap's `Collaboration` extension edits. One name, one place
/// — a replay that read a different fragment would attribute nothing and look
/// exactly like a document nobody has typed in.
export const PROSE_FRAGMENT = "default";

/// Replay the acts and keep, per paragraph, the act that last touched it.
///
/// The replay runs on its OWN `Y.Doc`: each act is applied alone, which is
/// what makes "which act touched this paragraph" an observed fact rather than
/// a guess. The caller owns the returned object, so each act is replayed once.
///
/// The answer is a function of the acts in their order (`ts`, then `actor`,
/// `seq`, `id`), not arrival order: an act that sorts before one already
/// replayed rebuilds the replay from the full ordered list.
export function createAttribution() {
  const seen = new Set();
  const absorbed = [];
  let frag = null;
  // Keyed by the paragraph's Yjs type, which is the identity that survives a
  // peer inserting a paragraph above it. An index would not.
  let by = null;
  let current = null;

  const reset = () => {
    const f = new Y.Doc().getXmlFragment(PROSE_FRAGMENT);
    const m = new Map();
    f.observeDeep((events) => {
      if (current === null) return;
      for (const event of events) {
        for (const block of touchedBlocks(event, f)) m.set(block, current);
      }
    });
    frag = f;
    by = m;
  };
  const replay = (act) => {
    current = { actor: act.actor, ts: act.ts, person: act.person || null };
    Y.applyUpdate(frag.doc, b64ToBytes(act.update));
  };
  reset();

  return {
    /// Replay every act not replayed yet, in order.
    absorb(acts) {
      const fresh = acts.filter((act) => !seen.has(act.id));
      if (fresh.length === 0) return;
      for (const act of fresh) seen.add(act.id);
      fresh.sort(order);
      const last = absorbed[absorbed.length - 1];
      const inOrder = last === undefined || order(last, fresh[0]) < 0;
      absorbed.push(...fresh);
      if (inOrder) {
        fresh.forEach(replay);
      } else {
        absorbed.sort(order);
        reset();
        absorbed.forEach(replay);
      }
      current = null;
    },
    /// One entry per top-level block, in document order: `{person, ts}` of
    /// the act that last touched it, or `null` where none has. What the
    /// server sends the pages (protocol.js `LINES`).
    entries() {
      return frag.toArray().map((block) => {
        const e = by.get(block);
        return e ? { person: e.person, ts: e.ts } : null;
      });
    },
    /// One line per top-level block, in document order; `null` where no act
    /// has touched that block yet.
    lines(nowMs) {
      return this.entries().map((e) => attributionLine(e, nowMs));
    },
  };
}

/// The order over acts: `(ts, actor, seq, id)`. The server gives each act a
/// `ts` later than the last (server.mjs `wrote`), so this is the order the
/// edits reached it.
const byText = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
const order = (x, y) =>
  x.ts - y.ts || byText(x.actor, y.actor) || x.seq - y.seq || byText(x.id, y.id);

/// The top-level blocks one Yjs event changed.
///
/// An event on the fragment itself is a paragraph added or removed, and the
/// added ones are in its delta; an event anywhere else is text inside one
/// block, so the block is found by walking up to the fragment's child.
function touchedBlocks(event, frag) {
  if (event.target === frag) {
    const added = [];
    for (const part of event.delta || []) {
      if (Array.isArray(part.insert)) added.push(...part.insert);
    }
    return added;
  }
  let node = event.target;
  while (node && node.parent && node.parent !== frag) node = node.parent;
  return node && node.parent === frag ? [node] : [];
}

/// "last edited by Bo 4s ago", from the entry, or `null` when no act has
/// touched this paragraph.
///
/// An act that came in on a socket svrngs named nobody on (the page was opened
/// at the server's own port, not through svrngs) is SAID to be unnamed rather
/// than rendered as "someone" — an absence reported, never defaulted.
export function attributionLine(entry, nowMs) {
  if (!entry) return null;
  const name = entry.person;
  const ago = Math.max(0, Math.round(nowMs / 1000 - entry.ts));
  return name === null || name === undefined
    ? `last edited by someone svrngs didn't name, ${ago}s ago`
    : `last edited by ${name} ${ago}s ago`;
}

/// A stable colour for a name, so one person is the same colour on every
/// screen without anyone agreeing on a palette.
///
/// Derived from the NAME rather than the Yjs clientID: a clientID is new on
/// every reload, which would recolour everyone whenever anybody refreshed.
///
/// **`#rrggbb`, and that is not a style preference.** Tiptap's caret runs the
/// colour through `isValidColor`, whose regex is exactly that, and substitutes
/// `transparent` for anything else — so an `hsl()` string renders an invisible
/// cursor with no error anywhere. The hue is spread over the wheel and
/// converted here rather than left to CSS for that reason alone.
export function colorFor(name) {
  const text = String(name || "");
  let hue = 0;
  for (let i = 0; i < text.length; i += 1) {
    hue = (hue * 31 + text.charCodeAt(i)) % 360;
  }
  // hsl(hue, 70%, 45%) by hand: chroma 0.63, mid 0.135.
  const c = 0.63;
  const x = c * (1 - Math.abs(((hue / 60) % 2) - 1));
  const m = 0.45 - c / 2;
  const sector = Math.floor(hue / 60) % 6;
  const rgb = [
    [c, x, 0],
    [x, c, 0],
    [0, c, x],
    [0, x, c],
    [x, 0, c],
    [c, 0, x],
  ][sector];
  const hex = (v) =>
    Math.round((v + m) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${hex(rgb[0])}${hex(rgb[1])}${hex(rgb[2])}`;
}
