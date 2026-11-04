// SPDX-License-Identifier: AGPL-3.0-or-later
// A program found running here, a row of "Already running here": a kind
// svrngs knows named with what it needs first, any other with fields for
// its name and what it is; one button lends it (lend-found.js LEND), and
// the row then says what happened, or why not.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LEND } from './lend-found.js?v=__VERSION__';
import { needs } from './running-look.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Shown } from './running-look.js?v=__VERSION__' */

/** @type {[field: 'name' | 'kind', words: string][]} */
const FIELDS = [
  ['name', 'Its name: '],
  ['kind', 'What it is: '],
];

/**
 * A program found: it, the kinds the table knows by name alone, and
 * HOME's `dispatch`, told once it is lent.
 * @typedef {{ r: Shown, named: string[], home: Home }} FoundProps
 */

/** @param {FoundProps} props */
export function Found({ r: shown, named, home }) {
  const [state, send] = useMachine(() =>
    machine('LEND', LEND(home), {
      at: { lend: 'unlent' },
      port: shown.port,
      name: '',
      kind: '',
      said: '',
    }),
  );
  const at = `http://localhost:${shown.port}`;
  if (state.at.lend === 'lent') {
    // `state.l` is the lend 'lent' made, which set 'lent' with it.
    const lent = /** @type {Shown} */ (state.l);
    return html`<li>${`You lent ${lent.name}, at ${at}: everyone in your groups can open it now. ${needs(lent)} `}</li>`;
  }
  const known = named.length ? `; if it is ${named.join(' or ')}, say that` : '';
  const said = shown.kind
    ? `${shown.kind}, at ${at}. ${needs(shown)} `
    : `Something at ${at}. To lend it, give it a name and say what it is, each one word${known}. `;
  const fields =
    !shown.kind &&
    FIELDS.map(
      ([name, words]) =>
        html`<${Field} name=${name} words=${words} value=${state[name]} send=${send} />`,
    );
  const busy = state.at.lend === 'lending' ? 'busy' : 'idle';
  return html`
    <li>
      ${said}
      ${fields}
      <${Act}
        words=${`Lend ${shown.kind ?? 'it'} to everyone in your groups`}
        doing=${{ state: busy }}
        on=${{ type: 'lend' }}
        send=${send}
      />
      ${state.said}
    </li>
  `;
}
