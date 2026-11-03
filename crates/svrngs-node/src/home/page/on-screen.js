// SPDX-License-Identifier: AGPL-3.0-or-later
// SCREEN, the machine of a group on a screen (group-screen.js `Screen`).
// Each read is asked by an edge timer a second after the last one began,
// `tick`, or as it lands if later: read back to back, a 25-computer
// group's founder spent 180% of a core
// (demo/runs/twentyfive/2026-10-17/README.md). People and lends are read
// together and drawn as one, so the door is judged before it takes the
// last one's place, and no window reads "asking…" again. Who joined and
// what was lent since the screen opened is kept by key (`seen`: a
// person's, or a lend's origin), at this computer's time, newest first,
// the newest marked (`newest`). It is kept by this page alone, so opened
// again it starts over; the first read's were there before it opened, so
// no line says them. The page's class and title are set at the edge
// (`look`), not by a view.
import { ask, assertNever, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { called } from '../question.js?v=__VERSION__';
import { get } from './routes.js?v=__VERSION__';
/** @import { Lends, Lent } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { PeopleRow } from '../question.js?v=__VERSION__' */

/**
 * Who or what arrived, by its key, a person's or a lend's origin, and how
 * a line says it arrived.
 * @typedef {[key: string, words: string]} Seen
 */

/**
 * A line of what arrived: at this computer's time, what.
 * @typedef {[time: string, words: string]} Since
 */

/**
 * SCREEN's state: where its read is, the group's id, the people and lends
 * last read, every key seen since the screen opened, what arrived since,
 * newest first, each at its time, the newest's key, and why the last read
 * failed.
 * @typedef {object} State
 * @property {{ screen: 'reading' | 'read' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {string | undefined} id
 * @property {string[]} seen
 * @property {Since[]} since
 * @property {PeopleRow[]} [rows]
 * @property {Lends} [lends]
 * @property {string} [newest]
 * @property {string} [why]
 */

/**
 * The messages SCREEN takes: the page's look set; the timer's tick; read
 * `n` of the people and lends, begun at `began`, at this computer's
 * `time`; or why not.
 * @typedef {{ type: 'looked', why: string }
 *   | { type: 'tick' }
 *   | { type: 'read', n: number, rows: PeopleRow[], lends: Lends, began: number, time: string, why: string }
 *   | { type: 'unread', n: number, began: number, why: string }} Message
 */

/**
 * The effects its step answers: the page's class and title set to
 * `title`; read `n` of the group `id`; and a tick at `at`, in
 * milliseconds.
 * @typedef {{ run: 'look', title: string }
 *   | { run: 'read', n: number, id: string, failed: 'unread' }
 *   | { run: 'tick', at: number, failed: 'unread' }} Effect
 */

// Who is in and what is lent, each once by its key, the first kept, with
// how a line says it arrived. A lend a group carries is no one's to lend.
/**
 * @param {PeopleRow[]} rows
 * @param {Lent[]} lent
 * @returns {Seen[]}
 */
const keyed = (rows, lent) => {
  /** @type {Seen[]} */
  const joined = rows.map((row) => [row.person_key, `${called(rows, row.person_key)} joined`]);
  /** @type {Seen[]} */
  const lends = lent
    .filter((lend) => !lend.group)
    .map((lend) => {
      const key = rows.find((row) => row.key === lend.key)?.person_key;
      return [lend.origin, `${called(rows, key) ?? lend.person} lent ${lend.name}`];
    });
  const all = [...joined, ...lends];
  return all.filter(([key], i) => all.findIndex(([other]) => other === key) === i);
};

/** @type {Machine<State, Message, Effect>} */
export const SCREEN = {
  states: { screen: ['reading', 'read', 'unread'] },
  step: (state, message) => {
    switch (message.type) {
      case 'looked':
        return go(state, {});
      case 'tick': {
        const [asked, serial] = ask(state, 'screen');
        // A tick follows a read, run only for a group found (`Screen`).
        return go(
          state,
          {},
          { asked },
          { run: 'read', n: serial, id: /** @type {string} */ (state.id), failed: 'unread' },
        );
      }
      case 'read': {
        const old = late(state, 'screen', message.n);
        if (old) return no(state, old);
        const all = keyed(message.rows, message.lends.lent);
        const now =
          state.at.screen === 'reading' ? [] : all.filter(([key]) => !state.seen.includes(key));
        const arrived = now
          .map(/** @returns {Since} */ ([, words]) => [message.time, words])
          .reverse();
        return go(
          state,
          { screen: 'read' },
          {
            rows: message.rows,
            lends: message.lends,
            seen: [...new Set([...state.seen, ...all.map(([key]) => key)])],
            since: [...arrived, ...state.since],
            newest: now.at(-1)?.[0] ?? state.newest,
          },
          { run: 'tick', at: message.began + 1000, failed: 'unread' },
        );
      }
      case 'unread': {
        const old = late(state, 'screen', message.n);
        if (old) return no(state, old);
        return go(
          state,
          { screen: 'unread' },
          { why: message.why },
          { run: 'tick', at: message.began + 1000, failed: 'unread' },
        );
      }
      default:
        return assertNever(state, message, 'a screen');
    }
  },
  io: {
    look: ({ title }) => {
      document.body.className = 'screen';
      document.title = title;
      return { type: 'looked', why: title };
    },
    read: ({ n: serial, id }) => {
      const began = Date.now();
      const namespace = encodeURIComponent(id);
      const people = get(`/v1/rail/people?namespace=${namespace}`);
      const lends = get(`/v1/lends?namespace=${namespace}`);
      return Promise.all([people, lends]).then(
        ([{ people: rows }, lent]) => ({
          type: 'read',
          n: serial,
          rows,
          lends: lent,
          began,
          time: new Date().toLocaleTimeString([], { timeStyle: 'short' }),
          why: `screen ${serial}`,
        }),
        (error) => ({ type: 'unread', n: serial, began, why: error.message }),
      );
    },
    tick: ({ at }) =>
      new Promise((resolve) => setTimeout(resolve, at - Date.now())).then(() => ({ type: 'tick' })),
  },
};
