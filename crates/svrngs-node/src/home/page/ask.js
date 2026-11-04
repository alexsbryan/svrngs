// SPDX-License-Identifier: AGPL-3.0-or-later
// The ask at /ask (asked.js `Asked`, its machine asking.js ASK), drawn in
// place of the page while the address asks (home-page.js `Home`).
import { Asked } from './asked.js?v=__VERSION__';
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */
/** @import { PlaceProps } from './home-reading.js?v=__VERSION__' */

/** @type {{ ask: (props: PlaceProps) => Drawn }} */
export const view = { ask: Asked };
