// SPDX-License-Identifier: AGPL-3.0-or-later
// Copy beside a link made (link-said.js): it says only its last answer,
// where ui/copy.js's `Copy` says each in turn, since a person presses it
// again and again as they send the link to one place after another.
import { useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { COPY, drawn } from '../ui/copy.js?v=__VERSION__';
/** @import { CopyProps } from '../ui/copy.js?v=__VERSION__' */

/** @param {CopyProps} props */
export function LinkCopy(props) {
  const [state, send] = useMachine(() => machine('COPY', COPY, { at: { copy: 'idle' }, said: [] }));
  const last = { ...state, said: state.said.slice(-1) };
  return drawn(last, send, props);
}
