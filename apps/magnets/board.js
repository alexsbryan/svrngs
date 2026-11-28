// SPDX-License-Identifier: AGPL-3.0-or-later
// Magnets' board apart from the page (index.js): the read it subscribes to,
// the writes a tap sends, the sentence, tray and kept sentences it draws
// from the rows, and every sentence it says. Nothing here reads the page or the network, so its
// laws run in Node (tests/board.test.mjs).

/**
 * The board in one read, each row by its `kind`: the sentence, its magnets
 * by position, then by ID where two moves made apart chose one position;
 * the sentences kept, newest first; and the prompt it is under. One read,
 * so the page never draws one part fresh beside another stale.
 */
export const BOARD =
  "SELECT 'magnet' AS kind, id, position, NULL AS prompt, NULL AS words, NULL AS at FROM magnets WHERE position IS NOT NULL" +
  " UNION ALL SELECT 'kept', id, NULL, prompt, words, at FROM kept" +
  " UNION ALL SELECT 'showing', id, NULL, prompt, NULL, NULL FROM showing" +
  ' ORDER BY kind, position, at DESC, id';

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

/**
 * Two neighbors in the sentence swapped, as two ops in one write: each takes
 * the other's position, and no other word's changes. Two at one position (a
 * tie two moves made apart left) can't trade it, so the one going first
 * keeps it and the other takes one past it.
 */
export function swap(first, second) {
  const [a, b] = [first.position, second.position];
  return [put(first.id, a === b ? a + 1 : b), put(second.id, a)];
}

/** The prompts a sentence is under, in turn: the pack's `prompt`, then its `prompts`. */
const prompts = (pack) => [pack.prompt, ...(pack.prompts ?? [])];
/** The prompt after `prompt`, back to the first past the last. */
export function following(pack, prompt) {
  const all = prompts(pack);
  return all[(all.indexOf(prompt) + 1) % all.length];
}

/** The sentence under `prompt` kept as one row, `id` and `at` the page's, its words by ID in order. */
const kept = (id, prompt, sentence, at) => ({
  insert: 'kept',
  row: { id, prompt, words: sentence.map((m) => m.id).join(' '), at },
});
const showing = (prompt) => ({ insert: 'showing', row: { id: 'now', prompt } });

/**
 * Keep it and start a new one, in one write: the sentence kept, its
 * magnets back in the tray, and the next prompt shown. Nothing is lost.
 */
export const keep = (pack, board, id, at) => [
  kept(id, board.prompt, board.sentence, at),
  ...board.sentence.map((m) => back(m.id)),
  showing(following(pack, board.prompt)),
];

/**
 * A kept sentence put back, in one write: the sentence on the fridge kept
 * first, if it has a word, then the kept one's magnets placed in its order,
 * any other back in the tray, its prompt shown, and its kept row gone, as
 * it is on the fridge again.
 */
export function restore(board, old, id, at) {
  const words = old.sentence.map((m) => m.id);
  return [
    ...(board.sentence.length ? [kept(id, board.prompt, board.sentence, at)] : []),
    ...board.sentence.filter((m) => !words.includes(m.id)).map((m) => back(m.id)),
    ...words.map((w, i) => put(w, i + 1)),
    showing(old.prompt),
    { delete: 'kept', key: { id: old.id } },
  ];
}

/** Once the sentence has this many words, it can be kept and a new one started. */
export const KEEP_AT = 3;

/** The IDs of `now`'s sentence that `before`, the ones the viewer last saw, lacks: new to them. */
export const fresh = (before, now) => now.filter((id) => !before.includes(id));
/** How long a word that arrives while someone watches stays outlined as new. */
export const FRESH_MS = 6_000;

/** The sentence as it goes to the group's chat: the words, and the link to change it, if the group has one. */
export function sent(board, link) {
  const words = [board.prompt, ...board.sentence.map((m) => m.word)].join(' ');
  return typeof link === 'string' ? `${words}. Change it: ${link}` : `${words}.`;
}

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
/** What the page says over the last sentence it showed, once no computer answers. */
export const NIGHT =
  'Goodnight from the fridge. None of your group’s computers is answering, so this is the sentence as you last saw it. It opens again by itself.';
export const CHANGED = 'Changed since you last looked.';
export const COPIED = 'Copied. Paste it in your group’s chat.';
export const UNCOPIED = (text) => `Copy this into your group’s chat: ${text}`;
/** A move that didn't happen, in the words the route gave for why. */
export const refused = (word, words) => `“${word}” didn’t move: ${words}`;
/** The board the route wouldn't read, in its words. */
export const shut = (words) => `The fridge didn’t open: ${words}`;
/** Why, where the route gave no words of its own. */
export const NONE = 'None of your group’s computers answered.';
export const answeredWith = (status) => `Your group’s computer answered ${status}.`;
export const UNREAD = 'Its answer couldn’t be read.';

/**
 * The board from its rows, already in BOARD's order: the prompt it is
 * under, the pack's first where none is shown; the sentence, each magnet
 * row's magnet the pack names; the tray, the pack's other magnets in the
 * pack's own order; and the sentences kept, each with its own prompt and
 * magnets. `unknown` holds the magnets the pack doesn't name (a fork's old
 * words), which none shows.
 */
export function order(pack, rows) {
  const words = new Map(pack.magnets.map((m) => [m.id, m]));
  const of = (kind) => rows.filter((r) => r.kind === kind);
  const magnets = of('magnet');
  const sentence = magnets
    .filter((r) => words.has(r.id))
    .map((r) => ({ ...words.get(r.id), position: r.position }));
  const placed = new Set(sentence.map((m) => m.id));
  const known = (ids) => ids.filter((id) => words.has(id)).map((id) => words.get(id));
  return {
    prompt: of('showing')[0]?.prompt ?? pack.prompt,
    sentence,
    tray: pack.magnets.filter((m) => !placed.has(m.id)),
    kept: of('kept').map((r) => ({
      id: r.id,
      prompt: r.prompt,
      sentence: known(String(r.words ?? '').split(' ')),
    })),
    unknown: magnets.filter((r) => !words.has(r.id)).map((r) => r.id),
  };
}
