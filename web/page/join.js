// SPDX-License-Identifier: AGPL-3.0-or-later
// Joining by a link no group kept here was joined by (page-join (a);
// web/app.js `join`, `askName`, :120-196; DESIGN §3, "Joining"). The link's
// issuer is asked which group it lets into; a group held here already, by
// another link, is in at once; a link made for a person's devices joins as
// that person, asking no name; else the name is asked once, in #join, and
// asked again, saying why, when svrngs-core refuses it or someone in the
// group goes by it (409). Joined, the group is held as enter.js holds any
// (`hold`). The key is made, used and kept at the edge, never in state.
// A guest's name is asked in this form too, first (web/page/visit.js), with
// no Home Screen line, as a visit is kept apart from every group, and sent
// to the visit as `guest named`.
import { html } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { Field } from '../ui/parts.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';
/** @import { Group } from '../store.js?v=__VERSION__' */
/** @import { Io, Takes } from '../ui/machine.js?v=__VERSION__' */
/** @import * as Wasm from '../wasm/svrngs_web.js?v=__VERSION__' */
/** @import { Wants } from './enter.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { At, Message as PageMessage, Props, Said, Send, Sender, State } from './page.js?v=__VERSION__' */

/**
 * The group a link lets into, as its issuer answered (svrngs-core
 * `believe`): a device link's person, whom the phone joins as.
 * @typedef {{ id: string, name: string, person?: string, maker: string }} Letting
 */

/**
 * A link being joined by: its token, the groups kept when it was opened,
 * what the address asks, and once asked, its group, the issuer's answer
 * (`told`), and the group held here already that it lets into (`known`).
 * @typedef {{ token: string, kept: Group[], wants: Wants | null, group?: Letting, told?: string, known?: Group }} Link
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
 * @typedef {{ type: 'join', token: string, kept: Group[], wants: Wants | null }
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
 * @typedef {{ run: 'ask', token: string, failed: string }
 *   | { run: 'admit', token: string, told: string, group: Letting, name: string, failed: string }} Effect
 */

// `joining`: the link's token, the groups kept when it was opened, what the
// address asks, and once asked, its group and the issuer's answer (`told`).
export const owns = ['joining', 'typed', 'why'];
export const states = /** @type {const} */ ({
  join: ['unasked', 'asking', 'naming', 'redeeming', 'taken', 'joined'],
});
const sending = /** @type {Sender} */ (
  (type, message) => ({ run: 'send', m: { type, ...message } })
);
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
  join: (state, { token, kept, wants }) =>
    state.at.join === 'unasked'
      ? go(
          state,
          { join: 'asking' },
          { joining: { token, kept, wants } },
          { run: 'ask', token, failed: 'unjoined' },
        )
      : no(state, `a link is ${state.at.join} already`),
  asked: (
    state,
    { group, told },
    known = state.joining?.kept.find((one) => one.id === group.id),
    // Read only while asking, once `join` set the link.
    joining = { .../** @type {Link} */ (state.joining), group, told, known },
  ) => {
    if (state.at.join !== 'asking') return no(state, `no link is asked of, it is ${state.at.join}`);
    if (known)
      return go(
        state,
        { join: 'joined' },
        { joining },
        sending('hold', {
          group: known,
          wants: joining.wants,
          joined: true,
          why: `this phone is in ${group.id} already, by another link`,
        }),
      );
    if (group.person) return redeem(state, joining, group.person);
    return go(state, { join: 'naming' }, { joining });
  },
  'typed name': (state, { value }) =>
    guest(state) || asks(state)
      ? go(state, {}, { typed: value })
      : no(state, `no name is asked: the link is ${state.at.join}`),
  named: (state) => {
    if (guest(state))
      return go(state, {}, {}, sending('guest named', { guest: state.typed ?? '' }));
    // A name is asked only of a link its issuer answered (`asked`).
    if (asks(state)) return redeem(state, /** @type {Told} */ (state.joining), state.typed ?? '');
    return no(state, `no name is asked: the link is ${state.at.join}`);
  },
  taken: (state, { why }) =>
    state.at.join === 'redeeming'
      ? go(state, { join: 'taken' }, { why })
      : no(state, 'no name was sent'),
  admitted: (state, { group }) =>
    state.at.join === 'redeeming'
      ? // Redeeming, a link is being joined by (`redeem`).
        go(
          state,
          { join: 'joined' },
          {},
          sending('hold', {
            group,
            wants: /** @type {Link} */ (state.joining).wants,
            joined: true,
          }),
        )
      : no(state, 'no name was sent'),
  // Why it did not join, said as the page's failure, in words (app.js@19b63b2b:78).
  unjoined: (state, { said, why }) =>
    go(state, { join: 'unasked' }, {}, sending('failed', { said, why })),
};
// Joined, its last words stand until the group is read (app.js@19b63b2b:162, :187).
// Joined, a link was joined by (`asked`, `admitted`).
const said = (/** @type {State} */ state) => {
  if (state.at.join !== 'joined' || state.at.list !== 'unasked') return state.at.join;
  return /** @type {Link} */ (state.joining).known ? 'asking' : 'redeeming';
};
/** @type {Record<string, (s: State) => Said | undefined>} */
export const heading = {
  // A state not named here says nothing.
  join: (state, group = state.joining?.group) =>
    /** @type {Partial<Record<At['join'], Said>>} */ ({
      asking: ['Opening your link…'],
      naming: [`Join ${group?.name}`],
      taken: [`Join ${group?.name}`],
      redeeming: [
        group?.person
          ? `Adding this phone to ${group.name} as ${group.person}’s…`
          : `Joining ${group?.name}…`,
      ],
    })[said(state)],
};

// The name asked, in the group's name. In iPhone Safari, the one browser
// defining `navigator.standalone`, false outside a Home Screen web app, the
// person is first told to join from the Home Screen: each web app there
// keeps a storage of its own, so one added after joining joins again, as
// someone new (the operator's iPhone, iOS 18.7, 2026-10-30). The form is
// drawn again each time it is shown, so its box takes the focus each time
// it is asked, as v1's `input.focus()` (app.js@19b63b2b:136). Pure, called as a function,
// so it is drawn as web/index.html@19b63b2b:70-76 has it (web/tests/pages/frame.test.mjs).
const ATTRS = { autocomplete: 'name', maxlength: 64, required: true };
/**
 * The name form's props: whether it is shown, the group named in it,
 * whether to say to join from the Home Screen first, why the last name was
 * refused, the name typed, and the message its Join sends.
 * @typedef {object} NameFormProps
 * @property {boolean} shown
 * @property {string} [group]
 * @property {boolean} [keep]
 * @property {string} [why]
 * @property {string} [value]
 * @property {Props['send']} send
 * @property {PageMessage} on
 */
/** @param {NameFormProps} props */
export const NameForm = ({ shown, group = '', keep, why, value, send, on }) =>
  html`<form key=${`${shown} ${why}`} id="join" hidden=${!shown} onSubmit=${(
    /** @type {Event} */ event,
  ) => {
    event.preventDefault();
    return send(on);
  }}><p id="keep" hidden=${!keep}>${keep && `To keep ${group} on this phone, add this page to your Home Screen first, and join from there.`}</p>${Field({ shape: 'for', id: 'name', words: html`What’s your name? Everyone in <span id="group">${group}</span> sees it.`, value, attrs: ATTRS, send })}<p id="why" class="bad">${why}</p><button>Join</button></form>`;
/** @type {Record<string, (props: Props) => Drawn>} */
export const view = {
  join: ({ s: state, send }) =>
    NameForm({
      value: state.typed,
      send,
      on: { type: 'named' },
      ...(guest(state)
        ? { shown: true, group: 'the group you’re visiting' }
        : {
            shown: asks(state),
            group: state.joining?.group?.name,
            keep: state.standalone === false,
            why: state.why,
          }),
    }),
};

// The edge: the key made fresh for this link, the issuer dialled as it
// (`connect_to_join`) within PATIENCE, and the ask and the redemption
// posted; the joined group kept, its key sealed (store.js `keep`).
const PATIENCE = 60_000;
const AWAY = 'The computer that sent this link did not answer. Try again later.';
const wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__');
const trace = (/** @type {string} */ line) => globalThis.__trace?.(line);
/**
 * @param {string} words
 * @param {number} [status]
 */
const refusal = (words, status) => Object.assign(new Error(words), { said: words, status });
/** @param {number} status */
const joining = (status) =>
  status === 403
    ? 'This link lets nobody in. Ask for a new one.'
    : 'The computer that sent this link cannot let anyone in just now. Try again later.';
// Made fresh by `ask`, which comes before every `admit` (`asked`).
/** @type {Uint8Array<ArrayBuffer>} */
let key;
/**
 * `body` posted at `path` to the issuer of `token`, its answer's text, or
 * the refusal `refused` words for its status.
 * @param {typeof Wasm} core
 * @param {string} token
 * @param {{ path: string, body: string }} request
 * @param {(status: number, text: string) => string} [refused]
 * @returns {Promise<string>}
 */
async function posted(core, token, { path, body }, refused = joining) {
  const dialled = await Promise.race([
    core.connect_to_join(key, token),
    /** @type {Promise<never>} */ (
      new Promise((_, reject) => setTimeout(() => reject(new Error('no answer')), PATIENCE))
    ),
  ]).catch((error) => Promise.reject((trace(`PAGE: the dial failed: ${error}`), refusal(AWAY))));
  const bytes = new TextEncoder().encode(body);
  const [res, text] = await dialled
    .fetch(
      'POST',
      path,
      [
        ['content-type', 'application/json'],
        ['content-length', String(bytes.length)],
      ],
      new Blob([bytes]).stream(),
    )
    // The wasm's `fetch` answers the page's `Response` (svrngs_web.d.ts `Dialled`).
    .then(
      async (/** @type {Response} */ response) =>
        /** @satisfies {[Response, string]} */ ([response, await response.text()]),
    )
    .finally(() => dialled.close());
  return res.ok
    ? text
    : Promise.reject(
        (trace(`PAGE: ${path} answered ${res.status}: ${text}`),
        refusal(refused(res.status, text), res.status)),
      );
}
const unjoined = (/** @type {Error & { said?: string }} */ error) =>
  /** @satisfies {PageMessage} */ ({
    type: 'unjoined',
    said: error.said,
    why: String(error),
  });
/** @type {Io<Effect, PageMessage>} */
export const io = {
  ask: ({ token }) =>
    wasm()
      .then(async (core) => {
        if (!core.relay_of(token))
          throw refusal(
            'This link was made by an older svrngs. Ask whoever sent it for a new one.',
          );
        key = crypto.getRandomValues(new Uint8Array(32));
        const told = await posted(core, token, { path: '/ask', body: core.ask(key, token) });
        const group = core.believe(told);
        return /** @satisfies {PageMessage} */ ({
          type: 'asked',
          told,
          group: { id: group.id, name: group.name, person: group.person, maker: group.maker },
          why: `the link lets into ${group.id}${group.person ? `, for ${group.person}’s devices` : ''}`,
        });
      })
      .catch(unjoined),
  // 409: someone in the group goes by the name (bash-person), so another is
  // asked, saying whose it is and how to add this phone if it is theirs.
  admit: ({ token, told, group, name }) =>
    wasm()
      .then(async (core) => {
        let body;
        try {
          body = core.redemption(key, token, told, name);
          // svrngs-core's refusal, thrown by the wasm as an `Error` with its words.
        } catch (/** @type {any} */ error) {
          return /** @satisfies {PageMessage} */ ({
            type: 'taken',
            why: `${error.message[0].toUpperCase()}${error.message.slice(1)}.`,
          });
        }
        await posted(core, token, { path: '/redeem', body }, (status, text) =>
          status === 409
            ? `Someone in ${group.name} already goes by ${JSON.parse(text).error}. If that’s you, open your home page on your computer, press “Add your phone or another computer”, and open that link here. Else join under another name.`
            : joining(status),
        );
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
        return /** @satisfies {PageMessage} */ ({
          type: 'admitted',
          group: /** @type {Group} */ ((await store.groups()).find((one) => one.id === group.id)),
        });
      })
      .catch((error) =>
        error.status === 409 ? { type: 'taken', why: error.said } : unjoined(error),
      ),
};
