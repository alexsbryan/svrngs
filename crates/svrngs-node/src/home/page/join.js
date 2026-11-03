// SPDX-License-Identifier: AGPL-3.0-or-later
// "Join a group from a link" (page-join (c); home.js@19b63b2b:404-414): a group
// joined from a link someone sent, by the svrngs running here as `svrngs
// join` joins (arch-command (c)). Its machine, JOIN, is the place's own
// (ui/view.js `useMachine`). Each link typed is asked of the computer that
// sent it, what it is for, each ask counted, so an older answer is refused
// by name (`late`), where v1 compared the typed value (home.js@19b63b2b:411); the answer
// is said on the button before it is pressed, and the name box hidden when
// the link names whom it joins as. Pressed, a second press is refused while
// it joins; joined, HOME says so (`take`) and reads the groups again, where
// v1 opened the page again (home.js@19b63b2b:409; page-join (c)'s fix), and the form is
// empty again; a refusal is said below, in place of the last.
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
/** @import { Machine, Step } from '../ui/machine.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/**
 * HOME's message this file takes: a group joined, by its name, as whom,
 * `null` when it was in already, and why its folder could not be made,
 * `null` when it was (home.rs `join`).
 * @typedef {{ type: 'joined', flow: 'join', q: { joined: string, person: string | null, unplaced: string | null } }} Taken
 */

/**
 * What a link is for, as the computer that sent it says (home.rs
 * `asked`): the group's name, and whom it joins as, `null` when the
 * person names themselves.
 * @typedef {[group: string, person: string | null]} For
 */

/**
 * The form's state: where its join is, the link and name typed, what the
 * link is for once asked, and what is said below.
 * @typedef {object} State
 * @property {{ join: 'typing' | 'checking' | 'checked' | 'refused' | 'joining' }} at
 * @property {Record<string, number>} [asked]
 * @property {string} link
 * @property {string} person
 * @property {For} [for]
 * @property {string} why
 */

/**
 * The messages the form takes: a link or a name typed; ask `n` of the
 * link answered, or refused, with why; Join pressed; the group joined, or
 * not, with why.
 * @typedef {{ type: 'typed', field: 'link' | 'person', value: string }
 *   | { type: 'checked', n: number, for: For, why: string }
 *   | { type: 'refused', n: number, why: string }
 *   | { type: 'join' }
 *   | { type: 'joined', why: string }
 *   | { type: 'unjoined', why: string }} Message
 */

/**
 * The effects its step answers: ask `n` of what `link` is for, and the
 * group joined from it.
 * @typedef {{ run: 'check', link: string, n: number, failed: 'refused' }
 *   | { run: 'join', link: string, person: string, failed: 'unjoined' }} Effect
 */

/** @type {State} */
const FIRST = { at: { join: 'typing' }, link: '', person: '', for: undefined, why: '' },
  who = /** @param {State} s */ (s) => s.for?.[1] ?? s.person.trim();
/**
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
export const JOIN = (home) => ({
  states: { join: ['typing', 'checking', 'checked', 'refused', 'joining'] },
  io: {
    check: ({ link, n }) =>
      get(`/join?${new URLSearchParams({ link })}`).then((a) => ({
        type: 'checked',
        n,
        for: a,
        why: a[0],
      })),
    join: ({ link, person }) =>
      post('/join', { link, person }).then(
        ([joined, person, unplaced]) => (
          home({ type: 'joined', flow: 'join', q: { joined, person, unplaced } }),
          home({ type: 'again' }),
          { type: 'joined', why: 'said, the groups read again' }
        ),
      ),
  },
  // An answer to an ask older than the last is refused by name, before any
  // case takes it.
  step: (s, m) => {
    const old = (m.type === 'checked' || m.type === 'refused') && late(s, 'link', m.n);
    if (old) return no(s, old);
    switch (m.type) {
      case 'typed':
        return m.field === 'person'
          ? go(s, {}, { person: m.value })
          : ((asked, n) =>
              go(
                s,
                { join: 'checking' },
                {
                  link: m.value,
                  asked,
                  why: 'Asking the computer that sent it what it is for…',
                },
                { run: 'check', link: m.value, n, failed: 'refused' },
              ))(...ask(s, 'link'));
      case 'checked':
        return go(s, { join: 'checked' }, { for: m.for, why: '' });
      case 'refused':
        return go(s, { join: 'refused' }, { for: undefined, why: m.why });
      case 'join':
        return s.at.join === 'joining'
          ? no(s, 'a join is under way, and is answered first')
          : s.for
            ? go(
                s,
                { join: 'joining' },
                {},
                { run: 'join', link: s.link, person: who(s), failed: 'unjoined' },
              )
            : no(s, 'no link is asked of yet');
      case 'joined':
        return go(s, FIRST.at, FIRST);
      case 'unjoined':
        return go(s, { join: 'refused' }, { why: m.why });
      default:
        return assertNever(s, m, 'joining');
    }
  },
});
/**
 * @param {HOME.State} s
 * @param {Taken} m
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (s, m) => go(s, { said: 'joined' }, { q: m.q });

/** @param {HOME.PlaceProps} props */
const Join = ({ home }) =>
  ((
    s,
    send,
    typed = /** @param {{ currentTarget: HTMLInputElement }} e */ (e) =>
      send({ type: 'typed', field: 'person', value: e.currentTarget.value }),
  ) =>
    html`<section id="join"><h2>Join a group from a link</h2><p><${Field} name="link" words="The link someone sent you: " value=${s.link} send=${send} /><label hidden=${Boolean(s.for?.[1])}>${'Your name in it: '}<input name="person" value=${s.person} onInput=${typed} /></label>${s.for && html`<${Act} words=${`Join ${s.for[0]}${who(s) ? ` as ${who(s)}` : ''}`} doing=${{ state: s.at.join === 'joining' ? 'busy' : 'idle' }} on=${{ type: 'join' }} send=${send} />`}</p><p>${s.why}</p></section>`)(
    ...useMachine(() => machine('JOIN', JOIN(home), FIRST)),
  );
/** @type {{ join: (props: HOME.PlaceProps) => Drawn }} */
export const view = { join: ({ home }) => html`<${Join} home=${home} />` };
