// SPDX-License-Identifier: AGPL-3.0-or-later
// A program found running here, a row of "Already running here": a kind
// svrngs knows named with what it needs first, any other with fields for
// its name and what it is; who can open it, one box a group of HOME's,
// each by its stable id, so it is lent to the groups ticked and never to
// everyone by default (SCOPE_PARALLEL.md, "Shared API shape"); one button
// lends it (lend-found.js LEND), and the row then says what happened, or
// why not.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LEND } from './lend-found.js?v=__VERSION__';
import { GroupChoice } from './group-choice.js?v=__VERSION__';
import { needs } from './running-look.js?v=__VERSION__';
/** @import { Home } from './group.js?v=__VERSION__' */
/** @import { Choice } from './group-choice.js?v=__VERSION__' */
/** @import { Shown } from './running-look.js?v=__VERSION__' */

/** @type {[field: 'name' | 'kind', words: string][]} */
const FIELDS = [
  ['name', 'Its name: '],
  ['kind', 'What it is: '],
];

/**
 * A program found: it, the kinds the table knows by name alone, HOME's
 * groups to lend it to, and HOME's `dispatch`, told once it is lent.
 * @typedef {{ r: Shown, named: string[], groups: Choice[], home: Home }} FoundProps
 */

// The names of the groups ticked, "House, Trip", for the lend's button.
/**
 * @param {Choice[]} groups
 * @param {string[]} ticked
 */
const chosen = (groups, ticked) =>
  groups.filter((group) => ticked.includes(group.id)).map((group) => group.name ?? 'this group');

/** @param {FoundProps} props */
export function Found({ r: shown, named, groups, home }) {
  const [state, send] = useMachine(() =>
    machine('LEND', LEND(home), {
      at: { lend: 'unlent' },
      port: shown.port,
      name: '',
      kind: '',
      said: '',
      groups: [],
    }),
  );
  const at = `http://localhost:${shown.port}`;
  if (state.at.lend === 'lent') {
    // `state.l` is the lend 'lent' made, which set 'lent' with it.
    const lent = /** @type {Shown} */ (state.l);
    return html`<li>${`You lent ${lent.name}, at ${at}: the groups you chose can open it now. ${needs(lent)} `}</li>`;
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
  const names = chosen(groups, state.groups);
  const who = shown.kind ? `Who can open ${shown.kind}?` : 'Who can open it?';
  const words = names.length
    ? `Lend ${shown.kind ?? 'it'} to ${names.join(' and ')}`
    : `Lend ${shown.kind ?? 'it'}`;
  return html`
    <li>
      ${said}
      ${fields}
      <p>${who}</p>
      <${GroupChoice}
        groups=${groups}
        ticked=${state.groups}
        waiting=${busy === 'busy'}
        send=${send}
      />
      <${Act}
        words=${words}
        doing=${{ state: busy }}
        on=${{ type: 'lend' }}
        send=${send}
      />
      ${state.said}
    </li>
  `;
}
