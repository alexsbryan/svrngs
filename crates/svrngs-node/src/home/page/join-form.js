// SPDX-License-Identifier: AGPL-3.0-or-later
// "Join a group from a link" on the home page: the link's box, the name
// box, and Join, which names the group once the link is asked of (join.js
// JOIN). The machine is the form's own (ui/view.js `useMachine`), so
// HOME's step never sees it. This file, not join.js, names the place: the
// form imports join.js, so join.js cannot import it back
// (`import/no-cycle`), and the law reads JOIN from join.js.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { FIRST, JOIN, who } from './join.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PlaceProps } from './home-reading.js?v=__VERSION__' */

/** @param {PlaceProps} props */
function JoinForm({ home }) {
  const [state, send] = useMachine(() => machine('JOIN', JOIN(home), FIRST));
  /** @param {{ currentTarget: HTMLInputElement }} typing */
  const typed = (typing) =>
    send({ type: 'typed', field: 'person', value: typing.currentTarget.value });
  const as = who(state) ? ` as ${who(state)}` : '';
  const doing = { state: state.at.join === 'joining' ? 'busy' : 'idle' };
  const join =
    state.for &&
    html`
      <${Act}
        words=${`Join ${state.for[0]}${as}`}
        doing=${doing}
        on=${{ type: 'join' }}
        send=${send}
      />
    `;
  return html`
    <section id="join">
      <h2>Join a group from a link</h2>
      <p>
        <${Field}
          name="link"
          words="The link someone sent you: "
          value=${state.link}
          send=${send}
        />
        <label hidden=${Boolean(state.for?.[1])}>
          ${'Your name in it: '}
          <input name="person" value=${state.person} onInput=${typed} />
        </label>
        ${join}
      </p>
      <p>${state.why}</p>
    </section>
  `;
}

/** @type {{ join: (props: PlaceProps) => Drawn }} */
export const view = { join: JoinForm };
