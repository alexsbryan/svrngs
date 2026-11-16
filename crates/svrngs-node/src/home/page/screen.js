// SPDX-License-Identifier: AGPL-3.0-or-later
// A group on a screen: HOME's place `screen`, drawn alone at `?screen=`
// (group-screen.js), and the link to it in each group's section, `show`,
// once its lends are read, which opens it in a tab of its own.
import { html } from '../ui/view.js?v=__VERSION__';
import { Screen } from './group-screen.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from './home-reading.js?v=__VERSION__' */

/** @param {PlaceProps} props */
function ScreenLink({ g: group, s: state }) {
  if (!state.lends) return null;
  const href = `/?screen=${encodeURIComponent(group.id)}`;
  return html`<p><a href=${href} target="_blank">Show ${group.name ?? 'this group'} on a screen</a></p>`;
}

/** @type {{ screen: (props: HOME.PlaceProps) => Drawn, show: (props: PlaceProps) => Drawn }} */
export const view = { screen: Screen, show: ScreenLink };
