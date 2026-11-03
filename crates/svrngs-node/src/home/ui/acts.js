// SPDX-License-Identifier: AGPL-3.0-or-later
// @ts-nocheck: not typed yet, so scripts/types.mjs skips it (the js-types row types it).
// "What acts as you", both pages' (page-acts (a); question.js `acts`,
// :231-242, which it moves; capability-what-acts; svrn-docs
// WHAT_CAN_ACT_AS_YOU §9): one section, folded under its heading, a list to
// look at when you want to check or stop something, never in the way. A
// part for each thing an app or a device can do as you, in this order,
// each drawn as v1 draws it, an empty `div` until it has a line, then
// headed, then `notes[part]`, the page's own words, then its lines. Each
// page gives `parts`, part -> its lines, derived from its own state, where
// v1 pushed each line in by `add`; the fold's open is the page's, `fold
// acts` (ui/parts.js `Fold`).
import { html } from './view.js?v=__VERSION__';
import { Fold } from './parts.js?v=__VERSION__';

const PARTS = {
  reads: 'Reads and writes for you',
  opens: 'Opens friends’ things as you',
  knows: 'Knows who you are',
  wifi: 'Open to this WiFi',
  play: 'Open to programs on this computer',
  decides: 'Can decide for you',
};
export const Acts = ({ parts, notes = {}, open, send }) =>
  html`<section id="acts"><${Fold} summary="What acts as you" heading=${true} fold="acts" open=${open} send=${send}>${Object.entries(PARTS).map(([k, head]) => html`<div>${parts[k]?.length > 0 && [html`<h3>${head}</h3>`, notes[k] && html`<p>${notes[k]}</p>`, parts[k]]}</div>`)}<//></section>`;
