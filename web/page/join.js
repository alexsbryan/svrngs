// SPDX-License-Identifier: AGPL-3.0-or-later
// Joining by a link no group kept here was joined by (DESIGN §3,
// "Joining"). The link's issuer is asked which group it lets into; a group
// held here already, by another link, is in at once; a link made for a
// person's devices joins as that person, asking no name; else the name is
// asked once, in #join (name-form.js), and asked again, saying why, when
// svrngs-core refuses it or someone in the group goes by it (409). Joined,
// the group is held as enter.js holds any (`hold`). The key is made, used
// and kept at the edge, never in state. A guest's name is asked in this
// form too, first (visit.js), with no Home Screen line, as a visit is kept
// apart from every group, and sent to the visit as `guest named`.
import { go, no } from '../ui/machine.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
import { AWAY, linkRefused, posted, readable, saying, trace, wasm } from './dial.js?v=__VERSION__';
import { NameForm } from './name-form.js?v=__VERSION__';
import { sending } from './page.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Wants } from './enter.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { At, Message as PageMessage, Props, Said, Send, State } from './page.js?v=__VERSION__' */

/**
 * The group a link lets into, as its issuer answered (svrngs-core
 * `believe`): a device link's person, whom the phone joins as.
 * @typedef {{ id: string, name: string, person?: string, maker: string, inviter?: string }} Letting
 */

/**
 * A link being joined by: its token, the groups kept when it was opened,
 * what the address asks, and once asked, its group, the issuer's answer
 * (`told`), and the group held here already that it lets into (`known`).
 * @typedef {{ token: string, kept: Group[], wants: Wants | null, native?: boolean, group?: Letting, told?: string, known?: Group }} Link
 */

/**
 * A link once its issuer answered which group it lets into (`asked`).
 * @typedef {Link & { group: Letting, told: string }} Told
 */

/**
 * The fields this flow owns (`owns`).
 * @typedef {object} Fields
 * @property {Link} [joining]
 * @property {string} [typed] the name typed
 * @property {string} [why] why the name was refused
 */

/**
 * The messages this flow takes.
 * @typedef {{ type: 'join', token: string, kept: Group[], wants: Wants | null, native?: boolean }
 *   | { type: 'asked', group: Letting, told: string }
 *   | { type: 'typed', field: 'name', value: string }
 *   | { type: 'named' }
 *   | { type: 'taken', why: string }
 *   | { type: 'admitted', group: Group }
 *   | { type: 'unjoined', said?: string, why: string }} Message
 */

/**
 * The effects its steps answer: the issuer asked which group the link
 * lets into, and the name redeemed there.
 * @typedef {{ run: 'ask', token: string, native?: boolean, failed: string }
 *   | { run: 'admit', token: string, told: string, group: Letting, name: string, failed: string }} Effect
 */

// `joining`: the link's token, the groups kept when it was opened, what the
// address asks, and once asked, its group and the issuer's answer (`told`).
export const owns = ['joining', 'typed', 'why'];
export const states = /** @type {const} */ ({
  join: ['unasked', 'asking', 'naming', 'installing', 'redeeming', 'taken', 'joined'],
});
/**
 * @param {State} state
 * @param {Told} joining
 * @param {string} name
 */
const redeem = (state, joining, name) =>
  go(
    state,
    { join: 'redeeming' },
    { joining },
    /** @satisfies {Effect} */ ({
      run: 'admit',
      token: joining.token,
      told: joining.told,
      group: joining.group,
      name,
      failed: 'unjoined',
    }),
  );
const asks = (/** @type {State} */ state) => ['naming', 'taken'].includes(state.at.join);
const guest = (/** @type {State} */ state) => state.at.visit === 'naming';
/** @type {Takes<State, Message, Effect | Send>} */
export const takes = {
  join: (state, { token, kept, wants, native }) => {
    if (state.at.join !== 'unasked') return no(state, `a link is ${state.at.join} already`);
    const joining = { token, kept, wants, native };
    return go(
      state,
      { join: 'asking' },
      { joining },
      { run: 'ask', token, native, failed: 'unjoined' },
    );
  },
  asked: (state, { group, told }) => {
    if (state.at.join !== 'asking') return no(state, `no link is asked of, it is ${state.at.join}`);
    // Asking, so `join` set the link.
    const link = /** @type {Link} */ (state.joining);
    const known = link.kept.find((one) => one.id === group.id);
    const joining = { ...link, group, told, known };
    if (link.native) return go(state, { join: 'naming' }, { joining });
    if (known) {
      const why = `this phone is in ${group.id} already, by another link`;
      const held = sending('hold', { group: known, wants: joining.wants, joined: true, why });
      return go(state, { join: 'joined' }, { joining }, held);
    }
    if (group.person) return redeem(state, joining, group.person);
    return go(state, { join: 'naming' }, { joining });
  },
  'typed name': (state, { value }) => {
    if (!guest(state) && !asks(state)) {
      return no(state, `no name is asked: the link is ${state.at.join}`);
    }
    return go(state, {}, { typed: value });
  },
  named: (state) => {
    if (guest(state))
      return go(state, {}, {}, sending('guest named', { guest: state.typed ?? '' }));
    if (asks(state) && state.joining?.native) {
      const { token, group } = /** @type {Told} */ (state.joining);
      return go(
        state,
        { join: 'installing' },
        {},
        sending('offer', {
          token,
          name: state.typed ?? '',
          destination: `/#group-${group.id}`,
        }),
      );
    }
    // A name is asked only of a link its issuer answered (`asked`).
    if (asks(state)) return redeem(state, /** @type {Told} */ (state.joining), state.typed ?? '');
    return no(state, `no name is asked: the link is ${state.at.join}`);
  },
  taken: (state, { why }) => {
    if (state.at.join !== 'redeeming') return no(state, 'no name was sent');
    return go(state, { join: 'taken' }, { why });
  },
  admitted: (state, { group }) => {
    if (state.at.join !== 'redeeming') return no(state, 'no name was sent');
    // Redeeming, a link is being joined by (`redeem`).
    const { wants } = /** @type {Link} */ (state.joining);
    return go(state, { join: 'joined' }, {}, sending('hold', { group, wants, joined: true }));
  },
  // Why it did not join, said as the page's failure, in words, so the
  // person reads it where every failure is said.
  unjoined: (state, { said, why }) =>
    go(state, { join: 'unasked' }, {}, sending('failed', { said, why })),
};
// Joined, its last words stand until the group is read, so the heading
// does not go blank between joining and the group's first read. Joined, a
// link was joined by (`asked`, `admitted`).
const said = (/** @type {State} */ state) => {
  if (state.at.join !== 'joined' || state.at.list !== 'unasked') return state.at.join;
  return /** @type {Link} */ (state.joining).known ? 'asking' : 'redeeming';
};
/** @type {Record<string, (s: State) => Said | undefined>} */
export const heading = {
  // A state not named here says nothing.
  join: (state) => {
    const group = state.joining?.group;
    const name = group?.name;
    const redeeming = group?.person
      ? `Adding this phone to ${group.name} as ${group.person}’s…`
      : `Joining ${name}…`;
    /** @type {Partial<Record<At['join'], Said>>} */
    const headings = {
      asking: ['Opening your link…'],
      naming: [state.joining?.native ? `${group?.inviter} invited you to ${name}` : `Join ${name}`],
      installing: [`Install svrngs to join ${name}`],
      taken: [`Join ${name}`],
      redeeming: [redeeming],
    };
    return headings[said(state)];
  },
};

// NameForm is called as a function, never drawn as a component
// (name-form.js says why).
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  join: ({ s: state, send }) => {
    /** @type {PageMessage} */
    const on = { type: 'named' };
    if (guest(state)) {
      const group = 'the group you’re visiting';
      return NameForm({ shown: true, group, value: state.typed, send, on });
    }
    return NameForm({
      shown: asks(state),
      group: state.joining?.group?.name,
      keep: !state.joining?.native && state.standalone === false,
      native: state.joining?.native,
      why: state.why,
      value: state.typed,
      send,
      on,
    });
  },
};

// The edge: the key made fresh for this link, the issuer dialled as it
// (`connect_to_join`) within dial.js's PATIENCE, and the ask and the
// redemption posted; the joined group kept, its key sealed (store.js
// `keep`).
// Made fresh by `ask`, which comes before every `admit` (`asked`).
/** @type {Uint8Array<ArrayBuffer>} */
let key;
const unjoined = (/** @type {Error & { said?: string }} */ error) =>
  /** @satisfies {PageMessage} */ ({
    type: 'unjoined',
    said: error.said,
    why: String(error),
  });
/**
 * The issuer of `token` asked which group it lets into, as a key made
 * fresh for this link.
 * @param {string} token
 * @param {boolean} [native]
 * @returns {Promise<PageMessage>}
 */
async function asking(token, native) {
  const core = await wasm();
  readable(core, token);
  key = crypto.getRandomValues(new Uint8Array(32));
  const reply = await posted(core, key, token, {
    path: native ? '/ask?native=true' : '/ask',
    body: core.ask(key, token),
  });
  const welcome = native ? JSON.parse(reply) : null;
  const told = native ? JSON.stringify(welcome.asked) : reply;
  if (native && typeof welcome.inviter !== 'string') throw saying(AWAY);
  const group = core.believe(told);
  // An older svrngs made the group: nobody joins it now, said in its words.
  if (group.older) throw saying(group.older);
  const devices = group.person ? `, for ${group.person}’s devices` : '';
  return {
    type: 'asked',
    told,
    group: {
      id: group.id,
      name: group.name,
      person: group.person,
      maker: group.maker,
      inviter: welcome?.inviter,
    },
    why: `the link lets into ${group.id}${devices}`,
  };
}
/**
 * What a 409 says: someone in the group goes by the name (bash-person), so
 * another is asked, saying whose it is and how to add this phone if it is
 * theirs.
 * @param {Letting} group
 * @param {number} status
 * @param {string} text
 */
const goesBy = (group, status, text) => {
  if (status !== 409) return linkRefused(status);
  const name = JSON.parse(text).error;
  return `Someone in ${group.name} already goes by ${name}. If that’s you, open your home page on your computer, press “Add your phone or another computer”, and open that link here. Else join under another name.`;
};
/**
 * `name` redeemed at the issuer, and the group it lets into kept here.
 * @param {Extract<Effect, { run: 'admit' }>} effect
 * @returns {Promise<PageMessage>}
 */
async function admitting({ token, told, group, name }) {
  const core = await wasm();
  let body;
  try {
    body = core.redemption(key, token, told, name);
    // svrngs-core's refusal, thrown by the wasm as an `Error` with its words.
  } catch (/** @type {any} */ error) {
    return { type: 'taken', why: `${error.message[0].toUpperCase()}${error.message.slice(1)}.` };
  }
  await posted(core, key, token, {
    path: '/redeem',
    body,
    refused: (status, text) => goesBy(group, status, text),
  });
  const person = JSON.parse(body).person;
  await store.keep({
    id: group.id,
    name: group.name,
    person,
    token,
    key,
    maker: group.maker,
    since: Math.floor(Date.now() / 1000),
  });
  trace(`PAGE: joined ${group.id} as ${person}; the key is kept sealed`);
  // Kept just above, so found.
  const kept = /** @type {Group} */ ((await store.groups()).find((one) => one.id === group.id));
  return { type: 'admitted', group: kept };
}
/** @type {Io<Effect, PageMessage>} */
export const io = {
  ask: ({ token, native }) => asking(token, native).catch(unjoined),
  // A name someone goes by (409) is asked again; any other refusal fails.
  admit: (effect) =>
    admitting(effect).catch((error) =>
      error.status === 409 ? { type: 'taken', why: error.said } : unjoined(error),
    ),
};
