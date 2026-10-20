// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai cmnwlth/apps/ring-doc/adapter.test.mjs@47b8acfd0
// The doc's rules, pinned. `node --test apps/doc/`
//
// Each name says which mistake puts the wrong name on a paragraph, or loses
// text on the way to the server's file. What is not tested here is that every
// page and the server converge — that is Yjs's, run end to end over the real
// server in `doc.test.mjs`.

import assert from "node:assert/strict";
import { test } from "node:test";
import { unpinned } from "../pinned.mjs";
import { Y } from "./vendor/doc-bundle.js";
import { PROSE_FRAGMENT, attributionLine, b64ToBytes, bytesToB64, colorFor, createAttribution } from "./adapter.js";

test("the bundle is the bytes vendor/README.md pins", () => {
  assert.deepEqual(unpinned(new URL("./vendor", import.meta.url).pathname), []);
});

test("an update survives the base64 trip whole", () => {
  // The chunked encoder exists because spreading a document-sized array onto
  // the call stack throws. A stride bug would corrupt exactly one boundary,
  // which is invisible on the short updates every other test here uses.
  const bytes = new Uint8Array(0x8000 * 2 + 7);
  for (let i = 0; i < bytes.length; i += 1) bytes[i] = i % 256;
  assert.deepEqual(Array.from(b64ToBytes(bytesToB64(bytes))), Array.from(bytes));
});

test("a caret colour is a hex triple, or Tiptap draws nothing", () => {
  // `isValidColor` in the caret extension is `/^#[0-9a-fA-F]{6}$/` and
  // substitutes `transparent` for anything else — silently. An `hsl()` string
  // here is an invisible cursor with no error on any surface.
  assert.match(colorFor("alex"), /^#[0-9a-f]{6}$/);
  assert.equal(colorFor("alex"), colorFor("alex"), "one person, two colours");
  assert.notEqual(colorFor("alex"), colorFor("bo"));
});

// --- attribution ---------------------------------------------------------
//
// Every "last edited by" line resolves from the act's `person`, which the
// server stamps from the name svrngs's door set on the socket. The test that
// earns it is the second one — the first would read the same if the adapter
// took the name from inside the update, which is the mistake this path avoids.

/// Paragraphs in the fragment Tiptap edits, as ONE update.
const typedParagraphs = (texts) => {
  const doc = new Y.Doc();
  const frag = doc.getXmlFragment(PROSE_FRAGMENT);
  for (const text of texts) {
    const para = new Y.XmlElement("paragraph");
    const body = new Y.XmlText();
    body.insert(0, text);
    para.insert(0, [body]);
    frag.push([para]);
  }
  return Y.encodeStateAsUpdate(doc);
};

/// A further update over `base`, optionally announcing `clientID` as the
/// client that wrote it. Yjs lets a client say who it is — `doc.clientID` is
/// one assignment — which is exactly the forgery the door's name is there to
/// make irrelevant.
const edit = (base, clientID, change) => {
  const doc = new Y.Doc();
  Y.applyUpdate(doc, base);
  if (clientID !== undefined) doc.clientID = clientID;
  const before = Y.encodeStateVector(doc);
  change(doc.getXmlFragment(PROSE_FRAGMENT));
  return Y.encodeStateAsUpdate(doc, before);
};

const NOW_SEC = 1_756_512_000;
const ALEX_KEY = "aa".repeat(32);
const BO_KEY = "bb".repeat(32);

/// One act as the server writes it (server.mjs, `doc.on("update")`).
const act = (id, update, extra = {}) => ({
  id,
  seq: Number(id),
  ts: NOW_SEC,
  actor: ALEX_KEY,
  person: "alex",
  update: bytesToB64(update),
  ...extra,
});

test("each paragraph says who last edited it, and how long ago", () => {
  const first = typedParagraphs(["one", "two"]);
  const second = edit(first, undefined, (frag) => frag.get(1).get(0).insert(3, " more"));

  const attribution = createAttribution();
  attribution.absorb([
    act("1", first, { ts: NOW_SEC - 12 }),
    act("2", second, { actor: BO_KEY, person: "bo", ts: NOW_SEC - 4 }),
  ]);

  assert.deepEqual(attribution.lines(NOW_SEC * 1000), [
    "last edited by alex 12s ago",
    "last edited by bo 4s ago",
  ]);
  assert.deepEqual(attribution.entries(), [
    { person: "alex", ts: NOW_SEC - 12 },
    { person: "bo", ts: NOW_SEC - 4 },
  ]);
});

test("an update whose clientID is another person's still names the socket's person", () => {
  // `bo`'s page announces alex's Yjs clientID — one number, changed in a page
  // anybody can edit, and Yjs has no way to refuse it. The act carrying it
  // came in on bo's socket, which svrngs's door named, so the paragraph must
  // read bo.
  const alexDoc = new Y.Doc();
  alexDoc.clientID = 1001;
  const alexFrag = alexDoc.getXmlFragment(PROSE_FRAGMENT);
  const para = new Y.XmlElement("paragraph");
  const body = new Y.XmlText();
  body.insert(0, "alex wrote this");
  para.insert(0, [body]);
  alexFrag.push([para]);
  const honest = Y.encodeStateAsUpdate(alexDoc);

  const forged = edit(honest, 1001, (frag) => frag.get(0).get(0).insert(0, "bo typed here. "));
  // The forgery, checked rather than assumed: every struct in bo's update
  // announces alex's client. A reader that took the name from in here would
  // say alex, which is the failure the line below rules out.
  assert.deepEqual(
    [...new Set(Y.decodeUpdate(forged).structs.map((s) => s.id.client))],
    [1001],
    "the update under test does not actually carry alex's clientID",
  );

  const attribution = createAttribution();
  attribution.absorb([
    act("1", honest, { ts: NOW_SEC - 30 }),
    act("2", forged, { actor: BO_KEY, person: "bo", ts: NOW_SEC - 1 }),
  ]);

  assert.deepEqual(attribution.lines(NOW_SEC * 1000), ["last edited by bo 1s ago"]);
});

test("an edit svrngs named nobody for is said to be unnamed, never invented", () => {
  // A socket that reached the server at its own port, not through svrngs:
  // no door set a name. The honest line says so; a fallback to the key, or
  // to "someone", would be a page substituting for an absence.
  const attribution = createAttribution();
  attribution.absorb([act("1", typedParagraphs(["hello"]), { actor: "", person: null })]);

  const [line] = attribution.lines(NOW_SEC * 1000);
  assert.match(line, /didn't name/);
  assert.doesNotMatch(line, /alex|bo/);
  assert.equal(attributionLine(null, NOW_SEC * 1000), null, "a paragraph no act touched gets no line");
});

test("two copies that absorbed the same acts in different orders name the same author", () => {
  // Two people edit one paragraph at one `ts`; one copy reads alex's act
  // first, the other bo's, and then both read all of them. The order is
  // (ts, actor, seq, id), so aa… before bb…: bo is last.
  const base = typedParagraphs(["shared"]);
  const acts = [
    act("1", base, { ts: NOW_SEC - 20 }),
    act("2", edit(base, undefined, (f) => f.get(0).get(0).insert(0, "A ")), { ts: NOW_SEC - 3 }),
    act("3", edit(base, undefined, (f) => f.get(0).get(0).insert(6, " B")), { actor: BO_KEY, person: "bo", ts: NOW_SEC - 3 }),
  ];

  const alexFirst = createAttribution();
  alexFirst.absorb([acts[0], acts[1]]);
  alexFirst.absorb(acts);
  const boFirst = createAttribution();
  boFirst.absorb([acts[0], acts[2]]);
  boFirst.absorb(acts);

  const expected = ["last edited by bo 3s ago"];
  assert.deepEqual(alexFirst.lines(NOW_SEC * 1000), expected);
  assert.deepEqual(boFirst.lines(NOW_SEC * 1000), expected);
});
