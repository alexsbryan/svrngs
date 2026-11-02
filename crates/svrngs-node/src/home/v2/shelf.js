// SPDX-License-Identifier: AGPL-3.0-or-later
// Bringing things to a group, in its section (page-shelf (a); home.js:203):
// the shelf (ui/shelf.js), once the group's lends are read, brought by
// /shelf, then the groups read again (HOME's `again`). Folded for the group
// HOME says was just joined here, its `q.joined` given through home.js and
// group.js, as v1 reads `?joined=` (home.js:203).
import { html, Shelf } from '../ui/kit.js?v=__VERSION__';

const bring = (g, apps) => fetch('/shelf', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify([g.id, apps]) }).then(async (r) => ((body) => (r.ok ? body : Promise.reject(new Error(body.error || `answered ${r.status}`))))(await r.json().catch(() => ({}))));
export const view = { shelf: ({ g, s, home, joined }) => s.lends && html`<${Shelf} g=${g} lent=${s.lends.lent} shelf=${s.lends.shelf} joined=${joined === g.name} bring=${bring} after=${() => home({ type: 'again' })} />` };
