// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's folders on this disk, in its section once its people are read:
// each with its button (group-folder.js), or why this computer cannot say
// where they are.
import { html } from '../ui/view.js?v=__VERSION__';
import { its } from './folder.js?v=__VERSION__';
import { Folder } from './group-folder.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */

/** @param {PlaceProps} props */
export function Folders({ g: group, s: state }) {
  if (!state.people) return null;
  if (group.folders === null) return html`<p>This computer cannot say where ${its(group)} is.</p>`;
  const folders = group.folders.map((dir) => html`<${Folder} key=${dir} g=${group} dir=${dir} />`);
  const waiting = group.waiting?.map(([who, files]) => {
    const [count, they] = files === 1 ? ['1 file is', 'It'] : [`${files} files are`, 'They'];
    return html`<p>${count} still only on ${who ?? 'someone'}’s computer. ${they}’ll arrive when it’s back on.</p>`;
  });
  return [folders, waiting ?? html`<p>This computer cannot say what is still on its way.</p>`];
}
