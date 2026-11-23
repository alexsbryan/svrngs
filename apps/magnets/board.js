// SPDX-License-Identifier: AGPL-3.0-or-later
// Magnets' board apart from the page (index.js): the read it subscribes to,
// the writes a tap sends, the sentence and tray it draws from the rows, and
// every sentence it says. Nothing here reads the page or the network, so its
// laws run in Node (tests/board.test.mjs).

/** The sentence: its magnets by position, then by ID where two moves made apart chose one position. */
export const BOARD =
  'SELECT id, position FROM magnets WHERE position IS NOT NULL ORDER BY position, id';

// A tap's write is one op of svrngs/sql's write form (tests/standin.mjs):
// its row named by key and its values given, never SQL. A phone's worker
// checks the one row it signs without a SQL parser (svrn-docs
// magnets-spikes, S2), and a write that ran no condition and computed
// nothing holds the same on every copy it reaches.

/**
 * A magnet put at the sentence's end: its row made, or its position set, at
 * `position`, which `next` reads from the board the page holds, since the
 * write itself can't ask for the last one.
 */
export const put = (id, position) => ({ insert: 'magnets', row: { id, position } });
/** A magnet back in the tray: its row kept, its position null. */
export const back = (id) => ({ update: 'magnets', key: { id }, set: { position: null } });
/** One past the last position among `rows`, the board as BOARD read it. */
export const next = (rows) => Math.max(0, ...rows.map((r) => r.position ?? 0)) + 1;

// How long after the board stopped answering it is read again: soon enough
// that someone reading why sees the fridge come back as a computer wakes,
// and 20 asks a minute from a page left open while every computer sleeps.
export const RETRY_MS = 3_000;

/** What the page says while it can't show the board, by why. */
export const SAID = {
  opening: 'Opening your group’s fridge…',
  syncing: 'Still syncing with your group. Your sentence shows once it has caught up.',
  unreachable:
    'None of your group’s computers is answering, so the fridge can’t open right now. It tries again by itself.',
};
/** A move that didn't happen, in the words the route gave for why. */
export const refused = (word, words) => `“${word}” didn’t move: ${words}`;
/** The board the route wouldn't read, in its words. */
export const shut = (words) => `The fridge didn’t open: ${words}`;
/** Why, where the route gave no words of its own. */
export const NONE = 'None of your group’s computers answered.';
export const answeredWith = (status) => `Your group’s computer answered ${status}.`;
export const UNREAD = 'Its answer couldn’t be read.';

/**
 * The board from its rows, already in BOARD's order: the sentence, each
 * row's magnet the pack names, and the tray, the pack's other magnets in the
 * pack's own order. `unknown` holds the rows the pack doesn't name (a fork's
 * old words), which neither shows.
 */
export function order(pack, rows) {
  const words = new Map(pack.magnets.map((m) => [m.id, m]));
  const sentence = rows.filter((r) => words.has(r.id)).map((r) => words.get(r.id));
  const placed = new Set(sentence.map((m) => m.id));
  return {
    sentence,
    tray: pack.magnets.filter((m) => !placed.has(m.id)),
    unknown: rows.filter((r) => !words.has(r.id)).map((r) => r.id),
  };
}
