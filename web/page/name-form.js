// SPDX-License-Identifier: AGPL-3.0-or-later
// The name asked, in #join, by join.js for a link and for a guest's visit.
// In iPhone Safari, the one browser defining `navigator.standalone`, false
// outside a Home Screen web app, the person is first told to join from the
// Home Screen: each web app there keeps a storage of its own, so one added
// after joining joins again, as someone new (the operator's iPhone, iOS
// 18.7, 2026-10-30). The form is keyed by whether it is shown and why the
// last name was refused, so it is drawn anew each time it is asked and its
// box takes the focus each time. It is pure and called as a function, as is
// its `Field`, so the first state draws the plain elements web/index.html
// has, as frame.test.mjs reads them.
import { html } from '../ui/view.js?v=__VERSION__';
import { Field } from '../ui/parts.js?v=__VERSION__';
/** @import { Message, Props } from './page.js?v=__VERSION__' */

const ATTRS = { autocomplete: 'name', maxlength: 64, required: true };

/**
 * @typedef {object} NameFormProps
 * @property {boolean} shown
 * @property {string} [group] the group named in it
 * @property {boolean} [keep] say to join from the Home Screen first
 * @property {string} [why] why the last name was refused
 * @property {string} [value] the name typed
 * @property {Props['send']} send
 * @property {Message} on the message its Join sends
 */

/** @param {NameFormProps} props */
export const NameForm = ({ shown, group = '', keep, why, value, send, on }) => {
  const submit = (/** @type {Event} */ event) => {
    event.preventDefault();
    return send(on);
  };
  const home =
    keep &&
    `To keep ${group} on this phone, add this page to your Home Screen first, and join from there.`;
  const words = html`What’s your name? Everyone in <span id="group">${group}</span> sees it.`;
  const field = Field({ shape: 'for', id: 'name', words, value, attrs: ATTRS, send });
  return html`
    <form key=${`${shown} ${why}`} id="join" hidden=${!shown} onSubmit=${submit}>
      <p id="keep" hidden=${!keep}>${home}</p>
      ${field}
      <p id="why" class="bad">${why}</p>
      <button>Join</button>
    </form>
  `;
};
