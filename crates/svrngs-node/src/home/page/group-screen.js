// SPDX-License-Identifier: AGPL-3.0-or-later
// A group on a screen, for a TV, a spare laptop or a projector: opened in
// a tab of its own at `/?screen=<its id>` (`ScreenLink`, in each group's
// section as `show`, once its lends are read), where HOME draws this alone,
// its place `screen` (home-page.js `Home`). Its door alone, big, each person's
// lends in their window (ui/door.js), read by SCREEN (on-screen.js), with
// what arrived since it opened beside it (`SinceOpened`).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { counted, Door } from '../ui/door.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';
import { colour } from './group.js?v=__VERSION__';
import { SCREEN } from './on-screen.js?v=__VERSION__';
/** @import { Group, Lends } from './group.js?v=__VERSION__' */
/** @import { Effect, Since, State } from './on-screen.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as GROUP from './group.js?v=__VERSION__' */
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

// Beside a group on a screen: who joined and what was lent since the
// screen opened, newest first, the newest marked new (on-screen.js
// SCREEN's `since`).
/**
 * What arrived since the screen opened, newest first.
 * @typedef {{ since: Since[] }} SinceOpenedProps
 */

/** @param {SinceOpenedProps} props */
function SinceOpened({ since }) {
  const lines = since.map(
    ([time, words], i) =>
      html`<li><time>${time}</time> ${words} ${i === 0 && html`<mark>new</mark>`}</li>`,
  );
  return html`
    <aside>
      <h2>Since this screen opened</h2>
      <p>Kept on this screen only: it starts over when the screen is opened again.</p>
      <ol>${lines}</ol>
      <p>Nothing new yet. Who joins, and what they lend, shows here as it happens.</p>
    </aside>
  `;
}

/** @param {PlaceProps} props */
function Screen({ s: { groups, q: query } }) {
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

/** @param {GROUP.PlaceProps} props */
function ScreenLink({ g: group, s: state }) {
  if (!state.lends) return null;
  const href = `/?screen=${encodeURIComponent(group.id)}`;
  return html`<p><a href=${href} target="_blank">Show ${group.name ?? 'this group'} on a screen</a></p>`;
}

/** @type {{ screen: (props: PlaceProps) => Drawn, show: (props: GROUP.PlaceProps) => Drawn }} */
export const view = { screen: Screen, show: ScreenLink };
