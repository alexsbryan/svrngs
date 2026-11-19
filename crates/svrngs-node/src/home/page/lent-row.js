// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend in "What you lend", one line: how many it lets in at once and how
// many use it now, and what it is lent to, paused when empty and needing
// selection when legacy (`null`). Folded under that line, since five lends'
// forms were two screens before anything was asked, and open at first only
// where apps cannot read it until its lender does something (`undone`): who
// can open it, one box a group of HOME's, each by its stable id, with a
// button for each change (lend-change.js CHANGE), and the app form's fill
// once signing people in is turned on (`SigningOn`).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { CHANGE } from './lend-change.js?v=__VERSION__';
import { GroupChoice, called } from './group-choice.js?v=__VERSION__';
import { notRunning } from '../question.js?v=__VERSION__';
/** @import { Lend } from '../question.js?v=__VERSION__' */
/** @import { AppsAccount } from './lent-read.js?v=__VERSION__' */
/** @import { State, Message } from './lend-change.js?v=__VERSION__' */
/** @import { Choice } from './group-choice.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/**
 * A lend's row: the lend, whether it signs people in with svrngs, HOME's
 * groups to lend it to, and HOME's `dispatch`, told to read again once it
 * changes.
 * @typedef {object} LentProps
 * @property {Lend} l
 * @property {boolean} signs
 * @property {AppsAccount} [account]
 * @property {string[]} [headers]
 * @property {Choice[]} groups
 * @property {HOME.PlaceProps['home']} home
 */

/** @import { Signed } from './lend-change.js?v=__VERSION__' */

// What a lend's app form is filled with, the one time its lender turns
// signing people in with svrngs on: the issuer, the client and its secret,
// and the deviations in their words. The secret is shown here once and
// nowhere else, never in a request, a link or a trace line
// (SIGN_IN_WITH_SVRNGS §5 "Turning it on"): a read again keeps its row, so
// keeps it shown, and a load of the page, which reads none, shows it no
// more.
/** @param {{ c: Signed }} props */
function SigningOn({ c: fill }) {
  const after = `. ${fill.deviations} Copy the secret now: it is not shown again. `;
  return html`<p>Sign-in credentials created; app setup still required. Follow the app’s OpenID Connect setup guide: install any required OpenID Connect plugin, then configure it with issuer <code>${fill.issuer}</code>, client id <code>${fill.client_id}</code>, client secret <code>${fill.client_secret}</code>. Reach the issuer from the app server on this computer, not the computer of the person opening the app; a container has its own loopback. Test sign-in through the group’s app list after setup. <a href="https://github.com/alexsbryan/svrngs/tree/main/apps">App setup guides</a>${after}</p>`;
}

// What the row says it is lent to: the names of the groups it holds, none
// when paused, and needing selection when the answer gave none.
/**
 * @param {Lend} lend
 * @param {Choice[]} groups
 */
export const lentTo = (lend, groups) => {
  // Absent is read as legacy, needing selection: never as every group.
  const ids = lend.groups ?? null;
  if (ids === null) return 'Choose which groups can open this.';
  if (ids.length === 0) return 'Paused: no group can open it.';
  // Every id named, one this computer holds no group for by its start.
  const names = ids.map((id) => {
    const group = groups.find((held) => held.id === id);
    return group ? called(group) : `a group this computer does not hold (${id.slice(0, 8)})`;
  });
  return `Lent to ${names.join(' and ')}.`;
};

/**
 * @typedef {{ name: string, account: AppsAccount, state: State, doing: { state: string }, send: (message: Message) => void }} AppsProps
 */

// What a password box hides and the browser never offers to fill.
const HIDDEN = { type: 'password', autocomplete: 'off' };

/**
 * How apps read a lend through its lender: whether an account or key is
 * made, how to make it, and its name and password, or the key its lender
 * makes in it (home.rs `readers`, `change`, `lent`; share.rs `key_of`),
 * typed here once, sent to this computer, and never drawn back.
 * @param {Omit<AppsProps, 'account'> & { account?: AppsAccount }} props
 */
function Apps({ name, account, state, doing, send }) {
  if (!account) return null;
  if (account.why) return html`<p>${account.why}</p>`;
  const what = account.key ? 'key' : 'account';
  const [typed, password] = [state.account ?? '', state.password ?? ''];
  const { make, how, field, to } = READS[what](name, typed, password);
  const said = account.kept
    ? `Apps your friends open can read your ${name} with the ${what} you made. They can’t change anything.`
    : `Apps can’t read your ${name} yet. To let them, ${make}.`;
  return html`
    <p>${said}</p>
    <details>
      <summary>${account.kept ? `Use a different ${what}` : `How to make the ${what}`}</summary>
      <p>In ${name}${account.by}: ${account.signs?.accounts}. ${how}</p>
      ${what === 'account' && html`<${Field} words="Account name: " name="account" value=${typed} send=${send} />`}
      <${Field} words=${field} name="password" value=${password} attrs=${HIDDEN} send=${send} />
      <${Act} words=${`Save the ${what}`} doing=${doing} on=${{ type: 'change', to }} send=${send} />
    </details>
  `;
}

// What Apps says and sends for each way apps read a lend: a key its lender
// makes and pastes, or an account's name and password, which svrngs signs
// in with once.
/** @type {Record<'key' | 'account', (name: string, typed: string, password: string) => { make: string, how: string, field: string, to: import('./lend-change.js?v=__VERSION__').To }>} */
const READS = {
  key: (name, _, password) => ({
    make: `make a key in ${name} and paste it here`,
    how: `Paste the key here. It stays on this computer and goes only to ${name}.`,
    field: 'The key: ',
    to: { key: password },
  }),
  account: (name, typed, password) => ({
    make: 'make one account for your group',
    how: `Then type that account’s name and password here. svrngs signs in once and keeps only what ${name} gives it, not the password.`,
    field: 'Its password: ',
    to: { readers: { name: typed, password } },
  }),
};

// What svrngs adds to each request it carries to a lend (home.rs `lent`,
// `change`; share.rs `adds_headers`): the names of the headers kept, never
// what they say, each with a button to stop it, and a header to add, its
// value sent once to this computer and kept there. A header of a name kept
// replaces it.
/** @param {Omit<AppsProps, 'account'> & { kept?: string[] }} props */
function Headers({ name, kept, state, doing, send }) {
  if (kept === undefined) return null;
  const header = state.header ?? 'Authorization';
  const said = kept.length
    ? `svrngs adds ${kept.join(' and ')} to each request your friends make to ${name}. They never receive what it says.`
    : `If ${name} wants a header on each request, add it here: for llama-server started with --api-key abc123, Authorization with Bearer abc123.`;
  /** @param {string} named @param {string} value */
  const to = (named, value) => ({ type: 'change', to: { header: { name: named, value } } });
  return html`
    <p>${said}</p>
    ${kept.map((named) => html`<${Act} words=${`Stop adding ${named}`} doing=${doing} on=${to(named, '')} send=${send} />`)}
    <${Field} words="Header: " name="header" value=${header} send=${send} />
    <${Field} words="What it says: " name="secret" value=${state.secret ?? ''} attrs=${HIDDEN} send=${send} />
    <${Act} words="Add this header" doing=${doing} on=${to(header, state.secret ?? '')} send=${send} />
  `;
}

// Why nobody can open a lend while its program runs (door.rs `answering`):
// whose program answers on its port isn't the account it was lent from.
const HELD = {
  another: (/** @type {string} */ name) =>
    ` Another account’s program is answering on ${name}’s port, so nobody can open ${name}. Start ${name} again, or stop lending it and lend what runs there now.`,
  unjudged: (/** @type {string} */ name) =>
    ` svrngs can’t tell whose program is on ${name}’s port, so nobody can open ${name}. Start it again, or lend it again.`,
};
/** @param {{ name: string, held?: 'another' | 'unjudged' }} lend */
const heldWords = (lend) => (lend.held ? HELD[lend.held](lend.name) : '');

// Whether apps wait on the lender: an account or key to make, or why not.
const undone = (/** @type {AppsAccount | undefined} */ account) =>
  account !== undefined && !account.kept;

/** @param {LentProps} props */
export function Lent({ l: lend, signs, account, headers, groups, home }) {
  const [state, send] = useMachine(() =>
    machine('CHANGE', CHANGE(lend.name, home), {
      at: { change: 'idle' },
      said: '',
      groups: lend.groups ?? [],
    }),
  );
  const limit = lend.limit;
  const limited = (limit ?? 0) > 0;
  const people = limit === 1 ? 'person' : 'people';
  const howMany = limited ? `up to ${limit} ${people}` : 'everyone in the groups you chose';
  const words = `${lend.name}: ${howMany} at once, ${lend.using} using it now.${notRunning(lend, ' Not running now: start it, and your people can open it.')}${heldWords(lend)} ${lentTo(lend, groups)}`;
  const value = state.many ?? String(limit ?? '');
  const doing = { state: state.at.change === 'changing' ? 'busy' : 'idle' };
  const letEveryone =
    limited &&
    html`
      <${Act}
        words="Let everyone in"
        doing=${doing}
        on=${{ type: 'change', to: { limit: null } }}
        send=${send}
      />
    `;
  const signIn = signs
    ? html`Sign-in credentials created; app setup still required. Follow the app’s OpenID Connect setup guide, then test sign-in through your group’s app list. <a href="https://github.com/alexsbryan/svrngs/tree/main/apps">App setup guides</a>. `
    : html`
        <${Act}
          words="Sign people in with svrngs"
          doing=${doing}
          on=${{ type: 'sign' }}
          send=${send}
        />
      `;
  const saveAudience = html`
    <${Act}
      words="Save who can open this"
      doing=${doing}
      on=${{ type: 'audience' }}
      send=${send}
    />
  `;
  return html`
    <li>
      <details open=${undone(account)} aria-label=${lend.name}>
      <summary>${words}</summary>
      <${Field}
        words="How many people at once: "
        value=${value}
        attrs=${{ type: 'number', min: 1 }}
        send=${send}
      />
      <${Act} words="Let in this many" doing=${doing} on=${{ type: 'many', value }} send=${send} />
      ${letEveryone}
      <p>Who can open it:</p>
      <${GroupChoice}
        groups=${groups}
        ticked=${state.groups}
        waiting=${state.at.change === 'changing'}
        send=${send}
      />
      ${saveAudience}
      <${Apps} name=${lend.name} account=${account} state=${state} doing=${doing} send=${send} />
      <${Headers} name=${lend.name} kept=${headers} state=${state} doing=${doing} send=${send} />
      ${signIn}
      <${Act} words="Stop lending" doing=${doing} on=${{ type: 'change', to: 'stop' }} send=${send} />
      ${state.said}
      ${state.shown && html`<${SigningOn} c=${state.shown} />`}
      </details>
    </li>
  `;
}
