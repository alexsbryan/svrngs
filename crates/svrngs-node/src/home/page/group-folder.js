// SPDX-License-Identifier: AGPL-3.0-or-later
// One of a group's folders on this disk: a button that opens it in this
// computer's file manager (folder.js FOLDER), Finder on a Mac, the files
// named apart from the group's apps (docs/APP_ACCESS_AND_STOP.md A2), and
// where it is under Folder location, closed at first. Of a group's several
// folders, each button names its own by the folder's name.
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
  const which = (group.folders?.length ?? 0) > 1 ? ` (${dir.split(/[\\/]/).pop()})` : '';
  const where = /Mac/.test(navigator.platform) ? 'Finder' : 'your file manager';
  return html`
    <p>
      <button onClick=${open}>Open shared files${which} in ${where}</button>
      ${state.said}
    </p>
    <details><summary>Folder location</summary><p class="folder">${its(group)} is on this computer at ${dir}</p></details>
  `;
}
