// SPDX-License-Identifier: AGPL-3.0-or-later
// Who is visiting a group, in its section on the home page (GUESTS §5 "The
// pages"), from the record as every member's page shows them, once GROUP
// has read its people: who you have over first, in words that are not an
// app's noun, each with its Stop (stop-visit.js); then everyone's, by host,
// with until when (ui/visitors.js).
import { html } from '../ui/view.js?v=__VERSION__';
import { Visiting } from '../ui/visitors.js?v=__VERSION__';
import { Stop } from './stop-visit.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/** @param {PlaceProps} props */
function Visits({ g: group, s: state, home }) {
  if (!state.people) return null;
  const rows = state.people.people;
  const name = group.name ?? 'this group';
  const visiting = group.visiting;
  const unknown = !visiting && html`<p>This computer cannot say who is visiting ${name} now.</p>`;
  const mine = (visiting ?? []).filter((visit) => visit.yours);
  const stops = mine.map(
    (visit) => html`<${Stop} key=${visit.id} g=${group} v=${visit} name=${name} home=${home} />`,
  );
  const hosted =
    mine.length > 0 &&
    html`
      <h3>Who you have over</h3>
      <ul class="yours">${stops}</ul>
    `;
  const now =
    (visiting?.length ?? 0) > 0 &&
    html`
      <h3>Visiting now</h3>
      <${Visiting} vs=${visiting} rows=${rows} />
    `;
  return html`${unknown}${hosted}${now}`;
}

/** @type {{ visits: (props: PlaceProps) => Drawn }} */
export const view = { visits: Visits };
