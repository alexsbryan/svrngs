// SPDX-License-Identifier: AGPL-3.0-or-later
// Play: a port a member lends, UDP or TCP, opened on this computer while
// Play is on, for a program here to connect to, said as /v1/play says it
// (play.rs `Open::said`), its address copied as it is turned on. PLAY is
// the page's one read of /v1/play, asked as this file loads and again once
// a round of the groups' reads. Each read is counted, so an older answer
// is refused by name. It feeds two lists: the index's cell for each port
// (play-cell.js), and each Play on with its Stop under "What acts as you"
// (`part`, for acting.js). What has Play on that cannot be read is said
// once, in `part`, never read as none; Play then writes it over.
import { html, useStore } from '../ui/view.js?v=__VERSION__';
import { ask, assertNever, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act } from '../ui/parts.js?v=__VERSION__';
import { keyOf, same } from './lend-key.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/**
 * A port another computer lends, by that computer's key and the lend's
 * name, and its lender's name when it is said.
 * @typedef {{ key: string, name: string, person?: string | null }} Port
 */

/**
 * A port with Play on, as /v1/play answers it (play.rs `Playing`): where
 * it is open or why not (play.rs `Open`, by `is`), and that said.
 * @typedef {object} Playing
 * @property {string | null} person
 * @property {string} key
 * @property {string} name
 * @property {number} port
 * @property {{ is: 'at', words: string | null } | { is: 'apart' | 'unread' | 'taken' } | { is: 'unopened', words: string }} open
 * @property {string} said
 */

/**
 * PLAY's state: where its read or turn is, the round last read, each
 * port's refusals and what its press copied, by `keyOf`, what has Play on,
 * and, while a turn is under way, where it was and the port it turns.
 * @typedef {object} State
 * @property {{ play: 'reading' | 'read' | 'unread' | 'turning' }} at
 * @property {Record<string, number>} [asked]
 * @property {number} round
 * @property {Record<string, string>} said
 * @property {Record<string, string>} copied
 * @property {Playing[]} [plays]
 * @property {State['at']['play']} [was]
 * @property {Port} [l]
 * @property {string} [why]
 */

/**
 * What has Play on, read or turned `n`, and what a turn on copied.
 * @typedef {{ type: 'answered', n: number, plays: Playing[], why: string, copied?: string }} Answered
 */

/**
 * The messages PLAY takes: a round's read; a Play or Stop pressed; what
 * has Play on; or a read or turn that failed, with why. A read and a turn
 * carry no read's number: `old` reads `message.n` of every message.
 * @typedef {{ type: 'read', round: number, n?: undefined }
 *   | { type: 'turn', l: Port, off: boolean, n?: undefined }
 *   | Answered
 *   | { type: 'unread', n: number, why: string }
 *   | { type: 'refused', n: number, why: string }} Message
 */

/**
 * The effects its step answers: read `n` of what has Play on, and turn
 * `n`, Play on or off for the port `l`.
 * @typedef {{ run: 'read', n: number, failed: 'unread' }
 *   | { run: 'turn', n: number, l: Port, off: boolean, failed: 'refused' }} Effect
 */

const answer =
  /** @param {number} serial */
  (serial) =>
    /**
     * @param {Playing[]} playing
     * @returns {Answered}
     */
    (playing) => ({ type: 'answered', n: serial, plays: playing, why: `${playing.length} on` });
// Where a program here connects to what Play opened, copied, or why not.
/** @param {Playing | undefined} playing */
const copy = (playing) =>
  playing?.open.is === 'at'
    ? navigator.clipboard.writeText(`127.0.0.1:${playing.port}`).then(
        () => ' Copied.',
        () => ' Copy it from here.',
      )
    : '';
// A turn, a cell's Play or a Stop, is PLAY's: answered by what has Play on
// then, written over what was read, with what it copied kept for its lend
// (`copied`), and each refusal appended to its lend's (`said`), so every
// list draws it.
/** @type {Machine<State, Message, Effect>} */
export const PLAY = {
  states: { play: ['reading', 'read', 'unread', 'turning'] },
  io: {
    read: ({ n: serial }) => get('/v1/play').then(answer(serial)),
    turn: ({ n: serial, l: lend, off }) =>
      post('/v1/play', { [off ? 'off' : 'on']: lend }).then(async (playing) => ({
        ...answer(serial)(playing),
        copied: off ? '' : await copy(playing.find(same(lend))),
      })),
  },
  // Each message's read numbered; an answer older than the last read asked
  // refused by name, and a read or a turn while a turn is under way, before
  // any case takes them.
  step: (state, message) => {
    const [asked, serial] = ask(state, 'play');
    const old = message.n === undefined ? '' : late(state, 'play', message.n);
    if (old) return no(state, old);
    if (state.at.play === 'turning' && ['read', 'turn'].includes(message.type))
      return no(state, 'a turn is under way, and its answer is what has Play on');
    // A refusal, and an answer with what it copied, answer a turn, which
    // set `state.l`.
    switch (message.type) {
      case 'read':
        if (message.round <= state.round)
          return no(state, `round ${message.round} is read already`);
        return go(
          state,
          { play: 'reading' },
          { asked, round: message.round },
          { run: 'read', n: serial, failed: 'unread' },
        );
      case 'turn':
        return go(
          state,
          { play: 'turning' },
          { asked, was: state.at.play, l: message.l },
          { run: 'turn', n: serial, l: message.l, off: message.off, failed: 'refused' },
        );
      case 'refused': {
        const key = keyOf(/** @type {Port} */ (state.l));
        const said = `${state.said[key] ?? ''} ${message.why}`;
        return go(state, { play: state.was }, { said: { ...state.said, [key]: said } });
      }
      case 'answered': {
        // A read copies nothing, and keeps what a turn copied before it.
        const copied =
          message.copied === undefined
            ? state.copied
            : { ...state.copied, [keyOf(/** @type {Port} */ (state.l))]: message.copied };
        return go(state, { play: 'read' }, { plays: message.plays, copied });
      }
      case 'unread':
        return go(state, { play: 'unread' }, { why: message.why });
      default:
        return assertNever(state, message, 'Play');
    }
  },
};
export const play = machine(
  'PLAY',
  PLAY,
  { at: { play: 'reading' }, asked: { play: 1 }, round: 1, said: {}, copied: {} },
  [{ run: 'read', n: 1, failed: 'unread' }],
);
/**
 * @param {State} state
 * @param {Port} lend
 */
const on = (state, lend) =>
  state.at.play === 'unread' ? undefined : state.plays?.find(same(lend));

/**
 * A port's Play or Stop: the port, and, in "What acts as you", what is
 * said of it before its Stop.
 * @typedef {{ l: Port, words?: (l: Port) => string }} TurnProps
 */

// A port's Play or Stop, in either list, drawn from PLAY alone: what has
// Play on said, with what its press copied, and why a turn was refused. It
// stays beside PLAY, whose store it reads, as `plays` draws it, and
// web/tests/pages/play.test.mjs imports PLAY and `plays` from this file.
/** @param {TurnProps} props */
export function Turn({ l: lend, words }) {
  const state = useStore(play);
  const open = on(state, lend);
  const said = open
    ? `${open.said}${state.copied[keyOf(lend)] ?? ''}`
    : 'Play opens it on this computer, for a program here to connect to.';
  const before = words ? words(lend) : `${said} `;
  const Tag = words ? 'p' : 'span';
  const doing = { state: state.at.play === 'turning' ? 'busy' : 'idle' };
  const turn = { type: 'turn', l: { key: lend.key, name: lend.name }, off: Boolean(open) };
  return html`
    <${Tag}>
      ${before}
      <${Act}
        words=${open ? 'Stop' : 'Play'}
        doing=${doing}
        on=${turn}
        send=${play.dispatch}
      />
      ${state.said[keyOf(lend)]}
    <//>
  `;
}

/** @param {Port} port */
const stop = (port) => {
  const whose = port.person ? ` (${port.person}’s)` : '';
  return `Open on this computer: ${port.name}${whose}, while Play is on. `;
};

// Each Play on, for acting.js, with its Stop, or why what has Play on
// cannot be read; none when nothing has.
/**
 * @param {State} state
 * @returns {Drawn[]}
 */
export function plays(state) {
  if (state.at.play === 'unread') return [html`<p>${state.why}</p>`];
  return (state.plays ?? []).map(
    (port) => html`<${Turn} key=${keyOf(port)} l=${port} words=${stop} />`,
  );
}

// A hook: acting.js calls it on each of its draws, so the list is drawn
// again with each read.
export const part = () => plays(useStore(play));
