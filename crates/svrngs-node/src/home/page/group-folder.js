// SPDX-License-Identifier: AGPL-3.0-or-later
// One of a group's folders on this disk, said where it is, with a button
// that opens it in a window of its own (folder.js FOLDER).
import { html, useMachine } from '../ui/view.js?v=__VERSION__';
import { machine } from '../ui/machine.js?v=__VERSION__';
import { FOLDER, its } from './folder.js?v=__VERSION__';
/** @import { Group } from './group.js?v=__VERSION__' */

/**
 * A folder of the group's, with its button.
 * @typedef {{ g: Group, dir: string }} FolderProps
 */

/** @param {FolderProps} props */
export function Folder({ g: group, dir }) {
  const [state, send] = useMachine(() =>
    machine('FOLDER', FOLDER, { at: { folder: 'idle' }, said: '' }),
  );
  const open = () => send({ type: 'open', group: group.id, dir });
  return html`
    <p class="folder">${its(group)} is on this computer at ${dir}</p>
    <p>
      <button onClick=${open}>Open this folder</button>
      ${state.said}
    </p>
  `;
}
