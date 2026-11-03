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
import {
  ask,
  go,
  html,
  late,
  machine,
  no,
  render,
  useStore,
  whose,
} from './ui/kit.js?v=__VERSION__';
import { Group, get, offered } from './page/group.js?v=__VERSION__';
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
    'make',
    'links',
    'open',
    'acts',
    'ask',
    'wifi',
    'play',
  ]),
  PLACES = ['acts', 'left', 'open', 'join', 'make', 'running', 'lent'];
// After a write here (`again`), the groups drawn stay, said as read again
// until they are, where v1 loaded the page again (home_group.rs:219, :335
// on v2: "You made Trip." over the old groups looked done).
/**
 * @param {State} s
 * @param {{ type: 'again' | 'visible' }} m
 * @returns {Step<State, Effect>}
 */
const reading = (s, m, [asked, n] = ask(s, 'groups')) =>
  go(
    s,
    m.type === 'again' && s.at.groups === 'read' ? { groups: 'rereading' } : {},
    { asked },
    { run: 'groups', n, round: m.type, failed: 'unread' },
  );
// A message no case takes is offered to the feature its `flow` names: each
// feature's `Taken` is that file's, and a message type HOME declares that
// no case takes and names no flow fails `types` at `offered`.
/**
 * @param {Flows} flows
 * @returns {Machine<State, Message, Effect>}
 */
const HOME = (flows) => ({
  states: {
    groups: ['reading', 'read', 'rereading', 'unread'],
    said: ['none', 'made', 'joined', 'left'],
  },
  step: (s, m) => {
    switch (m.type) {
      case 'again':
      case 'visible':
        return reading(s, m);
      case 'read':
        return late(s, 'groups', m.n)
          ? no(s, late(s, 'groups', m.n))
          : go(s, { groups: 'read' }, { groups: m.groups, key: m.key, round: { type: m.round } });
      case 'unread':
        return late(s, 'groups', m.n)
          ? no(s, late(s, 'groups', m.n))
          : go(s, { groups: 'unread' }, { why: m.why });
      default:
        return offered(flows, s, m);
    }
  },
  io: {
    groups: ({ n, round }) =>
      get('/v1/groups').then(
        /** @param {{ groups: Held[], key: string }} answer */
        ({ groups, key }) => ({
          type: 'read',
          n,
          groups,
          key,
          round,
          why: `${groups.length} group(s)`,
        }),
        (e) => ({ type: 'unread', n, why: e.message }),
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
  q = Object.fromEntries(new URLSearchParams(search)),
) => ({
  at: {
    groups: 'reading',
    said: /** @type {const} */ (['made', 'joined', 'left']).find((k) => k in q) ?? 'none',
  },
  asked: { groups: 1 },
  ask: pathname === '/ask',
  q,
  origin,
});
/** @type {Record<State['at']['said'], (q: Said, u: string) => string>} */
const SAID = {
  none: () => '',
  made: (q, u) =>
    `You made ${q.made}. ${q.also ? `${q.also} ` : ''}${u}Invite someone to it below.`,
  joined: (q, u) => `You’re in ${q.joined}${q.person ? `, as ${q.person}.` : ' already.'} ${u}`,
  left: (q) =>
    `You left ${q.left}. Rejoining needs a new invitation, and this computer can’t rejoin yet.`,
};
// The groups you left, with what stays of each here (home.js@19b63b2b:385-388).
// Drawn once the groups are read (`Home`), so `s.groups` is read then.
/** @param {PlaceProps} props */
const left = ({ s }) =>
  ((gone) =>
    gone.length > 0 &&
    html`<section id="left"><h2>Groups you left</h2>${gone.map((g) => html`<p>${g.name ?? 'A group'}${g.made ? ` (${whose(g.made)})` : ''}: what you have of it stays on this computer${g.folders?.length ? `, its folder at ${g.folders.join(' and ')}` : ''}, and nothing new of it reaches you.</p>`)}</section>`)(
    /** @type {Held[]} */ (s.groups).filter((g) => g.left),
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

// Past its first `return`, the groups are read, so `s.groups` is read then.
/** @param {HomeProps} props */
export function Home({ store, flows, places }) {
  const s = useStore(store),
    q = s.q,
    home = store.dispatch,
    held = () => Object.values(flows).some((f) => f.holds?.(s)),
    P = /** @param {string} k */ (k) => places[k] && html`<${places[k]} s=${s} home=${home} />`;
  if (s.at.groups === 'reading' || s.at.groups === 'unread')
    return html`<p class=${s.at.groups === 'reading' ? 'said' : undefined}>${s.at.groups === 'reading' ? 'Reading your groups…' : `This computer cannot read your groups: ${s.why}`}</p>`;
  if (s.ask || q.screen) return P(s.ask ? 'ask' : 'screen');
  const said = SAID[s.at.said](
    q,
    q.unplaced ? `Its folder could not be made: ${q.unplaced}. ` : '',
  );
  return html`${said && html`<p>${said}</p>`}${s.at.groups === 'rereading' && html`<p class="said">Reading your groups…</p>`}${!(/** @type {Held[]} */ (s.groups).length) && html`<p class="none">You’re in no group yet. Join one below from the link someone sent you, or make your own.</p>`}${/** @type {Held[]} */ (s.groups).filter((g) => !g.left).map((g) => html`<${Group} key=${g.id} g=${g} joined=${q.joined} me=${s.key} round=${s.round} places=${places} flows=${flows} home=${home} />`)}${PLACES.map((k) => (k !== 'lent' || !flows.acts.holds(s)) && P(k))}${!held() && html`<p class="address">Your home page, the same address each time svrngs starts: ${s.origin}/</p>`}`;
}

// The edge: each feature's file, by its one URL, then HOME, its first read
// run as it starts, drawn; and the page shown again, read again. Never
// awaited at the module's top (home.js@19b63b2b:347-353, WebKit's module loader).
Promise.all(FILES.map((f) => import(`./page/${f}.js?v=__VERSION__`)))
  .then((mods) => {
    // Each module imported by its name in FILES, so each name is there.
    const flows = /** @type {Flows} */ (Object.fromEntries(FILES.map((f, i) => [f, mods[i]]))),
      places = Object.assign({ left }, ...mods.map((f) => f.view));
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
  .catch(
    (e) => (
      trace(`HOME: its files did not load: ${e}`),
      render(
        html`<p>This computer cannot read your groups: ${e.message}</p>`,
        /** @type {HTMLElement} */ (document.getElementById('home')),
      )
    ),
  );
