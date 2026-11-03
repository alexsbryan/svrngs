// SPDX-License-Identifier: AGPL-3.0-or-later
// "Make a group" on the home page: the group's name and the person's, and
// Make a group (make.js MAKE). The machine is the form's own (ui/view.js
// `useMachine`), so HOME's step never sees it. This file, not make.js,
// names the place: the form imports make.js, so make.js cannot import it
// back (`import/no-cycle`), and the law reads MAKE from make.js.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { FIRST, MAKE } from './make.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PlaceProps } from '../home.js?v=__VERSION__' */

// Each field's name is a field of MAKE's `State`, by which `state[name]`
// reads what was typed in it.
/** @satisfies {['name' | 'person', string][]} */
const FIELDS = [
  ['name', 'A name for the group: '],
  ['person', 'Your name in it: '],
];

/** @param {PlaceProps} props */
function MakeForm({ home }) {
  const [state, send] = useMachine(() => machine('MAKE', MAKE(home), FIRST));
  const fields = FIELDS.map(
    ([name, words]) =>
      html`<${Field} name=${name} words=${words} value=${state[name]} send=${send} />`,
  );
  const doing = { state: state.at.make === 'making' ? 'busy' : 'idle', said: state.said };
  return html`
    <section id="make">
      <h2>Make a group</h2>
      <p>
        ${fields}
        <${Act} words="Make a group" doing=${doing} on=${{ type: 'make' }} send=${send} />
      </p>
      <p>${state.before}</p>
    </section>
  `;
}

/** @type {{ make: (props: PlaceProps) => Drawn }} */
export const view = { make: MakeForm };
