// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's links in its section, once the group's people are read
// (group-link.js LINK): Invite people, the invitation shown three ways
// (`PublicationControls`), then a button for each other link it makes, a
// guest's behind Have a guest, and below them the last link made
// (`LinkSaid`), or why it was not, or that a device link ends the
// invitation. The machine is the group's own (ui/view.js `useMachine`),
// so HOME's step never sees it.
import { html, useEffect, useMachine, useRef } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Field } from '../ui/parts.js?v=__VERSION__';
import { LINK, admits, busy } from './group-link.js?v=__VERSION__';
import { LinkCopy } from './link-copy.js?v=__VERSION__';
import { Copy } from '../ui/copy.js?v=__VERSION__';
/** @import { Group, PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { Made, State, Message, Publication } from './group-link.js?v=__VERSION__' */
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
      return { action: 'publish', words: 'Retry the short address' };
    case undefined:
    case 'open':
    case 'closed':
    case 'ended':
      return { action: 'publish', words: 'Invite people' };
  }
}

/** @param {Publication | null | undefined} receipt */
function publicationInfo(receipt) {
  switch (receipt?.state) {
    case 'open':
    case 'pending':
      return html`<p>Ends ${new Date(receipt.until * 1000).toLocaleString()}. Keep this computer on and awake until everyone’s in.${receipt.state === 'pending' && ' The short address isn’t ready: the service that makes it didn’t answer. Retry it in a minute from this computer.'}</p>`;
    case 'closed':
      return html`<p tabindex="-1">Joining is closed</p>`;
    case 'ended':
      return html`<p tabindex="-1">This invitation ended. Everyone already in stays.</p>`;
    case 'closing':
      return html`<p tabindex="-1">Closing isn’t confirmed: the service that holds the short address didn’t answer, so people can still join. Retry in a minute from this computer.</p>`;
    case undefined:
      return null;
  }
}

// The invitation, three ways (publication.rs `shown`): its link, for a
// chat; its short address, large, to read aloud; and the link's QR code,
// for a phone's camera, which never asks the resolver.
/** @param {Publication} receipt */
const ways = ({ link, address, qr, state }) => html`
  ${link && html`<p>Paste this link in a chat: <code>${link}</code> <${Copy} key=${link} text=${link} words=${{ copy: 'Copy the link', copied: 'Copied.' }} /></p>`}
  ${address && ['open', 'closing'].includes(state) && html`<p>Or read this address aloud: <code style="font-size:1.4em">${address}</code> <${Copy} key=${address} text=${address} words=${{ copy: 'Copy the address', copied: 'Copied.' }} /></p>`}
  ${qr && html`<p>Or have people point their phone’s camera at this code:</p><svg class="qr" viewBox=${`0 0 ${qr.size} ${qr.size}`} role="img" aria-label="QR code of the link above" shape-rendering="crispEdges"><path d=${qr.path} /></svg>`}
`;

// Invite people, its invitation (`ways`) and what it says, and Close
// joining. Each change is said as it is drawn (`aria-live`). A failure the
// receipt says itself, pending or closing, is not said twice. A press here
// disables its button while it is answered, or takes it away once it is,
// either of which drops a browser's focus to the top of the page; so once
// answered, the focus goes to the answer: the invitation's first Copy while
// it lets people in, else what joining now says (`tabindex="-1"`).
/** @param {{ state: State, send: Store<State, Message>['dispatch'], doing: { state: string } }} props */
function PublicationControls({ state, send, doing }) {
  const receipt = state.publication;
  const phase = receipt?.state ?? 'none';
  const { action, words } = publicationAction(state);
  const part = useRef(/** @type {HTMLDivElement | null} */ (null));
  useEffect(() => {
    const to = part.current?.querySelector(admits(receipt) ? 'button' : '[tabindex]');
    if (state.pressed && !busy(state)) /** @type {HTMLElement | null | undefined} */ (to)?.focus();
  }, [receipt, state.publicationWhy]);
  return html`<div class="publication" aria-live="polite" ref=${part}>
    ${receipt && ways(receipt)}
    ${publicationInfo(receipt)}
    ${state.publicationWhy && !['pending', 'closing'].includes(phase) && html`<p tabindex="-1">${state.publicationWhy}</p>`}
    <p>
      ${(phase !== 'open' || state.at.publication === 'refused') && html`<${Act} words=${words} doing=${doing} on=${{ type: 'publication', action }} send=${send} />`}
      ${admits(receipt) && html`<${Act} words="Close joining" doing=${doing} on=${{ type: 'publication', action: 'close' }} send=${send} />`}
    </p>
  </div>`;
}

// A link made for a group (group-link.js LINK), in the words of whom it is
// for: a guest, or your other computer or phone; then its Copy
// (link-copy.js).
const WORDS = {
  copy: 'Copy the link',
  copied: ' Copied.',
  unwritten: ' Select it above to copy it.',
};
const LAST =
  ' It works for a day, while this computer is on. A new link stops this one letting anyone new in; everyone it let in stays. ';

/**
 * The link made, and the name of the group it is for.
 * @typedef {{ shown: Made, name: string }} LinkSaidProps
 */

/** @param {LinkSaidProps} props */
function LinkSaid({ shown, name }) {
  const { a: answer, asked } = shown;
  // The page's link when this computer has one, as a browser opens it.
  const at = answer.https ?? answer.client;
  const code = html`<code>${at}</code>`;
  const copy = html`<${LinkCopy} key=${at} text=${at} words=${WORDS} beside=${true} />`;
  if (asked.guest)
    return [
      'Send this link to your guest, in any chat: ',
      code,
      ` They open it in a browser and give their name. For ${asked.hours} from then they can open what you lend, and everyone in ${name} sees they are your guest. It lets one guest in, once, within a day, while this computer is on. `,
      copy,
    ];
  const how = answer.https
    ? ' It opens in a browser, which says how to join from there.'
    : ' They join with svrngs join and this link.';
  const to =
    'Open this link on your other computer or phone; it joins as you, so send it to nobody else: ';
  return [to, code, how, LAST, copy];
}

/**
 * A group's links, made for it.
 * @typedef {{ g: Group }} LinksProps
 */

/** @param {LinksProps} props */
function Links({ g: group }) {
  const [state, send] = useMachine(() =>
    machine(
      'LINK',
      LINK(group.id),
      { at: { link: 'none', publication: 'reading' }, visit: '172800' },
      [{ run: 'publicationRead', failed: 'publicationUnread' }],
    ),
  );
  const name = group.name ?? 'this group';
  // The invitation's presses go busy while answered (`PublicationControls`).
  // A link's do not: disabled, a button drops the focus to the top of the
  // page, and LINK refuses a second press while one is made (`busy`).
  const doing = { state: busy(state) ? 'busy' : 'idle' };
  // A guest's hours: `state.visit` is the Field's string, as `Number` reads it.
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
          on=${{ type: 'make', device: false, guest: true, hours: `${Number(state.visit) / 3600} hours` }}
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
        words="Add your phone or another computer"
        on=${{ type: 'make', device: true }}
        send=${send}
      />
      ${guest}
    </p>
    <p class="link" role="status">${state.ending ? 'This ends the invitation people are using to join: its link, address and QR code stop letting anyone new in. Everyone already in stays. To go ahead, press Add your phone or another computer again.' : said}</p>
  `;
}

/** @param {PlaceProps} props */
function LinksPlace({ g: group, s: state }) {
  // Its people read, as the group's section reads them (group-read.js `GroupRead`).
  if (!state.people) return false;
  return html`<${Links} g=${group} />`;
}

/** @type {{ links: (props: PlaceProps) => Drawn }} */
export const view = { links: LinksPlace };
