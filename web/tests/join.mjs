// The browser half of crates/svrngs/tests/join_page.rs (DESIGN §3,
// "Joining"; u4-5): a phone opening a group's link. It serves the built
// page on 127.0.0.1, as `localhost`, whose every name this computer
// answers, as svrnme.sh's are (u5-2), and prints its origin as one JSON line; the Rust half
// prints a link on that origin with `svrngs new --origin` and answers with
// one JSON line on stdin, `{"link": ..., "name": ...}`. Then, in headless
// `engine` (page.mjs), it opens the link and types the name whenever the page asks for
// one; reads every storage the page can write for the person's key; opens
// the link again, then the page alone; opens the first lend listed, at its
// own origin, `<label>.localhost`, which its worker serves (u5-2); and
// prints what it saw as one JSON line.
//
//   node web/tests/join.mjs '{"engine": "chromium"|"webkit", "site": "<dir>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its browser is absent; 1 on anything else.
import { createInterface } from 'node:readline';
import { launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://localhost:${server.address().port}/`;
console.log(JSON.stringify({ origin }));
const input = createInterface({ input: process.stdin });
const told = await new Promise((done) => input.once('line', done));
input.close();
// `taken`, when told, is typed first: a name someone in the group goes by,
// which the page refuses and asks again, saying why (bash-person).
const { link, name, taken } = JSON.parse(told);

// In the page: the person's key as the page's store unseals it, and where
// else its 32 bytes appear — raw, hex, base64 or as a list of numbers — in
// every IndexedDB database's every record, in localStorage and in
// sessionStorage; and whether each CryptoKey held is extractable.
async function scan() {
  const app = document.querySelector('script[type=module]').src;
  const store = await import(new URL(`./store.js${new URL(app).search}`, app).href);
  const [group] = await store.groups();
  const key = await store.unseal(group);
  const hex = [...key].map((b) => b.toString(16).padStart(2, '0')).join('');
  const b64 = btoa(String.fromCharCode(...key));
  const spelled = [hex, b64, b64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, ''), key.join(',')];
  const holds = (bytes) => {
    for (let i = 0; i + key.length <= bytes.length; i++) {
      if (key.every((b, j) => bytes[i + j] === b)) return true;
    }
    return false;
  };
  const found = [];
  const extractable = [];
  const walk = (where, v) => {
    if (v instanceof CryptoKey) return extractable.push(v.extractable);
    if (v instanceof ArrayBuffer) return holds(new Uint8Array(v)) && found.push(where);
    if (ArrayBuffer.isView(v)) {
      return holds(new Uint8Array(v.buffer, v.byteOffset, v.byteLength)) && found.push(where);
    }
    if (typeof v === 'string') return spelled.some((s) => v.includes(s)) && found.push(where);
    if (Array.isArray(v) && v.every((n) => Number.isInteger(n))) return holds(v) && found.push(where);
    if (v && typeof v === 'object') for (const [k, x] of Object.entries(v)) walk(`${where}.${k}`, x);
  };
  const all = (db, name, how) =>
    new Promise((done, fail) => {
      const req = db.transaction(name).objectStore(name)[how]();
      req.onsuccess = () => done(req.result);
      req.onerror = () => fail(req.error);
    });
  const dbs = [];
  for (const { name } of await indexedDB.databases()) {
    const db = await new Promise((done, fail) => {
      const req = indexedDB.open(name);
      req.onsuccess = () => done(req.result);
      req.onerror = () => fail(req.error);
    });
    for (const s of db.objectStoreNames) {
      const keys = await all(db, s, 'getAllKeys');
      const values = await all(db, s, 'getAll');
      dbs.push(`${name}/${s}: ${values.length}`);
      keys.forEach((k, i) => walk(`indexedDB ${name}/${s}[${JSON.stringify(k)}]`, [k, values[i]]));
    }
    db.close();
  }
  for (const [label, area] of [['localStorage', localStorage], ['sessionStorage', sessionStorage]]) {
    for (let i = 0; i < area.length; i++) {
      const k = area.key(i);
      walk(`${label}[${k}]`, `${k}=${area.getItem(k)}`);
    }
  }
  return { key: hex, found, extractable, dbs };
}

const { context, close } = await launch(args.engine);
const logs = [];
try {
  // Every heading the page shows, in order, so a step said and gone — a
  // device link's "Adding this phone…" (bash-devices) — is read too.
  await context.addInitScript(() => {
    window.__said = [];
    new MutationObserver(() => {
      const s = document.getElementById('status')?.textContent;
      if (s && window.__said.at(-1) !== s) window.__said.push(s);
    }).observe(document, { subtree: true, childList: true, characterData: true });
  });
  const page = await context.newPage();
  page.on('console', (m) => logs.push(`${m.type()}: ${m.text()}`));
  page.on('pageerror', (e) => logs.push(`pageerror: ${e}`));

  // Open `url` as a phone would, typing the name each time the page asks
  // for one, until the page says it is in or why not.
  const opening = async (url) => {
    await page.goto('about:blank');
    await page.goto(url);
    const settled = () =>
      page.waitForFunction(
        () => !document.getElementById('join').hidden || /good|bad/.test(document.getElementById('status').className),
        null,
        { timeout: 90_000 },
      );
    await settled();
    const asked = await page.evaluate(() => !document.getElementById('join').hidden);
    let why = null;
    if (asked && taken) {
      await page.fill('#name', taken);
      await page.click('#join button');
      await page.waitForFunction(
        () => !document.getElementById('join').hidden && document.getElementById('why').textContent,
        null,
        { timeout: 90_000 },
      );
      why = await page.evaluate(() => document.getElementById('why').textContent);
    }
    if (asked) {
      await page.fill('#name', name);
      await page.click('#join button');
      await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    }
    return page.evaluate(([asked, why]) => ({
      asked,
      why,
      status: document.getElementById('status').textContent,
      lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
      answer: document.getElementById('response').textContent,
      said: window.__said,
      lines: window.__lines,
    }), [asked, why]);
  };

  const first = await opening(link);
  const stored = await page.evaluate(scan);
  const again = await opening(link);
  const alone = await opening(origin);
  const lines = await page.evaluate(() => window.__lines);
  // Open the first lend listed: the page goes to its own origin, whose
  // worker serves it from its lender's computer, dialled by key with a pass.
  let opened = null;
  if (alone.lent.length) {
    const away = (url) => url.hostname.endsWith('.localhost') && url.pathname === '/';
    await Promise.all([page.waitForURL(away, { timeout: 90_000 }), page.click('#lent button')]);
    await page.waitForFunction(() => document.body?.innerText.includes('hi'), null, { timeout: 60_000 }).catch(() => {});
    opened = { origin: new URL(page.url()).origin, text: await page.evaluate(() => document.body.innerText) };
  }
  console.log(JSON.stringify({ first, stored, again, alone, opened, lines, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await close();
  server.close();
}
