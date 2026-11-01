// SPDX-License-Identifier: AGPL-3.0-or-later
// Bringing things to a group, in its section (page-shelf (a); home.js:203):
// the shelf (ui/shelf.js), once the group's lends are read, brought by
// /shelf, then the groups read again (HOME's `again`). Folded for a group just joined here, as v1 reads `?joined=`
// (home.js:203). Read as this module loads, before home.js forgets the
// notice from the address (home.js `said`, once), since a group's place is
// given its group's state, not HOME's.
import { html } from '../ui/view.js?v=__VERSION__';
import { Shelf } from '../ui/shelf.js?v=__VERSION__';

const JOINED = new URLSearchParams(location.search).getAll('joined');
const bring = (g, apps) => fetch('/shelf', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify([g.id, apps]) }).then(async (r) => ((body) => (r.ok ? body : Promise.reject(new Error(body.error || `answered ${r.status}`))))(await r.json().catch(() => ({}))));
export const view = { shelf: ({ g, s, home }) => s.lends && html`<${Shelf} g=${g} lent=${s.lends.lent} shelf=${s.lends.shelf} joined=${JOINED.includes(g.name)} bring=${bring} after=${() => home({ type: 'again' })} />` };
