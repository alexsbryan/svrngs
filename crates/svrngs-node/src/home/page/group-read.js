// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's section once it is drawn as read (group.js `below`): GROUP's
// reads (group-reading.js), and what each read draws, each feature's in its
// place (PLACES), from home/page/<file>.js. A file that draws nothing
// draws nothing here.
import { html, useEffect, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { GROUP } from './group-reading.js?v=__VERSION__';
import { GroupPlaces } from './grouped.js?v=__VERSION__';
/** @import { GroupProps } from './group.js?v=__VERSION__' */

// Each place of a group's section, in the order it draws them: the index,
// its people, its folder, the links made here, leaving, removals, who is
// visiting, the shelf, what is open to this WiFi, and its link to a screen
// (`show`); then what it read, told to HOME for "What acts as you"
// (group-told.js `GroupTold`), drawing nothing.
export const PLACES = /** @type {const} */ ([
  'index',
  'people',
  'folder',
  'links',
  'leave',
  'removals',
  'visits',
  'shelf',
  'wifi',
  'show',
  'told',
]);

/** @typedef {(typeof PLACES)[number]} PlaceName */

// Until its people are first read, which each place drawn from them waits
// on, the section says it is being read, where its heading stood alone
// once a group was made or joined (the operator, 2026-11-03: "things
// should show as loading until they're loaded"; demo/runs/webkit-wall/
// 2026-11-03/home-group-gate-2.txt). Read again, it keeps what it drew.
/** @param {GroupProps} props */
export function GroupRead({ g: group, round, places, flows, home, me, joined, grouped }) {
  const [state, send] = useMachine(() =>
    machine('GROUP', GROUP(flows), { at: { people: 'reading', lends: 'reading' }, id: group.id }),
  );
  useEffect(() => send({ type: round.type }), [round]);
  const name = group.name ?? 'this group';
  const first = state.at.people === 'reading' && !state.people;
  const unread = state.at.people === 'unread' || state.at.lends === 'unread';
  /** @param {readonly PlaceName[]} names */
  const drawn = (names) =>
    names.map((place) => {
      const Place = places[place];
      if (!Place) return null;
      return html`
      <${Place} g=${group} s=${state} send=${send} home=${home} me=${me} joined=${joined} />
    `;
    });
  return html`
    ${first && html`<p class="said">Reading ${name}…</p>`}
    ${grouped ? html`<${GroupPlaces} draw=${drawn} />` : drawn(PLACES)}
    ${unread && html`<p>This computer cannot read what ${name} lends: ${state.why}</p>`}
  `;
}
