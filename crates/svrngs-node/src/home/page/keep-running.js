// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's offer to have svrngs keep its program running (lent-row.js
// `Lent`), and KEEP, its machine. It is made where /lent says the lend is
// shared only while its program runs, by its port (plan/words.rs
// `shared_while`: a lend kept to no job's restarts, so not one whose
// approval follows svrngs's copy). Keep it running asks the command and
// folder the person started it with, two empty fields: svrngs never reads
// how a program was started, since its arguments can hold a secret.
// Start runs it under the lend's own name and port (POST /run): the keeper
// refuses while the person's own copy holds the port, and stops nothing.
// Once svrngs's copy answers, the lend is held for it, as for anything not
// approved; `carry` asks to approve it at once, by the token /lent lists it
// with, only as the job svrngs keeps it to through restarts (`keep`): the
// approve route takes it only where svrngs verifies it is that job's
// (share.rs `kept`: on Linux its unit, on a Mac the launch agent the keeper
// runs it under), and else approves nothing, so the page never approves by
// itself what the person didn't see.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { assertNever, go, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { get, post } from './routes.js?v=__VERSION__';
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Lend } from '../question.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * The offer's state: where it is; the command and folder as typed; and
 * what was said last.
 * @typedef {object} State
 * @property {{ keep: 'offered' | 'asked' | 'starting' | 'started' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} [command]
 * @property {string} [folder]
 * @property {string} [said]
 */

/**
 * The messages it takes: Keep it running pressed; the command or folder
 * typed; Start pressed; it started, with what was said, or refused, with why.
 * @typedef {{ type: 'keep' }
 *   | { type: 'typed', field?: string, value: string }
 *   | { type: 'start' }
 *   | { type: 'started', said: string, why: string }
 *   | { type: 'refused', why: string }} Message
 */

/** @typedef {{ run: 'start', command: string, folder: string, failed: 'refused' }} Effect */

// What answers on `name`'s port once svrngs's copy does, approved by the
// token /lent lists it with only as the job svrngs keeps (`keep: true`;
// surface.rs `approve`); then HOME reads the list again. Started, it is
// never refused: a list not read or an approval refused is said after
// what Start said, and the lend stays held for Keep sharing it.
/**
 * @param {string} name
 * @param {HOME.PlaceProps['home']} home
 */
const carry = async (name, home) => {
  const listed = get('/lent').then(({ lent }) =>
    lent.find((/** @type {Lend} */ one) => one.name === name),
  );
  const lend = /** @type {Lend | undefined} */ (await listed.catch(() => undefined));
  const token = lend?.token;
  const approved = token ? post(`/v1/lends/${name}/approve`, { token, keep: true }) : listed;
  const refused = await approved.then(
    () => '',
    (/** @type {Error} */ error) => ` ${error.message}`,
  );
  home({ type: 'again' });
  return { refused, why: token ? 'approve asked, if kept' : 'nothing listed to approve' };
};

/**
 * @param {Lend & { sharing?: number }} lend
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
const KEEP = ({ name, sharing: port }, home) => ({
  states: { keep: ['offered', 'asked', 'starting', 'started'] },
  io: {
    start: async ({ command, folder }) => {
      const said = await post('/run', { name, port, folder, command });
      const { refused, why } = await carry(name, home);
      return { type: 'started', said: `${said}${refused}`, why };
    },
  },
  step: (state, message) => {
    switch (message.type) {
      case 'keep':
        if (state.at.keep !== 'offered') return no(state, `it is ${state.at.keep}`);
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
          { said: 'Starting…' },
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
// still held for svrngs's copy (not carried, or the approval refused),
// Keep sharing it, to press.
/** @param {Lend & { sharing?: number }} lend @param {string} who */
const after = (lend, who) => {
  if ((lend.sharing ?? 0) === 0) return who && ` ${who} can open it again after you sign in.`;
  return lend.token ? ' Press Keep sharing it so they can open it.' : '';
};

/** @typedef {{ state: State, doing: { state: string }, send: (message: Message) => void }} StepProps */

// The step Start asks in: the two fields Start sends besides the lend's
// own name and port, empty, the command's example made of the lend's
// program, where its kind is one word, and port; then what to do first,
// Start, and what was said.
/** @param {StepProps & { lend: Lend & { sharing?: number } }} props */
function Ask({ state, doing, send, lend }) {
  const program = /^[\w.+-]+$/.test(lend.kind) ? lend.kind : lend.name;
  const example = { placeholder: `${program} --port ${lend.sharing}` };
  return html`
    <${Field} name="command" words="The command you started it with: " value=${state.command ?? ''} attrs=${example} send=${send} />
    <${Field} name="folder" words="The folder you started it from: " value=${state.folder ?? ''} send=${send} />
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
  const [state, send] = useMachine(() =>
    machine('KEEP', KEEP(lend, home), { at: { keep: 'offered' } }),
  );
  const at = state.at.keep;
  if (at === 'started') return html`<p role="status">${state.said}${after(lend, who)}</p>`;
  if ((lend.sharing ?? 0) === 0) return null;
  const doing = { state: at === 'starting' ? 'busy' : 'idle' };
  const offer = html`<p>${who && `${who} can open it only while it’s running. `}svrngs can keep it running, even after a restart.
    <${Act} words="Keep it running" doing=${doing} on=${{ type: 'keep' }} send=${send} /></p>`;
  const asked = at === 'asked' || at === 'starting';
  return html`<div role="group" aria-label=${lend.name}>
    ${asked ? html`<${Ask} state=${state} doing=${doing} send=${send} lend=${lend} />` : offer}
  </div>`;
}
