// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's offer to have svrngs keep its program running (lent-row.js
// `Lent`), and KEEP, its machine. It is made where /lent says the lend is
// shared only while its program runs, by its port (plan/words.rs
// `shared_while`), and svrngs runs no program of the lend's name (GET /run's
// rows, read once as the row is drawn). Keep it running asks what svrngs
// would start it with (GET /run?prefill=<port>, a llama-server's), shown with
// a fold to change it, or two fields to fill where nothing could be said.
// Start runs it under the lend's own name and port (POST /run): the keeper
// refuses while the person's own copy holds the port, and stops nothing.
// Once svrngs's copy answers, the lend is held for it, as for anything not
// approved; it is approved at once (`carry`) only where what answers is the
// job svrngs just started, a unit named for the lend, by the token /lent
// lists it with. That approval keeps sharing it through the job's restarts
// (share.rs `kept`), on Linux; on a Mac what answers is a process, and
// Keep sharing it is left for the person to press.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { assertNever, go, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
import { answered } from './running-look.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Lend } from '../question.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * The offer's state: where it is, nothing drawn while GET /run is read or
 * where svrngs runs the program; the command and folder, as svrngs said
 * them or as typed, and whether svrngs said them; what is said while it
 * starts (plan/words.rs `STARTING`); and what was said last.
 * @typedef {object} State
 * @property {{ keep: 'none' | 'offered' | 'filling' | 'asked' | 'starting' | 'started' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} [command]
 * @property {string} [folder]
 * @property {boolean} [filled]
 * @property {string} [starting]
 * @property {string} [said]
 */

/**
 * The messages it takes: GET /run read, and whether svrngs runs a program
 * of the lend's name, or not read; Keep it running pressed; the command
 * and folder said, or not; one typed; Start pressed; it started, with what
 * was said, or refused, with why.
 * @typedef {{ type: 'read', runs: boolean, starting: string, why: string }
 *   | { type: 'unread', why: string }
 *   | { type: 'keep' }
 *   | { type: 'filled', command: string, folder: string, why: string }
 *   | { type: 'unfilled', why: string }
 *   | { type: 'typed', field?: string, value: string }
 *   | { type: 'start' }
 *   | { type: 'started', said: string, why: string }
 *   | { type: 'refused', why: string }} Message
 */

/**
 * @typedef {{ run: 'read', failed: 'unread' }
 *   | { run: 'fill', failed: 'unfilled' }
 *   | { run: 'start', command: string, folder: string, failed: 'refused' }} Effect
 */

// Whether what answers on the lend `name`'s port is svrngs's job for it
// alone, a unit /lent names for the lend (running-look.js `answered`),
// listed with the token approving it takes.
/** @param {Lend | undefined} lend @param {string} name */
const isJob = (lend, name) => {
  const [job, ...more] = lend?.answers ?? [];
  return Boolean(lend?.token && job?.unit && !more.length && answered(job) === name);
};

// What answers on `name`'s port once svrngs's copy does, approved only
// where it is that job (`isJob`), by the token /lent lists it with; then
// HOME reads the list again. Started, it is never refused: a list not read
// or an approval refused is said after what Start said.
/**
 * @param {string} name
 * @param {HOME.PlaceProps['home']} home
 */
const carry = async (name, home) => {
  const listed = get('/lent').then(({ lent }) =>
    lent.find((/** @type {Lend} */ one) => one.name === name),
  );
  const lend = /** @type {Lend | undefined} */ (await listed.catch(() => undefined));
  const ours = isJob(lend, name);
  const approved = ours ? post(`/v1/lends/${name}/approve`, { token: lend?.token }) : listed;
  const refused = await approved.then(
    () => '',
    (/** @type {Error} */ error) => ` ${error.message}`,
  );
  home({ type: 'again' });
  return { refused, why: ours ? 'its job approved' : 'left to Keep sharing it' };
};

/**
 * @param {Lend & { sharing?: number }} lend
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
const KEEP = ({ name, sharing: port }, home) => ({
  states: { keep: ['none', 'offered', 'filling', 'asked', 'starting', 'started'] },
  io: {
    read: () =>
      get('/run').then(({ rows, starting }) => {
        const runs = rows.some((/** @type {{ name: string }} */ row) => row.name === name);
        return { type: 'read', runs, starting: starting ?? '', why: `runs ${name}: ${runs}` };
      }),
    fill: () =>
      get(`/run?prefill=${port}`).then((got) =>
        got?.kind === 'prefill' && got.port === port
          ? { type: 'filled', command: got.command, folder: got.folder, why: 'svrngs said them' }
          : { type: 'unfilled', why: got?.why ?? 'svrngs has nothing to say of it' },
      ),
    start: async ({ command, folder }) => {
      const said = await post('/run', { name, port, folder, command });
      const { refused, why } = await carry(name, home);
      return { type: 'started', said: `${said}${refused}`, why };
    },
  },
  step: (state, message) => {
    switch (message.type) {
      case 'read':
        return go(
          state,
          { keep: message.runs ? 'none' : 'offered' },
          { starting: message.starting },
        );
      // Not read, it is offered: Start says why, as the keeper refuses.
      case 'unread':
        return go(state, { keep: 'offered' });
      case 'keep':
        if (state.at.keep !== 'offered') return no(state, `it is ${state.at.keep}`);
        return go(state, { keep: 'filling' }, {}, { run: 'fill', failed: 'unfilled' });
      case 'filled':
        return go(
          state,
          { keep: 'asked' },
          { command: message.command, folder: message.folder, filled: true },
        );
      case 'unfilled':
        return go(state, { keep: 'asked' });
      case 'typed':
        return go(state, {}, { [message.field ?? 'command']: message.value, said: '' });
      case 'start':
        if (state.at.keep !== 'asked') return no(state, `it is ${state.at.keep}`);
        if (!state.command?.trim() || !state.folder?.trim())
          return go(state, {}, { said: 'Fill in both, then press Start.' });
        return go(
          state,
          { keep: 'starting' },
          { said: state.starting },
          { run: 'start', command: state.command, folder: state.folder, failed: 'refused' },
        );
      case 'started':
        return go(state, { keep: 'started' }, { said: message.said });
      case 'refused':
        return go(state, { keep: 'asked' }, { said: message.why });
      default:
        return assertNever(state, message, 'keeping it running');
    }
  },
});

// What follows what Start said, once it started: where /lent no longer
// shares the lend only while it runs, its approval follows svrngs's job
// through restarts, so who can open it after a sign-in; else, where it is
// held for svrngs's copy, Keep sharing it, to press.
/** @param {Lend & { sharing?: number }} lend @param {string} who */
const after = (lend, who) => {
  if (!lend.sharing) return who && ` ${who} can open it again after you sign in.`;
  return lend.token ? ' Press Keep sharing it so they can open it.' : '';
};

/** @typedef {{ state: State, doing: { state: string }, send: (message: Message) => void }} StepProps */

// The step Start asks in: the command and folder svrngs said, a fold to
// change them, or the two fields to fill where it said none; then what to
// do first, Start, and what was said.
/** @param {StepProps} props */
function Ask({ state, doing, send }) {
  // The two fields Start sends besides the lend's own name and port.
  const fields = html`
    <${Field} name="command" words="Command you start it with: " value=${state.command ?? ''} send=${send} />
    <${Field} name="folder" words="Folder it starts in: " value=${state.folder ?? ''} send=${send} />
  `;
  return html`
    ${
      state.filled
        ? html`<p><code>${state.command}</code> in <code>${state.folder}</code></p>
          <details><summary>Change</summary>${fields}</details>`
        : fields
    }
    <p>Quit the copy you started, then press Start.
      <${Act} words="Start" doing=${doing} on=${{ type: 'start' }} send=${send} /></p>
    <p role="status">${state.said}</p>
  `;
}

/**
 * The offer, drawn under a lend's fold: who can open it (`who`, the names
 * of its groups, or '' for none), and HOME's `dispatch`, told to read
 * again once it starts.
 * @param {{ lend: Lend & { sharing?: number }, who: string, home: HOME.PlaceProps['home'] }} props
 */
export function KeepRunning({ lend, who, home }) {
  // Where /lent offers it, GET /run is read first, and nothing drawn till then.
  const read = lend.sharing ? [{ run: 'read', failed: 'unread' }] : [];
  const [state, send] = useMachine(() =>
    machine('KEEP', KEEP(lend, home), { at: { keep: 'none' } }, /** @type {Effect[]} */ (read)),
  );
  const at = state.at.keep;
  if (at === 'started') return html`<p role="status">${state.said}${after(lend, who)}</p>`;
  if (!lend.sharing || at === 'none') return null;
  const doing = { state: at === 'filling' || at === 'starting' ? 'busy' : 'idle' };
  const offer = html`<p>${who && `${who} can open it only while it’s running. `}svrngs can keep it running, even after a restart.
    <${Act} words="Keep it running" doing=${doing} on=${{ type: 'keep' }} send=${send} /></p>`;
  const asked = at === 'asked' || at === 'starting';
  return html`<div role="group" aria-label=${lend.name}>
    ${asked ? html`<${Ask} state=${state} doing=${doing} send=${send} />` : offer}
  </div>`;
}
