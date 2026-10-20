// SPDX-License-Identifier: AGPL-3.0-or-later
// What crosses the doc's socket, in one place for the page (sync.js) and
// the server (server.mjs): each message a varUint type, then its body, as
// y-websocket frames them. 0 and 1 are y-websocket's own, so the Yjs sync
// and awareness are the protocol's, not this app's; 100 and up are the
// doc's, kept clear of y-websocket's 2 and 3.
import { decoding, encoding } from "./vendor/doc-bundle.js";

/** A Yjs sync step or update (y-protocols/sync). */
export const SYNC = 0;
/** Who is here and where their caret is (y-protocols/awareness). */
export const AWARENESS = 1;
/**
 * The server to one page, on opening: `{you, key, lender}`. `you` is the
 * name svrngs's door gave the person on the upgrade (`x-svrngs-person`),
 * `null` when the page reached the server without svrngs; `lender` is the
 * name of the person lending the doc, `null` when the server was not told.
 */
export const HELLO = 100;
/** The server to every page: who last edited each paragraph, `[{person, ts} | null]`. */
export const LINES = 101;
/**
 * The server to every page, each second: nothing but that it answers. A
 * page that hears no message for `QUIET_MS` says goodnight (sync.js).
 */
export const HERE = 102;

/** How often the server says HERE. */
export const HERE_EVERY_MS = 1000;

/** A message of `type` whose body is `json`, written as text. */
export function said(type, json) {
  const e = encoding.createEncoder();
  encoding.writeVarUint(e, type);
  encoding.writeVarString(e, JSON.stringify(json));
  return encoding.toUint8Array(e);
}

/** A message of `type` with no body. */
export function bare(type) {
  const e = encoding.createEncoder();
  encoding.writeVarUint(e, type);
  return encoding.toUint8Array(e);
}

/** `bytes` read: its type, and a decoder at its body. */
export function read(bytes) {
  const d = decoding.createDecoder(bytes);
  return { type: decoding.readVarUint(d), body: d };
}

/** A body `said` wrote, as its JSON. */
export const json = (body) => JSON.parse(decoding.readVarString(body));
