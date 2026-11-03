// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page, its one root (page-open (d); home.js@19b63b2b's root,
// :347-429): served at /home.js (home.rs `served`) and drawn into
// home/index.html's #home. Where "v1" is named in the home page's files, it
// is the page before it, at 19b63b2b. HOME reads the person's
// groups from /v1/groups, and the address once: the ask (/ask), a group on
// a screen (?screen=), or the page, with what was just done here said
// (?made=, ?joined=, ?left=; home.js@19b63b2b:363-367), said once. It draws in v1's order, each
// group's section (home/page/group.js), then each feature's place, from
// home/page/<file>.js; a message no file takes is refused by name
// (group.js `offered`). It
// takes one fixed message, `again`, after a write, and `visible`, on
// coming back on screen: each reads the groups again, then each group.
// The edge, below the page, reads the address and forgets a notice in it.
import { trace } from './ui/trace.js?v=__VERSION__';
import { html, render, useStore } from './ui/view.js?v=__VERSION__';
import { ask, go, late, machine, no } from './ui/machine.js?v=__VERSION__';
import { whose } from './question.js?v=__VERSION__';
import { Group, offered } from './page/group.js?v=__VERSION__';
import { get } from './page/routes.js?v=__VERSION__';
/** @import { Machine, Step, Store } from './ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from './ui/view.js?v=__VERSION__' */
/** @import { Group as Held, GroupProps, Round, Taken } from './page/group.js?v=__VERSION__' */
/** @import * as acts from './page/acts.js?v=__VERSION__' */
/** @import * as lending from './page/lending.js?v=__VERSION__' */
/** @import * as join from './page/join.js?v=__VERSION__' */
/** @import * as make from './page/make.js?v=__VERSION__' */
/** @import * as leave from './page/leave.js?v=__VERSION__' */

/**
 * What the address says, by name (`?made=`, `?screen=`, …), and what a
 * feature's `take` writes into it: `null` for a folder that was placed.
 * @typedef {Partial<Record<string, string | null>>} Said
 */

/**
 * HOME's state: its regions, the address read once (`first`), the groups
 * and this computer's key as last read, the round of reading they began,
 * or why they could not be read; and the fields each feature's `take`
 * sets.
 * @typedef {object} Own
 * @property {{ groups: 'reading' | 'read' | 'rereading' | 'unread', said: 'none' | 'made' | 'joined' | 'left' }} at
 * @property {Record<string, number>} [asked]
 * @property {boolean} ask whether the address is /ask
 * @property {Said} q
 * @property {string} origin
 * @property {Held[]} [groups]
 * @property {string} [key]
 * @property {Round} [round]
 * @property {string} [why]
 * @typedef {Own & acts.Fields & lending.Fields} State
 */

/**
 * HOME's messages: a round of reading begun here (`again`, after a write;
 * `visible`), the groups read or unread, and each message a feature's
 * `take` takes, which names its file (`flow`).
 * @typedef {{ type: 'again' | 'visible' }
 *   | { type: 'read', n: number, groups: Held[], key: string, round: Round['type'], why: string }
 *   | { type: 'unread', n: number, why: string }
 *   | acts.Taken
 *   | lending.Taken
 *   | join.Taken
 *   | make.Taken
 *   | leave.Taken} Message
 */

/**
 * The effect HOME's step answers: the groups read, read `n`, in a round
 * of `round`.
 * @typedef {{ run: 'groups', n: number, round: Round['type'], failed: 'unread' }} Effect
 */

/**
 * A feature's module, as HOME reads it: the step that takes its messages,
 * whether it holds the page's foot while it reads, and its places' views.
 * `take` is a method, so a `take` of its own file's `Taken` stands for any.
 * @typedef {{
 *   take?(s: State, m: Taken): Step<State, Effect>,
 *   holds?(s: State): boolean,
 *   view?: Record<string, (props: any) => Drawn>,
 * }} Flow
 */

/**
 * The features by their file's name (FILES); "What acts as you" holds
 * "What you lend" while it reads.
 * @typedef {Record<(typeof FILES)[number], Flow> & { acts: { holds(s: State): boolean } }} Flows
 */

/**
 * A place of HOME's, after the groups (PLACES), drawn with its state and
 * its `dispatch`.
 * @typedef {{ s: State, home: Store<State, Message>['dispatch'] }} PlaceProps
 */

/**
 * Each place's view, HOME's and each group section's, by its name.
 * @typedef {Partial<Record<string, (props: PlaceProps) => Drawn>> & GroupProps['places']} Places
 */

// Each feature's file, and the page's places after the groups, in
// home.js@19b63b2b:378-427's order.
const FILES = /** @type {const} */ ([
  'shelf',
  'index',
  'running',
  'lending',
  'door',
  'visits',
  'leave',
  'screen',
  'join',
  'join-form',
  'make',
  'make-form',
  'links',
  'open',
  'acts',
  'ask',
  'wifi',
  'wifi-foot',
  'play',
]);
const PLACES = ['acts', 'left', 'open', 'join', 'make', 'running', 'lent'];
// After a write here (`again`), the groups drawn stay, said as read again
// until they are, where v1 loaded the page again (home_group.rs:219, :335
// on v2: "You made Trip." over the old groups looked done).
/**
 * @param {State} state
 * @param {{ type: 'again' | 'visible' }} message
 * @returns {Step<State, Effect>}
 */
const reading = (state, message, [asked, serial] = ask(state, 'groups')) =>
  go(
    state,
    message.type === 'again' && state.at.groups === 'read' ? { groups: 'rereading' } : {},
    { asked },
    { run: 'groups', n: serial, round: message.type, failed: 'unread' },
  );
// A feature's message is offered to the feature its `flow` names, as is a
// message no type declares, which `offered` refuses: each feature's `Taken`
// is that file's. A message type HOME declares that no case names fails
// `typed lint` (switch-exhaustiveness-check), and one that names no flow
// fails `types` at `offered`.
/**
 * @param {Flows} flows
 * @returns {Machine<State, Message, Effect>}
 */
const HOME = (flows) => ({
  states: {
    groups: ['reading', 'read', 'rereading', 'unread'],
    said: ['none', 'made', 'joined', 'left'],
  },
  step: (state, message) => {
    switch (message.type) {
      case 'again':
      case 'visible':
        return reading(state, message);
      case 'read':
        return late(state, 'groups', message.n)
          ? no(state, late(state, 'groups', message.n))
          : go(
              state,
              { groups: 'read' },
              { groups: message.groups, key: message.key, round: { type: message.round } },
            );
      case 'unread':
        return late(state, 'groups', message.n)
          ? no(state, late(state, 'groups', message.n))
          : go(state, { groups: 'unread' }, { why: message.why });
      case 'made':
      case 'joined':
      case 'left':
      case 'told':
      default:
        return offered(flows, state, message);
    }
  },
  io: {
    groups: ({ n: serial, round }) =>
      get('/v1/groups').then(
        /** @param {{ groups: Held[], key: string }} answer */
        ({ groups, key }) => ({
          type: 'read',
          n: serial,
          groups,
          key,
          round,
          why: `${groups.length} group(s)`,
        }),
        (error) => ({ type: 'unread', n: serial, why: error.message }),
      ),
  },
});
// The address, read once (home.js@19b63b2b:358-359), and what it says was done here.
/**
 * @param {Location} location
 * @returns {State}
 */
const first = (
  { pathname, search, origin },
  query = Object.fromEntries(new URLSearchParams(search)),
) => ({
  at: {
    groups: 'reading',
    said: /** @type {const} */ (['made', 'joined', 'left']).find((said) => said in query) ?? 'none',
  },
  asked: { groups: 1 },
  ask: pathname === '/ask',
  q: query,
  origin,
});
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
// The groups you left, with what stays of each here (home.js@19b63b2b:385-388).
// Drawn once the groups are read (`Home`), so `state.groups` is read then.
/** @param {PlaceProps} props */
const left = ({ s: state }) =>
  ((gone) =>
    gone.length > 0 &&
    html`<section id="left"><h2>Groups you left</h2>${gone.map((group) => html`<p>${group.name ?? 'A group'}${group.made ? ` (${whose(group.made)})` : ''}: what you have of it stays on this computer${group.folders && group.folders.length > 0 ? `, its folder at ${group.folders.join(' and ')}` : ''}, and nothing new of it reaches you.</p>`)}</section>`)(
    /** @type {Held[]} */ (state.groups).filter((group) => group.left),
  );

// The page's foot, last, once no feature's file `holds` it while it reads, as
// v1 drew it after every read (home.js@19b63b2b:326-328); and "What you lend" held
// until "What acts as you" is read, as v1 drew it after it and its WiFi (home.js@19b63b2b:377-424),
// which a driver's `text` waits on (home.mjs:166; main.rs:3536 on v2).
/**
 * The page: HOME's store, each feature's module, and each place's view.
 * @typedef {object} HomeProps
 * @property {Store<State, Message>} store
 * @property {Flows} flows
 * @property {Places} places
 */

// In the page's place while the groups are read, or once they cannot be.
/** @param {State} state */
const unread = (state) =>
  html`<p class=${state.at.groups === 'reading' ? 'said' : undefined}>${state.at.groups === 'reading' ? 'Reading your groups…' : `This computer cannot read your groups: ${state.why}`}</p>`;
// What was done here, said once (`SAID`), and the folder that could not be
// made when one could not.
/** @param {State} state */
const saidOnce = (state) =>
  SAID[state.at.said](
    state.q,
    state.q.unplaced ? `Its folder could not be made: ${state.q.unplaced}. ` : '',
  );

// Past its first `return`, the groups are read, so `state.groups` is read then.
/** @param {HomeProps} props */
export function Home({ store, flows, places }) {
  const state = useStore(store);
  const query = state.q;
  const home = store.dispatch;
  const held = () => Object.values(flows).some((flow) => flow.holds?.(state));
  const Place = /** @param {string} place */ (place) =>
    places[place] && html`<${places[place]} s=${state} home=${home} />`;
  if (state.at.groups === 'reading' || state.at.groups === 'unread') return unread(state);
  if (state.ask || query.screen) return Place(state.ask ? 'ask' : 'screen');
  // What was done here drawn, or nothing, never '': Preact 11.1.1 draws ''
  // as words, and those words turned into a <p> mounted each group's
  // section again, its GROUP new and its places gone until read again
  // (pages/join.rs, house; a page of these holes alone did the same).
  const said = saidOnce(state);
  return html`${said !== '' && html`<p>${said}</p>`}${state.at.groups === 'rereading' && html`<p class="said">Reading your groups…</p>`}${!(/** @type {Held[]} */ (state.groups).length) && html`<p class="none">You’re in no group yet. Join one below from the link someone sent you, or make your own.</p>`}${/** @type {Held[]} */ (state.groups).filter((group) => !group.left).map((group) => html`<${Group} key=${group.id} g=${group} joined=${query.joined} me=${state.key} round=${state.round} places=${places} flows=${flows} home=${home} />`)}${PLACES.map((place) => (place !== 'lent' || !flows.acts.holds(state)) && Place(place))}${!held() && html`<p class="address">Your home page, the same address each time svrngs starts: ${state.origin}/</p>`}`;
}

// The edge: each feature's file, by its one URL, then HOME, its first read
// run as it starts, drawn; and the page shown again, read again. Never
// awaited at the module's top (home.js@19b63b2b:347-353, WebKit's module loader).
Promise.all(FILES.map((file) => import(`./page/${file}.js?v=__VERSION__`)))
  .then((mods) => {
    // Each module imported by its name in FILES, so each name is there.
    const flows = /** @type {Flows} */ (
      Object.fromEntries(FILES.map((file, i) => [file, mods[i]]))
    );
    const places = Object.assign({ left }, ...mods.map((flow) => flow.view));
    const store = machine(
      'HOME',
      HOME(flows),
      first(location),
      /** @type {Effect[]} */ ([{ run: 'groups', n: 1, round: 'read', failed: 'unread' }]),
    );
    // What was done here is said once (page-open (e); CHARTER 9): the address
    // forgets it, so a reload, or the page opened again, says it no more.
    if (store.get().at.said !== 'none')
      history.replaceState(
        null,
        '',
        (trace(`HOME: ${store.get().at.said} said, once`), location.pathname),
      );
    // home/index.html holds #home, as it does where its files fail below.
    render(
      html`<${Home} store=${store} flows=${flows} places=${places} />`,
      /** @type {HTMLElement} */ (document.getElementById('home')),
    );
    document.addEventListener(
      'visibilitychange',
      () => document.visibilityState === 'visible' && store.dispatch({ type: 'visible' }),
    );
  })
  .catch((error) => {
    trace(`HOME: its files did not load: ${error}`);
    return render(
      html`<p>This computer cannot read your groups: ${error.message}</p>`,
      /** @type {HTMLElement} */ (document.getElementById('home')),
    );
  });
