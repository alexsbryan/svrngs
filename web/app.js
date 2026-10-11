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
// (store.js). Every word a person reads here is DESIGN §1's.
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
// (mnp-2, lend.js `READS`), and the lend's start page goes there. Either
// way this page steps aside, so the browser's Back skips it.
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
import init, {
  ask,
  believe,
  connect,
  connect_to_join,
  endpoint,
  label,
  pass,
  redemption,
  relay_of,
} from './wasm/svrngs_web.js?v=__VERSION__';
import * as store from './store.js?v=__VERSION__';

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
class Said extends Error {}

function show(words, good) {
  heading.textContent = words;
  heading.className = good ? 'good' : 'bad';
}

async function reach(dialling, away) {
  const timeout = new Promise((_, no) => setTimeout(() => no(new Error('no answer')), PATIENCE));
  try {
    return await Promise.race([dialling, timeout]);
  } catch (e) {
    trace(`PAGE: the dial failed: ${e}`);
    throw new Said(away);
  }
}

const joining = (status) =>
  status === 403
    ? 'This link lets nobody in. Ask for a new one.'
    : 'The computer that sent this link cannot let anyone in just now. Try again later.';

// POST `body`, JSON, on `dialled`: the answer's text, or the computer's
// refusal in the words `refused` gives its status.
async function post(dialled, path, body, refused = joining) {
  const bytes = new TextEncoder().encode(body);
  const res = await dialled.fetch(
    'POST',
    path,
    [['content-type', 'application/json'], ['content-length', String(bytes.length)]],
    new Blob([bytes]).stream(),
  );
  const text = await res.text();
  if (res.ok) return text;
  trace(`PAGE: ${path} answered ${res.status}: ${text}`);
  throw new Said(refused(res.status));
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
    const name = await askName(group.name, why);
    let body;
    try {
      body = redemption(key, token, asked, name);
    } catch (e) {
      why = `${e.message[0].toUpperCase()}${e.message.slice(1)}.`;
      continue;
    }
    heading.textContent = `Joining ${group.name}…`;
    issuer = await reach(connect_to_join(key, token), away);
    await post(issuer, '/redeem', body);
    await issuer.close();
    const person = JSON.parse(body).person;
    await store.keep({ id: group.id, name: group.name, person, token, key });
    trace(`PAGE: joined ${group.id} as ${person}; the key is kept sealed`);
    return (await store.groups()).find((g) => g.id === group.id);
  }
}

// What the group's people lend, as the bare command prints it: "Sam's
// <name>", and "<name> (<kind>)" when the two differ.
function what(l) {
  return `${l.person}’s ${l.name === l.kind ? l.name : `${l.name} (${l.kind})`}`;
}

const hex = (bytes) => [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');

// Open `l`, one of `lent`, as the person whose key is `key` in `group`: a
// pass to a fresh key for it and the lends of each kind it reads, handed
// to its origin in the fragment, which no origin is sent (the module docs),
// with whose each is and whose computer did not answer, which its worker
// tells its pages (lend.js `told`). Asked by a lend, at `at`.
function open(group, key, lent, unanswered, l, at) {
  if (!('serviceWorker' in navigator)) {
    said.textContent = `This browser cannot open ${what(l)}.`;
    return;
  }
  said.textContent = `Opening ${what(l)}…`;
  const reads = lent.filter((o) => o !== l && (l.reads ?? []).includes(o.kind));
  const lends = [l, ...reads].map((o) => ({ label: label(o.key, o.name), key: o.key, name: o.name, kind: o.kind, person: o.person, asks: o.asks }));
  const holder = crypto.getRandomValues(new Uint8Array(32));
  const until = Math.floor(Date.now() / 1000) + PASS_LIFE;
  // Each app at an address of its own under this page's host (DESIGN §7),
  // the page's files at its root.
  const back = at ? `&back=${encodeURIComponent(at)}` : '';
  const start = `${store.lendOrigin(location.protocol, lends[0].label, location.host)}/open.html?v=__VERSION__${back}`;
  const handoff = {
    label: lends[0].label,
    lends,
    pass: pass(key, holder, lends[0].label, lends.slice(1).map((o) => o.label), until),
    holder: hex(holder),
    relay: relay_of(group.token) ?? null,
    page: `${location.origin}${location.pathname}`,
    unanswered,
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
  const d = await reach((await dialler()).dial(l.key), away);
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
    said.textContent = 'That app, or what it asks to use, is not lent in your group now.';
    return;
  }
  trace(`PAGE: asking whether ${q.get('app')} may use ${q.get('ask')}`);
  const back = () => open(group, key, lent, unanswered, app, q.get('at') ?? '/');
  show(`Let ${what(app)} use ${what(lend)}?`, true);
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
  for (const l of lent.filter((o) => o.asks)) {
    const line = (words) => shown.appendChild(Object.assign(document.createElement('li'), { textContent: words }));
    try {
      const d = await reach((await dialler()).dial(l.key), '');
      const res = await d.fetch('GET', `/choices/${label(l.key, l.name)}`, [['accept', 'application/json']], null);
      await d.close();
      if (!res.ok) throw new Error(`answered ${res.status}`);
      for (const [app, c] of Object.entries((await res.json()).choices)) {
        const by = lent.find((o) => label(o.key, o.name) === app);
        const item = line(`${by ? what(by) : 'An app'} ${c.said === 'yes' ? 'uses' : 'was stopped using'} ${what(l)}. `);
        if (c.said !== 'yes') continue;
        const stop = Object.assign(document.createElement('button'), { textContent: 'Stop' });
        stop.onclick = () => choose(dialler, l, app, 'stop').then(() => uses(lent, dialler, shown), (e) => item.append(e.message));
        item.append(stop);
      }
    } catch (e) {
      trace(`PAGE: what apps use of ${l.name} not asked: ${e}`);
      line(`What apps use of ${what(l)} cannot be asked now.`);
    }
  }
}

// In `group`: list what its people lend, and open the one `asked` names
// at once, at its path, when a lend's origin asked for it; or ask whether
// an app may use a lend.
async function enter(group, asked) {
  group = await store.opened(group);
  document.title = group.name;
  const key = await store.unseal(group);
  heading.textContent = `Reaching ${group.name}…`;
  const d = await reach(connect(key, group.token), `Your group’s computer did not answer. Try again later.`);
  const res = await d.fetch('GET', PEOPLE_LENDS, [['accept', 'application/json']], null);
  if (!res.ok) {
    trace(`PAGE: ${PEOPLE_LENDS} answered ${res.status}`);
    await d.close();
    throw new Said('Your group’s computer said no.');
  }
  const { lent, unanswered } = await res.json();
  await d.close();
  show(`You’re in ${group.name}, as ${group.person}`, true);
  list.replaceChildren(
    ...lent.map((l) => {
      const item = document.createElement('li');
      const button = document.createElement('button');
      button.textContent = what(l);
      button.onclick = () => open(group, key, lent, unanswered, l);
      item.append(button);
      return item;
    }),
  );
  // One endpoint as the person, bound when first asked: a relay keeps one
  // per key (web/src/lib.rs `endpoint`).
  let bound;
  const dialler = () => (bound ??= endpoint(key, group.token));
  const again = asked?.get('open') && lent.find((l) => label(l.key, l.name) === asked.get('open'));
  if (again) open(group, key, lent, unanswered, again, asked.get('at') ?? '/');
  const using = document.getElementById('uses');
  using.hidden = !lent.some((l) => l.asks);
  using.ontoggle = () => using.open && uses(lent, dialler, document.getElementById('used'));
  said.textContent = [
    lent.length ? '' : `Nobody in ${group.name} lends anything yet.`,
    ...unanswered.map((p) => `${p}’s computer did not answer, so what they lend is not listed.`),
  ]
    .filter(Boolean)
    .join('\n');
  if (asked?.has('ask')) asking(group, key, lent, unanswered, dialler, asked);
}

// The service worker: the runtime that serves what the group's people lend
// (u4-6, sw.js), and the app shell's cache after first load, so a repeat
// visit needs no origin traffic at all (upstream's
// docs/RING_APP_LIBRARY.md: "no traffic after first load"). A module, as
// it imports the dial. Registered after load: joining and the list work
// without it. Asked for before joining awaits anyone, so a person typing
// their name does not miss the load.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker
      .register('./sw.js', { type: 'module' })
      .catch((e) => trace(`PAGE: the service worker did not register: ${e}`));
  });
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
    const asked = /^(open|ask)=/.test(fragment) ? new URLSearchParams(fragment) : null;
    const token = asked ? '' : fragment;
    const kept = await store.groups();
    let group = token ? kept.find((g) => g.token === token) : kept[0];
    if (token && !group) group = await join(token, kept);
    if (!group) throw new Said('Open this page from your group’s link.');
    await enter(group, asked);
  } catch (e) {
    console.error(e);
    show(e instanceof Said ? e.message : 'This browser could not open the page.', false);
  }
}
opening(location.hash.slice(1));
