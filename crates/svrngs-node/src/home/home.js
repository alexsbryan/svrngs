// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page (crates/svrngs-node/src/home.rs): the person's groups, the
// people in each and what each lends, from the routes that answer this
// computer's apps (/v1/groups, /v1/lends, /v1/rail/people), and who was
// removed or asked to be, from /v1/groups (v1-keeper-office (b)); the ask at
// /ask; a link from a group's page opened here; what apps use, each with
// Stop; and what this computer lends, each
// with how many it lets in at once, to set, clear or stop (bash-limit). Its
// only writes are the person's yes and stop, to its own origin's /choices,
// their change to what this computer lends, to its /lent, what runs here
// lent (/running, v1ui-lend (b)), a link (/link),
// a folder opened (/folder) and a group made (/group, start-group). Its words are
// DESIGN §1's: groups, people, what they lend, computer; its question and
// what apps use and its removals are question.js's, the group's page's too.
import { door, el, named, once, opens, question, removal, row, table, unlent, uses as used, whose } from './question.js';

const main = document.getElementById('home');
const button = (text, act) => Object.assign(el('button', text), { onclick: act });

// A route's answer, or its words when it refuses: never read as empty.
async function get(path) {
  const r = await fetch(path);
  const body = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(body.error || `answered ${r.status}`);
  return body;
}

const post = (path, body) => fetch(path, {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify(body),
}).then(async (r) => (r.ok ? r.json().catch(() => ({})) : Promise.reject(new Error((await r.json().catch(() => ({}))).error || `answered ${r.status}`))));
const said = (lend, app, what) => post('/choices', { lend, app, said: what });

// The lend's label, the first name of its origin here.
const label = (l) => new URL(l.origin).hostname.split('.')[0];

// What the people of group `g` lend, for the ask; none when it cannot be
// read. `removed` is null when who is in the group cannot be read here:
// never read as not removed (ralph/REVIEW_FINDINGS.md #123).
const lentIn = (g) => (g.removed === false ? get(`/v1/lends?namespace=${encodeURIComponent(g.id)}`).then((l) => l.lent, () => []) : []);


// The groups a new one shares its name with (/group, the node's one rule;
// #138), each said as question.js's `whose`, in the command's words (cli.rs
// `namesakes_said`): `verb` says it before the group is made or after.
const namesakes = (n, name, verb) => (n.length ? `You’re already in ${n.map((g) => `${g.name} (${whose(g)})`).join(' and ')}. ${verb} ${n.length > 1 ? 'another' : 'a second'} group called ${name}.` : '');

// Each group's section, headed by its name and, so two of one name are
// told apart, who made it and since when (#138); in its colour, derived
// from its id (look.css `data-colour`; the operator, "Derived from the
// group"): nothing to choose, and a group looks the same each time.
function section(g) {
  const s = el('section', '', el('h2', g.name ?? 'A group whose name this computer cannot read', g.made ? el('small', ` (${whose(g.made)})`) : ''));
  s.dataset.colour = [...g.id].reduce((h, c) => (h * 31 + c.charCodeAt(0)) % 12, 0);
  return main.appendChild(s);
}

// Group `g` in its section `s`: its people and folder as soon as they are
// read, then what its people lend once each computer has answered or not,
// so a computer that does not answer holds back that list alone
// (start-home). Answers what they lend.
async function show(g, s) {
  if (g.removed === null) return s.append(el('p', `This computer could not read who is in ${g.name ?? 'this group'}.`));
  if (g.removed) return s.append(el('p', `You are no longer in ${g.name ?? 'this group'}. What you already have stays on this computer; nothing new reaches it.`));
  const ns = encodeURIComponent(g.id);
  try {
    const [people, judge] = door((await get(`/v1/rail/people?namespace=${ns}`)).people);
    s.append(people);
    // Where the group's folder is on this disk (v1-onboarding (d)).
    const its = g.name ? `${g.name}’s folder` : 'The group’s folder';
    if (g.folders === null) s.append(el('p', `Where ${its} is cannot be read on this computer.`));
    // Each opened from here in this computer's file browser (start-home).
    for (const dir of g.folders ?? []) {
      const p = el('p', '', button('Open this folder', () => post('/folder', { group: g.id, dir }).then(() => p.append(' Opened, in a window of its own.'), (e) => p.append(` It could not be opened: ${e.message}.`))));
      s.append(Object.assign(el('p', `${its} is on this computer at ${dir}`), { className: 'folder' }), p);
    }
    // A link made here, as `svrngs link` makes it (start-home), each
    // button saying what it makes; then the link, said in its words, with
    // a button that copies it.
    const shown = (...words) => s.querySelector('.link').replaceChildren(...words);
    const copy = (link, p) => button('Copy the link', () => Promise.resolve().then(() => navigator.clipboard.writeText(link)).then(() => p.append(' Copied.'), () => p.append(' Select it above to copy it.')));
    const made = (device, words) => button(words, () => post('/link', { group: g.id, device }).then(({ client, https }) => shown(device ? 'Open this link on your other computer or phone; it joins as you, so send it to nobody else: ' : 'Send this link to the person you are inviting: ', el('code', https ?? client), https ? ' It opens in a browser, which says how to join from there.' : ' They join with svrngs join and this link.', ' It works for a day, while this computer is on. ', copy(https ?? client, s.querySelector('.link'))), (e) => shown(e.message)));
    s.append(el('p', '', made(false, `Make a link to invite someone to ${g.name ?? 'this group'}`), made(true, 'Make a link to add your phone or another computer of yours')), Object.assign(el('p'), { className: 'link' }));
    if (g.removals.length) s.append(Object.assign(el('ul', '', ...g.removals.map((r) => el('li', removal(r)))), { className: 'removals' }));
    // The index (V1_UI_BUILD §3.1): one table of what is in the group, on
    // any port — each thing by a link that opens it in a tab and says what
    // and whose (question.js `named`); what it is, its lender's word, never
    // a word of svrngs's own (scripts/vocab_check.py); whose; how it opens;
    // and whether it is full now.
    const body = el('tbody', '', row(['Asking each computer in the group what it lends…']));
    s.append(Object.assign(el('table', '', el('thead', '', row(['What', 'What it is', 'Whose', 'How it opens', 'Now'], 'th')), body), { className: 'index' }));
    const lends = await get(`/v1/lends?namespace=${ns}`).catch((e) => {
      judge();
      throw e;
    });
    judge(lends);
    body.replaceChildren();
    // Each with its address for a program, copied by a button that says
    // so, then said (start-home): the clipboard may refuse a page.
    for (const l of once(lends.lent)) {
      const [opened, now] = opens(l, 'In a tab');
      const how = el('td', `${opened}. `, button('Copy its address for a program', () => Promise.resolve().then(() => navigator.clipboard.writeText(l.origin)).then(() => how.append(` Copied: ${l.origin}`), () => how.append(` Copy it from here: ${l.origin}`))));
      if (l.room) how.append(` On your TV too, at ${l.room}`);
      body.append(el('tr', '', el('td', '', Object.assign(el('a', `Open ${named(l)}`), { href: l.origin, target: '_blank' })), el('td', l.kind), el('td', l.person), how, el('td', now)));
    }
    // Each computer that did not say, by how (bash-first-minute (c)), a row
    // of its own: what it lends never stale, and never nobody's. One that
    // answered is never said not to have.
    const said = (words) => body.append(el('tr', '', Object.assign(el('td', words), { colSpan: 5 })));
    for (const p of lends.unanswered) said(`${p}'s computer did not answer, so what they lend is not listed.`);
    for (const p of lends.unheard) said(`${p}'s computer has not heard of this computer yet, so what they lend is not listed yet.`);
    for (const p of lends.refused) said(`${p}'s computer answered without saying what they lend, so it is not listed.`);
    if (!body.children.length) said('Nobody lends anything here yet.');
    s.append(table(g, lends, (apps) => post('/shelf', [g.id, apps])));
    return lends.lent;
  } catch (e) {
    s.append(el('p', `What this group lends cannot be read on this computer: ${e.message}`));
  }
}

// What apps use of what the people of your groups lend, asked of each
// lender now, each with Stop.
async function uses(lent) {
  const s = el('section', '', el('h2', 'What apps use'));
  const stop = (l, app, line) => said(label(l), app, 'stop').then(() => location.reload(), (e) => line.append(e.message));
  await used(lent, label, async (l) => (await get(`/choices?lend=${label(l)}`)).choices, (words) => s.appendChild(el('p', words)), stop);
  // Shown only when there is something to say (start-home).
  if (s.children.length > 1) main.append(s);
}

// What a lend needs first, as /running says from svrngs's table, in a
// sentence each: its own accounts, a setting, or what svrngs keeps back.
const needs = (r) => [r.signs?.accounts && `It keeps its own accounts, so make each person one in it: ${r.signs.accounts}.`, r.signs?.header && `Before it knows who is who, set this in it: ${r.signs.header}.`, r.keeps?.length && `svrngs keeps these parts of it from everyone: ${r.keeps.join(', ')}.`].filter(Boolean).join(' ');

// What runs on this computer and is not lent yet (/running, v1ui-lend
// (b)), looked for only when the person presses Look, as often as they
// like: the look knocks at every address here, seconds of this computer's
// work, so not at each load of the page (v1ui-lend-ask, the operator's
// "On request, with a button"; demo/runs/v1ui-lend/2026-10-14/check-2.out). Each at its address: a
// kind svrngs knows named with what it needs first, any other lent only
// once the person names it and says what it is. One button lends it; the
// row then says what happened, or why not.
function running() {
  const [said, ul] = [el('p', 'This page lists the programs running on this computer only when you ask: looking takes it a few seconds of work.'), el('ul')];
  main.append(Object.assign(el('section', '', el('h2', 'Already running here'), said, el('p', '', button('Look for programs running on this computer', look)), ul), { id: 'running' }));
  async function look() {
    said.replaceChildren('Looking for programs running on this computer…');
    ul.replaceChildren();
    const answer = await get('/running').catch((e) => e);
    if (answer instanceof Error) return said.replaceChildren(`You looked, and what runs on this computer cannot be read: ${answer.message}`);
    const { running: found, named } = answer;
    said.replaceChildren(found.length ? 'You looked just now: the programs running on this computer, each at its address. Lend one, and everyone in your groups opens it in their browser, as themselves, while this computer is on. You can stop lending it below at any time.' : 'You looked just now: nothing else on this computer is running at an address. Start a program, then look again.');
    for (const r of found) {
      const at = `http://localhost:${r.port}`;
      const li = ul.appendChild(el('li', r.kind ? `${r.kind}, at ${at}. ${needs(r)} ` : `Something at ${at}. To lend it, give it a name and say what it is, each one word${named.length ? `; if it is ${named.join(' or ')}, say that` : ''}. `));
      const [name, kind] = r.kind ? [] : [['name', 'Its name: '], ['kind', 'What it is: ']].map(([n, w]) => li.appendChild(el('label', w, Object.assign(el('input'), { name: n }))));
      const typed = (l) => l?.lastChild.value.trim() || null;
      li.append(button(`Lend ${r.kind ?? 'it'} to everyone in your groups`, () => post('/running', { port: r.port, name: typed(name), kind: typed(kind) }).then((l) => { li.replaceChildren(`You lent ${l.name}, at ${at}: everyone in your groups can open it now. ${needs(l)}`); return lending(); }, (e) => li.append(` ${e.message}`))));
    }
  }
}

// What this computer lends, each with how many it lets in at once — its
// person's choice, a number or everyone in their groups — to set, clear or
// stop here, taken by the door at its next request; drawn again in place
// once something running here is lent.
async function lending() {
  const s = document.getElementById('lent') ?? main.appendChild(Object.assign(el('section'), { id: 'lent' }));
  s.replaceChildren(el('h2', 'What this computer lends'));
  let lent;
  try {
    // An app a group carries is the group's, not this computer's to change.
    lent = (await get('/lent')).lent.filter((l) => !l.group);
  } catch (e) {
    return s.append(el('p', `What this computer lends cannot be read on it: ${e.message}`));
  }
  if (!lent.length) s.append(el('p', 'This computer lends nothing.'));
  const ul = s.appendChild(el('ul'));
  for (const l of lent) {
    const n = l.limit;
    const li = el('li', `${l.name}: ${n ? `up to ${n} ${n === 1 ? 'person' : 'people'}` : 'everyone in your groups'} at once, ${l.using} using it now. `);
    const change = (to) => post('/lent', { lend: l.name, to }).then(() => location.reload(), (e) => li.append(e.message));
    const many = Object.assign(el('input'), { type: 'number', min: 1, value: n ?? '' });
    li.append(el('label', 'How many people at once: ', many), button('Let in this many', () => (/^[1-9][0-9]*$/.test(many.value) ? change({ limit: Number(many.value) }) : li.append('Say how many: 1 or more. '))));
    if (n) li.append(button('Let everyone in', () => change({ limit: null })));
    li.append(button('Stop lending', () => change('stop')));
    ul.append(li);
  }
}

// The ask: in svrngs's words around the lender's, then back to the app.
function ask(lent, q) {
  const [lend, app] = [q.get('lend'), q.get('app')].map((x) => lent.find((l) => label(l) === x));
  if (!lend || !app || !lend.asks) return main.append(el('p', unlent));
  const back = q.get('back')?.startsWith('/') && !q.get('back').startsWith('//') ? q.get('back') : '/';
  const go = () => location.assign(new URL(back, app.origin));
  const s = el('section', '', el('h2', question(app, lend)), el('blockquote', lend.asks));
  s.append(button('Yes', () => said(label(lend), label(app), 'yes').then(go, (e) => s.append(e.message))), button('Not now', go));
  main.append(s);
}

try {
  const { groups } = await get('/v1/groups');
  main.replaceChildren();
  if (location.pathname === '/ask') ask((await Promise.all(groups.map(lentIn))).flat(), new URLSearchParams(location.search));
  else {
    // Said after a group is made here (start-group), with where its folder
    // could not be made, if it could not.
    const q = new URLSearchParams(location.search);
    if (q.has('made')) main.append(el('p', `You made ${q.get('made')}. ${q.get('also') ? `${q.get('also')} ` : ''}${q.has('unplaced') ? `Its folder could not be made: ${q.get('unplaced')}. ` : ''}Make a link below to invite someone to it.`));
    // With no group, how to make one or join one (start-home; start-group).
    if (!groups.length) main.append(Object.assign(el('p', 'This computer is in no group yet. Make one below, or join one by the line the page of the link you were sent gives.'), { className: 'none' }));
    const lent = Promise.all(groups.map((g) => show(g, section(g))));
    // A link from a group's page, `<page>#open=<label>&at=<path>`, pasted
    // (events-links (c)): the lend it names opened at its path in a tab of
    // its own, as the group's page opens it on a phone; the lend's origin
    // says so when the label names nothing lent here.
    const open = el('p', '', el('label', 'The link: ', el('input')), button('Open it', () => {
      const q = new URLSearchParams(open.querySelector('input').value.split('#')[1] ?? '');
      const [lend, at] = [q.get('open') ?? '', q.get('at') ?? '/'];
      if (!/^[a-z0-9]+$/.test(lend) || !at.startsWith('/') || at.startsWith('//')) return open.append(' That is not a link to something lent in your groups.');
      window.open(new URL(at, `http://${lend}.localhost:${location.port}`), '_blank');
    }));
    main.append(Object.assign(el('section', '', el('h2', 'Open a link from your group'), open), { id: 'open' }));
    // A group made here as `svrngs new` makes one, by the svrngs running
    // here (start-group): its name and the person's, then the page again,
    // saying so; a refusal said beside the button. A name a group here has
    // already is said as it is typed, and again once made (#138).
    const [name, person] = [['name', 'A name for the group: '], ['person', 'Your name in it: ']].map(([n, w]) => el('label', w, Object.assign(el('input'), { name: n })));
    const [typed, before] = [name.lastChild, el('p')];
    typed.oninput = () => ((n) => get(`/group?${new URLSearchParams({ name: n })}`).then((b) => namesakes(b, n, 'This makes'), (e) => (n ? e.message : '')).then((w) => n === typed.value && before.replaceChildren(w)))(typed.value);
    const make = el('p', '', name, person, button('Make a group', () => post('/group', [name, person].map((l) => l.lastChild.value)).then(([made, unplaced, beside]) => location.assign(`/?${new URLSearchParams({ made, also: namesakes(beside, made, 'This is'), ...(unplaced && { unplaced }) })}`), (e) => make.append(` ${e.message}`))));
    main.append(Object.assign(el('section', '', el('h2', 'Make a group'), make, before), { id: 'make' }));
    running();
    await lending();
    await uses((await lent).flatMap((l) => l ?? []));
    // Said as the command and install.sh say it: it is kept (start-address).
    main.append(Object.assign(el('p', `Your home page, the same address each time svrngs starts: ${location.origin}/`), { className: 'address' }));
  }
} catch (e) {
  main.replaceChildren(el('p', `Your groups cannot be read on this computer: ${e.message}`));
}
