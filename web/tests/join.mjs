// The browser half of crates/svrngs/tests/join_page.rs (DESIGN §3,
// "Joining"; u4-5): a phone opening a group's link. It serves the built
// page on 127.0.0.1 and prints its origin as one JSON line; the Rust half
// prints a link on that origin with `svrngs new --origin` and answers with
// one JSON line on stdin, `{"link": ..., "name": ...}`. Then, in headless
// Chromium, it opens the link and types the name whenever the page asks for
// one; reads every storage the page can write for the person's key; opens
// the link again, then the page alone; opens the first lend listed, which
// the service worker serves; and prints what it saw as one JSON line.
//
//   node web/tests/join.mjs '{"site": "<dir>"}'
//
// Exit 0 with the JSON line; 3, naming the install command, when Playwright
// or its Chromium is absent; 1 on anything else.
import { createInterface } from 'node:readline';
import { launch, serve } from './page.mjs';

const args = JSON.parse(process.argv[2]);
const server = await serve(args.site);
const origin = `http://127.0.0.1:${server.address().port}/`;
console.log(JSON.stringify({ origin }));
const input = createInterface({ input: process.stdin });
const told = await new Promise((done) => input.once('line', done));
input.close();
const { link, name } = JSON.parse(told);

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

const browser = await launch();
const logs = [];
try {
  const page = await browser.newPage();
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
    if (asked) {
      await page.fill('#name', name);
      await page.click('#join button');
      await page.waitForSelector('#status.good, #status.bad', { timeout: 90_000 });
    }
    return page.evaluate((asked) => ({
      asked,
      status: document.getElementById('status').textContent,
      lent: [...document.querySelectorAll('#lent button')].map((b) => b.textContent),
      answer: document.getElementById('response').textContent,
      lines: window.__lines,
    }), asked);
  };

  const first = await opening(link);
  const stored = await page.evaluate(scan);
  const again = await opening(link);
  const alone = await opening(origin);
  const lines = await page.evaluate(() => window.__lines);
  // Open the first lend listed: the page goes to its address, which the
  // service worker serves from its lender's computer, dialled by key.
  let opened = null;
  if (alone.lent.length) {
    await Promise.all([page.waitForURL(/\/l\//, { timeout: 90_000 }), page.click('#lent button')]);
    await page.waitForLoadState('load');
    opened = { path: new URL(page.url()).pathname, text: await page.evaluate(() => document.body.innerText) };
  }
  console.log(JSON.stringify({ first, stored, again, alone, opened, lines, logs }));
} catch (e) {
  console.error(`the page did not finish: ${e}\n${logs.join('\n')}`);
  process.exitCode = 1;
} finally {
  await browser.close();
  server.close();
}
