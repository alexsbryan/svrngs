// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's links (group-link.js LINK): a button for each link it makes,
// a guest's behind Have a guest, and below them the last link made
// (link-said.js), or why it was not. The machine is the group's own
// (ui/view.js `useMachine`), so HOME's step never sees it.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LINK } from './group-link.js?v=__VERSION__';
import { LinkSaid } from './link-said.js?v=__VERSION__';
import { Copy } from '../ui/copy.js?v=__VERSION__';
/** @import { Group } from './group.js?v=__VERSION__' */
/** @import { State, Message, Publication } from './group-link.js?v=__VERSION__' */
/** @import { Store } from '../ui/machine.js?v=__VERSION__' */

// A guest's visit, longest first and chosen first: its seconds, as the
// Field holds them, and its words.
const VISITS = [48, 24, 6].map((hours) => [hours * 3600, `${hours} hours`]);

/** @param {State} state */
function publicationAction(state) {
  if (state.at.publication === 'refused') return { action: 'read', words: 'Retry reading joining' };
  switch (state.publication?.state) {
    case 'closing':
      return { action: 'close', words: 'Retry closing' };
    case 'pending':
      return { action: 'publish', words: 'Retry joining' };
    case undefined:
    case 'open':
    case 'closed':
    case 'ended':
      return { action: 'publish', words: 'Invite people here' };
  }
}

/** @param {Publication | null | undefined} receipt */
function publicationInfo(receipt) {
  switch (receipt?.state) {
    case 'open':
      return html`<p>Ends ${new Date(receipt.until * 1000).toLocaleString()}. This computer must remain reachable. A new invitation or device link ends this one; everyone already in stays.</p>`;
    case 'closed':
      return html`<p>Joining is closed</p>`;
    case 'ended':
      return html`<p>This invitation ended. Everyone already in stays.</p>`;
    case 'pending':
      return html`<p>joining is temporarily unavailable; retry in a minute from this computer. Your invitation is kept for retry.</p>`;
    case 'closing':
      return html`<p>joining is temporarily unavailable; retry in a minute from this computer. Closing has not been confirmed.</p>`;
    case undefined:
      return null;
  }
}

/** @param {{ state: State, send: Store<State, Message>['dispatch'], doing: { state: string } }} props */
function PublicationControls({ state, send, doing }) {
  const receipt = state.publication;
  const phase = receipt?.state ?? 'none';
  const { action, words } = publicationAction(state);
  return html`<div class="publication">
    ${receipt?.address && ['open', 'closing'].includes(phase) && html`<p><code style="font-size:1.4em">${receipt.address}</code> <${Copy} key=${receipt.address} text=${receipt.address} words=${{ copy: 'Copy the address', copied: 'Copied.' }} /></p>`}
    ${publicationInfo(receipt)}
    ${state.publicationWhy && html`<p role="status">${state.publicationWhy}</p>`}
    <p>
      ${(phase !== 'open' || state.at.publication === 'refused') && html`<${Act} words=${words} doing=${doing} on=${{ type: 'publication', action }} send=${send} />`}
      ${['open', 'pending'].includes(phase) && html`<${Act} words="Close joining" doing=${doing} on=${{ type: 'publication', action: 'close' }} send=${send} />`}
    </p>
  </div>`;
}

/**
 * A group's links, made for it.
 * @typedef {{ g: Group }} LinksProps
 */

/** @param {LinksProps} props */
export function Links({ g: group }) {
  const [state, send] = useMachine(() =>
    machine(
      'LINK',
      LINK(group.id),
      { at: { link: 'none', publication: 'reading' }, visit: '172800' },
      [{ run: 'publicationRead', failed: 'publicationUnread' }],
    ),
  );
  const name = group.name ?? 'this group';
  const busy = state.at.link === 'making' || ['reading', 'working'].includes(state.at.publication);
  const doing = { state: busy ? 'busy' : 'idle' };
  // `state.visit` is the Field's string, as `Number` reads it.
  const hours = `${Number(state.visit) / 3600} hours`;
  const guest = state.guest
    ? html`
        <${Field}
          name="visit"
          words="For how long, from when they open the link: "
          value=${state.visit}
          options=${VISITS}
          send=${send}
        />
        <${Act}
          words="Make a link for your guest"
          doing=${doing}
          on=${{ type: 'make', device: false, guest: true, hours }}
          send=${send}
        />
      `
    : html`<${Act} words="Have a guest" on=${{ type: 'guest' }} send=${send} />`;
  const said =
    state.at.link === 'refused'
      ? state.why
      : state.shown && html`<${LinkSaid} shown=${state.shown} name=${name} />`;
  return html`
    <${PublicationControls} state=${state} send=${send} doing=${doing} />
    <p>
      <${Act}
        words=${`Invite someone to ${name}`}
        doing=${doing}
        on=${{ type: 'make', device: false }}
        send=${send}
      />
      <${Act}
        words="Add your phone or another computer"
        doing=${doing}
        on=${{ type: 'make', device: true }}
        send=${send}
      />
      ${guest}
    </p>
    <p class="link">${said}</p>
  `;
}
