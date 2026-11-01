// SPDX-License-Identifier: AGPL-3.0-or-later
// The flow that opens the page and reads the group: page-open's. Until its
// part (b), `start` gives only a link's token to the computer's line
// (web/v2/computer.js), and the rest is refused by name.
import { go, no } from '../ui/machine.js?v=__VERSION__';

export const unbuilt = ['hold', 'reread'];
export const takes = {
  start: (s, { fragment }, token = fragment.split('&')[0]) => (token && !token.includes('=') ? go(s, {}, {}, { run: 'send', m: { type: 'offer', token } }) : no(s, 'not built yet: enter')),
};
