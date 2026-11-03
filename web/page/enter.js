// SPDX-License-Identifier: AGPL-3.0-or-later
// The flow that opens the page and reads the group, the spine every phone
// flow reads (page-open (b)): web/app.js `opening` (:667-706), `hold` and
// `elsewhere` (:385-411), and `enter`'s read (:413-458), as two regions.
// `tab` is the group's one lock, held by one tab of this browser (#176);
// `list` its read, of the first computer to answer, then the next (#205).
// What the address asks is decided here; what it opens elsewhere is sent to
// its flow (`visit`, `join`, `among`, `cold`, `opens`, `offer`), and what
// `#open=` names, once the group is read (`wanted`).
// The key and the endpoint are the edge's (`io`), never in state.
import {
  Act,
  alias,
  apart,
  ask,
  called,
  go,
  html,
  late,
  no,
  once,
  whose,
} from '../ui/kit.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { offered } from './among.js?v=__VERSION__';
/** @import { Group, Computer } from '../store.js?v=__VERSION__' */
/** @import { Lend, Removal, PeopleRow, Shelved } from '../question.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Io, Step, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Dialled, Endpoint } from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Message as PageMessage, Nothing, Props, Said, Send, Sender, State } from './page.js?v=__VERSION__' */

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
 * @typedef {{ open?: string, at?: string, anew?: string, ask?: string, app?: string, cold?: string }} Wants
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
 * @property {number} [version]
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'start', fragment: string }
 *   | { type: 'looked', fragment: string, kept: Group[], missing: boolean, cold: boolean, linked: boolean }
 *   | { type: 'hold', group: Group, wants?: Wants | null, joined?: boolean }
 *   | { type: 'held' }
 *   | { type: 'elsewhere' }
 *   | { type: 'take' }
 *   | { type: 'released' }
 *   | { type: 'opened', group: Group }
 *   | { type: 'shown' }
 *   | { type: 'reread' }
 *   | { type: 'read', n: number, ok: boolean, refusal: string | null, via?: Computer, link?: Computer, body: Body, lent: Lent[], me: string, older: boolean }
 *   | { type: 'unread', n?: number, said?: string, why: string }
 *   | { type: 'kept' }} Message
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
 *   | { run: 'keep', group: Group, failed: string }} Effect
 */

export const owns = [
  'group',
  'wants',
  'joined',
  'via',
  'stale',
  'lent',
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

// The parts of a fragment this page reads, besides a link's token (app.js@19b63b2b:665).
const READ = ['open', 'at', 'anew', 'cold', 'ask', 'app'];
const sending = /** @type {Sender} */ (
  (type, message) => ({ run: 'send', m: { type, ...message } })
);
const say = (/** @type {string | undefined} */ said) => sending('failed', { said, why: said });
const titled = (/** @type {Group} */ group) =>
  `${group.name}${group.maker ? ` (${whose(group)})` : ''}`;
const whoseComputer = (/** @type {Computer | undefined} */ computer) =>
  computer ? `${computer.person}’s computer` : 'The computer that sent your link';
// The read, of `group` as kept: counted, so an older answer is refused by name.
const reading = (/** @type {State} */ state, /** @type {Partial<State>} */ patch) =>
  ((asked, serial) =>
    go(
      state,
      { list: 'reading' },
      { ...patch, asked },
      /** @satisfies {Effect} */ ({
        run: 'read',
        group: patch.group ?? state.group,
        n: serial,
        failed: 'unread',
      }),
    ))(...ask(state, 'list'));
// A read again that fails keeps the list drawn, and says why (page-open (c)).
// Its type said whole, as a second declarator's JSDoc is not read.
const failing =
  /** @satisfies {(s: State, said?: string, why?: string) => Step<State, Effect | Send>} */ (
    (state, said, why = said) =>
      state.lent
        ? go(state, { list: 'unread' }, { stale: why })
        : go(
            state,
            { list: 'unread' },
            {},
            // Typed as both effects, so `go` takes them as one list.
            /** @type {Effect | Send} */ ({ run: 'release', failed: 'failed' }),
            say(said),
          )
  );
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
  const offer = token ? [sending('offer', { token })] : [];
  const lending = lendingIn(kept, wants);
  // Lent in a group, `wants` names what is open (`lending`).
  if (!token && lending.length > 1)
    return go(
      state,
      {},
      {},
      sending('among', {
        groups: lending,
        wants: /** @type {Wants} */ (wants),
        why: `#open=${/** @type {Wants} */ (wants).open} is lent in ${lending.length} of the ${kept.length} groups held here`,
      }),
    );
  const group = token ? kept.find((one) => one.token === token) : (lending[0] ?? kept[0]);
  if (group)
    return ((result) => ({ ...result, effects: [...offer, ...result.effects] }))(
      // `hold` reads only the group and what the address wants.
      takes.hold(state, /** @type {Extract<Message, { type: 'hold' }>} */ ({ group, wants })),
    );
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
  return go(
    state,
    {},
    {},
    ...(wants ? [] : [sending('offer', { token: '' })]),
    say(
      wants?.open
        ? 'This browser hasn’t joined your group yet, so it can’t open this. Open your group’s link from the chat here first, then this again.'
        : 'Open this page from your group’s link.',
    ),
  );
};

/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  start: (state, { fragment }) => go(state, {}, {}, { run: 'look', fragment, failed: 'failed' }),
  // A link's token first, if any, then what it opens or asks; a part this
  // page cannot read, or a token svrngs-core cannot, is said, never joined
  // (app.js@19b63b2b:681-687). `#open=` opens in the group whose addresses, as last
  // read, hold it: one, or each offered when more do (`among`).
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
  // One tab at a time reaches a group (app.js@19b63b2b:375-384): the one holding its lock.
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
  // Read again, after a write here or on coming back on screen (PAGE_STATE §3, 6).
  // `reread` reads no message.
  shown: (state) => /** @type {(s: State) => Step<State, Effect | Send>} */ (takes.reread)(state),
  reread: (state) =>
    state.at.tab === 'holding' && ['read', 'unread'].includes(state.at.list)
      ? reading(state, {})
      : no(state, `the list is ${state.at.list}, in a tab ${state.at.tab}`),
  // What the group lends, its refusals said by whose computer gave them
  // (app.js@19b63b2b:438-451), and the computers and addresses kept (app.js@19b63b2b:456-458).
  read: (state, message) => {
    if (late(state, 'list', message.n)) return no(state, late(state, 'list', message.n));
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
    const {
      lent,
      body: {
        unanswered,
        unheard,
        refused,
        older: behind,
        removals,
        visiting,
        people,
        shelf,
        changed,
      },
    } = message;
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
      { run: 'keep', group, failed: 'failed' },
    );
  },
  unread: (state, message) =>
    late(state, 'list', message.n)
      ? no(state, late(state, 'list', message.n))
      : failing(state, message.said, message.said ?? message.why),
  // Read, what `#open=` names is sent to be named (among.js `wanted`).
  // Sent only when `wants` names what is open.
  kept: (state) =>
    go(
      state,
      {},
      {},
      ...(state.wants?.open
        ? [sending('wanted', { wants: /** @type {Wants & { open: string } } */ (state.wants) })]
        : []),
    ),
};

/** @type {Record<string, (s: State) => Said | Nothing>} */
export const heading = {
  elsewhere: (state) =>
    state.at.tab === 'elsewhere' && [`You have ${titled(state.group)} open in another tab`, 'good'],
  enter: (state) =>
    state.lent
      ? [
          `You’re in ${state.group.name}, as ${state.group.person}${state.via ? `, through ${state.via.person}’s computer` : ''}`,
          'good',
        ]
      : state.at.list === 'reading' && [`Reaching ${state.group.name}…`],
};
// Each computer that did not say by how, and nobody lending said only when
// every computer did (app.js@19b63b2b:578-581).
const unlisted = (/** @type {State} */ state) =>
  /** @type {[keyof typeof HOW, string][]} */ (Object.entries(HOW)).flatMap(([how, why]) =>
    state[how].map(
      (person) =>
        `${called(state.people, person.person_key) ?? person.person ?? person}’s computer ${why}.`,
    ),
  );
/** @type {Record<string, (s: State) => string[] | Nothing>} */
export const response = {
  // Read, `missing` is the list's, as `state.lent` is.
  enter: (state, missing = state.lent && unlisted(state)) =>
    state.lent && [
      ...(state.stale
        ? [`What ${state.group.name} lends cannot be read again just now: ${state.stale}`]
        : []),
      ...(state.lent.length || /** @type {string[]} */ (missing).length
        ? /** @type {string[]} */ (missing)
        : [`Nobody in ${state.group.name} lends anything yet.`]),
    ],
};
// The group open in another tab, and taking it here (app.js@19b63b2b:403-410), drawn in
// #door, as is the group to open `#open=` in, when more than one lends it.
/**
 * @param {State} state
 * @param {Props['send']} send
 */
export const chosen = (state, send) =>
  offered(state, send) ||
  (state.at.tab === 'elsewhere' &&
    html`<p>One tab at a time reaches ${state.group.name}. Open it here, and the other tab stops reaching it.</p><${Act} words="Open it here" on=${{ type: 'take' }} send=${send} />`);

// The edge: the wasm, imported here only once app.js has run its init,
// the key unsealed and the one endpoint bound as the person (app.js@19b63b2b:427-428),
// and the lock's letting go.
// The page's own lines go to ui/trace.js's sink, as ui/machine.js's do.
const PATIENCE = 60_000;
const PEOPLE_LENDS = '/people/lends';
const wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__');
const trace = (/** @type {string} */ line) => globalThis.__trace?.(line);
const said = (/** @type {string} */ words) => Object.assign(new Error(words), { said: words });
// Each typed apart, as JSDoc types only the first of several declared at once.
// The key, unsealed by `open`, which comes before every read (`opened`).
/** @type {Uint8Array<ArrayBuffer>} */
let key;
/** @type {Promise<Endpoint> | null} */
let bound;
/** @type {((value?: unknown) => void) | undefined} */
let letGo;
// `group` dialled through the first to answer of its link's computer and
// each its last read named, all at once (app.js@19b63b2b:355-369).
/**
 * @param {typeof Wasm} core
 * @param {Group} group
 * @param {string[]} tried the computers asked already, by key
 * @returns {Promise<[Dialled] | [Dialled, Computer]>}
 */
async function reach(core, group, tried) {
  if (!core.relay_of(group.token))
    throw said('This link was made by an older svrngs. Ask whoever sent it for a new one.');
  const issuer = core.issuer_of(group.token);
  const link = group.computers?.find((one) => one.key === issuer);
  const linked = !tried.includes(issuer);
  const others = (group.computers ?? []).filter((one) => one !== link && !tried.includes(one.key));
  const ep = () => (bound ??= core.endpoint(key, group.token));
  trace(
    `PAGE: dialling ${linked ? "the link's computer and " : ''}${others.length} other computer(s) at once`,
  );
  const tries = [
    ...(linked
      ? [
          ep()
            .then((endpoint) => endpoint.dial_link())
            .then((reached) => /** @satisfies {[Dialled]} */ ([reached])),
        ]
      : []),
    ...others.map((other) =>
      ep()
        .then((endpoint) => endpoint.dial(other.key, other.relay))
        .then((reached) => /** @satisfies {[Dialled, Computer]} */ ([reached, other])),
    ),
  ];
  const away = `${[...(linked ? [link] : []), ...others].map((one) => `${whoseComputer(one)} did not answer.`).join(' ')} Try again later.`;
  const [dialled, computer] = await Promise.race([
    Promise.any(tries),
    new Promise((_, reject) => setTimeout(() => reject(new Error('no answer')), PATIENCE)),
  ]).catch((error) => Promise.reject((trace(`PAGE: the dial failed: ${error}`), said(away))));
  trace(
    `PAGE: reached ${group.name} through ${computer ? `${computer.person}'s` : "the link's"} computer`,
  );
  tries.forEach((attempt) =>
    attempt.then(
      ([answered]) =>
        answered !== dialled &&
        (trace('PAGE: a computer answered late, so closed'), answered.close()),
      () => {},
    ),
  );
  return [dialled, computer];
}
// Its lends, asked of the next to answer when an answer fails (app.js@19b63b2b:430-436).
/**
 * @param {typeof Wasm} core
 * @param {Group} group
 * @param {string[]} tried
 * @returns {Promise<[Dialled, Computer | undefined, Response]>}
 */
const lends = (core, group, tried) =>
  reach(core, group, tried).then(([dialled, via]) =>
    dialled.fetch('GET', PEOPLE_LENDS, [['accept', 'application/json']], null).then(
      (res) => [dialled, via, res],
      (error) => {
        trace(
          `PAGE: ${PEOPLE_LENDS} failed through ${whoseComputer(via)}, so the next to answer is asked: ${error}`,
        );
        dialled.close();
        return lends(core, group, [...tried, via?.key ?? core.issuer_of(group.token)]).catch(() =>
          Promise.reject(error),
        );
      },
    ),
  );
// A write through the computer the page reaches, for another flow's io
// (page-shelf): `body` posted with the group's id, as JSON, and its answer
// read, or the computer's refusal in its own words (app.js@19b63b2b:110-116, :485-488).
// Here, as the key and the one endpoint are: a second endpoint of the key
// would displace this one at the relay, which keeps one per key (app.js@19b63b2b:352).
// One lender's computer, `lend`, dialled as the person through that one
// endpoint, for another flow's io (page-acts: what apps use, and the ask's
// yes, posted to its door's /choices; app.js@19b63b2b:288-296), within PATIENCE.
/**
 * @param {Group} group
 * @param {Lent} lend
 */
export const dialled = (group, lend) =>
  wasm().then((core) =>
    Promise.race([
      Promise.resolve((bound ??= core.endpoint(key, group.token))).then((endpoint) =>
        endpoint.dial(lend.key, lend.relay ?? null),
      ),
      new Promise((_, reject) => setTimeout(() => reject(new Error('no answer')), PATIENCE)),
    ]),
  );
/**
 * @param {Group} group
 * @param {string} path
 * @param {unknown} body
 */
export const through = (group, path, body) =>
  wasm().then(async (core) => {
    const [connection] = await reach(core, group, []);
    const bytes = new TextEncoder().encode(JSON.stringify([group.id, body]));
    const [ok, status, text] = await connection
      .fetch(
        'POST',
        path,
        [
          ['content-type', 'application/json'],
          ['content-length', String(bytes.length)],
        ],
        new Blob([bytes]).stream(),
      )
      .then(async (answer) => [answer.ok, answer.status, await answer.text()])
      .finally(() => connection.close());
    return ok
      ? JSON.parse(text)
      : Promise.reject(
          (trace(`PAGE: ${path} answered ${status}: ${text}`), new Error(JSON.parse(text).error)),
        );
  });
/** @type {Io<Effect, PageMessage>} */
export const io = {
  // `split` always answers one part.
  look: ({ fragment }, token = /** @type {string} */ (fragment.split('&')[0])) =>
    Promise.all([wasm(), store.groups()]).then(([core, kept]) => ({
      type: 'looked',
      fragment,
      kept,
      missing: document.body.id === 'missing',
      cold: core.is_label(store.labelOf(location.hostname)),
      linked: (() => {
        try {
          return Boolean(core.issuer_of(token));
        } catch {
          return false;
        }
      })(),
    })),
  // The lock held forever once given; taken by another tab, this one opens again (app.js@19b63b2b:386-398).
  lock: ({ id, steal }) =>
    !navigator.locks
      ? { type: 'held', why: 'this browser has no locks, so each tab reaches the group' }
      : new Promise((answer, failed) =>
          navigator.locks
            .request(`svrngs ${id}`, steal ? { steal } : { ifAvailable: true }, (lock) =>
              lock
                ? (answer({
                    type: 'held',
                    why: `this tab holds ${id}${steal ? ', taken from another tab' : ''}`,
                  }),
                  new Promise((resolve) => (letGo = resolve)))
                : answer({ type: 'elsewhere', why: `${id} is open in another tab` }),
            )
            .catch((error) =>
              error.name === 'AbortError'
                ? (trace(`PAGE: another tab took ${id}, so this one opens again`),
                  location.reload())
                : failed(error),
            ),
        ),
  release: () => {
    letGo?.();
    return { type: 'released' };
  },
  // Asked on each open, never awaited, so a browser asking its person first holds nothing (app.js@19b63b2b:417-421).
  open: ({ group }) =>
    store.opened(group).then(async (opened) => {
      store
        .persist()
        .catch((error) => `asking the browser to keep the page's storage failed: ${error}`)
        .then((line) => trace(`PAGE: ${line}`));
      [key, bound] = [await store.unseal(opened), null];
      return { type: 'opened', group: opened };
    }),
  // Each lend with its label, and whose lender `apart` says; and `me`, the
  // key this page holds as the record spells it, public, which the record's
  // rows name (app.js@19b63b2b:467), never the key itself.
  read: ({ group, n: serial }) =>
    wasm()
      .then(async (core) => {
        const [connection, via, res] = await lends(core, group, []);
        // Refused, its body is read for `lent` and `version` alone, both optional.
        const body = /** @type {Body} */ (res.ok ? await res.json() : {});
        const refusal = res.headers.get('x-svrngs-refused');
        const lent = (await connection.close(), body.lent ?? []).map((lend) => ({
          ...lend,
          label: core.label(lend.key, lend.name),
        }));
        const link = group.computers?.find(
          (computer) => computer.key === core.issuer_of(group.token),
        );
        apart(lent, body.people ?? []);
        return /** @satisfies {PageMessage} */ ({
          type: 'read',
          n: serial,
          ok: res.ok,
          refusal,
          via,
          link,
          body,
          lent,
          me: core.actor(key),
          older: res.ok && core.older(body.version),
          why: `${res.status}${refusal ? `, refused ${refusal}` : ''}, at version ${body.version}, ${lent.length} lent`,
        });
      })
      .catch((error) => ({ type: 'unread', n: serial, said: error.said, why: String(error) })),
  keep: ({ group }) => store.opened(group).then(() => ({ type: 'kept' })),
};
