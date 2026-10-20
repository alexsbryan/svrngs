// House rules' page in a browser (canon-house (c)): Chromium as a desktop and
// WebKit as an iPhone (web/tests/page.mjs, `launch`). A server of the test's
// own serves the app's files under a carried page's policy, read from the
// trust floor (scripts/trust_floor.txt, `page carried`), and answers
// `svrngs/acts` as svrngs's door does (crates/svrngs-node/src/carried.rs
// `acts`): the asker, the group, its people and their time in it, and the
// app's entries, each new one written as the asker, through the computer the
// test names. Four people founded the house. Ana brings it, answers the
// question, "Most of you agree", and proposes "Quiet hours are 10pm to 7am";
// Bo agrees through Dee's computer, and Ana's page says so; Cy agrees, and it
// reads as a rule. No page says canon's own words. Without Playwright or its
// browser this fails naming the install (`launch` exits 3); it never skips.

import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import http from 'node:http';
import { extname, join, normalize } from 'node:path';
import { test } from 'node:test';

import { launch } from '../../web/tests/page.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const ROOT = join(HERE, '../..');
/** canon's words and svrngs's own, which the page never says (fence 6). */
const NOT_SAID = /\b(scope|ratif\w*|commitments?|acts?|actors?|canon|member|key|record)\b/i;

/** A carried page's policy as the floor pins it: its directives under `page carried`. */
async function carriedPolicy() {
  const floor = await readFile(join(ROOT, 'scripts/trust_floor.txt'), 'utf8');
  const block = floor.split('\npage carried ')[1].split('\n').slice(1);
  const lines = block.slice(0, block.findIndex((l) => !l.startsWith('  ')));
  return lines.map((l) => l.trim().split(' | ')[0]).join('; ');
}

const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8' };

/** The app at an origin of its own, for `spans`: who asks is the `x-person` header, whose computer wrote it `x-computer`. */
async function serve(spans) {
  const policy = await carriedPolicy();
  const ops = [];
  let n = 0;
  const server = http.createServer(async (req, res) => {
    res.setHeader('content-security-policy', policy);
    const you = req.headers['x-person'];
    const path = new URL(req.url, 'http://app').pathname;
    if (path === '/svrngs/acts') {
      if (req.method === 'GET') {
        const people = spans.filter((s) => s.left == null).map((s) => s.person);
        return res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({ you, group: 'house1', people, spans, ops }));
      }
      let body = '';
      for await (const chunk of req) body += chunk;
      const { op } = JSON.parse(body);
      ops.push({ id: `op${++n}`, ts_unix: Math.floor(Date.now() / 1000), person: req.headers['x-computer'] ?? you, on_behalf_of: you, voided: false, payload: op });
      return res.writeHead(200, { 'content-type': 'application/json' }).end('{}');
    }
    const file = path === '/look.css' ? join(ROOT, 'crates/svrngs-node/src/home/look.css') : join(HERE, normalize(path === '/' ? 'index.html' : path));
    try {
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' }).end(await readFile(file));
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  return { at: `http://127.0.0.1:${server.address().port}/`, stop: () => server.close() };
}

for (const engine of ['chromium', 'webkit']) {
  test(`a house answers the question, proposes a rule, and two others agree to it (${engine})`, async () => {
    const founders = ['ana', 'bo', 'cy', 'dee'].map((person) => ({ person, joined: null, left: null }));
    const server = await serve(founders);
    const { context, close } = await launch(engine);
    const errors = [];
    try {
      const page = await context.newPage();
      page.on('pageerror', (e) => errors.push(String(e)));
      const traced = [];
      page.on('console', (m) => m.type() === 'debug' && traced.push(m.text()));
      const said = [];
      const text = () => page.innerText('main');
      const as = async (person, computer = person) => {
        await context.setExtraHTTPHeaders({ 'x-person': person, 'x-computer': computer });
        await page.goto(server.at);
        await page.waitForFunction(() => !document.body.innerText.includes('Reading your house'), null, { timeout: 60_000 });
        said.push(await text());
      };
      const press = async (name) => {
        const before = await text();
        await page.getByRole('button', { name, exact: true }).first().click();
        await page.waitForFunction((b) => document.querySelector('main').innerText !== b, before, { timeout: 30_000 });
        said.push(await text());
      };

      await as('ana');
      assert.match(await text(), /How does your house decide its rules\?/);
      assert.match(await text(), /Most of you agree\. A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them\./);
      await page.check('input[name=way][value=most]');
      await press('Make this how your house decides');
      assert.match(await text(), /How your house decides\n+A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them\./);
      await page.fill('#rule', 'Quiet hours are 10pm to 7am');
      await press('Propose this rule to everyone in your house');
      assert.match(await page.innerText('ul.proposed'), /Quiet hours are 10pm to 7am\n+You proposed it on/);

      await as('bo', 'dee');
      await press('Agree');
      await as('ana');
      assert.match(await page.innerText('ul.proposed'), /Agreed: bo, through dee’s computer\./);
      assert.equal(await page.locator('ul.proposed button').count(), 0, 'Ana is not asked about her own proposal');

      await as('cy');
      await press('Agree');
      assert.match(await page.innerText('ul.rules'), /^Quiet hours are 10pm to 7am \(ana proposed it on /);
      assert.match(await text(), /Nothing is waiting on anyone\./);
      assert.match(await text(), /Everyone in your house has a say: ana, bo, you and dee\./);
      for (const words of said) assert.doesNotMatch(words, NOT_SAID);
      // Each decision traced at debug, house.js's included (#210).
      for (const what of ['svrngs/acts answered', 'asking how the house decides', 'sent', 'canon asked', 'how the house decides']) {
        assert.ok(traced.some((l) => l.startsWith(`House rules: ${what} `)), `traced: ${what}`);
      }
      assert.deepEqual(errors, []);
    } finally {
      await close();
      server.stop();
    }
  });
}
