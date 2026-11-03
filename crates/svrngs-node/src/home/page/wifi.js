// SPDX-License-Identifier: AGPL-3.0-or-later
// What is open to this WiFi (page-home-rooms (a); home.js `wifi` and the
// foot, :69-80, :202-203, :377-381; DESIGN §3, "The room"): a lend opened
// to this WiFi, or closed, for any device on it, a TV among them, said as
// /v1/room says it (room.rs `Served::said`). ROOM, the page's one read of
// /v1/room, asked as this file loads, beside HOME's read of the groups, as
// v1 reads it before it draws a group (home.js@19b63b2b:377), feeds both of its lists:
// the foot of each group's section (`ul.wifi`, its `view`), each lend with
// its address for a program, and "Open to this WiFi" under "What acts as
// you" (`part`, for acts.js, `ul#room`). Each read is counted, so an older
// answer is refused by name, and is asked again on each of the groups'
// rounds (HOME's `again` and `visible`), once a round however many groups
// draw a foot. What is open that cannot be read is said once, in `part`,
// never read as none; a press in a line then writes it over.
import {
  Act,
  ask,
  assertNever,
  called,
  go,
  html,
  Keep,
  late,
  machine,
  named,
  no,
  once,
  useEffect,
  useMachine,
  useStore,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';
/** @import { Lent, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/**
 * A lend by its lender's computer's key and its name, as ROOM is asked to
 * turn it, and its lender's name when it is said.
 * @typedef {{ key: string, name: string, person?: string | null }} Which
 */

/**
 * Where a lend open to this WiFi is served now, or why not (room.rs
 * `Served`, serde's snake_case): at an address, or closed for now.
 * @typedef {{ at: string } | 'elsewhere' | 'unread' | 'no_address' | 'taken'} Served
 */

/**
 * A lend open to this WiFi, as /v1/room answers it (room.rs `InRoom`): its
 * lender, `null` for a key no group held here names now, where it is
 * served, and that said.
 * @typedef {object} InRoom
 * @property {string | null} person
 * @property {string} key
 * @property {string} name
 * @property {Served} served
 * @property {string} said
 */

/**
 * A turn answered: the lend it turned, and whether off.
 * @typedef {{ l: Which, off: boolean }} Turned
 */

/**
 * ROOM's state: where its read or turn is, the round last read, each
 * lend's refusals, by `keyOf`, what a press here closed, what is open, and,
 * while a turn is under way, where it was and the lend it turns.
 * @typedef {object} State
 * @property {{ room: 'reading' | 'read' | 'unread' | 'turning' }} at
 * @property {Record<string, number>} [asked]
 * @property {number} round
 * @property {Which[]} closed
 * @property {Record<string, string>} said
 * @property {InRoom[]} [room]
 * @property {State['at']['room']} [was]
 * @property {Which} [l]
 * @property {string} [why]
 */

/**
 * What is open, read or turned `n`, and the turn it answers.
 * @typedef {{ type: 'answered', n: number, room: InRoom[], turned?: Turned, why: string }} Answered
 */

/**
 * The messages ROOM takes: a round's read; a line's press; what is open;
 * or a read or turn that failed, with why. A read and a turn carry no
 * read's number: `old` reads `message.n` of every message.
 * @typedef {{ type: 'read', round: number, n?: undefined }
 *   | { type: 'turn', l: Which, off: boolean, n?: undefined }
 *   | Answered
 *   | { type: 'unread', n: number, why: string }
 *   | { type: 'refused', n: number, why: string }} Message
 */

/**
 * The effects its step answers: read `n` of what is open, and turn `n`,
 * the lend `l` opened or closed to this WiFi.
 * @typedef {{ run: 'read', n: number, failed: 'unread' }
 *   | { run: 'turn', n: number, l: Which, off: boolean, failed: 'refused' }} Effect
 */

const answer =
    /**
     * @param {number} serial
     * @param {Turned} [turned]
     */
    (serial, turned) =>
      /**
       * @param {InRoom[]} room
       * @returns {Answered}
       */
      (room) => ({
        type: 'answered',
        n: serial,
        room,
        turned,
        why: `${room.length} open`,
      }),
  same = /** @param {Which} one */ (one) => /** @param {Which} other */ (other) =>
    one.key === other.key && one.name === other.name,
  keyOf = /** @param {Which} lend */ (lend) => `${lend.key}:${lend.name}`;
// A turn is answered by what is open then, written over what was read, and
// what it closed is kept, said closed in both lists until the page is left.
/** @type {Machine<State, Message, Effect>} */
export const ROOM = {
  states: { room: ['reading', 'read', 'unread', 'turning'] },
  io: {
    read: ({ n: serial }) => get('/v1/room').then(answer(serial)),
    turn: ({ n: serial, l: lend, off }) =>
      post('/v1/room', { [off ? 'off' : 'on']: { key: lend.key, name: lend.name } }).then(
        answer(serial, { l: lend, off }),
      ),
  },
  // Each message's read numbered; an answer older than the last read asked
  // refused by name, and a read or a turn while a turn is under way, before
  // any case takes them.
  step: (state, message) => {
    const [asked, serial] = ask(state, 'room');
    const old = message.n && late(state, 'room', message.n);
    if (old) return no(state, old);
    if (state.at.room === 'turning' && ['read', 'turn'].includes(message.type))
      return no(state, 'a turn is under way, and its answer is what is open');
    // A refusal answers a turn, which set `state.l`.
    switch (message.type) {
      case 'read':
        return message.round <= state.round
          ? no(state, `round ${message.round} is read already`)
          : go(
              state,
              { room: 'reading' },
              { asked, round: message.round },
              { run: 'read', n: serial, failed: 'unread' },
            );
      case 'turn':
        return go(
          state,
          { room: 'turning' },
          { asked, was: state.at.room, l: message.l },
          { run: 'turn', n: serial, l: message.l, off: message.off, failed: 'refused' },
        );
      case 'refused':
        return go(
          state,
          { room: state.was },
          { said: { ...state.said, [keyOf(/** @type {Which} */ (state.l))]: ` ${message.why}` } },
        );
      case 'answered': {
        // The turn read once, as tsc does not narrow `message.turned` inside
        // the filter's callback.
        const turned = message.turned;
        return go(
          state,
          { room: 'read' },
          {
            room: message.room,
            closed: turned
              ? [
                  ...state.closed.filter((closed) => !same(closed)(turned.l)),
                  ...(turned.off ? [turned.l] : []),
                ]
              : state.closed,
          },
        );
      }
      case 'unread':
        return go(state, { room: 'unread' }, { why: message.why });
      default:
        return assertNever(state, message, 'the room');
    }
  },
};
const room = machine(
  'ROOM',
  ROOM,
  { at: { room: 'reading' }, asked: { room: 1 }, round: 1, closed: [], said: {} },
  [{ run: 'read', n: 1, failed: 'unread' }],
);
/**
 * @param {State} state
 * @param {Which} lend
 */
const on = (state, lend) => (state.at.room === 'unread' ? undefined : state.room?.find(same(lend)));

// A copy of the lend's address for a program, said beside it once, the
// last press's words in place of the one before, where v1 appends each
// (home.js@19b63b2b:202).
/**
 * A copy's state: whether it is copied, and its last press's words.
 * @typedef {{ at: { copy: 'uncopied' | 'copied' }, asked?: Record<string, number>, said: string }} Copied
 */

/**
 * The messages a copy takes: Copy pressed; and the address copied, in its
 * words, or why not.
 * @typedef {{ type: 'copy' } | { type: 'copied', words?: string, why?: string }} Copying
 */

/**
 * @param {string} at
 * @returns {Machine<Copied, Copying, { run: 'copy', failed: 'copied' }>}
 */
const COPY = (at) => ({
  states: { copy: ['uncopied', 'copied'] },
  io: {
    copy: () =>
      Promise.resolve()
        .then(() => navigator.clipboard.writeText(at))
        .then(
          () => ({
            type: 'copied',
            words: ` Copied: ${at}. Paste it where a program on this computer, such as a chat app, asks for an address.`,
          }),
          () => ({ type: 'copied', why: 'the clipboard refused' }),
        ),
  },
  step: (state, message) => {
    switch (message.type) {
      case 'copy':
        return go(state, {}, {}, { run: 'copy', failed: 'copied' });
      case 'copied':
        return go(
          state,
          { copy: 'copied' },
          { said: message.words ?? ` Copy it from here: ${at}` },
        );
      default:
        return assertNever(state, message, 'a copy');
    }
  },
});
/**
 * A copy of the lend's address `at`, for a program.
 * @typedef {{ at: string }} CopyProps
 */

/** @param {CopyProps} props */
const Copy = ({ at }) =>
  (([state, send]) =>
    html`${' '}<${Act} words="Copy its address for a program" on=${{ type: 'copy' }} send=${send} />${' '}<${Keep} at=${at} />${state.said}`)(
    useMachine(() => machine('COPY', COPY(at), { at: { copy: 'uncopied' }, said: '' })),
  );
// One lend's line, in either list, drawn from ROOM alone: what is open
// said, or closed, if a press here closed it, and why a turn was refused.
/**
 * One lend's line: the lend, at its origin in a group's foot, what is said
 * of it before its press, and whether it is in the foot, with its Copy.
 * @typedef {{ l: Which & { origin?: string }, words: string, foot?: boolean }} LineProps
 */

/** @param {LineProps} props */
const Line = ({ l: lend, words, foot }) =>
  ((state, open = on(state, lend)) =>
    html`<li>${words}<span>${` ${open?.said ?? `${state.closed.some(same(lend)) ? 'Closed to this WiFi: nobody on it can open it through this computer now. ' : ''}Open it to this WiFi, and anyone on it can open it as you while svrngs runs.`} `}<${Act} words=${open ? 'Close it to this WiFi' : 'Open it to this WiFi'} doing=${{ state: state.at.room === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', l: open ?? { key: lend.key, name: lend.name }, off: Boolean(open) }} send=${room.dispatch} />${state.said[keyOf(lend)]}</span>${foot && html`<${Copy} at=${lend.origin} />`}</li>`)(
    useStore(room),
  );
// Each lend of the group's that is a page, at the section's foot (home.js@19b63b2b:202),
// once the room has answered, as v1 draws none before (home.js@19b63b2b:377).
/**
 * @param {PlaceProps} props
 * @returns {Drawn}
 */
function Foot({ s: state }) {
  useEffect(
    () => room.dispatch({ type: 'read', round: state.asked?.lends ?? 1 }),
    [state.asked?.lends],
  );
  const roomState = useStore(room),
    people = state.people?.people ?? [],
    whose = /** @param {Lent} lend */ (lend) => ({
      ...lend,
      called: called(people, people.find((row) => row.key === lend.key)?.person_key),
    });
  return (
    state.lends &&
    (roomState.room || roomState.at.room === 'unread') &&
    html`<ul class="wifi">${once(state.lends.lent)
      .filter((lend) => !lend.raw)
      .map((lend) =>
        ((owned) =>
          html`<${Line} key=${keyOf(lend)} l=${owned} words=${`${named(owned)}:`} foot />`)(
          whose(lend),
        ),
      )}</ul>`
  );
}
export const view = { wifi: Foot };

// "Open to this WiFi", for acts.js: what ROOM read, each with its line, or
// why it cannot be read; none when nothing is open. A hook: acts.js calls
// it on each of its draws, so it is drawn again with each read.
/**
 * @param {State} state
 * @param {Which[]} [all]
 * @returns {Drawn[]}
 */
export const rooms = (
  state,
  all = [
    ...(state.at.room === 'unread' ? [] : (state.room ?? [])),
    ...state.closed.filter((closed) => !on(state, closed)),
  ],
) =>
  state.at.room === 'unread' || all.length
    ? [
        html`<p>${state.at.room === 'unread' ? state.why : 'Anyone on this WiFi can open these as you while svrngs runs: your TV, a phone, another computer. Each stays on the WiFi where you turned it on.'}</p>`,
        html`<ul id="room">${all.map((row) => html`<${Line} key=${keyOf(row)} l=${row} words=${`${row.person ? `${row.person}’s ` : ''}${row.name}:`} />`)}</ul>`,
      ]
    : [];
export const part = () => rooms(useStore(room));
