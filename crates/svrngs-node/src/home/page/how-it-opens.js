// SPDX-License-Identifier: AGPL-3.0-or-later
// A lend's third cell in the group's index: how it opens. A port is this
// computer's own, or another's, which Play opens here (play.js `cell`).
import { html } from '../ui/view.js?v=__VERSION__';
import { cell } from './play.js?v=__VERSION__';
/** @import { Lent, PlaceProps } from './group.js?v=__VERSION__' */

/**
 * The lend, how it opens as question.js `opens` says it, and what a port's
 * cell is drawn with.
 * @typedef {Omit<PlaceProps, 'joined'> & { l: Lent, opened: string | undefined }} HowItOpensProps
 */

/** @param {HowItOpensProps} props */
export function HowItOpens({ l: lend, opened, g: group, s: state, send, home, me }) {
  if (!lend.raw) return `${opened}.`;
  if (lend.key === me) return 'A port this computer lends.';
  return html`<${cell} l=${lend} g=${group} s=${state} send=${send} home=${home} />`;
}
