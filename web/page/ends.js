// SPDX-License-Identifier: AGPL-3.0-or-later
// While anyone is visiting, the earliest end of a visit, sent to door.js
// to wait for. It draws nothing.
import { useEffect } from '../ui/view.js?v=__VERSION__';
/** @import { Visitor } from '../ui/visitors.js?v=__VERSION__' */
/** @import { Props } from './page.js?v=__VERSION__' */

/**
 * @typedef {object} EndsProps
 * @property {Visitor[]} vs who is visiting
 * @property {Props['send']} send
 */

/** @param {EndsProps} props */
export function Ends({ vs, send }) {
  useEffect(() => send({ type: 'ends', until: Math.min(...vs.map((visit) => visit.until)) }), [vs]);
  return null;
}
