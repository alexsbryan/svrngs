// SPDX-License-Identifier: AGPL-3.0-or-later
// Copy beside a lend in a group's foot (wifi.js `Line`): its address for a
// program on this computer, shown beside it too (ui/parts.js `Keep`), and
// what the last press said. The machine is the button's own
// (ui/view.js `useMachine`), one for each lend's line.
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { Act, Keep } from '../ui/parts.js?v=__VERSION__';
import { COPY } from './address-copying.js?v=__VERSION__';

/**
 * A copy of the lend's address `at`, for a program.
 * @typedef {{ at: string }} AddressCopyProps
 */

/** @param {AddressCopyProps} props */
export function AddressCopy({ at }) {
  const [state, send] = useMachine(() =>
    machine('COPY', COPY(at), { at: { copy: 'uncopied' }, said: '' }),
  );
  return html`
    ${' '}
    <${Act} words="Copy its address for a program" on=${{ type: 'copy' }} send=${send} />
    ${' '}
    <${Keep} at=${at} />
    ${state.said}
  `;
}
