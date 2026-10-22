// SPDX-License-Identifier: AGPL-3.0-or-later
// The home page (crates/svrngs-node/src/home.rs): the person's groups, the
// people in each and what each lends, from the routes that answer this
// computer's apps (/v1/groups, /v1/lends, /v1/rail/people), and who was
// removed or asked to be, from /v1/groups (v1-keeper-office (b)); the ask at
// /ask; a group on a screen at /?screen=<its id> (`screen`); a link from a
// group's page opened here; what apps use, each with
// Stop; and what this computer lends, each
// with how many it lets in at once, to set, clear or stop (bash-limit). Its
// only writes are the person's yes and stop, to its own origin's /choices,
// their change to what this computer lends, to its /lent, what runs here
// lent (/running, v1ui-lend (b)), a link (/link), a guest's visit stopped
// (/visits, guests-pages),
// a folder opened (/folder), a group made (/group, start-group), one
// joined from a link (/join, arch-command (c)), the shelf's apps brought
// (/shelf), a yes to a carried app's new version (/runs, canon-pin (c)) and
// a lend shown on a TV here or not (/v1/room, room.rs). Its words are
// DESIGN §1's: groups, people, what they lend, computer; its question and
// what apps use and its removals are question.js's, the group's page's too.
import { door, el, named, namesakes, once, opens, owner, question, removal, row, table, unlent, until, uses as used, visiting, whose } from './question.js';

const main = document.getElementById('home');
const button = (text, act) => Object.assign(el('button', text), { onclick: act });

// A route's answer, or its words when it refuses: never read as empty. One
// reading for a read and a write alike.
async function answer(r) {
  const body = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(body.error || `answered ${r.status}`);
  return body;
}
const get = (path) => fetch(path).then(answer);

// `body` sent as JSON, as every write of the page sends it.
const sent = (body) => ({ method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
const post = (path, body) => fetch(path, sent(body)).then(answer);
const said = (lend, app, what) => post('/choices', { lend, app, said: what });

// A leave or a hand-on asked of the node (removal.rs), its answer said in
// `p` beside its button; a leave goes on to the page again, saying so.
const asked = (path, body, p, name) => fetch(path, sent(body)).then((r) => r.json()).then(({ outcome: o }) => (o === 'left' ? location.assign(`/?${new URLSearchParams({ left: name })}`) : p.append(` ${{ handed: `${body.person} keeps the order of removals in ${name} now, so you can leave it.`, decides: `You keep the order of removals in ${name}: choose who keeps it after you, then leave.`, not_in: `You are no longer in ${name}.` }[o] ?? `${name} cannot be changed on this computer now.`}`)), (e) => p.append(` ${name} cannot be changed on this computer now: ${e.message}`));

// The lend's label, the first name of its origin here.
const label = (l) => new URL(l.origin).hostname.split('.')[0];

// What the people of group `g` lend, for the ask; none when it cannot be
// read. `removed` is null when who is in the group cannot be read here:
// never read as not removed (ralph/REVIEW_FINDINGS.md #123).
const lentIn = (g) => (g.removed === false ? get(`/v1/lends?namespace=${encodeURIComponent(g.id)}`).then((l) => l.lent, () => []) : []);


// A lend on a TV here, or not (DESIGN §3, "The room"): said as /v1/room
// says it (room.rs `Served::said`), its button turning it on or off there,
// then drawn again in place from the room's answer, or its refusal said.
// `room` is the error when what is on cannot be read here, said once in
// "On your TV" and never read as none; the button then writes it over.
function tv(l, room, was, p = el('span')) {
  const on = room.find?.((r) => r.key === l.key && r.name === l.name);
  const turn = () => post('/v1/room', { [on ? 'off' : 'on']: { key: l.key, name: l.name } }).then((now) => p.replaceWith(tv(l, now, on)), (e) => p.append(` ${e.message}`));
  p.replaceChildren(` ${on?.said ?? `${was ? 'Turned off for the TV: nobody on this WiFi can open it through this computer now. ' : ''}Show it on a TV here, and anyone on this WiFi can open it as you while svrngs runs.`} `, button(on ? 'Turn it off for the TV' : 'Show it on a TV here', turn));
  return p;
}

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
async function show(g, s, room) {
  if (g.removed === null) return s.append(el('p', `This computer could not read who is in ${g.name ?? 'this group'}.`));
  if (g.removed) return s.append(el('p', `You are no longer in ${g.name ?? 'this group'}. What you already have stays on this computer; nothing new reaches it.`));
  const ns = encodeURIComponent(g.id);
  try {
    const rows = (await get(`/v1/rail/people?namespace=${ns}`)).people;
    const [people, judge] = door(rows);
    // Its screen, in a tab of its own: for a TV, a spare laptop or a projector.
    s.append(people, el('p', '', Object.assign(el('a', `Show ${g.name ?? 'this group'} on a screen`), { href: `/?screen=${ns}`, target: '_blank' })));
    // Where the group's folder is on this disk (v1-onboarding (d)).
    const its = g.name ? `${g.name}’s folder` : 'The group’s folder';
    if (g.folders === null) s.append(el('p', `This computer cannot say where ${its} is.`));
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
    // A guest (guests-link; GUESTS §5): how long asked first, 48 hours
    // chosen to start and nothing longer offered, counted from when the
    // guest opens the link; then the link, for any chat.
    const length = el('select', '', ...[48, 24, 6].map((h) => Object.assign(el('option', `${h} hours`), { value: h * 3600 })));
    const guest = button('Have a guest', () => guest.replaceWith(el('label', 'For how long, from when they open the link: ', length), button('Make a link for your guest', () => post('/link', { group: g.id, device: false, visit: Number(length.value) }).then(({ https }) => shown('Send this link to your guest, in any chat: ', el('code', https), ` They open it in a browser and give their name. For ${length.selectedOptions[0].textContent} from then they can open what you lend, and everyone in ${g.name ?? 'this group'} sees they are your guest. It lets one guest in, once, within a day, while this computer is on. `, copy(https, s.querySelector('.link'))), (e) => shown(e.message)))));
    s.append(el('p', '', made(false, `Make a link to invite someone to ${g.name ?? 'this group'}`), made(true, 'Make a link to add your phone or another computer of yours'), guest), Object.assign(el('p'), { className: 'link' }));
    // Leaving (groups-leave), what stays and what stops said before; whoever
    // keeps the order of removals chooses who keeps it next, beside it.
    const name = g.name ?? 'this group';
    const others = [...new Map(rows.filter((r) => r.device === 'computer' && r.person !== g.person).map((r) => [r.person_key, r])).values()];
    const to = el('select', '', ...others.map((r, i) => Object.assign(el('option', r.person), { value: i })));
    const hand = el('p', `You keep the order of removals in ${name}. Choose who keeps it after you, then leave: `, to, button('Hand it on', () => asked('/v1/handovers', { group: g.id, ...others[to.value] }, hand, name)));
    const leave = el('p', `Leaving ${name} stops it reaching you, and stops what you lend reaching its people. What you have of it stays on this computer, its folder too. Rejoining needs a new invitation, and this computer can’t rejoin yet. `, button(`Leave ${name}`, () => asked('/v1/leaves', { group: g.id }, leave, name)));
    s.append(...(g.keeps && others.length ? [hand] : []), Object.assign(leave, { className: 'leave' }));
    if (g.removals.length) s.append(Object.assign(el('ul', '', ...g.removals.map((r) => el('li', removal(r)))), { className: 'removals' }));
    // Who is visiting (guests-pages; GUESTS §5 "The pages"), from the
    // record, as every member's page shows them: who you have over first
    // (GUESTS's "Your guests", in words that are not events's noun), each
    // with Stop, which says what it does before it does it; then everyone's,
    // by host, with until when. The page is drawn again at the Stop's own
    // time, which is after every act its computer holds, so may be a second
    // on (svrngs-core command.rs `sign_lines`): sooner, Sam reads as here.
    if (!g.visiting) s.append(el('p', `This computer cannot say who is visiting ${name} now.`));
    const stop = (v, li = el('li', `${v.guest}, until ${until(v)}. Stop ends the visit now: what you lend stops opening to ${v.guest}, and everyone in ${name} sees it ended. `)) => (li.append(button(`Stop ${v.guest}’s visit`, () => post('/visits', [g.id, v.id]).then((a) => setTimeout(() => location.reload(), a.ts_unix * 1000 - Date.now()), (e) => li.append(` ${e.message}`)))), li);
    const mine = (g.visiting ?? []).filter((v) => v.yours);
    if (mine.length) s.append(el('h3', 'Who you have over'), Object.assign(el('ul', '', ...mine.map((v) => stop(v))), { className: 'yours' }));
    if (g.visiting?.length) s.append(el('h3', 'Visiting now'), visiting(g.visiting));
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
      how.append(tv(l, room));
      body.append(el('tr', '', el('td', '', Object.assign(el('a', `Open ${named(l)}`), { href: l.origin, target: '_blank' })), el('td', l.group ? named(l) : l.kind), el('td', owner(l, g)), how, el('td', now)));
    }
    // Each computer that did not say, by how (bash-first-minute (c)), a row
    // of its own: what it lends never stale, and never nobody's. One that
    // answered is never said not to have.
    const said = (words) => body.append(el('tr', '', Object.assign(el('td', words), { colSpan: 5 })));
    for (const p of lends.unanswered) said(`${p}'s computer did not answer, so what they lend is not listed.`);
    for (const p of lends.unheard) said(`${p}'s computer has not heard of yours yet, so what they lend is not listed yet.`);
    for (const p of lends.refused) said(`${p}'s computer answered without saying what they lend, so it is not listed.`);
    for (const p of lends.older) said(`${p}'s computer runs an older svrngs, so what they lend is not listed.`);
    if (!body.children.length) said('Nobody lends anything here yet.');
    s.append(table(g, lends, (apps) => post('/shelf', [g.id, apps]), (app) => post('/runs', [g.id, app])));
    return lends.lent;
  } catch (e) {
    s.append(el('p', `This computer cannot read what ${g.name ?? 'this group'} lends: ${e.message}`));
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
      li.append(button(`Lend ${r.kind ?? 'it'} to everyone in your groups`, () => post('/running', { port: r.port, name: typed(name), kind: typed(kind) }).then((l) => { li.replaceChildren(`You lent ${l.name}, at ${at}: everyone in your groups can open it now. ${needs(l)} `); return lending(); }, (e) => li.append(` ${e.message}`))));
    }
  }
}

// What a lend's app form is filled with, the one time its lender turns
// signing people in with svrngs on: the issuer, the client and its secret,
// and the deviations in their words. The secret is shown here and nowhere
// again (SIGN_IN_WITH_SVRNGS §5 "Turning it on").
const signed = (c) => el('p', 'Signing people in with svrngs. In the app’s own settings, choose OpenID Connect, and fill in: issuer ', el('code', c.issuer), ', client id ', el('code', c.client_id), ', client secret ', el('code', c.client_secret), `. ${c.deviations} Copy the secret now: it is not shown again. `);

// What this computer lends, each with how many it lets in at once — its
// person's choice, a number or everyone in their groups — to set, clear,
// sign people in to or stop here, taken by the door at its next request;
// drawn again in place once something running here is lent.
async function lending() {
  const s = document.getElementById('lent') ?? main.appendChild(Object.assign(el('section'), { id: 'lent' }));
  s.replaceChildren(el('h2', 'What you lend'));
  let lent, signing;
  try {
    // An app a group carries is the group's, not this computer's to change.
    ({ lent, signing } = await get('/lent'));
    lent = lent.filter((l) => !l.group);
  } catch (e) {
    return s.append(el('p', `This computer cannot read what you lend: ${e.message}`));
  }
  if (!lent.length) s.append(el('p', 'You lend nothing yet.'));
  const ul = s.appendChild(el('ul'));
  for (const l of lent) {
    const n = l.limit;
    const li = el('li', `${l.name}: ${n ? `up to ${n} ${n === 1 ? 'person' : 'people'}` : 'everyone in your groups'} at once, ${l.using} using it now. `);
    const change = (to) => post('/lent', { lend: l.name, to }).then(() => location.reload(), (e) => li.append(e.message));
    const many = Object.assign(el('input'), { type: 'number', min: 1, value: n ?? '' });
    li.append(el('label', 'How many people at once: ', many), button('Let in this many', () => (/^[1-9][0-9]*$/.test(many.value) ? change({ limit: Number(many.value) }) : li.append('Say how many: 1 or more. '))));
    if (n) li.append(button('Let everyone in', () => change({ limit: null })));
    if (signing?.includes(l.name)) li.append('Signs people in with svrngs. ');
    else li.append(button('Sign people in with svrngs', () => post('/lent', { lend: l.name, to: 'sign-in' }).then((c) => li.append(signed(c)), (e) => li.append(` ${e.message}`))));
    li.append(button('Stop lending', () => change('stop')));
    ul.append(li);
  }
}

// A group on a screen, `/?screen=<its id>`, opened from its section in a tab
// of its own, for a TV, a spare laptop or a projector (the operator, "Wall
// is approved", of the Draft 2 canvas's frames 8-10): its door alone, big,
// each person's lends in their window (question.js `door`), drawn again
// from a read begun a second after the last one began, or once that one
// lands. Read back to back, a 25-computer group's founder spent 180% of a
// core and grew to 442 MiB in 80 s; read once a second, at most 10% and
// 65.5 MiB (demo/runs/twentyfive/2026-10-17/README.md). Each door is drawn
// and judged before it takes the last one's place, so no window reads
// "asking…" again. Beside it, who joined and what was lent since the screen
// opened, at this computer's time, newest first, the newest marked new
// there and in its window: kept by this page alone, so opened again it
// starts over, and never who went off, which the door never says.
function screen(g) {
  [document.body.className, document.title] = ['screen', `${g?.name ?? 'A group'} on a screen`];
  if (!g || g.left || g.removed !== false) return main.append(el('p', 'This computer cannot show that group on a screen: you are not in it here.'));
  const [ns, count, at, list, seen, many] = [encodeURIComponent(g.id), el('p'), el('div'), el('ol'), new Set(), (n, one, more) => `${n} ${n === 1 ? one : more}`];
  section(g).append(count, at, el('aside', '', el('h2', 'Since this screen opened'), el('p', 'Kept on this screen only: it starts over when the screen is opened again.'), list, el('p', 'Nothing new yet. Who joins, and what they lend, shows here as it happens.')), el('footer', '', el('p', 'Lit: answered just now. Dashed: did not answer. Dotted: in from a phone.'), el('p', `Each thing lent opens in a tab as ${g.person}, from this computer.`)));
  let newest;
  // The first read's people and lends were here before the screen opened:
  // seen, and said by no line.
  const read = (first, began = Date.now()) => Promise.all([get(`/v1/rail/people?namespace=${ns}`), get(`/v1/lends?namespace=${ns}`)]).then(([{ people: rows }, lends]) => {
    const [people, judge] = door(rows, lends.lent);
    judge(lends);
    const arrived = [...rows.map((r) => [r.person_key, `${r.person} joined`]), ...lends.lent.filter((l) => !l.group).map((l) => [l.origin, `${l.person} lent ${l.name}`])].filter(([key]) => !seen.has(key) && seen.add(key));
    for (const [key, words] of first ? [] : arrived) {
      list.querySelector('mark')?.remove();
      list.prepend(el('li', '', el('time', new Date().toLocaleTimeString([], { timeStyle: 'short' })), ` ${words} `, el('mark', 'new')));
      newest = key;
    }
    people.querySelector(`a[href="${newest}"]`)?.after(' ', el('mark', 'new'));
    at.replaceChildren(people);
    count.replaceChildren(`${many(people.children.length, 'person', 'people')} · ${people.querySelectorAll('.on').length} on now · ${many(people.querySelectorAll('a').length, 'thing', 'things')} lent`);
  }, (e) => count.replaceChildren(`This computer cannot read ${g.name ?? 'this group'} now: ${e.message}`)).finally(() => setTimeout(read, began + 1000 - Date.now(), false));
  read(true);
}

// `at` (`/` when none) as an address at `origin`, or null: resolved, and
// kept only if it is still there, as open.js judges its way back — a path
// `startsWith` let through, `/\evil.example`, left for another host (#153).
const within = (at, origin) => ((u) => (u.origin === new URL(origin).origin ? u.href : null))(new URL(at ?? '/', origin));

// The ask: in svrngs's words around the lender's, then back to the app.
function ask(lent, q) {
  const [lend, app] = [q.get('lend'), q.get('app')].map((x) => lent.find((l) => label(l) === x));
  if (!lend || !app || !lend.asks) return main.append(el('p', unlent));
  const go = () => location.assign(within(q.get('back'), app.origin) ?? app.origin);
  const s = el('section', '', el('h2', question(app, lend)), el('blockquote', lend.asks));
  s.append(button('Yes', () => said(label(lend), label(app), 'yes').then(go, (e) => s.append(e.message))), button('Not now', go));
  main.append(s);
}

try {
  const { groups } = await get('/v1/groups');
  main.replaceChildren();
  const q = new URLSearchParams(location.search);
  if (location.pathname === '/ask') ask((await Promise.all(groups.map(lentIn))).flat(), q);
  else if (q.has('screen')) screen(groups.find((g) => g.id === q.get('screen')));
  else {
    // Said after a group is made here (start-group), with where its folder
    // could not be made, if it could not.
    const unplaced = q.has('unplaced') ? `Its folder could not be made: ${q.get('unplaced')}. ` : '';
    if (q.has('made')) main.append(el('p', `You made ${q.get('made')}. ${q.get('also') ? `${q.get('also')} ` : ''}${unplaced}Make a link below to invite someone to it.`));
    // Said after a group is joined here (arch-command (c)).
    if (q.has('joined')) main.append(el('p', `You’re in ${q.get('joined')}${q.has('person') ? `, as ${q.get('person')}.` : ' already.'} ${unplaced}`));
    if (q.has('left')) main.append(el('p', `You left ${q.get('left')}. Rejoining needs a new invitation, and this computer can’t rejoin yet.`));
    // With no group, how to make one or join one (start-home; start-group).
    if (!groups.length) main.append(Object.assign(el('p', 'You’re in no group yet. Make one below, or join one from a link someone sent you.'), { className: 'none' }));
    // What is on for a TV here (room.rs), each said in its lend's row and
    // all of it in a section of its own, a lend whose lender is away too.
    const room = await get('/v1/room').catch((e) => e);
    const lent = Promise.all(groups.filter((g) => !g.left).map((g) => show(g, section(g), room)));
    if (room.length || room.message) main.append(Object.assign(el('section', '', el('h2', 'On your TV'), el('p', room.message ?? 'While svrngs runs here, anyone on this WiFi can open these as you. Each stays on the WiFi where you turned it on.'), el('ul', '', ...(room.map?.((r) => el('li', `${r.person ? `${r.person}’s ` : ''}${r.name}:`, tv(r, room))) ?? []))), { id: 'room' }));
    // The groups you left (groups-leave), with what stays of each here.
    const gone = groups.filter((g) => g.left).map((g) => el('p', `${g.name ?? 'A group'}${g.made ? ` (${whose(g.made)})` : ''}: what you have of it stays on this computer${g.folders?.length ? `, its folder at ${g.folders.join(' and ')}` : ''}, and nothing new of it reaches you.`));
    if (gone.length) main.append(Object.assign(el('section', '', el('h2', 'Groups you left'), ...gone), { id: 'left' }));
    // A link from a group's page, `<page>#open=<label>&at=<path>`, pasted
    // (events-links (c)): the lend it names opened at its path in a tab of
    // its own, as the group's page opens it on a phone; the lend's origin
    // says so when the label names nothing lent here.
    const open = el('p', '', el('label', 'The link: ', el('input')), button('Open it', () => {
      const q = new URLSearchParams(open.querySelector('input').value.split('#')[1] ?? '');
      const lend = q.get('open') ?? '';
      const to = /^[a-z0-9]+$/.test(lend) && within(q.get('at'), `http://${lend}.localhost:${location.port}`);
      if (!to) return open.append(' That is not a link to something lent in your groups.');
      window.open(to, '_blank');
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
    // A group joined from a link someone sent, by the svrngs running here
    // as `svrngs join` joins (arch-command (c)): pasted, what it is for
    // asked of the computer that sent it and said on the button before it
    // is pressed, then the page again, saying so; a refusal said below.
    const [link, as] = [['link', 'The link someone sent you: '], ['person', 'Your name in it: ']].map(([n, w]) => el('label', w, Object.assign(el('input'), { name: n })));
    const [join, why] = [el('p', '', link, as), el('p')];
    let asked;
    const who = () => asked[1] ?? as.lastChild.value.trim();
    const go = button('', () => post('/join', { link: link.lastChild.value, person: who() }).then(([joined, person, unplaced]) => location.assign(`/?${new URLSearchParams({ joined, ...(person && { person }), ...(unplaced && { unplaced }) })}`), (e) => why.replaceChildren(e.message)));
    const says = () => join.append(Object.assign(go, { textContent: `Join ${asked[0]}${who() ? ` as ${who()}` : ''}` }));
    link.lastChild.oninput = () => ((l) => (why.replaceChildren('Asking the computer that sent it what it is for…'), get(`/join?${new URLSearchParams({ link: l })}`).then((a) => l === link.lastChild.value && ((asked = a), (as.hidden = Boolean(a[1])), why.replaceChildren(), says()), (e) => l === link.lastChild.value && (go.remove(), why.replaceChildren(e.message)))))(link.lastChild.value);
    as.lastChild.oninput = () => asked && says();
    main.append(Object.assign(el('section', '', el('h2', 'Join a group from a link'), join, why), { id: 'join' }));
    running();
    await lending();
    await uses((await lent).flatMap((l) => l ?? []));
    // Said as the command and install.sh say it: it is kept (start-address).
    main.append(Object.assign(el('p', `Your home page, the same address each time svrngs starts: ${location.origin}/`), { className: 'address' }));
  }
} catch (e) {
  main.replaceChildren(el('p', `This computer cannot read your groups: ${e.message}`));
}
