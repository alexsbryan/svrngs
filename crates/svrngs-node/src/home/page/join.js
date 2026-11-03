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
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { ask, assertNever, go, late, machine, no } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
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
const FIRST = { at: { join: 'typing' }, link: '', person: '', for: undefined, why: '' };
const who = /** @param {State} state */ (state) => state.for?.[1] ?? state.person.trim();
/**
 * @param {HOME.PlaceProps['home']} home
 * @returns {Machine<State, Message, Effect>}
 */
export const JOIN = (home) => ({
  states: { join: ['typing', 'checking', 'checked', 'refused', 'joining'] },
  io: {
    check: ({ link, n: serial }) =>
      get(`/join?${new URLSearchParams({ link })}`).then((answer) => ({
        type: 'checked',
        n: serial,
        for: answer,
        why: answer[0],
      })),
    join: ({ link, person }) =>
      post('/join', { link, person }).then(([joined, joinedAs, unplaced]) => {
        home({ type: 'joined', flow: 'join', q: { joined, person: joinedAs, unplaced } });
        home({ type: 'again' });
        return { type: 'joined', why: 'said, the groups read again' };
      }),
  },
  // An answer to an ask older than the last is refused by name, before any
  // case takes it.
  step: (state, message) => {
    const old =
      message.type === 'checked' || message.type === 'refused'
        ? late(state, 'link', message.n)
        : '';
    if (old) return no(state, old);
    switch (message.type) {
      case 'typed':
        return message.field === 'person'
          ? go(state, {}, { person: message.value })
          : ((asked, serial) =>
              go(
                state,
                { join: 'checking' },
                {
                  link: message.value,
                  asked,
                  why: 'Asking the computer that sent it what it is for…',
                },
                { run: 'check', link: message.value, n: serial, failed: 'refused' },
              ))(...ask(state, 'link'));
      case 'checked':
        return go(state, { join: 'checked' }, { for: message.for, why: '' });
      case 'refused':
        return go(state, { join: 'refused' }, { for: undefined, why: message.why });
      case 'join':
        if (state.at.join === 'joining')
          return no(state, 'a join is under way, and is answered first');
        if (!state.for) return no(state, 'no link is asked of yet');
        return go(
          state,
          { join: 'joining' },
          {},
          { run: 'join', link: state.link, person: who(state), failed: 'unjoined' },
        );
      case 'joined':
        return go(state, FIRST.at, FIRST);
      case 'unjoined':
        return go(state, { join: 'refused' }, { why: message.why });
      default:
        return assertNever(state, message, 'joining');
    }
  },
});
/**
 * @param {HOME.State} state
 * @param {Taken} message
 * @returns {Step<HOME.State, HOME.Effect>}
 */
export const take = (state, message) => go(state, { said: 'joined' }, { q: message.q });

/** @param {HOME.PlaceProps} props */
const Join = ({ home }) =>
  ((
    state,
    send,
    typed = /** @param {{ currentTarget: HTMLInputElement }} typing */ (typing) =>
      send({ type: 'typed', field: 'person', value: typing.currentTarget.value }),
  ) =>
    html`<section id="join"><h2>Join a group from a link</h2><p><${Field} name="link" words="The link someone sent you: " value=${state.link} send=${send} /><label hidden=${Boolean(state.for?.[1])}>${'Your name in it: '}<input name="person" value=${state.person} onInput=${typed} /></label>${state.for && html`<${Act} words=${`Join ${state.for[0]}${who(state) ? ` as ${who(state)}` : ''}`} doing=${{ state: state.at.join === 'joining' ? 'busy' : 'idle' }} on=${{ type: 'join' }} send=${send} />`}</p><p>${state.why}</p></section>`)(
    ...useMachine(() => machine('JOIN', JOIN(home), FIRST)),
  );
/** @type {{ join: (props: HOME.PlaceProps) => Drawn }} */
export const view = { join: ({ home }) => html`<${Join} home=${home} />` };
