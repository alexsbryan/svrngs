// SPDX-License-Identifier: AGPL-3.0-or-later
// The flow that opens the page and reads the group, the spine every phone
// flow reads, as two regions. `tab` is the group's one lock, held by one
// tab of this browser (#176); `list` its read, of the first computer to
// answer, then the next (#205). What the address asks is decided here;
// what it opens elsewhere is sent to its flow (`visit`, `join`, `among`,
// `cold`, `opens`, `offer`), and what `#open=` names, once the group is
// read (`wanted`). The key and the endpoint are the edge's (dial.js), never
// in state. It draws nothing: #door draws its tab (group-door.js).
import { ask, go, late, no } from '../ui/machine.js?v=__VERSION__';
import { alias, apart, called, once, titled, whoseComputer } from '../question.js?v=__VERSION__';
import { named as saidOf } from '../question.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { actor, hold, lends, trace, wasm } from './dial.js?v=__VERSION__';
import { sending } from './page.js?v=__VERSION__';
/** @import { Group, Computer } from '../store.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Lend, Removal, PeopleRow, Shelved } from '../question.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Io, Step, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Said, Send, State } from './page.js?v=__VERSION__' */

/**
 * A lend as this page lists it: as the node answered, with its label
 * (svrngs-core `label`, `read`'s io).
 * @typedef {Lend & { label: string }} Lent
 */

/**
 * Someone whose computer did not say what they lend (node.rs `Named`).
 * @typedef {{ person: string, person_key: string }} Whom
 */

/**
 * What the address asks of the page once the group is read, by the
 * fragment's names (`READ`).
 * @typedef {object} Wants
 * @property {string} [open] a lend's label or alias, opened once the group is read
 * @property {string} [at] where in the lend it opens
 * @property {string} [anew] present when the lend's origin holds no key its pass names
 * @property {string} [ask] the lend an app asks to use (ask-when-read.js)
 * @property {string} [app] the app that asks
 * @property {string} [cold] present when sent from a lend's address with nothing serving it
 */

/**
 * The fields this flow owns (`owns`): the group this tab reaches and, once
 * read, what its computers answered of it.
 * @typedef {object} Fields
 * @property {Group} group the group this tab reaches, once one is held
 * @property {Wants | null} wants
 * @property {boolean} joined
 * @property {Computer} [via] the computer the last read went through, none for the link's
 * @property {string} stale why the last read again failed, or ''
 * @property {string[]} [unready] what keeps this phone from going on at home (store.js `unready`)
 * @property {Lent[]} [lent] what the group lends, once read
 * @property {PeopleRow[]} people
 * @property {Shelved[]} shelf
 * @property {Whom[]} unanswered
 * @property {Whom[]} unheard
 * @property {Whom[]} refused
 * @property {Whom[]} behind
 * @property {Removal[] | null} [removals] `null` when they cannot be read
 * @property {Visitor[] | null} [visiting] `null` when they cannot be read
 * @property {Change[] | null} [changed]
 * @property {string} me this page's key, as the record spells it
 */

/**
 * What the computer answered `/people/lends` (surface.rs).
 * @typedef {object} Body
 * @property {Lend[]} [lent]
 * @property {Whom[]} unanswered
 * @property {Whom[]} unheard
 * @property {Whom[]} refused
 * @property {Whom[]} older
 * @property {Removal[] | null} [removals]
 * @property {Visitor[] | null} [visiting]
 * @property {PeopleRow[]} people
 * @property {Shelved[]} shelf
 * @property {Change[] | null} [changed]
 * @property {unknown} [meets] the relays its keeper says the group meets at, unchecked; none from an older computer or an unread record
 * @property {number} [version]
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'start', fragment: string }
 *   | {
 *       type: 'looked',
 *       fragment: string,
 *       kept: Group[],
 *       missing: boolean,
 *       cold: boolean,
 *       linked: boolean,
 *     }
 *   | { type: 'hold', group: Group, wants?: Wants | null, joined?: boolean }
 *   | { type: 'held' }
 *   | { type: 'elsewhere' }
 *   | { type: 'take' }
 *   | { type: 'released' }
 *   | { type: 'opened', group: Group }
 *   | { type: 'shown' }
 *   | { type: 'reread' }
 *   | {
 *       type: 'read',
 *       n: number,
 *       ok: boolean,
 *       refusal: string | null,
 *       via?: Computer,
 *       link?: Computer,
 *       body: Partial<Body>,
 *       lent: Lent[],
 *       meets?: string[],
 *       me: string,
 *       older: boolean,
 *     }
 *   | { type: 'unread', n?: number, said?: string, why: string }
 *   | { type: 'kept', unready?: string[] }} Message
 */

/**
 * The effects its steps answer: the address looked at; the group's lock
 * asked for, or taken from another tab, and let go; the group opened, its
 * key unsealed; read, the `n`th time; and kept as read.
 * @typedef {{ run: 'look', fragment: string, failed: string }
 *   | { run: 'lock', id: string, steal?: boolean, failed: string }
 *   | { run: 'release', failed: string }
 *   | { run: 'open', group: Group, failed: string }
 *   | { run: 'read', group: Group, n: number, failed: string }
 *   | { run: 'keep', group: Group, lent: Lent[], failed: string }} Effect
 */

export const owns = [
  'group',
  'wants',
  'joined',
  'via',
  'stale',
  'lent',
  'unready',
  'people',
  'shelf',
  'unanswered',
  'unheard',
  'refused',
  'behind',
  'removals',
  'visiting',
  'changed',
  'me',
];
export const states = /** @type {const} */ ({
  tab: ['unheld', 'holding', 'elsewhere', 'taking'],
  list: ['unasked', 'reading', 'read', 'unread'],
});

// The parts of a fragment this page reads, besides a link's token.
const READ = ['open', 'at', 'anew', 'cold', 'ask', 'app', 'native'];
const say = (/** @type {string | undefined} */ said) => sending('failed', { said, why: said });
// The browser's locks: absent where the page is not a secure context (Web
// Locks, `navigator.locks` is `[SecureContext]`), and in a browser before them.
const locks = () => /** @type {LockManager | undefined} */ (navigator.locks);
// The read, of `group` as kept: counted, so an older answer is refused by name.
/**
 * @param {State} state
 * @param {Partial<State>} patch
 * @returns {Stepped}
 */
const reading = (state, patch) => {
  const [asked, serial] = ask(state, 'list');
  /** @type {Effect} */
  const read = { run: 'read', group: patch.group ?? state.group, n: serial, failed: 'unread' };
  return go(state, { list: 'reading' }, { ...patch, asked }, read);
};
// A read again that fails keeps the list drawn, and says why over it; a
// first read that fails lets the lock go, and says why in the heading.
/**
 * @param {State} state
 * @param {string} [said]
 * @param {string} [why]
 * @returns {Stepped}
 */
const failing = (state, said, why = said) => {
  if (state.lent) return go(state, { list: 'unread' }, { stale: why });
  // Cast as either effect, so `go` takes it and the `say` as one list.
  const release = /** @type {Effect | Send} */ ({ run: 'release', failed: 'failed' });
  return go(state, { list: 'unread' }, {}, release, say(said));
};
// Read again, after a write here or on coming back on screen (PAGE_STATE
// §3, 6), once a read has answered.
/**
 * @param {State} state
 * @returns {Stepped}
 */
const readAgain = (state) => {
  if (state.at.tab === 'holding' && ['read', 'unread'].includes(state.at.list)) {
    return reading(state, {});
  }
  return no(state, `the list is ${state.at.list}, in a tab ${state.at.tab}`);
};
const HOW = {
  unanswered: 'did not answer, so what they lend is not listed',
  unheard: 'has not heard of the computer you reached yet, so what they lend is not listed yet',
  refused: 'answered without saying what they lend, so it is not listed',
  behind: 'runs an older svrngs, so what they lend is not listed',
};

/** @typedef {Extract<Message, { type: 'looked' }>} Looked */
/** @typedef {Step<State, Effect | Send>} Stepped */

// A link's token: the fragment's first part, unless it names a part.
/** @param {string} fragment */
const tokenOf = (fragment) => {
  // `split` always answers one part.
  const first = /** @type {string} */ (fragment.split('&')[0]);
  return first.includes('=') ? '' : first;
};

// Why a fragment cannot be read: a part this page does not read, or a
// token svrngs-core cannot (`linked`). None when it can be.
/**
 * @param {URLSearchParams} parts
 * @param {string} token
 * @param {boolean} linked
 */
const unreadableIn = (parts, token, linked) => {
  const unread = [...parts.keys()].filter((key) => key !== token && !READ.includes(key));
  if (unread.length)
    return `This address asks for something this page can’t read: ${unread.join(', ')}. Open your group’s link again.`;
  if (token && !linked)
    return 'This link isn’t one svrngs can read. Ask whoever sent it for a new one.';
  return null;
};

// What the address wants opened or asked, when it names either.
/** @param {URLSearchParams} parts */
const wantsOf = (parts) =>
  parts.has('open') || parts.has('ask') ? /** @type {Wants} */ (Object.fromEntries(parts)) : null;

// The groups kept whose addresses, as last read, hold the lend `#open=` names.
/**
 * @param {Group[]} kept
 * @param {Wants | null} wants
 */
const lendingIn = (kept, wants) =>
  wants?.open
    ? kept.filter((group) => group.addresses?.includes(/** @type {string} */ (wants.open)))
    : [];

// A readable address, `parts`, with its `token` if any: the group it
// opens, held, or each group offered that lends what it opens, or the
// link's join; else said why not.
/**
 * @param {State} state
 * @param {Looked} message
 * @param {URLSearchParams} parts
 * @param {string} token
 * @returns {Stepped}
 */
const addressed = (state, { kept }, parts, token) => {
  const wants = wantsOf(parts);
  if (token && parts.has('native'))
    return go(state, {}, {}, sending('join', { token, kept: [], wants: null, native: true }));
  const offer = token ? [sending('offer', { token })] : [];
  const lending = lendingIn(kept, wants);
  if (!token && lending.length > 1) {
    // Lent in a group, `wants` names what is open (`lending`).
    const named = /** @type {Wants} */ (wants);
    const why = `#open=${named.open} is lent in ${lending.length} of the ${kept.length} groups held here`;
    return go(state, {}, {}, sending('among', { groups: lending, wants: named, why }));
  }
  const group = token ? kept.find((one) => one.token === token) : (lending[0] ?? kept[0]);
  if (group) {
    // The link's line offered first, then the group held.
    const held = takes.hold(state, { type: 'hold', group, wants });
    return { ...held, effects: [...offer, ...held.effects] };
  }
  if (token) return go(state, {}, {}, ...offer, sending('join', { token, kept, wants }));
  return ungrouped(state, wants);
};

// An address that opens no group kept here: the lend asked for where
// this browser may (`fine`), else said why not.
/**
 * @param {State} state
 * @param {Wants | null} wants
 * @returns {Stepped}
 */
const ungrouped = (state, wants) => {
  if (wants?.open && state.fine) return go(state, {}, {}, sending('opens', { wants }));
  // Offered with no token only when the address asks nothing.
  const offer = wants ? [] : [sending('offer', { token: '' })];
  const said = wants?.open
    ? 'This browser hasn’t joined your group yet, so it can’t open this. Open your group’s link from the chat here first, then this again.'
    : 'Open this page from your group’s link.';
  return go(state, {}, {}, ...offer, say(said));
};

/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  start: (state, { fragment }) => go(state, {}, {}, { run: 'look', fragment, failed: 'failed' }),
  // A link's token first, if any, then what it opens or asks; a part this
  // page cannot read, or a token svrngs-core cannot, is said, never joined.
  // `#open=` opens in the group whose addresses, as last read, hold it:
  // one, or each offered when more do (`among`).
  looked: (state, message) => {
    const { fragment, cold, missing, linked } = message;
    if (cold) return go(state, {}, {}, sending('cold'));
    if (missing)
      return go(
        state,
        {},
        {},
        say('There’s no page at this address. Open your group’s link to reach your group.'),
      );
    if (fragment.startsWith('visit='))
      return go(state, {}, {}, sending('visit', { token: fragment.slice('visit='.length) }));
    const parts = new URLSearchParams(fragment);
    const token = tokenOf(fragment);
    const unreadable = unreadableIn(parts, token, linked);
    if (unreadable) return go(state, {}, {}, say(unreadable));
    return addressed(state, message, parts, token);
  },
  // One tab at a time reaches a group: the one holding its lock.
  hold: (state, { group, wants = null, joined = false }) =>
    state.at.tab === 'unheld'
      ? go(state, {}, { group, wants, joined }, { run: 'lock', id: group.id, failed: 'failed' })
      : no(state, `${state.group.name} is ${state.at.tab} here already`),
  held: (state) =>
    ['unheld', 'taking'].includes(state.at.tab)
      ? go(state, { tab: 'holding' }, {}, { run: 'open', group: state.group, failed: 'failed' })
      : no(state, 'no lock was asked for'),
  elsewhere: (state) =>
    state.at.tab === 'unheld'
      ? go(state, { tab: 'elsewhere' })
      : no(state, 'no lock was asked for'),
  take: (state) =>
    state.at.tab === 'elsewhere'
      ? go(
          state,
          { tab: 'taking' },
          {},
          { run: 'lock', id: state.group.id, steal: true, failed: 'failed' },
        )
      : no(state, 'the group is not open in another tab'),
  released: (state) => go(state, { tab: 'unheld' }),
  opened: (state, { group }) =>
    state.at.tab === 'holding' ? reading(state, { group }) : no(state, 'this tab holds no lock'),
  shown: readAgain,
  reread: readAgain,
  // What the group lends, its refusals said by whose computer gave them,
  // and the computers and addresses kept.
  read: (state, message) => {
    const later = late(state, 'list', message.n);
    if (later) return no(state, later);
    const whom = whoseComputer(message.via ?? message.link);
    const name = state.group.name;
    if (!message.ok)
      return failing(
        state,
        message.refusal === 'removed'
          ? `You are no longer in ${name}.`
          : `${whom} said no when you asked what ${name} lends.`,
      );
    if (message.older)
      return failing(
        state,
        `${whom} runs an older svrngs, so this page cannot show what ${name} lends. Installing svrngs again there updates it.`,
      );
    const { lent } = message;
    // Answered, at this version or later, so the body is whole: an answer
    // is never read field by field (version.rs `older`).
    const {
      unanswered,
      unheard,
      refused,
      older: behind,
      removals,
      visiting,
      people,
      shelf,
      changed,
    } = /** @type {Body} */ (message.body);
    const computers = [
      ...new Map(
        lent.map((lend) => [
          lend.key,
          { key: lend.key, relay: lend.relay ?? null, person: lend.person },
        ]),
      ).values(),
    ];
    const group = {
      ...state.group,
      computers,
      meets: message.meets ?? state.group.meets,
      addresses: [
        ...lent.map((lend) => lend.label),
        ...once(lent).map((lend) => alias(lend, lent, people)),
      ],
    };
    return go(
      state,
      { list: 'read' },
      {
        group,
        via: message.via,
        stale: '',
        lent,
        people,
        shelf,
        unanswered,
        unheard,
        refused,
        behind,
        removals,
        visiting,
        changed,
        me: message.me,
      },
      { run: 'keep', group, lent, failed: 'failed' },
    );
  },
  unread: (state, message) => {
    const later = late(state, 'list', message.n);
    if (later) return no(state, later);
    return failing(state, message.said, message.said ?? message.why);
  },
  // Read, what `#open=` names is sent to be named (among.js `wanted`).
  // Sent only when `wants` names what is open.
  kept: (state, { unready }) => {
    if (!state.wants?.open) return go(state, {}, { unready });
    // Its `open` is the string just tested.
    const wants = /** @type {Wants & { open: string }} */ (state.wants);
    return go(state, {}, { unready }, sending('wanted', { wants }));
  },
};

/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  elsewhere: (state) =>
    state.at.tab === 'elsewhere' && [`You have ${titled(state.group)} open in another tab`, 'good'],
  enter: (state) => {
    if (state.lent) {
      const through = state.via ? `, through ${state.via.person}’s computer` : '';
      return [`You’re in ${state.group.name}, as ${state.group.person}${through}`, 'good'];
    }
    return state.at.list === 'reading' && [`Reaching ${state.group.name}…`];
  },
};
// Each computer that did not say what its person lends, and by how; and
// nobody lending said only when every computer did.
/**
 * @param {State} state
 * @returns {string[]}
 */
const unlisted = (state) => {
  const hows = /** @type {[keyof typeof HOW, string][]} */ (Object.entries(HOW));
  return hows.flatMap(([how, why]) =>
    state[how].map((whom) => {
      const name = called(state.people, whom.person_key) ?? whom.person;
      return `${name}’s computer ${why}.`;
    }),
  );
};
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  enter: (state) => {
    if (!state.lent) return undefined;
    const missing = unlisted(state);
    const stale = state.stale
      ? [`What ${state.group.name} lends cannot be read again just now: ${state.stale}`]
      : [];
    const none = state.lent.length + missing.length === 0;
    const nobody = none ? [`Nobody in ${state.group.name} lends anything yet.`] : [];
    return [...stale, ...missing, ...nobody, ...(state.unready ?? [])];
  },
};

// The edge.
// Whether svrngs-core reads `token` as a link's, naming who issued it; a
// token it cannot parse is no link.
/**
 * @param {typeof Wasm} core
 * @param {string} token
 */
const issued = (core, token) => {
  try {
    return Boolean(core.issuer_of(token));
  } catch {
    return false;
  }
};
// The lock's letting go, set as the lock is held.
/** @type {((value?: unknown) => void) | undefined} */
let letGo;
// The lock asked for, or taken from another tab: held until `release` lets
// it go, as the browser holds a lock while its callback's promise is
// unsettled. Taken by another tab, this one opens again.
/**
 * @param {string} id
 * @param {boolean} [steal]
 * @returns {Promise<PageMessage>}
 */
const locking = (id, steal) =>
  new Promise((answer, failed) => {
    const options = steal ? { steal } : { ifAvailable: true };
    /** @param {Lock | null} lock */
    const granted = (lock) => {
      if (!lock) {
        answer({ type: 'elsewhere', why: `${id} is open in another tab` });
        return undefined;
      }
      const taken = steal ? ', taken from another tab' : '';
      answer({ type: 'held', why: `this tab holds ${id}${taken}` });
      return new Promise((resolve) => {
        letGo = resolve;
      });
    };
    navigator.locks.request(`svrngs ${id}`, options, granted).catch((error) => {
      if (error.name !== 'AbortError') {
        failed(error);
        return;
      }
      trace(`PAGE: another tab took ${id}, so this one opens again`);
      location.reload();
    });
  });
// How many of the group's relays a phone keeps, so a computer that lies
// cannot slow every later dial with a flood of them (KEEP_GOING_AT_HOME 6).
export const MEETS_KEPT = 8;
// The relays a reply names that this page takes, each once, at most
// MEETS_KEPT; none when it names no list, as an older computer's does, or
// none it takes, as one yet to read the group's record, so what is kept stands.
// No command leaves a group with no relay, so an empty list never means one.
/**
 * @param {typeof Wasm} core
 * @param {unknown} said
 */
export const meeting = (core, said) => {
  const named = Array.isArray(said) ? [...new Set(said)] : [];
  const taken = named.filter((one) => typeof one === 'string' && core.relay_taken(one));
  trace(`PAGE: kept ${Math.min(taken.length, MEETS_KEPT)} of ${named.length} relay(s) named`);
  return taken.length > 0 ? taken.slice(0, MEETS_KEPT) : undefined;
};
// Each lend with its label, and whose lender `apart` says; and `me`, the
// key this page holds as the record spells it, public, which the record's
// rows name, never the key itself.
/**
 * @param {Group} group
 * @param {number} serial
 * @returns {Promise<PageMessage>}
 */
async function readLends(group, serial) {
  const core = await wasm();
  const [connection, via, res] = await lends(core, group, []);
  // Refused, its body is read for `lent` and `version` alone, both optional.
  const body = /** @type {Partial<Body>} */ (res.ok ? await res.json() : {});
  const refusal = res.headers.get('x-svrngs-refused');
  await connection.close();
  const lent = (body.lent ?? []).map((lend) => ({
    ...lend,
    label: core.label(lend.key, lend.name),
  }));
  const link = group.computers?.find((computer) => computer.key === core.issuer_of(group.token));
  apart(lent, body.people ?? []);
  const refused = refusal ? `, refused ${refusal}` : '';
  return {
    type: 'read',
    n: serial,
    ok: res.ok,
    refusal,
    via,
    link,
    body,
    lent,
    meets: res.ok ? meeting(core, body.meets) : undefined,
    me: actor(core),
    older: res.ok && core.older(body.version),
    why: `${res.status}${refused}, at version ${body.version}, ${lent.length} lent`,
  };
}
// A readiness check that could not run, said rather than failing a good read.
/** @param {unknown} error */
const unchecked = (error) => [`This phone could not check what it has saved: ${String(error)}`];
/** @type {Io<Effect, PageMessage>} */
export const io = {
  look: async ({ fragment }) => {
    const [core, kept] = await Promise.all([wasm(), store.groups()]);
    // `split` always answers one part.
    const token = /** @type {string} */ (fragment.split('&')[0]);
    return {
      type: 'looked',
      fragment,
      kept,
      missing: document.body.id === 'missing',
      cold: core.is_label(store.labelOf(location.hostname)),
      linked: issued(core, token),
    };
  },
  lock: ({ id, steal }) => {
    if (!locks())
      return { type: 'held', why: 'this browser has no locks, so each tab reaches the group' };
    return locking(id, steal);
  },
  release: () => {
    letGo?.();
    return { type: 'released' };
  },
  open: async ({ group }) => {
    const opened = await store.opened(group);
    // Asked on each open, never awaited, so a browser asking its person
    // first holds nothing.
    void store
      .persist()
      .catch((error) => `asking the browser to keep the page's storage failed: ${error}`)
      .then((line) => trace(`PAGE: ${line}`));
    hold(await store.unseal(opened));
    return { type: 'opened', group: opened };
  },
  read: ({ group, n: serial }) =>
    readLends(group, serial).catch((error) => ({
      type: 'unread',
      n: serial,
      said: error.said,
      why: String(error),
    })),
  keep: async ({ group, lent }) => {
    await store.opened(group);
    // Only a group with a relay of its own goes on when the internet goes.
    if ((group.meets?.length ?? 0) === 0) return { type: 'kept' };
    return { type: 'kept', unready: await store.unready(lent, saidOf).catch(unchecked) };
  },
};
