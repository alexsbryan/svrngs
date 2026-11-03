// SPDX-License-Identifier: AGPL-3.0-or-later
// Sends the ask the address names (ask.js) once the group's list is read,
// and then not again. No flow sends it, as the read is enter.js's, so the
// page sends it once it draws the list. It draws nothing.
import { useEffect } from '../ui/view.js?v=__VERSION__';
/** @import { Wants } from './enter.js?v=__VERSION__' */
/** @import { Props, State } from './page.js?v=__VERSION__' */

/**
 * Drawn only once the address asks (ask-form.js).
 * @typedef {object} AskWhenReadProps
 * @property {State & { wants: Wants }} s
 * @property {Props['send']} send
 */

/** @param {AskWhenReadProps} props */
export function AskWhenRead({ s: state, send }) {
  useEffect(() => {
    if (state.at.list !== 'read' || state.at.ask !== 'none') return;
    send({
      type: 'ask',
      lend: state.wants.ask,
      app: state.wants.app,
      at: state.wants.at,
      why: `whether ${state.wants.app} may use ${state.wants.ask}`,
    });
  }, [state.at.list]);
  return null;
}
