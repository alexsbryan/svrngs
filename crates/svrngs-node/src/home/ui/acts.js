// SPDX-License-Identifier: AGPL-3.0-or-later
// "What acts as you", both pages' (svrn-docs WHAT_CAN_ACT_AS_YOU §9): one
// section, folded under its heading, a list to look at when you want to
// check or stop something, never in the way. A part for each thing an app
// or a device can do as you, in PARTS' order, each an empty `div` until it
// has a line, then headed, then `notes[part]`, the page's own words, then
// its lines. Each page gives `parts`, part -> its lines, derived from its
// own state; the fold's open is the page's, `fold acts` (ui/parts.js
// `Fold`).
import { html } from './view.js?v=__VERSION__';
import { Fold } from './parts.js?v=__VERSION__';
/** @import { Drawn } from './view.js?v=__VERSION__' */

const PARTS = {
  reads: 'Reads and writes for you',
  opens: 'Opens friends’ things as you',
  knows: 'Knows who you are',
  wifi: 'Open to this WiFi',
  play: 'Open to programs on this computer',
  decides: 'Can decide for you',
};

/** @typedef {keyof typeof PARTS} Part a part of the section, by its name */

/**
 * The section's props: each part's lines, by its name, a page giving the
 * parts it has; the page's words for a part; whether the fold is open;
 * and where its fold's opening is sent.
 * @typedef {object} ActsProps
 * @property {Partial<Record<Part, Drawn[]>>} parts
 * @property {Partial<Record<Part, string>>} [notes]
 * @property {boolean} [open]
 * @property {(message: { type: 'fold', fold: string, open: boolean }) => void} send
 */

/** @param {ActsProps} props */
export function Acts({ parts, notes = {}, open, send }) {
  const named = /** @type {[Part, string][]} */ (Object.entries(PARTS));
  const drawn = named.map(([part, head]) => {
    const lines = parts[part] ?? [];
    const shown = lines.length > 0 && [
      html`<h3>${head}</h3>`,
      notes[part] && html`<p>${notes[part]}</p>`,
      lines,
    ];
    return html`<div>${shown}</div>`;
  });
  return html`
    <section id="acts">
      <${Fold} summary="What acts as you" heading=${true} fold="acts" open=${open} send=${send}>
        ${drawn}
      <//>
    </section>
  `;
}
