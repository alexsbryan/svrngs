// SPDX-License-Identifier: AGPL-3.0-or-later
// Bringing things to the group: the shelf (ui/shelf.js), drawn in #shelf
// once the group is read, brings through the computer the page reaches
// (dial.js `through`), then has the group read again (enter.js `reread`).
import { html } from '../ui/view.js?v=__VERSION__';
import { Shelf } from '../ui/shelf.js?v=__VERSION__';
import { through } from './dial.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/**
 * @param {Group} group
 * @param {unknown} apps what the shelf brings, as ui/shelf.js `Shelf` asks it
 */
const bring = (group, apps) => through(group, '/shelf', apps);

/** @param {Props} props */
function GroupShelf({ s: state, send }) {
  const shelf =
    state.lent &&
    html`
      <${Shelf}
        g=${state.group}
        lent=${state.lent}
        shelf=${state.shelf}
        joined=${state.joined}
        bring=${bring}
        after=${() => send({ type: 'reread' })}
      />
    `;
  return html`<div id="shelf">${shelf}</div>`;
}

/** @type {Record<string, (props: Props) => Drawn>} */
export const view = { shelf: GroupShelf };
