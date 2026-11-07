// SPDX-License-Identifier: AGPL-3.0-or-later
// B is the home page's layout. It changes presentation only. Hidden groups and
// home places stay mounted: their reads still supply the computer-wide access
// and lending controls.
import { html } from '../ui/view.js?v=__VERSION__';
import { whose } from '../question.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { State } from './home-reading.js?v=__VERSION__' */
/** @import { PlaceName } from './group-read.js?v=__VERSION__' */
/** @import { Group } from './group.js?v=__VERSION__' */

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

/** @param {{ s: State }} props */
function GroupLinks({ s: state }) {
  return state.groups
    ?.filter((group) => !group.left)
    .map((group) => {
      const made = group.made ? ` (${whose(group.made)})` : '';
      return html`<li><a href=${address('groups', group.id)} aria-current=${group.id === state.q.group ? 'page' : undefined}>${group.name ?? 'A group whose name cannot be read'}${made}</a><small>${group.person ? `You are ${group.person} here.` : 'This computer cannot say who you are here.'}</small></li>`;
    });
}

/** @param {{ s: State }} props */
function Notice({ s: state }) {
  const selected = state.groups?.find((group) => group.id === state.q.group && !group.left);
  const changes = selected && state.told?.[selected.id]?.lends?.changed;
  if (!selected || !changes || changes.length === 0) return null;
  return html`
    <p class="grouped-notice"><a href=${`${address('computer')}&review=${encodeURIComponent(selected.id)}`}>${changes.length} ${changes.length === 1 ? 'app has a new version' : 'apps have new versions'} you can review.</a> Your current versions keep running.</p>
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
        <ul class="group-list"><${GroupLinks} s=${state} /></ul>
        <p><a href=${address('join')}>Join a group</a> <a href=${address('make')}>Make a group</a> <a href=${address('open')}>Open a link</a> <a href=${address('left')}>Groups you left</a></p>
      `
      }
    </nav>
    <${Context} s=${state} />
    <${Notice} s=${state} />
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

/** @param {{ draw: (names: readonly PlaceName[]) => Drawn }} props */
export function GroupPlaces({ draw }) {
  return html`
    <h3>Shared apps and folder</h3>
    ${draw(['index', 'folder'])}
    <details class="group-place"><summary>People</summary>${draw(['people', 'visits', 'removals'])}</details>
    <details class="group-place"><summary>Add an app</summary>${draw(['shelf'])}</details>
    <details class="group-place"><summary>Manage this group</summary>${draw(['links', 'wifi', 'show', 'leave'])}</details>
    ${draw(['told'])}
  `;
}
