// SPDX-License-Identifier: AGPL-3.0-or-later
// B is the home page's layout. It changes presentation only. Hidden groups and
// home places stay mounted: their reads still supply the computer-wide access
// and lending controls.
import { html } from '../ui/view.js?v=__VERSION__';
import { once, whose } from '../question.js?v=__VERSION__';
import { counted } from '../ui/door.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { State } from './home-reading.js?v=__VERSION__' */
/** @import { PlaceName } from './group-read.js?v=__VERSION__' */
/** @import { Group } from './group.js?v=__VERSION__' */
/** @import { Told } from './acts.js?v=__VERSION__' */

/** @type {Record<string, readonly string[]>} */
const VIEWS = {
  groups: [],
  computer: ['acts', 'running', 'lent', 'connection'],
  open: ['open'],
  join: ['join'],
  make: ['make'],
  left: ['left'],
};

/** @param {string | null} [view] @param {string | null} [group] */
export function address(view = 'groups', group = '') {
  if (group) return `/?${new URLSearchParams({ group })}`;
  return view && view !== 'groups' ? `/?${new URLSearchParams({ view })}` : '/';
}

/** @param {State} state @param {string} place */
export function exposes(state, place) {
  const view = state.q.view ?? 'groups';
  if (!state.grouped) return true;
  return !state.q.group && Object.hasOwn(VIEWS, view) && VIEWS[view]?.includes(place) === true;
}

// A group's card says what a person opens its page to learn, from what its
// hidden section last told HOME (group-told.js): no read is added. Nothing is
// said until it has told; when either read failed, that is said, never the
// older answer as now's; and versions to review only when that was read.
/** @param {{ told?: Told }} props */
function Live({ told }) {
  if (!told) return null;
  const { people, lends } = told;
  if (!told.fresh || !people || !lends)
    return html`<small>This computer cannot read who is on now.</small>`;
  const [all, on] = counted(people, [], lends);
  const changed = lends.changed?.length ?? 0;
  return html`<small>On now: ${on} of ${all}. Shared apps: ${once(lends.lent).length}.${changed > 0 && ` New versions to review: ${changed}.`}</small>`;
}

// The overview's cards. Who the person is in a group is said once above
// the cards when it is the same in every one, as it mostly is, and on a
// card only when it differs or cannot be read there.
/** @param {{ s: State }} props */
function GroupLinks({ s: state }) {
  const groups = (state.groups ?? []).filter((group) => !group.left);
  const who = groups[0]?.person ?? null;
  const same = who !== null && groups.every((group) => group.person === who);
  const cards = groups.map((group) => {
    const made = group.made ? ` (${whose(group.made)})` : '';
    const here = group.person
      ? `You are ${group.person} here.`
      : 'This computer cannot say who you are here.';
    return html`<li><a href=${address('groups', group.id)} aria-current=${group.id === state.q.group ? 'page' : undefined}>${group.name ?? 'A group whose name cannot be read'}${made}</a>${!same && html`<small>${here}</small>`}<${Live} told=${state.told?.[group.id]} /></li>`;
  });
  return html`${same && html`<p>You are ${who} here.</p>`}<ul class="group-list">${cards}</ul>`;
}

// What needs the person on a group's page, a new version to review, is
// said directly under its banner (home-page.js gives it to group.js
// `Group`), where they look first, never above the banner among the rules.
/** @param {{ s: State, g: Group }} props */
export function Notice({ s: state, g: group }) {
  const changes = state.told?.[group.id]?.lends?.changed;
  if (!changes || changes.length === 0) return null;
  return html`
    <p class="grouped-notice"><a href=${`${address('computer')}&review=${encodeURIComponent(group.id)}`}>${changes.length} ${changes.length === 1 ? 'app has a new version' : 'apps have new versions'} you can review.</a> Your current versions keep running.</p>
  `;
}

/** @param {{ s: State }} props */
function Context({ s: state }) {
  const groups = /** @type {Group[]} */ (state.groups);
  if (state.q.group) {
    const found = groups.some((group) => group.id === state.q.group && !group.left);
    return (
      !found &&
      html`<p class="bad">This group is not available here. Open All your groups to choose another, or to see Groups you left.</p>`
    );
  }
  const view = state.q.view ?? 'groups';
  if (!Object.hasOwn(VIEWS, view))
    return html`<p class="bad">This page is not available. Choose a destination above.</p>`;
  if (view === 'computer')
    return html`<h2>This computer</h2><p>What you share, and what can act as you, across your groups.</p>`;
  if (view === 'left' && !groups.some((group) => group.left))
    return html`<p>You have no groups you left on this computer.</p>`;
  return null;
}

/** @param {{ s: State }} props */
export function Navigation({ s: state }) {
  const view = state.q.view ?? 'groups';
  const overview = !state.q.group && view === 'groups';
  return html`
    <nav class="grouped-nav" aria-label="Your groups" data-view=${state.q.group ? 'group' : view}>
      <p><a href=${address()} aria-current=${overview ? 'page' : undefined}>All your groups</a> <a href=${address('computer')} aria-current=${!state.q.group && view === 'computer' ? 'page' : undefined}>This computer</a></p>
      ${
        overview &&
        html`
        <${GroupLinks} s=${state} />
        <p><a href=${address('join')}>Join a group</a> <a href=${address('make')}>Make a group</a> <a href=${address('open')}>Open a link</a> <a href=${address('left')}>Groups you left</a></p>
      `
      }
    </nav>
    <${Context} s=${state} />
  `;
}

/** @param {{ s: State }} props */
export function Choices({ s: state }) {
  const where = state.grouped ? 'above' : 'below';
  return html`
    ${state.grouped && html`<${Navigation} s=${state} />`}
    ${state.groups?.length === 0 && html`<p class="none">You’re in no group yet. Join one ${where} from the link someone sent you, or make your own.</p>`}
  `;
}

/** @param {{ s: State, g?: Group }} props */
export function ArrivalLinks({ s: state, g: group }) {
  return html`<p>${state.grouped && group && html`<a href=${address('groups', group.id)}>Continue to ${group.name ?? 'your group'}</a> `}<a href="/">All your groups</a></p>`;
}

// A group's page, what is before what is done: who is here, the door, a
// few rows however many; then what they share, which is what a person
// opens the page for, and its files. After those, what a person does to
// the group now and then: who they have over, with who else is visiting,
// and the invitation buttons, never folded with leaving. Last the folds:
// each app's address and WiFi line has one of its own, being about an app,
// not about managing the group.
/** @param {{ draw: (names: readonly PlaceName[]) => Drawn }} props */
export function GroupPlaces({ draw }) {
  return html`
    <details class="group-place" open><summary>People</summary>${draw(['people', 'removals'])}</details>
    <h3>Shared apps</h3>
    ${draw(['index'])}
    <h3>Shared files</h3>
    ${draw(['folder'])}
    ${draw(['visits', 'links'])}
    <details class="group-place"><summary>Add an app</summary>${draw(['shelf'])}</details>
    <details class="group-place"><summary>Addresses and WiFi</summary>${draw(['wifi'])}</details>
    <details class="group-place"><summary>Manage this group</summary>${draw(['show', 'leave'])}</details>
    ${draw(['told'])}
  `;
}
