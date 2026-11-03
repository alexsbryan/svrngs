// SPDX-License-Identifier: AGPL-3.0-or-later
// A group on a screen: HOME's place `screen`, drawn alone at `?screen=`
// (group-screen.js), and the link to it in each group's section, `show`
// (screen-link.js).
import { Screen } from './group-screen.js?v=__VERSION__';
import { ScreenLink } from './screen-link.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import * as HOME from '../home.js?v=__VERSION__' */

/** @type {{ screen: (props: HOME.PlaceProps) => Drawn, show: (props: PlaceProps) => Drawn }} */
export const view = { screen: Screen, show: ScreenLink };
