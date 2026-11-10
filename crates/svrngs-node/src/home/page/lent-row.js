// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend in "What you lend": how many it lets in at once and how many use
// it now, who can open it, one box a group of HOME's, each by its stable
// id, showing what it is lent to, paused when empty and needing selection
// when legacy (`null`), with a button for each change (lend-change.js
// CHANGE), and the app form's fill once signing people in is turned on
// (signing-on.js).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { CHANGE } from './lend-change.js?v=__VERSION__';
import { GroupChoice, called } from './group-choice.js?v=__VERSION__';
import { SigningOn } from './signing-on.js?v=__VERSION__';
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
 * @property {Choice[]} groups
 * @property {HOME.PlaceProps['home']} home
 */

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

/**
 * How apps read a lend through its lender: whether an account or key is
 * made, how to make it, and its name and password, or the key, typed here
 * once (home.rs `readers`, `change`).
 * @param {Omit<AppsProps, 'account'> & { account?: AppsAccount }} props
 */
function Apps(props) {
  const { account } = props;
  if (!account) return null;
  if (account.why) return html`<p>${account.why}</p>`;
  return account.key ? html`<${AppsKey} ...${props} />` : html`<${AppsAccount} ...${props} />`;
}

// What Apps draws when its program takes a key its lender makes (home.rs
// `lent`, `key`, share.rs `key_of`): pasted here, sent once to this
// computer, and never drawn back.
/** @param {AppsProps} props */
function AppsKey({ name, account, state, doing, send }) {
  const save = { type: 'change', to: { key: state.password ?? '' } };
  const said = account.kept
    ? `Apps your friends open can read your ${name} with the key you made. They can’t change anything.`
    : `Apps can’t read your ${name} yet. To let them, make a key in ${name} and paste it here.`;
  return html`
    <p>${said}</p>
    <details>
      <summary>${account.kept ? 'Use a different key' : 'How to make the key'}</summary>
      <p>In ${name}${account.by}: ${account.signs?.accounts}. Paste the key here. It stays on this computer and goes only to ${name}.</p>
      <${Field}
        words="The key: "
        name="password"
        value=${state.password ?? ''}
        attrs=${{ type: 'password', autocomplete: 'off' }}
        send=${send}
      />
      <${Act} words="Save the key" doing=${doing} on=${save} send=${send} />
    </details>
  `;
}

/** @param {AppsProps} props */
function AppsAccount({ name, account, state, doing, send }) {
  const said = account.kept
    ? `Apps your friends open can read your ${name} with the account you made. They can’t change anything.`
    : `Apps can’t read your ${name} yet. To let them, make one account for your group.`;
  const save = {
    type: 'change',
    to: { readers: { name: state.account ?? '', password: state.password ?? '' } },
  };
  return html`
    <p>${said}</p>
    <details>
      <summary>${account.kept ? 'Use a different account' : 'How to make the account'}</summary>
      <p>In ${name}${account.by}: ${account.signs?.accounts}. Then type that account’s name and password here. svrngs signs in once and keeps only what ${name} gives it, not the password.</p>
      <${Field} words="Account name: " name="account" value=${state.account ?? ''} send=${send} />
      <${Field}
        words="Its password: "
        name="password"
        value=${state.password ?? ''}
        attrs=${{ type: 'password', autocomplete: 'off' }}
        send=${send}
      />
      <${Act} words="Save the account" doing=${doing} on=${save} send=${send} />
    </details>
  `;
}

/** @param {LentProps} props */
export function Lent({ l: lend, signs, account, groups, home }) {
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
  const words = `${lend.name}: ${howMany} at once, ${lend.using} using it now. ${lentTo(lend, groups)}`;
  const value = state.many ?? String(limit ?? '');
  const doing = { state: state.at.change === 'changing' ? 'busy' : 'idle' };
  const letMany = html`
    <${Act} words="Let in this many" doing=${doing} on=${{ type: 'many', value }} send=${send} />
  `;
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
  const stop = html`
    <${Act} words="Stop lending" doing=${doing} on=${{ type: 'change', to: 'stop' }} send=${send} />
  `;
  return html`
    <li>
      ${words}
      <${Field}
        words="How many people at once: "
        value=${value}
        attrs=${{ type: 'number', min: 1 }}
        send=${send}
      />
      ${letMany}
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
      ${signIn}
      ${stop}
      ${state.said}
      ${state.shown && html`<${SigningOn} c=${state.shown} />`}
    </li>
  `;
}
