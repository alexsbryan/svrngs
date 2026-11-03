// SPDX-License-Identifier: AGPL-3.0-or-later
// A group's people on the home page, each in its place in the group's
// section (group.js PLACES), once GROUP has read them: the door
// (people.js), the group's folders on this disk (folders.js), and who was
// removed or asked to be (removals.js).
import { Folders } from './folders.js?v=__VERSION__';
import { People } from './people.js?v=__VERSION__';
import { Removals } from './removals.js?v=__VERSION__';
/** @import { PlaceProps } from './group.js?v=__VERSION__' */
/** @import { Drawn } from '../ui/view.js?v=__VERSION__' */

/** @type {Record<'people' | 'folder' | 'removals', (props: PlaceProps) => Drawn>} */
export const view = { people: People, folder: Folders, removals: Removals };
