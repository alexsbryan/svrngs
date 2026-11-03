// SPDX-License-Identifier: AGPL-3.0-or-later
// A group on a screen, for a TV, a spare laptop or a projector: opened in
// a tab of its own at `/?screen=<its id>` (screen-link.js), where HOME
// draws this alone (home-page.js `Home`). Its door alone, big, each person's
// lends in their window (ui/door.js), read by SCREEN (on-screen.js), with
// what arrived since it opened beside it (since-opened.js).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { counted, Door } from '../ui/door.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';
import { colour } from './group.js?v=__VERSION__';
import { SCREEN } from './on-screen.js?v=__VERSION__';
import { SinceOpened } from './since-opened.js?v=__VERSION__';
/** @import { Group, Lends } from './group.js?v=__VERSION__' */
/** @import { Effect, State } from './on-screen.js?v=__VERSION__' */
/** @import { PlaceProps } from './home-reading.js?v=__VERSION__' */

/**
 * @param {number} count
 * @param {string} one
 * @param {string} more
 */
const many = (count, one, more) => `${count} ${count === 1 ? one : more}`;

// What the screen says of `group`'s count: why this computer cannot read
// it, else, once read, who is in it, who is on now and what is lent.
/**
 * @param {State} state
 * @param {Group} group
 */
const countLine = (state, group) => {
  if (state.at.screen === 'unread')
    return `This computer cannot read ${group.name ?? 'this group'} now: ${state.why}`;
  if (!state.rows) return '';
  // `state.lends` is read with `state.rows`, by one read ('read' sets
  // both), so it is there whenever they are.
  const lends = /** @type {Lends} */ (state.lends);
  const [people, on, things] = counted(state.rows, lends.lent, lends);
  return `${many(people, 'person', 'people')} · ${on} on now · ${many(things, 'thing', 'things')} lent`;
};

/** @param {PlaceProps} props */
export function Screen({ s: { groups, q: query } }) {
  // HOME draws the screen once the groups are read (home-page.js `Home`).
  const group = /** @type {Group[]} */ (groups).find((one) => one.id === query.screen);
  const here = group && !group.left && group.removed === false;
  const [state] = useMachine(() => {
    /** @type {Effect[]} */
    const first = [{ run: 'look', title: `${group?.name ?? 'A group'} on a screen` }];
    if (here) first.push({ run: 'read', n: 1, id: group.id, failed: 'unread' });
    return machine(
      'SCREEN',
      SCREEN,
      { at: { screen: 'reading' }, asked: { screen: 1 }, id: group?.id, seen: [], since: [] },
      first,
    );
  });
  if (!here)
    return html`<p>This computer cannot show that group on a screen: you are not in it here.</p>`;
  const name = group.name ?? 'A group whose name this computer cannot read';
  const made = group.made && html`<small> (${whose(group.made)})</small>`;
  const lends = /** @type {Lends} */ (state.lends);
  const door =
    state.rows &&
    html`<${Door} rows=${state.rows} lent=${lends.lent} lends=${lends} newest=${state.newest} />`;
  return html`
    <section data-colour=${colour(group.id)}>
      <h2>${name}${made}</h2>
      <p>${countLine(state, group)}</p>
      <div>${door}</div>
      <${SinceOpened} since=${state.since} />
      <footer>
        <p>Lit: answered just now. Dashed: did not answer. Dotted: in from a phone.</p>
        <p>Each thing lent opens in a tab as ${group.person}, from this computer.</p>
      </footer>
    </section>
  `;
}
