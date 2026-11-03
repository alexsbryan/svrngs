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
 * lend's refusals, by `k`, what a press here closed, what is open, and,
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
 * read's number: `old` reads `m.n` of every message.
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
     * @param {number} n
     * @param {Turned} [turned]
     */
    (n, turned) =>
      /**
       * @param {InRoom[]} room
       * @returns {Answered}
       */
      (room) => ({
        type: 'answered',
        n,
        room,
        turned,
        why: `${room.length} open`,
      }),
  same = /** @param {Which} a */ (a) => /** @param {Which} b */ (b) =>
    a.key === b.key && a.name === b.name,
  k = /** @param {Which} l */ (l) => `${l.key}:${l.name}`;
// A turn is answered by what is open then, written over what was read, and
// what it closed is kept, said closed in both lists until the page is left.
/** @type {Machine<State, Message, Effect>} */
export const ROOM = {
  states: { room: ['reading', 'read', 'unread', 'turning'] },
  io: {
    read: ({ n }) => get('/v1/room').then(answer(n)),
    turn: ({ n, l, off }) =>
      post('/v1/room', { [off ? 'off' : 'on']: { key: l.key, name: l.name } }).then(
        answer(n, { l, off }),
      ),
  },
  // Each message's read numbered; an answer older than the last read asked
  // refused by name, and a read or a turn while a turn is under way, before
  // any case takes them.
  step: (s, m) => {
    const [asked, n] = ask(s, 'room');
    const old = m.n && late(s, 'room', m.n);
    if (old) return no(s, old);
    if (s.at.room === 'turning' && ['read', 'turn'].includes(m.type))
      return no(s, 'a turn is under way, and its answer is what is open');
    // A refusal answers a turn, which set `s.l`.
    switch (m.type) {
      case 'read':
        return m.round <= s.round
          ? no(s, `round ${m.round} is read already`)
          : go(
              s,
              { room: 'reading' },
              { asked, round: m.round },
              { run: 'read', n, failed: 'unread' },
            );
      case 'turn':
        return go(
          s,
          { room: 'turning' },
          { asked, was: s.at.room, l: m.l },
          { run: 'turn', n, l: m.l, off: m.off, failed: 'refused' },
        );
      case 'refused':
        return go(
          s,
          { room: s.was },
          { said: { ...s.said, [k(/** @type {Which} */ (s.l))]: ` ${m.why}` } },
        );
      case 'answered': {
        // The turn read once, as tsc does not narrow `m.turned` inside
        // the filter's callback.
        const turned = m.turned;
        return go(
          s,
          { room: 'read' },
          {
            room: m.room,
            closed: turned
              ? [...s.closed.filter((c) => !same(c)(turned.l)), ...(turned.off ? [turned.l] : [])]
              : s.closed,
          },
        );
      }
      case 'unread':
        return go(s, { room: 'unread' }, { why: m.why });
      default:
        return assertNever(s, m, 'the room');
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
 * @param {State} s
 * @param {Which} l
 */
const on = (s, l) => (s.at.room === 'unread' ? undefined : s.room?.find(same(l)));

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
  step: (s, m) => {
    switch (m.type) {
      case 'copy':
        return go(s, {}, {}, { run: 'copy', failed: 'copied' });
      case 'copied':
        return go(s, { copy: 'copied' }, { said: m.words ?? ` Copy it from here: ${at}` });
      default:
        return assertNever(s, m, 'a copy');
    }
  },
});
/**
 * A copy of the lend's address `at`, for a program.
 * @typedef {{ at: string }} CopyProps
 */

/** @param {CopyProps} props */
const Copy = ({ at }) =>
  (([s, send]) =>
    html`${' '}<${Act} words="Copy its address for a program" on=${{ type: 'copy' }} send=${send} />${' '}<${Keep} at=${at} />${s.said}`)(
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
const Line = ({ l, words, foot }) =>
  ((s, r = on(s, l)) =>
    html`<li>${words}<span>${` ${r?.said ?? `${s.closed.some(same(l)) ? 'Closed to this WiFi: nobody on it can open it through this computer now. ' : ''}Open it to this WiFi, and anyone on it can open it as you while svrngs runs.`} `}<${Act} words=${r ? 'Close it to this WiFi' : 'Open it to this WiFi'} doing=${{ state: s.at.room === 'turning' ? 'busy' : 'idle' }} on=${{ type: 'turn', l: r ?? { key: l.key, name: l.name }, off: Boolean(r) }} send=${room.dispatch} />${s.said[k(l)]}</span>${foot && html`<${Copy} at=${l.origin} />`}</li>`)(
    useStore(room),
  );
// Each lend of the group's that is a page, at the section's foot (home.js@19b63b2b:202),
// once the room has answered, as v1 draws none before (home.js@19b63b2b:377).
/**
 * @param {PlaceProps} props
 * @returns {Drawn}
 */
function Foot({ s }) {
  useEffect(() => room.dispatch({ type: 'read', round: s.asked?.lends ?? 1 }), [s.asked?.lends]);
  const r = useStore(room),
    people = s.people?.people ?? [],
    whose = /** @param {Lent} l */ (l) => ({
      ...l,
      called: called(people, people.find((r) => r.key === l.key)?.person_key),
    });
  return (
    s.lends &&
    (r.room || r.at.room === 'unread') &&
    html`<ul class="wifi">${once(s.lends.lent)
      .filter((l) => !l.raw)
      .map((l) =>
        ((w) => html`<${Line} key=${k(l)} l=${w} words=${`${named(w)}:`} foot />`)(whose(l)),
      )}</ul>`
  );
}
export const view = { wifi: Foot };

// "Open to this WiFi", for acts.js: what ROOM read, each with its line, or
// why it cannot be read; none when nothing is open. A hook: acts.js calls
// it on each of its draws, so it is drawn again with each read.
/**
 * @param {State} s
 * @param {Which[]} [all]
 * @returns {Drawn[]}
 */
export const rooms = (
  s,
  all = [...(s.at.room === 'unread' ? [] : (s.room ?? [])), ...s.closed.filter((c) => !on(s, c))],
) =>
  s.at.room === 'unread' || all.length
    ? [
        html`<p>${s.at.room === 'unread' ? s.why : 'Anyone on this WiFi can open these as you while svrngs runs: your TV, a phone, another computer. Each stays on the WiFi where you turned it on.'}</p>`,
        html`<ul id="room">${all.map((r) => html`<${Line} key=${k(r)} l=${r} words=${`${r.person ? `${r.person}’s ` : ''}${r.name}:`} />`)}</ul>`,
      ]
    : [];
export const part = () => rooms(useStore(room));
