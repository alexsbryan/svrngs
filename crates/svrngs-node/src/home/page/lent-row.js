// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend in "What you lend", one line: its name, who can open it, paused
// when lent to no group and a choice to make when legacy (`null`), and how
// many use it now, of how many it lets in at once when that is limited;
// where friends can't open it, Closed or Not running first, marked (`line`).
// Folded under that line, since five lends' forms were two screens before
// anything was asked, and open at first only where apps cannot read it
// until its lender does something (`undone`): who can open it, one box a
// group of HOME's, each by its stable id, Keep sharing it when it is held,
// whether it is for programs, so friends see its address first, the apps'
// account, and Stop lending, a button for each change
// (lend-change.js CHANGE). What is seldom changed is folded again under
// More settings: how many at once, a header, and signing people in with
// svrngs, with the app form's fill once that is turned on (`SigningOn`).
// Under the fold, never in it, the offer to keep its program running,
// where it is shared only while that runs (keep-running.js). Just lent from
// its found row, whose line still holds the focus (running-here.js
// `Found`), it opens and takes the focus, in view with that offer under it.
import { html, useEffect, useMachine, useRef } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field, Tick } from '../ui/parts.js?v=__VERSION__';
import { CHANGE } from './lend-change.js?v=__VERSION__';
import { GroupChoice, called } from './group-choice.js?v=__VERSION__';
import { notRunning } from '../question.js?v=__VERSION__';
import { answered } from './running-look.js?v=__VERSION__';
import { KeepRunning } from './keep-running.js?v=__VERSION__';
/** @import { Lend } from '../question.js?v=__VERSION__' */
/** @import { AppsAccount } from './lent-read.js?v=__VERSION__' */
/** @import { State, Message } from './lend-change.js?v=__VERSION__' */
/** @import { Choice } from './group-choice.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */
/** @import { Targeted } from '../ui/view.js?v=__VERSION__' */

/**
 * A lend's row: the lend, whether it signs people in with svrngs, HOME's
 * groups to lend it to, and HOME's `dispatch`, told to read again once it
 * changes.
 * @typedef {object} LentProps
 * @property {Lend & { sharing?: number }} l
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

// The names of the groups a lend is lent to, "House and Trip", every id
// named, one this computer holds no group for by its start; '' for none.
/**
 * @param {Lend} lend
 * @param {Choice[]} groups
 */
const groupNames = (lend, groups) =>
  (lend.groups ?? [])
    .map((id) => {
      const group = groups.find((held) => held.id === id);
      return group
        ? called(group, groups)
        : `a group this computer does not hold (${id.slice(0, 8)})`;
    })
    .join(' and ');

// Who the row says can open it: the names of the groups it is lent to,
// none when paused, and a choice to make when the answer gave none.
/**
 * @param {Lend} lend
 * @param {Choice[]} groups
 */
export const lentTo = (lend, groups) => {
  // Absent is read as legacy, needing selection: never as every group.
  if (!lend.groups) return 'choose which groups can open it';
  if (lend.groups.length === 0) return 'paused, no group can open it';
  return `lent to ${groupNames(lend, groups)}`;
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

// Why nobody can open a held lend (door.rs `answering`, SHARE_BINDING
// §5.5), said after its line's Closed, so none says closed again; then what
// answers on its port now, which Keep sharing it approves.
const HELD = {
  unapproved: 'Nobody can open it until you approve what answers on its port now.',
  another: 'Something you didn’t approve was on its port.',
  unjudged: 'svrngs can’t tell what answers on its port. Start it again.',
  changed: 'What runs it changed since you chose to keep sharing it when it starts again.',
};
// What svrngs read answering on a held lend's port now: nothing answering
// said apart from a read that failed, which the row names in `unread`
// (door.rs `offered`), with no `answers`.
/** @param {Lend} lend */
const answering = ({ answers = [], unread }) => {
  if (unread) return 'svrngs couldn’t read what answers on its port. Reload to look again.';
  const now = answers.map(answered).join(' and ');
  return now ? `Now answering: ${now}.` : 'Nothing answers on its port now.';
};

// A lend's line. One friends can't open leads with that, marked and in
// words, so the color is not the only sign: Closed when it is held, with
// why and what answers now, else Not running (question.js `notRunning`),
// then who it is lent to, and no count of who uses it. An open one says who
// can open it and how many use it now.
/**
 * @param {Lend} lend
 * @param {string} to who can open it (`lentTo`)
 * @param {number | string | undefined} using
 */
const line = (lend, to, using) => {
  const [state, why] = lend.held
    ? ['Closed', `${HELD[lend.held]} ${answering(lend)}`]
    : ['Not running', notRunning(lend, 'Start it, and the groups you chose can open it again.')];
  if (!why) return `${lend.name}: ${to}, ${using} using it now.`;
  return html`${lend.name}: <strong class="bad">${state}</strong>. ${why} ${to.charAt(0).toUpperCase()}${to.slice(1)}.`;
};

// Keep sharing it, offered by the token alone: svrngs lists one only where
// approving it would be taken (share.rs `approve`), so the page doesn't
// judge the hold itself. It approves everything the line says answers now,
// by that token, with the first process shown, if one (lend-change.js
// `approve`).
/** @param {{ lend: Lend, doing: { state: string }, send: (message: Message) => void }} props */
function Keep({ lend, doing, send }) {
  if (!lend.token) return null;
  const first = lend.answers?.[0];
  const shown = { token: lend.token, pid: first?.pid, start: first?.start };
  return html`<${Act} words="Keep sharing it" doing=${doing} on=${{ type: 'approve', shown }} send=${send} />`;
}

// How friends' cells lead with a lend that opens in a tab: its name, or,
// ticked, for a program such as a chat app or a coding tool, Play and the
// address it copies (lend-name.js). The lender's to say, since what answers
// on its port never says which its friends want; a tick is said as one to
// a screen reader, ticked or not, and posts as it changes.
/** @param {{ lend: Lend, doing: { state: string }, send: (message: Message) => void }} props */
function Programs({ lend, doing, send }) {
  if (lend.raw) return null;
  /** @param {Targeted<HTMLInputElement>} box */
  const change = (box) =>
    send({ type: 'change', to: { 'for-programs': box.currentTarget.checked } });
  const words = ' For programs, such as a chat app: your friends see its address first.';
  return html`<${Tick} on=${Boolean(lend.for_programs)} disabled=${doing.state === 'busy'} change=${change}>${words}<//>`;
}

// Whether the lend waits on its lender: an account or key for apps to make,
// or what answers on its port to approve.
const undone = (/** @type {AppsAccount | undefined} */ account, /** @type {Lend} */ lend) =>
  (account !== undefined && !account.kept) || Boolean(lend.held);

/** @param {LentProps} props */
export function Lent({ l: lend, signs, account, headers, groups, home }) {
  /** @type {() => State} */
  const first = () => ({ at: { change: 'idle' }, said: '', groups: lend.groups ?? [] });
  const [state, send] = useMachine(() => machine('CHANGE', CHANGE(lend.name, home), first()));
  const fold = useRef(/** @type {HTMLDetailsElement | null} */ (null));
  // Focus scrolls what takes it into view, so the offer under it shows.
  useEffect(() => {
    if (!fold.current || document.activeElement?.getAttribute('data-lent') !== lend.name) return;
    fold.current.open = true;
    fold.current.querySelector('summary')?.focus();
  }, []);
  const limit = lend.limit;
  const limited = (limit ?? 0) > 0;
  const using = limited ? `${lend.using} of ${limit}` : lend.using;
  const value = state.many ?? String(limit ?? '');
  const doing = { state: state.at.change === 'changing' ? 'busy' : 'idle' };
  // What three of the row's acts send, named so each act is one line.
  const everyone = { type: 'change', to: { limit: null } };
  const [sign, audience] = [{ type: 'sign' }, { type: 'audience' }];
  const letEveryone =
    limited && html`<${Act} words="Let everyone in" doing=${doing} on=${everyone} send=${send} />`;
  const signIn = signs
    ? html`Sign-in credentials created; app setup still required. Follow the app’s OpenID Connect setup guide, then test sign-in through your group’s app list. <a href="https://github.com/alexsbryan/svrngs/tree/main/apps">App setup guides</a>. `
    : html`<${Act} words="Sign people in with svrngs" doing=${doing} on=${sign} send=${send} />`;
  return html`
    <li>
      <details ref=${fold} open=${undone(account, lend)} aria-label=${lend.name}>
        <summary>${line(lend, lentTo(lend, groups), using)}</summary>
        <${Keep} lend=${lend} doing=${doing} send=${send} />
        <p>Who can open it:</p>
        <${GroupChoice}
          groups=${groups}
          ticked=${state.groups}
          waiting=${state.at.change === 'changing'}
          send=${send}
        />
        <${Act} words="Save who can open this" doing=${doing} on=${audience} send=${send} />
        <${Programs} lend=${lend} doing=${doing} send=${send} />
        <${Apps} name=${lend.name} account=${account} state=${state} doing=${doing} send=${send} />
        <details>
          <summary>More settings</summary>
          <${Field}
            words="How many people at once: "
            value=${value}
            attrs=${{ type: 'number', min: 1 }}
            send=${send}
          />
          <${Act} words="Let in this many" doing=${doing} on=${{ type: 'many', value }} send=${send} />
          ${letEveryone}
          <${Headers} name=${lend.name} kept=${headers} state=${state} doing=${doing} send=${send} />
          ${signIn}
          ${state.shown && html`<${SigningOn} c=${state.shown} />`}
        </details>
        <${Act} words="Stop lending" doing=${doing} on=${{ type: 'change', to: 'stop' }} send=${send} />
        ${state.said}
      </details>
      <${KeepRunning} lend=${lend} who=${groupNames(lend, groups)} home=${home} />
    </li>
  `;
}
