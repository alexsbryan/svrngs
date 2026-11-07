// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page as HOME (home-reading.js) draws it into home/index.html's
// #home: while the groups are read, or why they cannot be; the ask or a
// group on a screen alone, when the address names one; else what was just
// done here, each group's section (group.js), each place after them
// (PLACES), and the page's address last.
import { html, useEffect, useStore } from '../ui/view.js?v=__VERSION__';
import { Group } from './group.js?v=__VERSION__';
import { Folder } from './group-folder.js?v=__VERSION__';
import { ArrivalLinks, Choices, exposes } from './grouped.js?v=__VERSION__';
/** @import { Store } from '../ui/machine.js?v=__VERSION__' */
/** @import { Group as Held } from './group.js?v=__VERSION__' */
/** @import { Flows, Message, Places, Said, State } from './home-reading.js?v=__VERSION__' */

// The page's places after the groups, in the order they are drawn.
const PLACES = ['acts', 'left', 'open', 'join', 'make', 'running', 'lent'];

/** @type {Record<State['at']['said'], (q: Said, u: string, grouped: boolean) => string>} */
const SAID = {
  none: () => '',
  made: (query, invite, grouped) =>
    `You made ${query.made}. ${query.also ? `${query.also} ` : ''}${invite}${grouped ? 'Open its group to invite someone.' : 'Invite someone to it below.'}`,
  joined: (query, invite) =>
    `You’re in ${query.joined}${query.person ? `, as ${query.person}.` : ' already.'} ${invite}`,
  left: (query) =>
    `You left ${query.left}. Rejoining needs a new invitation, and this computer can’t rejoin yet.`,
  // The button below, as home.rs `connection` answers it.
  stayed: () => 'The public way answers, so nothing changed.',
  moved: () =>
    'The public way did not answer, so this computer moved to where your group usually meets.',
  nowhere: () => 'The public way did not answer, and neither did anywhere else your groups meet.',
};

// What was done here, said once (`SAID`), and the folder that could not be
// made when one could not.
/** @param {State} state */
export const saidOnce = (state) => {
  const unplaced = state.q.unplaced ? `Its folder could not be made: ${state.q.unplaced}. ` : '';
  return SAID[state.at.said](state.q, unplaced, state.grouped);
};

// For someone who cannot wait for this computer to find out on its own
// (KEEP_GOING_AT_HOME 7): a form, as ./reach.js is, so the answer is said
// once by `SAID`.
export const Connection = () =>
  html`<form method="post" action="/connection"><button>Check the connection now</button></form>`;

/**
 * The page: HOME's store, each feature's module, and each place's view.
 * @typedef {object} HomeProps
 * @property {Store<State, Message>} store
 * @property {Flows} flows
 * @property {Places} places
 */

/** @param {Held} group */
function welcomeFolders(group) {
  if (group.folders === null)
    return html`<p>This computer cannot say where the shared folder is. Try refreshing this page.</p>`;
  if (group.folders.length === 0)
    return html`<p>This group has no shared folder on this computer yet. Ask the person who invited you for help.</p>`;
  return group.folders.map((dir) => html`<${Folder} key=${dir} g=${group} dir=${dir} />`);
}

/** @param {{ state: State }} props */
function Arrival({ state }) {
  const target = state.q.welcome;
  const welcome = state.groups?.find(
    (group) => group.id === target && !group.left && !group.removed,
  );
  useEffect(() => {
    if (target)
      globalThis.__trace?.(
        `HOME: welcome ${target}: ${state.at.groups}, ${welcome ? 'found' : 'unavailable'}`,
      );
  }, [target, state.at.groups, welcome]);
  const back = html`<${ArrivalLinks} s=${state} g=${welcome} />`;
  if (state.at.groups === 'reading') return html`<p class="said">Reading your groups…</p>${back}`;
  if (state.at.groups === 'unread')
    return html`<p>This computer cannot read your groups: ${state.why}. Try refreshing this page.</p>${back}`;
  return html`
    <section id=${`group-${target}`}>
      <h2>${welcome ? `Welcome to ${welcome.name ?? 'your group'}` : 'This group is not available on this computer'}</h2>
      ${
        welcome
          ? html`
        <p>Start with your shared folder. What you put there reaches everyone in this group, and what they add reaches you.</p>
        ${welcomeFolders(welcome)}
        <p>Open the folder to get started. If there is a WELCOME.txt, read it for your host’s welcome. Add a small text file named after you to say hello.</p>
      `
          : html`<p>Try refreshing this page, or ask the person who invited you for help.</p>`
      }
      ${back}
    </section>
  `;
}

// The page's address, drawn last, once no feature's file `holds` it while it
// reads: a driver reads the whole page once it is there (web/tests/home.mjs
// `all`, web/tests/acts.mjs `loaded`). A names `/` though it is `/?ui=a`.
/** @param {{ s: State, held: boolean }} props */
const Foot = ({ s: state, held }) => html`
  <${Connection} />
  ${!held && html`<p class="address">Your home page, the same address each time svrngs starts: ${state.origin}/</p>`}
`;

/** @param {HomeProps} props */
export function Home({ store, flows, places }) {
  const state = useStore(store);
  const query = state.q;
  const { grouped } = state;
  const home = store.dispatch;
  /** @param {string} place */
  const Place = (place) => places[place] && html`<${places[place]} s=${state} home=${home} />`;
  if (query.welcome) return html`<${Arrival} state=${state} />`;
  if (state.at.groups === 'reading') return html`<p class="said">Reading your groups…</p>`;
  if (state.at.groups === 'unread')
    return html`<p>This computer cannot read your groups: ${state.why}</p>`;
  if (state.ask) return Place('ask');
  if (query.screen) return Place('screen');
  // The page's own svrngs, above the groups: it says nothing while it is
  // current or still asking, so the page's top stays what the person has.
  const update = Place('update');
  // What was done here drawn, or nothing, never '': Preact 11.1.1 draws ''
  // as words, and those words turned into a <p> mounted each group's
  // section again, its GROUP new and its places gone until read again
  // (crates/svrngs/tests/pages/join.rs, its group house).
  const said = saidOnce(state);
  const groups = /** @type {Held[]} */ (state.groups);
  const sections = groups
    .filter((group) => !group.left)
    .map(
      (group) => html`
        <${Group}
          key=${group.id}
          g=${group}
          joined=${query.joined}
          me=${state.key}
          round=${state.round}
          places=${places}
          flows=${flows}
          home=${home}
          grouped=${grouped}
          hidden=${grouped && query.group !== group.id}
        />
      `,
    );
  // "What you lend" waits until "What acts as you" is read, which draws
  // above it, so the two never trade places as they load.
  const after = PLACES.map((place) => {
    const waiting = place === 'lent' && flows.acts.holds(state);
    const drawn = !waiting && Place(place);
    return grouped
      ? html`<div data-home-place=${place} hidden=${!exposes(state, place)}>${drawn}</div>`
      : drawn;
  });
  const held = Object.values(flows).some((flow) => flow.holds?.(state));
  return html`
    ${update}
    ${said !== '' && html`<p>${said}</p>`}
    ${state.at.groups === 'rereading' && html`<p class="said">Reading your groups…</p>`}
    <${Choices} s=${state} />
    ${sections}
    ${after}
    ${exposes(state, 'connection') && html`<${Foot} s=${state} held=${held} />`}
  `;
}
