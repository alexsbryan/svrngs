// ported from commonwealth-ai sovereign/apps/ring-runtime/web/app.js@e1a43bcd2
// The page's logic. External (not inline) so a strict CSP can run here
// — see index.html's `script-src 'self' 'wasm-unsafe-eval'`: the wasm module
// needs `wasm-unsafe-eval`, and nothing else needs an allowance.
//
// The version query on the wasm import is stamped at build time (the content
// hash of the module), so a service worker or CDN cache cannot serve a stale
// runtime after a rebuild.
//
// Joining (u4-5; DESIGN §3, "Joining"): a group's link, `<origin>#<token>`,
// opens this page. A link this phone joined by, or the page with no link,
// is in at once: the group kept last opened is listed. A new link is asked
// of its issuer — which group it lets into — and a group this phone is in
// already, by another link, is in too. Otherwise the page asks the
// person's name once, makes the person's key, redeems the link at its
// issuer as a browser, and keeps the key sealed and the group's name
// (store.js). A link made for a person's devices (`svrngs link --device`,
// bash-devices) names that person: the page asks no name and joins as
// them. Every word a person reads here is DESIGN §1's.
//
// Opening a lend (u5-2; DESIGN §3, "Lent apps", calls 1 and 6; §7): the
// page signs a pass — a fresh key may act as this person toward this lend,
// and the lends its lender declared it reads, until PASS_LIFE from now —
// and opens the lend's own origin, handing it the pass, the fresh key, the
// lends and the relay in the fragment: never the person's key, nor the
// link's token, which would let it join. Its origin is
// `<label>.<this page's host>`. A lend's origin whose pass has lapsed,
// or is gone, opens this page at `#open=<label>`, and the page signs again;
// a page that reads a lend opens it at a path by `#open=<label>&at=<path>`
// (mnp-2, lend.js `READS`), and the lend's start page goes there. A lend's
// address opened where its worker is not, a bookmark in a browser that
// never opened it, opens this page there, which goes on to the group's
// page at `#open=<label>&at=<path>&cold` (`cold`). Either way this page
// steps aside, so the browser's Back skips it.
//
// Asking the person (sup-3; DESIGN §3, "What an app gets"): a lent page
// sends its person here at `#ask=<lend>&app=<its label>&at=<path>` (lend.js
// `asking`), and the page asks in the home page's words around the
// lender's. A yes is posted to the lender's door, dialled as the person,
// which no app's page can; yes or not, the app opens again at the path.
// "What apps use" asks each lender that asks, when the person opens it,
// what they said, each yes with Stop. On the page host this page is never
// framed (vercel.json's `frame-ancestors 'none'`): a lent page is a name
// under that host and could frame it at `#ask=` to borrow a click.
import init, { actor, ask, believe, carried_label, connect_to_join, endpoint, is_label, issuer_of, label, links, older, pass, redemption, relay_of, sign_line } from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';
import { acts, agreed, alias, aliased, apart, called, carrying, changes, deciders, door, el, keeping, named, once, opens, question, reading, removal, row, spoken, table, unlent, uses as used, visiting, whose } from './question.js?v=__VERSION__';

const heading = document.getElementById('status');
const said = document.getElementById('response');
const list = document.getElementById('lent');
const form = document.getElementById('join');
// The dial's layers, and the page's own steps, go to the console at debug,
// never onto the page; the lines are kept on `window.__lines` for a test to
// read. Upstream printed them in a "Layers" panel (index.html:42).
const lines = [];
window.__lines = lines;
window.__trace = (l) => {
  lines.push(l);
  console.debug(l);
};
const trace = window.__trace;

// How long a computer has to answer a dial before the page says it did not.
const PATIENCE = 60_000;

// The people's lends, asked of the computer the page dials (u4-5;
// crates/svrngs-node/src/node.rs, PEOPLE_LENDS_ROUTE).
const PEOPLE_LENDS = '/people/lends';

// How long a pass opens a lend, in seconds: its origin asks for another
// after, which the page signs without a word to the person.
const PASS_LIFE = 12 * 60 * 60;

// Why the page stopped, in a person's words.
class Said extends Error {
  constructor(words, status) {
    super(words);
    this.status = status;
  }
}

const show = (words, good) => Object.assign(heading, { textContent: words, className: good ? 'good' : 'bad' });

// Why the page stopped, said: a `Said`'s words, else that it could not.
const failed = (e) => (console.error(e), show(e instanceof Said ? e.message : 'This browser could not open the page.', false));

// A link naming no relay — made by a computer that reached no relay, or
// before relay-park (c) by one with n0's lookup — is refused, as the page
// asks n0 for nothing (relay-default (b)). The relay is never in a
// person's words: they cannot act on its host (v1ui-reach (d)).
function relayed(token) {
  if (!relay_of(token)) throw new Said('This link was made by an older svrngs. Ask whoever sent it for a new one.');
}

// Each of `computers` that did not answer, by whose it is, the link's
// computer by what it did when its person is not known (v1ui-reach (d)).
// Whose computer the page reached or dialled, by its person when known.
const whoseComputer = (c) => (c ? `${c.person}’s computer` : 'The computer that sent your link');
const unanswered = (computers) => `${computers.map((c) => `${whoseComputer(c)} did not answer.`).join(' ')} Try again later.`;

const reach = (dialling, away) => Promise.race([dialling, new Promise((_, no) => setTimeout(() => no(new Error('no answer')), PATIENCE))]).catch((e) => Promise.reject((trace(`PAGE: the dial failed: ${e}`), new Said(away))));

const joining = (status) => (status === 403 ? 'This link lets nobody in. Ask for a new one.' : 'The computer that sent this link cannot let anyone in just now. Try again later.');

// POST `body`, JSON, on `dialled`: the answer's text, or the computer's
// refusal in the words `refused` gives its status and text.
async function post(dialled, path, body, refused = joining) {
  const bytes = new TextEncoder().encode(body);
  const res = await dialled.fetch('POST', path, [['content-type', 'application/json'], ['content-length', String(bytes.length)]], new Blob([bytes]).stream());
  const text = await res.text();
  if (res.ok) return text;
  trace(`PAGE: ${path} answered ${res.status}: ${text}`);
  throw new Said(refused(res.status, text), res.status);
}

// The person's name, asked once, in the group's name.
function askName(group, why) {
  trace(`PAGE: asking the person's name to join ${group}`);
  document.getElementById('group').textContent = group;
  document.getElementById('why').textContent = why ?? '';
  heading.textContent = `Join ${group}`;
  heading.className = '';
  form.hidden = false;
  const input = document.getElementById('name');
  input.focus();
  return new Promise((done) => {
    form.onsubmit = (e) => {
      e.preventDefault();
      form.hidden = true;
      done(input.value);
    };
  });
}

// Join by `token`, a link no group kept was joined by: the group joined,
// kept, or one kept already, which the link lets into.
async function join(token, kept) {
  relayed(token);
  const away = 'The computer that sent this link did not answer. Try again later.';
  const key = crypto.getRandomValues(new Uint8Array(32));
  heading.textContent = 'Opening your link…';
  let issuer = await reach(connect_to_join(key, token), away);
  const asked = await post(issuer, '/ask', ask(key, token));
  await issuer.close();
  const group = believe(asked);
  const known = kept.find((g) => g.id === group.id);
  if (known) {
    trace(`PAGE: this phone is in ${group.id} already, by another link`);
    return known;
  }
  let why;
  for (;;) {
    const mine = group.person;
    if (mine) {
      trace(`PAGE: a link for ${mine}’s devices, so no name is asked`);
      heading.textContent = `Adding this phone to ${group.name} as ${mine}’s…`;
    }
    const name = mine ?? (await askName(group.name, why));
    let body;
    try {
      body = redemption(key, token, asked, name);
    } catch (e) {
      why = `${e.message[0].toUpperCase()}${e.message.slice(1)}.`;
      continue;
    }
    if (!mine) heading.textContent = `Joining ${group.name}…`;
    issuer = await reach(connect_to_join(key, token), away);
    // 409: someone in the group goes by the name (bash-person), so another
    // is asked, saying whose it is and how to add this phone if it is theirs.
    const taken = (status, text) => (status === 409 ? `Someone in ${group.name} already goes by ${JSON.parse(text).error}. If that’s you, open your home page on your computer, press “Add your phone or another computer”, and open that link here. Else join under another name.` : joining(status));
    try {
      await post(issuer, '/redeem', body, taken);
    } catch (e) {
      if (e.status !== 409) throw e;
      why = e.message;
      continue;
    } finally {
      await issuer.close();
    }
    const person = JSON.parse(body).person;
    await store.keep({ id: group.id, name: group.name, person, token, key, maker: group.maker, since: Math.floor(Date.now() / 1000) });
    trace(`PAGE: joined ${group.id} as ${person}; the key is kept sealed`);
    return (await store.groups()).find((g) => g.id === group.id);
  }
}

// A visit (guests-link; GUESTS §5, "How a visit is made"): a link a member
// made for a guest, `<origin>#visit=<token>`. The page asks the guest's
// name once, makes a key fresh for this visit, never one it keeps for a
// group (§3.9), and redeems the link at the member's computer, which
// writes the visit; it keeps the visit apart from every group
// (store.js `keepVisit`), so the link opened again shows it again, or says
// it has ended. Each pass lasts past the visit, so on a lend's page the
// lender's door, which asks the fold, says it has ended (door.rs `guest_of`).
async function visit(token) {
  relayed(token);
  let { key, ...v } = (await store.visitOf(token)) ?? {};
  if (!key) {
    key = crypto.getRandomValues(new Uint8Array(32));
    const asking = askName('the group you’re visiting');
    heading.textContent = 'You’re invited over as a guest';
    const guest = await asking;
    heading.textContent = 'Opening your link…';
    const issuer = await reach(connect_to_join(key, token), 'The computer that sent this link did not answer. Try again later.');
    v = JSON.parse(await post(issuer, '/visit', JSON.stringify({ ask: JSON.parse(ask(key, token)), guest })).finally(() => issuer.close()));
    await store.keepVisit(token, v, key);
  }
  const until = new Date(v.until * 1000).toLocaleString();
  trace(`PAGE: a guest of ${v.host} until ${v.until}, opening ${Object.keys(v.lent).join(', ')}`);
  if (v.until <= Date.now() / 1000) return show(`Your visit as ${v.host}’s guest ended ${until}.`, false);
  const lent = Object.entries(v.lent).map(([name, kind]) => ({ key: issuer_of(token), relay: relay_of(token), name, kind, person: v.host, version: v.version }));
  show(`You’re ${v.host}’s guest until ${until}`, true);
  list.replaceChildren(...lent.map((l) => row([Object.assign(el('button', named(l)), { onclick: () => open({ token }, key, lent, [], l, undefined, v.until + PASS_LIFE) })])));
  said.textContent = lent.length ? '' : `${v.host} lends nothing yet.`;
}

const hex = (bytes) => [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');

// Open `l`, one of `lent`, as the person whose key is `key` in `group`: a
// pass to its origin's holder for it and the lends of each kind it reads,
// handed to its origin in the fragment, which no origin is sent (the
// module docs), with whose each is and whose computer did not answer,
// which its worker tells its pages (lend.js `told`). Asked by a lend, at
// `at`. A guest's pass lasts until `until` (guests-link), past the visit.
async function open(group, key, lent, unanswered, l, at, until = Math.floor(Date.now() / 1000) + PASS_LIFE) {
  if (!('serviceWorker' in navigator)) {
    said.textContent = `This browser cannot open ${named(l)}.`;
    return;
  }
  said.textContent = `Opening ${named(l)}…`;
  // The person's Stop of what it reads as them holds from this pass
  // (capability-what-acts), the last one lapsing within PASS_LIFE.
  const stopped = group.reads?.[label(l.key, l.name)]?.said === 'stop';
  if (stopped) trace(`PAGE: ${l.name}'s reads were stopped here, so its pass names none`);
  const reads = stopped ? [] : lent.filter((o) => o !== l && (l.reads ?? []).includes(o.kind));
  const lends = [l, ...reads].map((o) => ({ label: label(o.key, o.name), key: o.key, relay: o.relay ?? null, name: o.name, kind: o.kind, group: o.group ?? null, person: o.person, asks: o.asks, version: o.version ?? null }));
  // The holder (capability-signed (H); DESIGN §3): one key per lend's
  // origin, kept there as a key no page can copy, so each pass names it
  // again and the phone's acts in the app are one chain. Its bytes go to
  // the origin only when it holds none, and each open when a lend of it is
  // at a computer whose door takes no holder's link (an older svrngs),
  // which the worker then dials as (F2).
  const raw = lends.some((o) => !links(o.version));
  const kept = !raw && (await store.holder(lends[0].label));
  const seed = kept ? null : crypto.getRandomValues(new Uint8Array(32));
  const holder = kept || actor(seed);
  if (raw) trace(`PAGE: a lend of this open is at a computer on an older svrngs, so its holder goes as bytes`);
  if (!kept && !raw) await store.holder(lends[0].label, holder);
  // Each app at an address of its own under this page's host (DESIGN §7),
  // the page's files at its root.
  const back = at ? `&back=${encodeURIComponent(at)}` : '';
  const start = `${store.lendOrigin(location.protocol, lends[0].label, location.host)}/open.html?v=__VERSION__${back}`;
  const handoff = {
    label: lends[0].label,
    lends,
    pass: pass(key, holder, lends[0].label, lends.slice(1).map((o) => o.label), until),
    // An app on a port writes its kind's part of the group's record as you
    // by naming the act (`x-svrngs-append`), under a pass whose app is that
    // part's label, which the fold reads (DESIGN §3, "Lent apps").
    // A guest's open names no group, so writes none as a member.
    writes: l.group || !group.id ? null : pass(key, holder, carried_label(l.key, l.kind, group.id), [], until),
    group: group.id,
    holder,
    seed: seed && hex(seed),
    raw,
    relay: relay_of(group.token) ?? null,
    page: `${location.origin}${location.pathname}`,
    // By name, as a computer's /svrngs-reads tells a lent page (surface.rs `reads`).
    unanswered: unanswered.map((u) => u.person ?? u),
  };
  trace(`PAGE: opening ${handoff.label} at ${start}, a pass to ${lends.length} lend(s) until ${until}`);
  const json = new TextEncoder().encode(JSON.stringify(handoff));
  const fragment = btoa(String.fromCharCode(...json)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  location[at === undefined ? 'assign' : 'replace'](`${start}#${fragment}`);
}

// The person's answer `said` for the app labelled `app` on `l`, posted to
// its lender's door, dialled as the person from `dialler`'s endpoint.
async function choose(dialler, l, app, said) {
  const away = `${l.person}’s computer did not answer. Try again later.`;
  const d = await reach((await dialler()).dial(l.key, l.relay ?? null), away);
  try {
    await post(d, `/choices/${label(l.key, l.name)}`, JSON.stringify({ app, said }), () => `${l.person}’s computer did not keep your answer.`);
  } finally {
    await d.close();
  }
}

// Ask the person whether the app `q` names may use the lend it names,
// then open the app again at `q`'s path, whatever they said.
function asking(group, key, lent, unanswered, dialler, q) {
  const [lend, app] = [q.get('ask'), q.get('app')].map((x) => lent.find((l) => label(l.key, l.name) === x));
  if (!lend?.asks || !app) {
    trace(`PAGE: asked whether ${q.get('app')} may use ${q.get('ask')}, not both lent here or not asking`);
    said.textContent = unlent;
    return;
  }
  trace(`PAGE: asking whether ${q.get('app')} may use ${q.get('ask')}`);
  const back = () => open(group, key, lent, unanswered, app, q.get('at') ?? '/');
  show(question(app, lend), true);
  document.getElementById('words').textContent = lend.asks;
  const form = document.getElementById('ask');
  form.hidden = false;
  document.getElementById('not').onclick = back;
  form.onsubmit = async (e) => {
    e.preventDefault();
    form.hidden = true;
    try {
      await choose(dialler, lend, q.get('app'), 'yes');
      trace(`PAGE: the yes kept at ${lend.person}'s computer`);
      back();
    } catch (err) {
      show(err instanceof Said ? err.message : 'Your answer could not be sent.', false);
    }
  };
}

// What apps use of what lends ask, asked of each lender now, each yes
// with Stop.
async function uses(lent, dialler, shown) {
  shown.replaceChildren();
  const line = (words) => shown.appendChild(Object.assign(document.createElement('li'), { textContent: words }));
  const choicesOf = async (l) => {
    const d = await reach((await dialler()).dial(l.key, l.relay ?? null), `${l.person}’s computer did not answer.`);
    const res = await d.fetch('GET', `/choices/${label(l.key, l.name)}`, [['accept', 'application/json']], null);
    // The body is read off the dial, so read whole before it closes: a
    // body that comes after its head is cut off by the close (bash-film-load).
    const answer = res.ok ? await res.json() : null;
    await d.close();
    if (!res.ok) throw new Error(`answered ${res.status}`);
    return answer.choices;
  };
  const traced = (l) => choicesOf(l).catch((e) => (trace(`PAGE: what apps use of ${l.name} not asked: ${e}`), Promise.reject(e)));
  const stop = (l, app, item) => choose(dialler, l, app, 'stop').then(() => uses(lent, dialler, shown), (e) => item.append(e.message));
  await used(lent, (o) => label(o.key, o.name), traced, line, stop);
}

// `group` dialled as the person whose key is `key`, through the first to
// answer of the computer that made its link and each the group's last
// answer named as lending (v1ui-reach (c)), all dialled at once, each
// within its cap (dial.rs `LINK_WITHIN`, `DIAL_WITHIN`), so none is waited
// out before the next is asked; one answering late is closed (v1ui-race
// (a)). Each from `dialler`'s one endpoint, as a relay keeps one per key.
// None whose key is in `tried`, the link's computer by its issuer's.
// The dial, and whose computer it is when not the link's.
async function reachGroup(group, key, dialler, tried = []) {
  relayed(group.token);
  const link = (group.computers ?? []).find((c) => c.key === issuer_of(group.token));
  const others = (group.computers ?? []).filter((c) => c !== link && !tried.includes(c.key));
  const linked = !tried.includes(issuer_of(group.token));
  trace(`PAGE: dialling ${linked ? "the link's computer and " : ''}${others.length} other computer(s) at once`);
  const tries = [
    ...(linked ? [dialler().then((ep) => ep.dial_link()).then((d) => [d])] : []),
    ...others.map((c) => dialler().then((ep) => ep.dial(c.key, c.relay)).then((d) => [d, c])),
  ];
  const [d, c] = await reach(Promise.any(tries), unanswered([...(linked ? [link] : []), ...others]));
  trace(`PAGE: reached ${group.name} through ${c ? `${c.person}'s` : "the link's"} computer`);
  tries.forEach((t) => t.then(([o]) => o !== d && (trace('PAGE: a computer answered late, so closed'), o.close()), () => {}));
  return [d, c];
}

// The group's name, with who made it and since when this phone has held
// it, so two of one name are told apart (#138); its name alone for a
// group kept before.
const titled = (group) => `${group.name}${group.maker ? ` (${whose(group)})` : ''}`;

// One tab at a time reaches a group (groups-tabs (b); #176). Each tab
// binds an endpoint as the person, and a relay keeps one connection per
// key (iroh-relay-1.0.2/src/server/clients.rs:80-92): a second tab's set
// the first's aside, which then said every computer it dialled for a
// minute did not answer (demo/runs/tabs/2026-10-16/README.md). So the tab
// holding the group's lock reaches it, and lets it go when it fails or
// closes. One opened while another holds it says so and offers to take
// it (`steal`); a tab it is taken from opens again, and says so too. A
// browser with no locks reaches the group from each tab, as before.
function hold(group, asked, steal = false, joined = false) {
  if (!navigator.locks) return (trace('PAGE: this browser has no locks, so each tab reaches the group'), enter(group, asked, joined));
  return new Promise((entered, failed) => {
    const reached = (lock) => {
      if (!lock) return entered(elsewhere(group, asked));
      trace(`PAGE: this tab holds ${group.id}${steal ? ', taken from another tab' : ''}`);
      return enter(group, asked, joined).then(() => (entered(), new Promise(() => {})), failed);
    };
    navigator.locks.request(`svrngs ${group.id}`, steal ? { steal } : { ifAvailable: true }, reached).catch((e) => {
      if (e.name !== 'AbortError') return failed(e);
      trace(`PAGE: another tab took ${group.id}, so this one opens again`);
      location.reload();
    });
  });
}

// `group` open in another tab of this browser: said, with a button that
// takes it here.
function elsewhere(group, asked) {
  trace(`PAGE: ${group.id} is open in another tab`);
  document.title = group.name;
  show(`You have ${titled(group)} open in another tab`, true);
  const at = document.getElementById('door');
  const here = Object.assign(el('button', 'Open it here'), { onclick: () => (at.replaceChildren(), hold(group, asked, true).catch(failed)) });
  at.replaceChildren(el('p', `One tab at a time reaches ${group.name}. Open it here, and the other tab stops reaching it.`), here);
}

// In `group`: list what its people lend, and open the one `asked` names
// at once, at its path, when a lend's origin asked for it; or ask whether
// an app may use a lend.
async function enter(group, asked, joined) {
  group = await store.opened(group);
  document.title = group.name;
  const key = await store.unseal(group);
  heading.textContent = `Reaching ${group.name}…`;
  // One endpoint as the person, bound when first asked: a relay keeps one
  // per key (web/src/lib.rs `endpoint`).
  let bound;
  const dialler = () => (bound ??= endpoint(key, group.token));
  // What the group lends, asked of the first computer to answer; when its
  // answer fails, as a computer whose person left still answers the dial
  // and no more (#205), of the next to answer of those not yet asked. Only
  // once every one has been asked does the first failure stop the page.
  const lends = (tried) => reachGroup(group, key, dialler, tried).then(([d, via]) => d.fetch('GET', PEOPLE_LENDS, [['accept', 'application/json']], null).then((res) => [d, via, res], (e) => (
    trace(`PAGE: ${PEOPLE_LENDS} failed through ${whoseComputer(via)}, so the next to answer is asked: ${e}`), d.close(),
    lends([...tried, via?.key ?? issuer_of(group.token)]).catch(() => Promise.reject(e)))));
  const [d, via, res] = await lends([]);
  const whom = whoseComputer(via ?? (group.computers ?? []).find((c) => c.key === issuer_of(group.token)));
  if (!res.ok) {
    const why = res.headers.get('x-svrngs-refused');
    trace(`PAGE: ${PEOPLE_LENDS} answered ${res.status}, refused ${why}`);
    await d.close();
    if (why === 'removed') throw new Said(`You are no longer in ${group.name}.`);
    throw new Said(`${whom} said no when you asked what ${group.name} lends.`);
  }
  const { version, lent, unanswered, unheard, refused, older: behind, removals, visiting: on, people, shelf, changed } = await res.json();
  await d.close();
  // One version check (arch-version; the node's version.rs): a computer
  // running an older svrngs is said so once, by whose it is, and nothing
  // of its answer is read field by field.
  trace(`PAGE: ${PEOPLE_LENDS} answered, at version ${version}`);
  if (older(version)) throw new Said(`${whom} runs an older svrngs, so this page cannot show what ${group.name} lends. Installing svrngs again there updates it.`);
  // The computers that lend, kept to reach the group through next time.
  const computers = new Map(lent.map((l) => [l.key, { key: l.key, relay: l.relay ?? null, person: l.person }]));
  group = await store.opened({ ...group, computers: [...computers.values()] });
  apart(lent, people);
  // Who was removed or asked to be, shown to everyone in the group
  // (v1-keeper-office (b)); `null` when the computer could not read it.
  const gone = (removals ?? [null]).map((r) => el('li', r ? removal(r, people) : `Who was removed from ${group.name} cannot be read just now.`));
  document.getElementById('removals').replaceChildren(...gone);
  // Who is visiting now, as the home page lists them (guests-pages;
  // question.js `visiting`), `null` when the computer could not read it;
  // a guest's Stop is on the home page of the computer that made their link.
  // Yours by your first key (keys-names), from a computer at version 2 by name.
  const you = people.find((r) => r.key === actor(key))?.person_key;
  const mine = on?.some((v) => (v.host_key ? v.host_key === you : v.host === group.person)) ? [el('p', 'You stop your guest’s visit on the home page of the computer you made their link on.')] : [];
  document.getElementById('visitors').replaceChildren(...(on ? (on.length ? [el('h2', 'Visiting now'), visiting(on, people), ...mine] : []) : [el('p', `Who is visiting ${group.name} cannot be read just now.`)]));
  // The heading says only that you are in; who made the group and since
  // when, which tells two of one name apart (#138), heads its people below
  // the group's things: in the heading they were words read before the
  // first tap (the fifth set, Ana 60-70 words of her 20;
  // demo/xenia/findings/wordy-the-group-s-page.md).
  show(`You’re in ${group.name}, as ${group.person}${via ? `, through ${via.person}’s computer` : ''}`, true);
  // The door and the index as the home page draws them (question.js;
  // V1_UI_BUILD §3.1), judged by the answer this computer gave: whose
  // computer did not answer it is drawn off.
  const [drawn, judge] = door(people);
  judge({ unanswered });
  document.getElementById('door').replaceChildren(el('p', titled(group)), drawn);
  // The shelf, brought through the computer the page reaches (v1ui-shelf
  // (c)), and a yes to an app's new version through it, under "What acts
  // as you" (canon-pin (c); capability-what-acts).
  const through = (path) => async (body) => {
    const [b] = await reachGroup(group, key, dialler);
    return JSON.parse(await post(b, path, JSON.stringify([group.id, body]), (_, text) => JSON.parse(text).error).finally(() => b.close()));
  };
  document.getElementById('shelf').replaceChildren(table(group, { lent, shelf }, through('/shelf'), joined));
  // What acts as you, as the home page says it (question.js `acts`). The
  // yes to a lend opening friends' lends as the person is its open here,
  // kept with the group, `reads`, which its Stop changes (`saying`).
  const [acting, add] = acts({ opens: `A Stop here holds from the next pass this phone signs for the app, within ${PASS_LIFE / 3600} hours.` });
  document.getElementById('acts').replaceWith(acting);
  const saying = (l, what) => (l.reads?.length ? store.opened({ ...group, reads: { ...group.reads, [label(l.key, l.name)]: { said: what, at: Math.floor(Date.now() / 1000) } } }).then((g) => (group = g)) : Promise.resolve());
  for (const l of once(lent).filter((o) => o.group)) add('reads', el('p', carrying(l.kind, shelf)));
  // A yes is this phone's own (capability-signed): the computer it reaches
  // names the act, and the phone signs it with its own key as its chain's
  // next line, kept with the group before it is sent, so no seq is signed
  // twice, and carries every line not yet taken.
  const yes = async (app) => {
    const { sign } = await through('/runs')(app);
    const at = group.line ?? { seq: -1, id: null };
    const line = JSON.parse(sign_line(key, group.id, Math.floor(Date.now() / 1000), at.seq + 1, at.id, JSON.stringify(sign)));
    group = await store.opened({ ...group, line: { seq: line.seq, id: line.id }, unsent: [...(group.unsent ?? []), line] });
    for (const l of group.unsent) await through('/carry')(l);
    group = await store.opened({ ...group, unsent: [] });
  };
  const me = JSON.parse(ask(key, group.token)).key;
  const news = changes(group, { changed }, yes, people, me);
  if (news.length) add('reads', ...news);
  for (const l of lent.filter((o) => o.reads?.length && group.reads?.[label(o.key, o.name)])) {
    const c = group.reads[label(l.key, l.name)];
    const p = el('p', reading(l, c));
    const stop = () => saying(l, 'stop').then(() => p.replaceChildren(`${named(l)} stops opening your friends’ ${l.reads.map(spoken).join(' and ')} as you within ${PASS_LIFE / 3600} hours, when the pass it holds ends.`));
    if (c.said === 'yes') p.append(Object.assign(el('button', 'Stop'), { onclick: stop }));
    add('opens', p);
  }
  add('knows', el('p', 'What you open knows your name.'));
  // This phone's key, as the ask a link's issuer is sent names it.
  add('decides', el('p', `${deciders(people, me, 'browser')}.`));
  // How this reaches you (IN_THE_BROWSER §3), for someone at a computer
  // who likes to see how a thing works; on a phone it is left out of the
  // first look, which Ana spends on the group's things (join_page.rs).
  document.getElementById('how').hidden = !computer();
  // The index, one line each on a phone: the open, what it is for and what
  // opening it lets it do (`agreed`), and whether it is full now. Whose it
  // is the open names (`named`), and each opens in this tab, so no column
  // says either: with the head they were words Ana read before her first
  // tap (the fifth set; demo/xenia/findings/wordy-the-group-s-page.md).
  // Last, the address to keep (svrn-docs rings/current/IN_THE_BROWSER.md
  // §4): its alias at this page, or, where two share one, its label, which
  // never collides; the alias as the page reads it, a space as `+`.
  const kept = (l, a = alias(l, lent, people)) => `${location.origin}${location.pathname}#open=${aliased(a, lent, people).length === 1 ? encodeURIComponent(a).replaceAll('%20', '+').replaceAll('%40', '@') : label(l.key, l.name)}`;
  list.replaceChildren(
    ...once(lent).map((l) => {
      const button = Object.assign(el('button', named(l)), { onclick: () => saying(l, 'yes').then(() => open(group, key, lent, unanswered, l)) });
      return row([el('span', '', button, ` ${agreed(l, shelf, group)}${opens(l, 'Here, in this tab')[1] === 'full' ? ' Full now.' : ''} `, keeping(kept(l), !computer()))]);
    }),
  );
  // What `#open=` names (DESIGN §3, "Lent apps", call 1): a lend by its
  // label, else by its alias (question.js `alias`), one of what the group
  // lends now; none, or more than one, is said, never a nearest match.
  const wanted = asked?.get('open');
  const byLabel = lent.filter((l) => label(l.key, l.name) === wanted);
  const by = byLabel.length || !wanted ? byLabel : aliased(wanted, lent, people);
  const missed = !wanted || by.length === 1 ? '' : by.length ? `${by.length} things in ${group.name} go by ${wanted}. Open the one you mean from the list.` : is_label(wanted) ? `What that address opens is not lent in ${group.name} now.` : `Nothing in ${group.name} goes by ${wanted} now.`;
  if (wanted) trace(`PAGE: #open=${wanted} names ${by.length} of what ${group.name} lends${byLabel.length ? ', by its label' : ', by its alias'}`);
  const again = by.length === 1 && by[0];
  const its = again && label(again.key, again.name);
  // A lend's origin holding no key its pass names asks for a new one
  // (open.js), never from a window another page opened, which could read
  // its bytes as they arrive.
  if (again && asked.has('anew') && !window.opener) await store.holder(its, null);
  // One opened here before its open was kept as the yes is that yes.
  if (again && !group.reads?.[its]) await saying(again, 'yes');
  if (again) open(group, key, lent, unanswered, again, asked.get('at') ?? '/');
  const using = document.getElementById('uses');
  using.hidden = !lent.some((l) => l.asks);
  if (!using.hidden) add('opens', using);
  using.ontoggle = () => using.open && uses(lent, dialler, document.getElementById('used'));
  // Each computer that did not say by how (bash-first-minute (c)), and
  // nobody lending said only when every computer did; each person by first
  // key, told apart as `called` does (keys-names), from a computer at
  // version 2 by name.
  const how = [[unanswered, 'did not answer, so what they lend is not listed'], [unheard, 'has not heard of the computer you reached yet, so what they lend is not listed yet'], [refused, 'answered without saying what they lend, so it is not listed'], [behind, 'runs an older svrngs, so what they lend is not listed']];
  const unlisted = how.flatMap(([ps, why]) => ps.map((p) => `${called(people, p.person_key) ?? p.person ?? p}’s computer ${why}.`));
  said.textContent = [missed, ...(lent.length || unlisted.length ? unlisted : [`Nobody in ${group.name} lends anything yet.`])].filter(Boolean).join('\n');
  if (asked?.has('ask')) asking(group, key, lent, unanswered, dialler, asked);
}

// On a computer, the one line that gets svrngs and joins it by this link
// (v1-onboarding (c); DESIGN §7), with a button that copies it: composed
// here from the `#` part, which no request carries. With no token, a
// founder's line, the page's address in the link's place, which joins
// nothing and opens the home page, where a group is made (start-group).
// A phone's pointer is coarse, and it has no terminal: no line there.
const computer = () => matchMedia('(pointer: fine)').matches;
function offer(token) {
  if (!computer() || !/^[A-Za-z0-9_-]*$/.test(token)) {
    trace('PAGE: no line offered: not a computer, or not a token');
    return;
  }
  const page = `${location.origin}${location.pathname}`;
  line(token ? 'joins' : 'founds', `curl -fsSL ${page}install.sh | sh -s -- '${page}${token && `#${token}`}'`);
}

// The paragraph `words` shown on a computer, and `text` beneath it with a
// button that copies it.
function line(words, text) {
  document.getElementById(words).hidden = false;
  document.getElementById('line').textContent = text;
  const copy = document.getElementById('copy');
  copy.onclick = () => navigator.clipboard.writeText(text).then(() => (copy.textContent = 'Copied'), (e) => (trace(`PAGE: the line was not copied: ${e}`), (copy.textContent = 'Select the line and copy it')));
  document.getElementById('computer').hidden = false;
  trace(`PAGE: the line offered, for a computer: ${words}`);
}

// The service worker: the runtime that serves what the group's people lend
// (u4-6, sw.js), and the app shell's cache after first load, so a repeat
// visit needs no origin traffic at all (upstream's
// docs/RING_APP_LIBRARY.md: "no traffic after first load"). A module, as
// it imports the dial. Registered after load: joining and the list work
// without it. Asked for before joining awaits anyone, so a person typing
// their name does not miss the load.
// An address typed or tapped on this page, such as an alias kept, is a
// new fragment on the same page, which the browser opens without loading
// it: so the page loads again, to open what it asks.
window.addEventListener('hashchange', () => (trace('PAGE: the address changed here, so the page opens it'), location.reload()));
if ('serviceWorker' in navigator) window.addEventListener('load', () => navigator.serviceWorker.register('./sw.js', { type: 'module' }).catch((e) => trace(`PAGE: the service worker did not register: ${e}`)));

// A lend's address with nothing serving it in this browser (svrn-docs
// rings/current/IN_THE_BROWSER.md §4; DESIGN §3, "Lent apps", call 1): its
// worker is not here, as in a browser that never opened it or cleared it,
// so the page's host answered with this page. It sends the browser to the
// group's page, this host past its first name, at `#open=<label>&at=<path>`,
// which signs a pass and comes back at the path, or says why it cannot
// (`cold` asked). At most once within ONCE from one tab, marked in this
// origin's session: a lend whose worker never takes its pass comes back
// here, and is said, never sent on again.
const ONCE = 60_000;
function cold() {
  const [at, last] = [location.href.slice(location.origin.length), Number(sessionStorage.getItem('svrngs cold'))];
  if (Date.now() - last < ONCE) {
    trace(`PAGE: a lend's address with nothing serving it, sent to the group's page ${Date.now() - last} ms ago, so not again`);
    throw new Said('This could not be opened. Open it again from your group’s page.');
  }
  sessionStorage.setItem('svrngs cold', String(Date.now()));
  const page = `${location.protocol}//${location.host.slice(location.host.indexOf('.') + 1)}/`;
  trace(`PAGE: a lend's address with nothing serving it: to the group's page, ${page}, to come back at ${at}`);
  location.replace(`${page}#${new URLSearchParams({ open: store.labelOf(location.hostname), at })}&cold`);
}

// The link's fragment is its token, `<origin>#<token>`, which svrngs-core
// parses as the command does (DESIGN §3, call 5). The fragment is never
// sent to this page's origin, which is why the link rides there and not in
// the query.
//
// Opened from a function the module does not await: WebKit wedged the tab
// for good when a person left the page while its module still awaited the
// wasm's fetch at its top, then opened it again — the next navigation never
// reached the origin (u4-7; ralph/REVIEW_FINDINGS.md has the runs).
//
// A lend's origin asking for another pass opens `#open=<label>`, a
// page opening a lend it reads `#open=<label>&at=<path>`, and one asking
// its person `#ask=<lend>&app=<label>&at=<path>`.
async function opening(fragment) {
  try {
    // The wasm by its one URL, from the cache the worker reads it from
    // too (store.js `runtime`, u4-9).
    await init({ module_or_path: store.runtime() });
    if (is_label(store.labelOf(location.hostname))) return cold();
    if (fragment.startsWith('visit=')) return await visit(fragment.slice('visit='.length));
    const asked = /^(open|ask)=/.test(fragment) ? new URLSearchParams(fragment) : null;
    const token = asked ? '' : fragment;
    if (token) offer(token);
    const kept = await store.groups();
    let group = token ? kept.find((g) => g.token === token) : kept[0];
    const joined = Boolean(token && !group);
    if (joined) group = await join(token, kept);
    if (!group && !asked) offer('');
    // A link into a lend on a computer keeping no group here, its person's
    // svrngs elsewhere on it: opened from its home page (events-links (c)).
    if (!group && asked?.has('open') && computer()) return (line('opens', location.href), show('This link opens in svrngs on your computer.', true));
    // A lend asked for in a browser holding no group, a cold address's
    // among them: said why, and what to do.
    if (!group) throw new Said(asked?.has('open') ? 'This browser hasn’t joined your group yet, so it can’t open this. Open your group’s link from the chat here first, then this again.' : 'Open this page from your group’s link.');
    await hold(group, asked, false, joined);
  } catch (e) {
    failed(e);
  }
}
opening(location.hash.slice(1));
