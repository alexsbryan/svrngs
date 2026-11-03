// SPDX-License-Identifier: AGPL-3.0-or-later
// "Already running here" (page-index (d); home.js `running`, :225-250):
// what runs on this computer and is not lent yet (/running, v1ui-lend
// (b)), looked for only when the person presses Look, as often as they
// like, never as the page loads: the look knocks at every address here,
// seconds of this computer's work (v1ui-lend-ask, the operator's "On
// request, with a button"). Its machine, RUNNING, is the place's own
// (ui/view.js `useMachine`), each look counted, so an older look's answer
// is refused by name. Each program found is a row with a machine of its
// own, LEND: a kind svrngs knows named with what it needs first, any other
// lent only once the person names it and says what it is; one button
// lends it, and the row then says what happened, or why not, each refusal
// beside the button as v1 appends it, kept there while it lends again. Drawn once the person is in a group.
import {
  Act,
  ask,
  assertNever,
  Field,
  go,
  html,
  late,
  machine,
  no,
  useMachine,
} from '../ui/kit.js?v=__VERSION__';
import { get, post } from './group.js?v=__VERSION__';
/** @import { Group, Home } from './group.js?v=__VERSION__' */
/** @import { Machine } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * How a kind svrngs's table knows signs each person in, and what its
 * lender does for it (share.rs `Signs`, serde's lowercase): it keeps its
 * own accounts, or takes the person from a header.
 * @typedef {{ accounts: string, header?: undefined } | { header: string, accounts?: undefined }} Signs
 */

/**
 * A program running here, as /running answers it (home.rs `shown`): its
 * port, the kind the table knows, its name once lent, how it signs each
 * person in, and the paths svrngs keeps back of it.
 * @typedef {object} Shown
 * @property {number} port
 * @property {string | null} kind
 * @property {string | null} name
 * @property {Signs | null} signs
 * @property {string[] | null} keeps
 */

/**
 * RUNNING's state: where its look is, what it found and the kinds the
 * table knows by name alone, or why it could not look.
 * @typedef {object} State
 * @property {{ running: 'unlooked' | 'looking' | 'looked' | 'unread' }} at
 * @property {Record<string, number>} [asked]
 * @property {Shown[]} [found]
 * @property {string[]} [named]
 * @property {string} [why]
 */

/**
 * The messages RUNNING takes: Look pressed; look `n` answered, or why not.
 * @typedef {{ type: 'look' }
 *   | { type: 'looked', n: number, found: Shown[], named: string[], why: string }
 *   | { type: 'unread', n: number, why: string }} Message
 */

/**
 * The effect its step answers: look `n`.
 * @typedef {{ run: 'look', n: number, failed: 'unread' }} Effect
 */

// What a lend needs first, as /running says from svrngs's table, in a
// sentence each: its own accounts, a setting, or what svrngs keeps back.
/** @param {Shown} r */
export const needs = (r) =>
  [
    r.signs?.accounts &&
      `It keeps its own accounts, so make each person one in it: ${r.signs.accounts}.`,
    r.signs?.header && `Before it knows who is who, set this in it: ${r.signs.header}.`,
    r.keeps?.length && `svrngs keeps these parts of it from everyone: ${r.keeps.join(', ')}.`,
  ]
    .filter(Boolean)
    .join(' ');

// Each look counted, an older look's answer refused by name.
/** @type {Machine<State, Message, Effect>} */
const RUNNING = {
  states: { running: ['unlooked', 'looking', 'looked', 'unread'] },
  io: {
    look: ({ n }) =>
      get('/running').then(({ running: found, named }) => ({
        type: 'looked',
        n,
        found,
        named,
        why: `${found.length} running`,
      })),
  },
  step: (s, m) => {
    const [asked, n] = ask(s, 'running');
    const old = m.type !== 'look' && late(s, 'running', m.n);
    if (old) return no(s, old);
    switch (m.type) {
      case 'look':
        return go(s, { running: 'looking' }, { asked }, { run: 'look', n, failed: 'unread' });
      case 'looked':
        return go(s, { running: 'looked' }, { found: m.found, named: m.named });
      case 'unread':
        return go(s, { running: 'unread' }, { why: m.why });
      default:
        return assertNever(s, m, 'what runs here');
    }
  },
};
// A second press while one lends is refused; what it is called is typed.
// Once lent, HOME reads the groups again, so the index lists it and "What
// you lend" with it (page-index (d)'s fix), where v1 redrew only the last.
/**
 * A program found's state: where its lend is, its port, its name and what
 * it is as typed, the refusals said beside its button, and the lend made.
 * @typedef {object} Lending
 * @property {{ lend: 'unlent' | 'lending' | 'lent' }} at
 * @property {Record<string, number>} [asked]
 * @property {number} port
 * @property {string} name
 * @property {string} kind
 * @property {string} said
 * @property {Shown} [l]
 */

/**
 * The messages a program found takes: its name or what it is typed; Lend
 * pressed; it lent; HOME told; or the lend refused, with why.
 * @typedef {{ type: 'typed', field: 'name' | 'kind', value: string }
 *   | { type: 'lend' }
 *   | { type: 'lent', l: Shown, why: string }
 *   | { type: 'told', why: string }
 *   | { type: 'refused', why: string }} Lends
 */

/**
 * The effects its step answers: the port lent, by the name and kind
 * typed, `null` where none is; and HOME told to read the groups again.
 * @typedef {{ run: 'lend', port: number, name: string | null, kind: string | null, failed: 'refused' }
 *   | { run: 'again', failed: 'told' }} Lent
 */

/**
 * @param {Home} home
 * @returns {Machine<Lending, Lends, Lent>}
 */
const LEND = (home) => ({
  states: { lend: ['unlent', 'lending', 'lent'] },
  io: {
    lend: ({ port, name, kind }) =>
      post('/running', { port, name, kind }).then((l) => ({ type: 'lent', l, why: l.name })),
    again: () => (home({ type: 'again' }), { type: 'told', why: 'the groups read again' }),
  },
  step: (s, m) => {
    switch (m.type) {
      case 'typed':
        return go(s, {}, { [m.field]: m.value });
      case 'lend':
        return s.at.lend === 'lending'
          ? no(s, 'a lend is under way, and is answered first')
          : go(
              s,
              { lend: 'lending' },
              {},
              {
                run: 'lend',
                port: s.port,
                name: s.name.trim() || null,
                kind: s.kind.trim() || null,
                failed: 'refused',
              },
            );
      case 'lent':
        return go(s, { lend: 'lent' }, { l: m.l }, { run: 'again', failed: 'told' });
      case 'told':
        return go(s, {});
      case 'refused':
        return go(s, { lend: 'unlent' }, { said: `${s.said} ${m.why}` });
      default:
        return assertNever(s, m, 'a program found');
    }
  },
});

/**
 * A program found: it, the kinds the table knows by name alone, and
 * HOME's `dispatch`, told once it is lent.
 * @typedef {{ r: Shown, named: string[], home: Home }} FoundProps
 */

/** @param {FoundProps} props */
function Found({ r, named, home }) {
  const [s, send] = useMachine(() =>
      machine('LEND', LEND(home), {
        at: { lend: 'unlent' },
        port: r.port,
        name: '',
        kind: '',
        said: '',
      }),
    ),
    at = `http://localhost:${r.port}`;
  // `s.l` is the lend 'lent' made, which set 'lent' with it.
  if (s.at.lend === 'lent')
    return html`<li>${`You lent ${/** @type {Shown} */ (s.l).name}, at ${at}: everyone in your groups can open it now. ${needs(/** @type {Shown} */ (s.l))} `}</li>`;
  return html`<li>${r.kind ? `${r.kind}, at ${at}. ${needs(r)} ` : `Something at ${at}. To lend it, give it a name and say what it is, each one word${named.length ? `; if it is ${named.join(' or ')}, say that` : ''}. `}${
    !r.kind &&
    /** @satisfies {[field: 'name' | 'kind', words: string][]} */ ([
      ['name', 'Its name: '],
      ['kind', 'What it is: '],
    ]).map(([n, w]) => html`<${Field} name=${n} words=${w} value=${s[n]} send=${send} />`)
  }<${Act} words=${`Lend ${r.kind ?? 'it'} to everyone in your groups`} doing=${{ state: s.at.lend === 'lending' ? 'busy' : 'idle' }} on=${{ type: 'lend' }} send=${send} />${s.said}</li>`;
}
// `s.found` is the look's, which set 'looked' with it.
/** @type {Record<State['at']['running'], (s: State) => string>} */
const SAID = {
  unlooked: () =>
    'This page lists the programs running on this computer only when you ask: looking takes it a few seconds of work.',
  looking: () => 'Looking for programs running on this computer…',
  unread: (s) => `You looked, and what runs on this computer cannot be read: ${s.why}`,
  looked: (s) =>
    /** @type {Shown[]} */ (s.found).length
      ? 'You looked just now: the programs running on this computer, each at its address. Lend one, and everyone in your groups opens it in their browser, as themselves, while this computer is on. You can stop lending it below at any time.'
      : 'You looked just now: nothing else on this computer is running at an address. Start a program, then look again.',
};
/**
 * "Already running here": HOME's `dispatch`, which each lend tells.
 * @typedef {{ home: Home }} RunningProps
 */

/** @param {RunningProps} props */
function Running({ home }) {
  const [s, send] = useMachine(() => machine('RUNNING', RUNNING, { at: { running: 'unlooked' } }));
  // A look again empties the list, and each row it finds is a new one (home.js@19b63b2b:237).
  // A look, which set 'looked', asked first and found.
  return html`<section id="running"><h2>Already running here</h2><p>${SAID[s.at.running](s)}</p><p><${Act} words="Look for programs running on this computer" doing=${{ state: s.at.running === 'looking' ? 'busy' : 'idle' }} on=${{ type: 'look' }} send=${send} /></p><ul>${s.at.running === 'looked' && /** @type {Shown[]} */ (s.found).map((r) => html`<${Found} key=${`${/** @type {Record<string, number>} */ (s.asked).running}:${r.port}`} r=${r} named=${s.named} home=${home} />`)}</ul></section>`;
}
// HOME draws its places once the groups are read (home.js `Home`).
/** @type {{ running: (props: HOME.PlaceProps) => Drawn }} */
export const view = {
  running: ({ s, home }) =>
    /** @type {Group[]} */ (s.groups).length > 0 && html`<${Running} home=${home} />`,
};
