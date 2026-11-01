// SPDX-License-Identifier: AGPL-3.0-or-later
// Joining by a link no group kept here was joined by (page-join (a);
// web/app.js `join`, `askName`, :120-196; DESIGN §3, "Joining"). The link's
// issuer is asked which group it lets into; a group held here already, by
// another link, is in at once; a link made for a person's devices joins as
// that person, asking no name; else the name is asked once, in #join, and
// asked again, saying why, when svrngs-core refuses it or someone in the
// group goes by it (409). Joined, the group is held as enter.js holds any
// (`hold`). The key is made, used and kept at the edge, never in state.
// A guest's name is asked in this form too, first (web/v2/visit.js), with
// no Home Screen line, as a visit is kept apart from every group, and sent
// to the visit as `guest named`.
import { html } from '../ui/view.js?v=__VERSION__';
import { go, no } from '../ui/machine.js?v=__VERSION__';
import { Field } from '../ui/parts.js?v=__VERSION__';
import * as store from '../store.js?v=__VERSION__';

// `joining`: the link's token, the groups kept when it was opened, what the
// address asks, and once asked, its group and the issuer's answer (`told`).
export const owns = ['joining', 'typed', 'why'];
export const states = { join: ['unasked', 'asking', 'naming', 'redeeming', 'taken', 'joined'] };
const send = (type, m) => ({ run: 'send', m: { type, ...m } });
const redeem = (s, joining, name) => go(s, { join: 'redeeming' }, { joining }, { run: 'admit', token: joining.token, told: joining.told, group: joining.group, name, failed: 'unjoined' });
const asks = (s) => ['naming', 'taken'].includes(s.at.join), guest = (s) => s.at.visit === 'naming';
export const takes = {
  join: (s, { token, kept, wants }) => (s.at.join === 'unasked' ? go(s, { join: 'asking' }, { joining: { token, kept, wants } }, { run: 'ask', token, failed: 'unjoined' }) : no(s, `a link is ${s.at.join} already`)),
  asked: (s, { group, told }, known = s.joining?.kept.find((g) => g.id === group.id), joining = { ...s.joining, group, told, known }) => (s.at.join !== 'asking' ? no(s, `no link is asked of, it is ${s.at.join}`)
    : known ? go(s, { join: 'joined' }, { joining }, send('hold', { group: known, wants: joining.wants, joined: true, why: `this phone is in ${group.id} already, by another link` })) : group.person ? redeem(s, joining, group.person) : go(s, { join: 'naming' }, { joining })),
  'typed name': (s, { value }) => (guest(s) || asks(s) ? go(s, {}, { typed: value }) : no(s, `no name is asked: the link is ${s.at.join}`)),
  named: (s) => (guest(s) ? go(s, {}, {}, send('guest named', { guest: s.typed ?? '' })) : asks(s) ? redeem(s, s.joining, s.typed ?? '') : no(s, `no name is asked: the link is ${s.at.join}`)),
  taken: (s, { why }) => (s.at.join === 'redeeming' ? go(s, { join: 'taken' }, { why }) : no(s, 'no name was sent')),
  admitted: (s, { group }) => (s.at.join === 'redeeming' ? go(s, { join: 'joined' }, {}, send('hold', { group, wants: s.joining.wants, joined: true })) : no(s, 'no name was sent')),
  // Why it did not join, said as the page's failure, in words (a:78).
  unjoined: (s, { said, why }) => go(s, { join: 'unasked' }, {}, send('failed', { said, why })),
};
// Joined, its last words stand until the group is read (a:162, :187).
const said = (s) => (s.at.join === 'joined' && s.at.list === 'unasked' ? (s.joining.known ? 'asking' : 'redeeming') : s.at.join);
export const heading = {
  join: (s, g = s.joining?.group) => ({ asking: ['Opening your link…'], naming: [`Join ${g?.name}`], taken: [`Join ${g?.name}`],
    redeeming: [g?.person ? `Adding this phone to ${g.name} as ${g.person}’s…` : `Joining ${g?.name}…`] })[said(s)],
};

// The name asked, in the group's name. In iPhone Safari, the one browser
// defining `navigator.standalone`, false outside a Home Screen web app, the
// person is first told to join from the Home Screen: each web app there
// keeps a storage of its own, so one added after joining joins again, as
// someone new (the operator's iPhone, iOS 18.7, 2026-10-30). The form is
// drawn again each time it is shown, so its box takes the focus each time
// it is asked, as v1's `input.focus()` (a:136). Pure, called as a function,
// so it is drawn as web/index.html:70-76 has it (web/tests/v2/frame.test.mjs).
const ATTRS = { autocomplete: 'name', maxlength: 64, required: true };
export const NameForm = ({ shown, group = '', keep, why, value, send, on }) => html`<form key=${`${shown} ${why}`} id="join" hidden=${!shown} onSubmit=${(e) => (e.preventDefault(), send(on))}><p id="keep" hidden=${!keep}>${keep && `To keep ${group} on this phone, add this page to your Home Screen first, and join from there.`}</p>${Field({ shape: 'for', id: 'name', words: html`What’s your name? Everyone in <span id="group">${group}</span> sees it.`, value, attrs: ATTRS, send })}<p id="why" class="bad">${why}</p><button>Join</button></form>`;
export const view = { join: ({ s, send }) => NameForm({ value: s.typed, send, on: { type: 'named' }, ...(guest(s) ? { shown: true, group: 'the group you’re visiting' } : { shown: asks(s), group: s.joining?.group?.name, keep: s.standalone === false, why: s.why }) }) };

// The edge: the key made fresh for this link, the issuer dialled as it
// (`connect_to_join`) within PATIENCE, and the ask and the redemption
// posted; the joined group kept, its key sealed (store.js `keep`).
const PATIENCE = 60_000, AWAY = 'The computer that sent this link did not answer. Try again later.', wasm = () => import('../wasm/svrngs_web.js?v=__VERSION__'), trace = (l) => globalThis.__trace?.(l);
const refusal = (words, status) => Object.assign(new Error(words), { said: words, status });
const joining = (status) => (status === 403 ? 'This link lets nobody in. Ask for a new one.' : 'The computer that sent this link cannot let anyone in just now. Try again later.');
let key;
async function posted(w, token, path, body, refused = joining) {
  const dialled = await Promise.race([w.connect_to_join(key, token), new Promise((_, n) => setTimeout(() => n(new Error('no answer')), PATIENCE))]).catch((e) => Promise.reject((trace(`PAGE: the dial failed: ${e}`), refusal(AWAY))));
  const bytes = new TextEncoder().encode(body), [res, text] = await dialled.fetch('POST', path, [['content-type', 'application/json'], ['content-length', String(bytes.length)]], new Blob([bytes]).stream())
    .then(async (r) => [r, await r.text()]).finally(() => dialled.close());
  return res.ok ? text : Promise.reject((trace(`PAGE: ${path} answered ${res.status}: ${text}`), refusal(refused(res.status, text), res.status)));
}
const unjoined = (e) => ({ type: 'unjoined', said: e.said, why: String(e) });
export const io = {
  ask: ({ token }) => wasm().then(async (w) => {
    if (!w.relay_of(token)) throw refusal('This link was made by an older svrngs. Ask whoever sent it for a new one.');
    key = crypto.getRandomValues(new Uint8Array(32));
    const told = await posted(w, token, '/ask', w.ask(key, token)), g = w.believe(told);
    return { type: 'asked', told, group: { id: g.id, name: g.name, person: g.person, maker: g.maker }, why: `the link lets into ${g.id}${g.person ? `, for ${g.person}’s devices` : ''}` };
  }).catch(unjoined),
  // 409: someone in the group goes by the name (bash-person), so another is
  // asked, saying whose it is and how to add this phone if it is theirs.
  admit: ({ token, told, group, name }) => wasm().then(async (w) => {
    let body;
    try { body = w.redemption(key, token, told, name); } catch (e) { return { type: 'taken', why: `${e.message[0].toUpperCase()}${e.message.slice(1)}.` }; }
    await posted(w, token, '/redeem', body, (status, text) => (status === 409 ? `Someone in ${group.name} already goes by ${JSON.parse(text).error}. If that’s you, open your home page on your computer, press “Add your phone or another computer”, and open that link here. Else join under another name.` : joining(status)));
    const person = JSON.parse(body).person;
    await store.keep({ id: group.id, name: group.name, person, token, key, maker: group.maker, since: Math.floor(Date.now() / 1000) });
    trace(`PAGE: joined ${group.id} as ${person}; the key is kept sealed`);
    return { type: 'admitted', group: (await store.groups()).find((g) => g.id === group.id) };
  }).catch((e) => (e.status === 409 ? { type: 'taken', why: e.said } : unjoined(e))),
};
