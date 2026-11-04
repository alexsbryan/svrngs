// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page as HOME (home-reading.js) draws it into home/index.html's
// #home: while the groups are read, or why they cannot be; the ask or a
// group on a screen alone, when the address names one; else what was just
// done here, each group's section (group.js), each place after them
// (PLACES), and the page's address last.
import { html, useStore } from '../ui/view.js?v=__VERSION__';
import { Group } from './group.js?v=__VERSION__';
/** @import { Store } from '../ui/machine.js?v=__VERSION__' */
/** @import { Group as Held } from './group.js?v=__VERSION__' */
/** @import { Flows, Message, Places, Said, State } from './home-reading.js?v=__VERSION__' */

// The page's places after the groups, in the order they are drawn.
const PLACES = ['acts', 'left', 'open', 'join', 'make', 'running', 'lent'];

/** @type {Record<State['at']['said'], (q: Said, u: string) => string>} */
const SAID = {
  none: () => '',
  made: (query, invite) =>
    `You made ${query.made}. ${query.also ? `${query.also} ` : ''}${invite}Invite someone to it below.`,
  joined: (query, invite) =>
    `You’re in ${query.joined}${query.person ? `, as ${query.person}.` : ' already.'} ${invite}`,
  left: (query) =>
    `You left ${query.left}. Rejoining needs a new invitation, and this computer can’t rejoin yet.`,
};

// What was done here, said once (`SAID`), and the folder that could not be
// made when one could not.
/** @param {State} state */
const saidOnce = (state) => {
  const unplaced = state.q.unplaced ? `Its folder could not be made: ${state.q.unplaced}. ` : '';
  return SAID[state.at.said](state.q, unplaced);
};

/**
 * The page: HOME's store, each feature's module, and each place's view.
 * @typedef {object} HomeProps
 * @property {Store<State, Message>} store
 * @property {Flows} flows
 * @property {Places} places
 */

// Past its first two `return`s, the groups are read, so `state.groups` is
// read then.
/** @param {HomeProps} props */
export function Home({ store, flows, places }) {
  const state = useStore(store);
  const query = state.q;
  const home = store.dispatch;
  /** @param {string} place */
  const Place = (place) => places[place] && html`<${places[place]} s=${state} home=${home} />`;
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
        />
      `,
    );
  // "What you lend" waits until "What acts as you" is read, which draws
  // above it, so the two never trade places as they load.
  const after = PLACES.map((place) => {
    const waiting = place === 'lent' && flows.acts.holds(state);
    return !waiting && Place(place);
  });
  // The page's address is drawn last, once no feature's file `holds` it
  // while it reads: a driver reads the whole page once it is there
  // (web/tests/home.mjs `all`, web/tests/acts.mjs `loaded`).
  const held = Object.values(flows).some((flow) => flow.holds?.(state));
  return html`
    ${update}
    ${said !== '' && html`<p>${said}</p>`}
    ${state.at.groups === 'rereading' && html`<p class="said">Reading your groups…</p>`}
    ${groups.length === 0 && html`<p class="none">You’re in no group yet. Join one below from the link someone sent you, or make your own.</p>`}
    ${sections}
    ${after}
    ${!held && html`<p class="address">Your home page, the same address each time svrngs starts: ${state.origin}/</p>`}
  `;
}
