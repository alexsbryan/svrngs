// SPDX-License-Identifier: AGPL-3.0-or-later
// A link made for a group (group-link.js LINK), in the words of whom it is
// for: a guest, your other computer or phone, or someone invited; then
// its Copy (link-copy.js).
import { html } from '../ui/view.js?v=__VERSION__';
import { LinkCopy } from './link-copy.js?v=__VERSION__';
/** @import { Made } from './group-link.js?v=__VERSION__' */

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
export function LinkSaid({ shown, name }) {
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
  const to = asked.device
    ? 'Open this link on your other computer or phone; it joins as you, so send it to nobody else: '
    : 'Send this link to the person you are inviting: ';
  const how = answer.https
    ? ' It opens in a browser, which says how to join from there.'
    : ' They join with svrngs join and this link.';
  return [to, code, how, LAST, copy];
}
