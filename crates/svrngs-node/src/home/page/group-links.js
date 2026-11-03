// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's links (group-link.js LINK): a button for each link it makes,
// a guest's behind Have a guest, and below them the last link made
// (link-said.js), or why it was not. The machine is the group's own
// (ui/view.js `useMachine`), so HOME's step never sees it.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LINK } from './group-link.js?v=__VERSION__';
import { LinkSaid } from './link-said.js?v=__VERSION__';
/** @import { Group } from './group.js?v=__VERSION__' */

// A guest's visit, longest first and chosen first: its seconds, as the
// Field holds them, and its words.
const VISITS = [48, 24, 6].map((hours) => [hours * 3600, `${hours} hours`]);

/**
 * A group's links, made for it.
 * @typedef {{ g: Group }} LinksProps
 */

/** @param {LinksProps} props */
export function Links({ g: group }) {
  const [state, send] = useMachine(() =>
    machine('LINK', LINK(group.id), { at: { link: 'none' }, visit: '172800' }),
  );
  const name = group.name ?? 'this group';
  const doing = { state: state.at.link === 'making' ? 'busy' : 'idle' };
  // `state.visit` is the Field's string, as `Number` reads it.
  const hours = `${Number(state.visit) / 3600} hours`;
  const guest = state.guest
    ? html`
        <${Field}
          name="visit"
          words="For how long, from when they open the link: "
          value=${state.visit}
          options=${VISITS}
          send=${send}
        />
        <${Act}
          words="Make a link for your guest"
          doing=${doing}
          on=${{ type: 'make', device: false, guest: true, hours }}
          send=${send}
        />
      `
    : html`<${Act} words="Have a guest" on=${{ type: 'guest' }} send=${send} />`;
  const said =
    state.at.link === 'refused'
      ? state.why
      : state.shown && html`<${LinkSaid} shown=${state.shown} name=${name} />`;
  return html`
    <p>
      <${Act}
        words=${`Invite someone to ${name}`}
        doing=${doing}
        on=${{ type: 'make', device: false }}
        send=${send}
      />
      <${Act}
        words="Add your phone or another computer"
        doing=${doing}
        on=${{ type: 'make', device: true }}
        send=${send}
      />
      ${guest}
    </p>
    <p class="link">${said}</p>
  `;
}
