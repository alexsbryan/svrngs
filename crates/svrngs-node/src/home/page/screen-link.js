// SPDX-License-Identifier: AGPL-3.0-or-later
// The link at the foot of a group's section, once its lends are read, that
// opens it on a screen in a tab of its own (group-screen.js).
import { html } from '../ui/view.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */

/** @param {PlaceProps} props */
export function ScreenLink({ g: group, s: state }) {
  if (!state.lends) return null;
  const href = `/?screen=${encodeURIComponent(group.id)}`;
  return html`<p><a href=${href} target="_blank">Show ${group.name ?? 'this group'} on a screen</a></p>`;
}
