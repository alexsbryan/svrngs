// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend in "What you lend": how many it lets in at once and how many use
// it now, with a button for each change (lend-change.js CHANGE), and the
// app form's fill once signing people in is turned on (signing-on.js).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { CHANGE } from './lend-change.js?v=__VERSION__';
import { SigningOn } from './signing-on.js?v=__VERSION__';
/** @import { Lend } from '../question.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * A lend's row: the lend, whether it signs people in with svrngs, and
 * HOME's `dispatch`, told to read again once it changes.
 * @typedef {object} LentProps
 * @property {Lend} l
 * @property {boolean} signs
 * @property {HOME.PlaceProps['home']} home
 */

/** @param {LentProps} props */
export function Lent({ l: lend, signs, home }) {
  const [state, send] = useMachine(() =>
    machine('CHANGE', CHANGE(lend.name, home), { at: { change: 'idle' }, said: '' }),
  );
  const limit = lend.limit;
  const limited = (limit ?? 0) > 0;
  const people = limit === 1 ? 'person' : 'people';
  const howMany = limited ? `up to ${limit} ${people}` : 'everyone in your groups';
  const words = `${lend.name}: ${howMany} at once, ${lend.using} using it now. `;
  const value = state.many ?? String(limit ?? '');
  const doing = { state: state.at.change === 'changing' ? 'busy' : 'idle' };
  const letMany = html`
    <${Act} words="Let in this many" doing=${doing} on=${{ type: 'many', value }} send=${send} />
  `;
  const letEveryone =
    limited &&
    html`
      <${Act}
        words="Let everyone in"
        doing=${doing}
        on=${{ type: 'change', to: { limit: null } }}
        send=${send}
      />
    `;
  const signIn = signs
    ? html`Sign-in credentials created; app setup still required. Follow the app’s OpenID Connect setup guide, then test sign-in through your group’s app list. <a href="https://github.com/alexsbryan/svrngs/tree/main/apps">App setup guides</a>. `
    : html`
        <${Act}
          words="Sign people in with svrngs"
          doing=${doing}
          on=${{ type: 'sign' }}
          send=${send}
        />
      `;
  const stop = html`
    <${Act} words="Stop lending" doing=${doing} on=${{ type: 'change', to: 'stop' }} send=${send} />
  `;
  return html`
    <li>
      ${words}
      <${Field}
        words="How many people at once: "
        value=${value}
        attrs=${{ type: 'number', min: 1 }}
        send=${send}
      />
      ${letMany}
      ${letEveryone}
      ${signIn}
      ${stop}
      ${state.said}
      ${state.shown && html`<${SigningOn} c=${state.shown} />`}
    </li>
  `;
}
