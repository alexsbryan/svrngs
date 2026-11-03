// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
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
/** @import { Lend, Removal, Row, Shelved } from '../question.js?v=__VERSION__' */
/** @import { Change } from '../ui/changes.js?v=__VERSION__' */
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */

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
 * @property {Row[]} people
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
 * @property {Row[]} people
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
const READ = ['open', 'at', 'anew', 'cold', 'ask', 'app'],
  send = (type, m) => ({ run: 'send', m: { type, ...m } }),
  say = (said) => send('failed', { said, why: said });
const titled = (g) => `${g.name}${g.maker ? ` (${whose(g)})` : ''}`,
  whoseComputer = (c) => (c ? `${c.person}’s computer` : 'The computer that sent your link');
// The read, of `group` as kept: counted, so an older answer is refused by name.
const reading = (s, patch) =>
    ((asked, n) =>
      go(
        s,
        { list: 'reading' },
        { ...patch, asked },
        { run: 'read', group: patch.group ?? s.group, n, failed: 'unread' },
      ))(...ask(s, 'list')),
  // A read again that fails keeps the list drawn, and says why (page-open (c)).
  failing = (s, said, why = said) =>
    s.lent
      ? go(s, { list: 'unread' }, { stale: why })
      : go(s, { list: 'unread' }, {}, { run: 'release', failed: 'failed' }, say(said));
const HOW = {
  unanswered: 'did not answer, so what they lend is not listed',
  unheard: 'has not heard of the computer you reached yet, so what they lend is not listed yet',
  refused: 'answered without saying what they lend, so it is not listed',
  behind: 'runs an older svrngs, so what they lend is not listed',
};

export const takes = {
  start: (s, { fragment }) => go(s, {}, {}, { run: 'look', fragment, failed: 'failed' }),
  // A link's token first, if any, then what it opens or asks; a part this
  // page cannot read, or a token svrngs-core cannot, is said, never joined
  // (app.js@19b63b2b:681-687). `#open=` opens in the group whose addresses, as last
  // read, hold it: one, or each offered when more do (`among`).
  looked: (s, { fragment, cold, missing, linked, kept }) => {
    if (cold) return go(s, {}, {}, send('cold'));
    if (missing)
      return go(
        s,
        {},
        {},
        say('There’s no page at this address. Open your group’s link to reach your group.'),
      );
    if (fragment.startsWith('visit='))
      return go(s, {}, {}, send('visit', { token: fragment.slice('visit='.length) }));
    const parts = new URLSearchParams(fragment),
      first = fragment.split('&')[0],
      token = first.includes('=') ? '' : first;
    const unread = [...parts.keys()].filter((k) => k !== token && !READ.includes(k));
    if (unread.length)
      return go(
        s,
        {},
        {},
        say(
          `This address asks for something this page can’t read: ${unread.join(', ')}. Open your group’s link again.`,
        ),
      );
    if (token && !linked)
      return go(
        s,
        {},
        {},
        say('This link isn’t one svrngs can read. Ask whoever sent it for a new one.'),
      );
    const wants = parts.has('open') || parts.has('ask') ? Object.fromEntries(parts) : null,
      offer = token ? [send('offer', { token })] : [];
    const lending = wants?.open ? kept.filter((g) => g.addresses?.includes(wants.open)) : [];
    if (!token && lending.length > 1)
      return go(
        s,
        {},
        {},
        send('among', {
          groups: lending,
          wants,
          why: `#open=${wants.open} is lent in ${lending.length} of the ${kept.length} groups held here`,
        }),
      );
    const group = token ? kept.find((g) => g.token === token) : (lending[0] ?? kept[0]);
    if (group)
      return ((r) => ({ ...r, effects: [...offer, ...r.effects] }))(
        takes.hold(s, { group, wants }),
      );
    if (token) return go(s, {}, {}, ...offer, send('join', { token, kept, wants }));
    if (wants?.open && s.fine) return go(s, {}, {}, send('opens', { wants }));
    return go(
      s,
      {},
      {},
      ...(wants ? [] : [send('offer', { token: '' })]),
      say(
        wants?.open
          ? 'This browser hasn’t joined your group yet, so it can’t open this. Open your group’s link from the chat here first, then this again.'
          : 'Open this page from your group’s link.',
      ),
    );
  },
  // One tab at a time reaches a group (app.js@19b63b2b:375-384): the one holding its lock.
  hold: (s, { group, wants = null, joined = false }) =>
    s.at.tab === 'unheld'
      ? go(s, {}, { group, wants, joined }, { run: 'lock', id: group.id, failed: 'failed' })
      : no(s, `${s.group.name} is ${s.at.tab} here already`),
  held: (s) =>
    ['unheld', 'taking'].includes(s.at.tab)
      ? go(s, { tab: 'holding' }, {}, { run: 'open', group: s.group, failed: 'failed' })
      : no(s, 'no lock was asked for'),
  elsewhere: (s) =>
    s.at.tab === 'unheld' ? go(s, { tab: 'elsewhere' }) : no(s, 'no lock was asked for'),
  take: (s) =>
    s.at.tab === 'elsewhere'
      ? go(s, { tab: 'taking' }, {}, { run: 'lock', id: s.group.id, steal: true, failed: 'failed' })
      : no(s, 'the group is not open in another tab'),
  released: (s) => go(s, { tab: 'unheld' }),
  opened: (s, { group }) =>
    s.at.tab === 'holding' ? reading(s, { group }) : no(s, 'this tab holds no lock'),
  // Read again, after a write here or on coming back on screen (PAGE_STATE §3, 6).
  shown: (s) => takes.reread(s),
  reread: (s) =>
    s.at.tab === 'holding' && ['read', 'unread'].includes(s.at.list)
      ? reading(s, {})
      : no(s, `the list is ${s.at.list}, in a tab ${s.at.tab}`),
  // What the group lends, its refusals said by whose computer gave them
  // (app.js@19b63b2b:438-451), and the computers and addresses kept (app.js@19b63b2b:456-458).
  read: (s, m) => {
    if (late(s, 'list', m.n)) return no(s, late(s, 'list', m.n));
    const whom = whoseComputer(m.via ?? m.link),
      name = s.group.name;
    if (!m.ok)
      return failing(
        s,
        m.refusal === 'removed'
          ? `You are no longer in ${name}.`
          : `${whom} said no when you asked what ${name} lends.`,
      );
    if (m.older)
      return failing(
        s,
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
    } = m;
    const computers = [
        ...new Map(
          lent.map((l) => [l.key, { key: l.key, relay: l.relay ?? null, person: l.person }]),
        ).values(),
      ],
      group = {
        ...s.group,
        computers,
        addresses: [...lent.map((l) => l.label), ...once(lent).map((l) => alias(l, lent, people))],
      };
    return go(
      s,
      { list: 'read' },
      {
        group,
        via: m.via,
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
        me: m.me,
      },
      { run: 'keep', group, failed: 'failed' },
    );
  },
  unread: (s, m) =>
    late(s, 'list', m.n) ? no(s, late(s, 'list', m.n)) : failing(s, m.said, m.said ?? m.why),
  // Read, what `#open=` names is sent to be named (among.js `wanted`).
  kept: (s) => go(s, {}, {}, ...(s.wants?.open ? [send('wanted', { wants: s.wants })] : [])),
};

export const heading = {
  elsewhere: (s) =>
    s.at.tab === 'elsewhere' && [`You have ${titled(s.group)} open in another tab`, 'good'],
  enter: (s) =>
    s.lent
      ? [
          `You’re in ${s.group.name}, as ${s.group.person}${s.via ? `, through ${s.via.person}’s computer` : ''}`,
          'good',
        ]
      : s.at.list === 'reading' && [`Reaching ${s.group.name}…`],
};
// Each computer that did not say by how, and nobody lending said only when
// every computer did (app.js@19b63b2b:578-581).
const unlisted = (s) =>
  Object.entries(HOW).flatMap(([k, why]) =>
    s[k].map((p) => `${called(s.people, p.person_key) ?? p.person ?? p}’s computer ${why}.`),
  );
export const response = {
  enter: (s, u = s.lent && unlisted(s)) =>
    s.lent && [
      ...(s.stale ? [`What ${s.group.name} lends cannot be read again just now: ${s.stale}`] : []),
      ...(s.lent.length || u.length ? u : [`Nobody in ${s.group.name} lends anything yet.`]),
    ],
};
// The group open in another tab, and taking it here (app.js@19b63b2b:403-410), drawn in
// #door, as is the group to open `#open=` in, when more than one lends it.
export const chosen = (s, send) =>
  offered(s, send) ||
  (s.at.tab === 'elsewhere' &&
    html`<p>One tab at a time reaches ${s.group.name}. Open it here, and the other tab stops reaching it.</p><${Act} words="Open it here" on=${{ type: 'take' }} send=${send} />`);

// The edge: the wasm, imported here only once app.js has run its init,
// the key unsealed and the one endpoint bound as the person (app.js@19b63b2b:427-428),
// and the lock's letting go.
// The page's own lines go to ui/trace.js's sink, as ui/machine.js's do.
const PATIENCE = 60_000,
  PEOPLE_LENDS = '/people/lends',
  wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__'),
  trace = (l) => globalThis.__trace?.(l);
const said = (words) => Object.assign(new Error(words), { said: words });
let key, bound, letGo;
// `group` dialled through the first to answer of its link's computer and
// each its last read named, all at once (app.js@19b63b2b:355-369).
async function reach(w, group, tried) {
  if (!w.relay_of(group.token))
    throw said('This link was made by an older svrngs. Ask whoever sent it for a new one.');
  const issuer = w.issuer_of(group.token),
    link = group.computers?.find((c) => c.key === issuer),
    linked = !tried.includes(issuer);
  const others = (group.computers ?? []).filter((c) => c !== link && !tried.includes(c.key)),
    ep = () => (bound ??= w.endpoint(key, group.token));
  trace(
    `PAGE: dialling ${linked ? "the link's computer and " : ''}${others.length} other computer(s) at once`,
  );
  const tries = [
      ...(linked
        ? [
            ep()
              .then((e) => e.dial_link())
              .then((d) => [d]),
          ]
        : []),
      ...others.map((c) =>
        ep()
          .then((e) => e.dial(c.key, c.relay))
          .then((d) => [d, c]),
      ),
    ],
    away = `${[...(linked ? [link] : []), ...others].map((c) => `${whoseComputer(c)} did not answer.`).join(' ')} Try again later.`;
  const [d, c] = await Promise.race([
    Promise.any(tries),
    new Promise((_, n) => setTimeout(() => n(new Error('no answer')), PATIENCE)),
  ]).catch((e) => Promise.reject((trace(`PAGE: the dial failed: ${e}`), said(away))));
  trace(`PAGE: reached ${group.name} through ${c ? `${c.person}'s` : "the link's"} computer`);
  tries.forEach((t) =>
    t.then(
      ([o]) => o !== d && (trace('PAGE: a computer answered late, so closed'), o.close()),
      () => {},
    ),
  );
  return [d, c];
}
// Its lends, asked of the next to answer when an answer fails (app.js@19b63b2b:430-436).
const lends = (w, group, tried) =>
  reach(w, group, tried).then(([d, via]) =>
    d.fetch('GET', PEOPLE_LENDS, [['accept', 'application/json']], null).then(
      (res) => [d, via, res],
      (e) => (
        trace(
          `PAGE: ${PEOPLE_LENDS} failed through ${whoseComputer(via)}, so the next to answer is asked: ${e}`,
        ),
        d.close(),
        lends(w, group, [...tried, via?.key ?? w.issuer_of(group.token)]).catch(() =>
          Promise.reject(e),
        )
      ),
    ),
  );
// A write through the computer the page reaches, for another flow's io
// (page-shelf): `body` posted with the group's id, as JSON, and its answer
// read, or the computer's refusal in its own words (app.js@19b63b2b:110-116, :485-488).
// Here, as the key and the one endpoint are: a second endpoint of the key
// would displace this one at the relay, which keeps one per key (app.js@19b63b2b:352).
// One lender's computer, `l`, dialled as the person through that one
// endpoint, for another flow's io (page-acts: what apps use, and the ask's
// yes, posted to its door's /choices; app.js@19b63b2b:288-296), within PATIENCE.
export const dialled = (group, l) =>
  wasm().then((w) =>
    Promise.race([
      Promise.resolve((bound ??= w.endpoint(key, group.token))).then((e) =>
        e.dial(l.key, l.relay ?? null),
      ),
      new Promise((_, n) => setTimeout(() => n(new Error('no answer')), PATIENCE)),
    ]),
  );
export const through = (group, path, body) =>
  wasm().then(async (w) => {
    const [d] = await reach(w, group, []),
      bytes = new TextEncoder().encode(JSON.stringify([group.id, body]));
    const [ok, status, text] = await d
      .fetch(
        'POST',
        path,
        [
          ['content-type', 'application/json'],
          ['content-length', String(bytes.length)],
        ],
        new Blob([bytes]).stream(),
      )
      .then(async (r) => [r.ok, r.status, await r.text()])
      .finally(() => d.close());
    return ok
      ? JSON.parse(text)
      : Promise.reject(
          (trace(`PAGE: ${path} answered ${status}: ${text}`), new Error(JSON.parse(text).error)),
        );
  });
export const io = {
  look: ({ fragment }, token = fragment.split('&')[0]) =>
    Promise.all([wasm(), store.groups()]).then(([w, kept]) => ({
      type: 'looked',
      fragment,
      kept,
      missing: document.body.id === 'missing',
      cold: w.is_label(store.labelOf(location.hostname)),
      linked: (() => {
        try {
          return Boolean(w.issuer_of(token));
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
                  new Promise((r) => (letGo = r)))
                : answer({ type: 'elsewhere', why: `${id} is open in another tab` }),
            )
            .catch((e) =>
              e.name === 'AbortError'
                ? (trace(`PAGE: another tab took ${id}, so this one opens again`),
                  location.reload())
                : failed(e),
            ),
        ),
  release: () => (letGo?.(), { type: 'released' }),
  // Asked on each open, never awaited, so a browser asking its person first holds nothing (app.js@19b63b2b:417-421).
  open: ({ group }) =>
    store.opened(group).then(
      async (g) => (
        store
          .persist()
          .catch((e) => `asking the browser to keep the page's storage failed: ${e}`)
          .then((l) => trace(`PAGE: ${l}`)),
        ([key, bound] = [await store.unseal(g), null]),
        { type: 'opened', group: g }
      ),
    ),
  // Each lend with its label, and whose lender `apart` says; and `me`, the
  // key this page holds as the record spells it, public, which the record's
  // rows name (app.js@19b63b2b:467), never the key itself.
  read: ({ group, n }) =>
    wasm()
      .then(async (w) => {
        const [d, via, res] = await lends(w, group, []),
          body = res.ok ? await res.json() : {},
          refusal = res.headers.get('x-svrngs-refused');
        const lent = (await d.close(), body.lent ?? []).map((l) => ({
            ...l,
            label: w.label(l.key, l.name),
          })),
          link = group.computers?.find((c) => c.key === w.issuer_of(group.token));
        apart(lent, body.people ?? []);
        return {
          type: 'read',
          n,
          ok: res.ok,
          refusal,
          via,
          link,
          body,
          lent,
          me: w.actor(key),
          older: res.ok && w.older(body.version),
          why: `${res.status}${refusal ? `, refused ${refusal}` : ''}, at version ${body.version}, ${lent.length} lent`,
        };
      })
      .catch((e) => ({ type: 'unread', n, said: e.said, why: String(e) })),
  keep: ({ group }) => store.opened(group).then(() => ({ type: 'kept' })),
};
