// SPDX-License-Identifier: AGPL-3.0-or-later
// Copy beside a lend in a group's foot (wifi.js `Line`): its address for a
// program on this computer, shown beside it too (ui/parts.js `Keep`), and
// what the last press said (link-copy.js `LinkCopy`, whose answer takes the
// place of the one before, as the address's must).
import { html } from '../ui/view.js?v=__VERSION__';
import { Keep } from '../ui/parts.js?v=__VERSION__';
import { LinkCopy } from './link-copy.js?v=__VERSION__';

/**
 * A copy of the lend's address `at`, for a program.
 * @typedef {{ at: string }} AddressCopyProps
 */

/** @param {AddressCopyProps} props */
export function AddressCopy({ at }) {
  const words = {
    copy: 'Copy its address for a program',
    copied: ` Copied: ${at}. Paste it where a program on this computer, such as a chat app, asks for an address.`,
    unwritten: ` Copy it from here: ${at}`,
  };
  const beside = html`${' '}<${Keep} at=${at} />`;
  return html`${' '}<${LinkCopy} text=${at} words=${words} beside=${beside} />`;
}
