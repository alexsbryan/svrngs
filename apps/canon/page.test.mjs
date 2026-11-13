// House rules' page in a browser, B's own (docs/seams.tsv): Chromium as a
// desktop and WebKit as an iPhone (web/tests/page.mjs, `launch`). A server
// of the test's own serves the app's files under a carried page's policy,
// read from the trust floor (scripts/trust_floor.txt, `page carried`), and
// answers `svrngs/acts` as svrngs's door does (crates/svrngs-node/src/
// carried.rs `acts`). What only a browser does with the page: its module
// and the vendored build load under that policy, it reads the house, and a
// press writes and draws again, with no page error. The whole scene, every
// word, is house-page.test.mjs's, index.js in Node. Without Playwright or
// its browser this fails naming the install (`launch` exits 3); it never
// skips.

import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import http from 'node:http';
import { extname, join, normalize } from 'node:path';
import { test } from 'node:test';

import { launch } from '../../web/tests/page.mjs';

const HERE = new URL('.', import.meta.url).pathname;
const ROOT = join(HERE, '../..');

/** A carried page's policy as the floor pins it: its directives under `page carried`. */
async function carriedPolicy() {
  const floor = await readFile(join(ROOT, 'scripts/trust_floor.txt'), 'utf8');
  const block = floor.split('\npage carried ')[1].split('\n').slice(1);
  const lines = block.slice(
    0,
    block.findIndex((l) => !l.startsWith('  ')),
  );
  return lines.map((l) => l.trim().split(' | ')[0]).join('; ');
}

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
};

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
        return res
          .writeHead(200, { 'content-type': 'application/json' })
          .end(JSON.stringify({ you, group: 'house1', people, spans, ops }));
      }
      let body = '';
      for await (const chunk of req) body += chunk;
      const { op } = JSON.parse(body);
      ops.push({
        id: `op${++n}`,
        ts_unix: Math.floor(Date.now() / 1000),
        person: req.headers['x-computer'] ?? you,
        on_behalf_of: you,
        voided: false,
        payload: op,
      });
      return res.writeHead(200, { 'content-type': 'application/json' }).end('{}');
    }
    const file =
      path === '/look.css'
        ? join(ROOT, 'crates/svrngs-node/src/home/look.css')
        : join(HERE, normalize(path === '/' ? 'index.html' : path));
    try {
      res
        .writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' })
        .end(await readFile(file));
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  return { at: `http://127.0.0.1:${server.address().port}/`, stop: () => server.close() };
}

for (const engine of ['chromium', 'webkit']) {
  test(`the page loads under a carried policy, reads the house and writes a choice (${engine})`, async () => {
    const founders = ['ana', 'bo', 'cy', 'dee'].map((person) => ({
      person,
      joined: null,
      left: null,
    }));
    const server = await serve(founders);
    const { context, close } = await launch(engine);
    const errors = [];
    try {
      const page = await context.newPage();
      page.on('pageerror', (e) => errors.push(String(e)));
      await context.setExtraHTTPHeaders({ 'x-person': 'ana', 'x-computer': 'ana' });
      await page.goto(server.at);
      await page.waitForFunction(
        () => !document.body.innerText.includes('Reading your house'),
        null,
        { timeout: 60_000 },
      );
      assert.match(await page.innerText('main'), /How does your house decide its rules\?/);
      await page.check('input[name=way][value=most]');
      const before = await page.innerText('main');
      await page
        .getByRole('button', { name: 'Make this how your house decides', exact: true })
        .click();
      await page.waitForFunction((b) => document.querySelector('main').innerText !== b, before, {
        timeout: 30_000,
      });
      assert.match(
        await page.innerText('main'),
        /How your house decides\n+A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them\./,
      );
      assert.deepEqual(errors, []);
    } finally {
      await close();
      server.stop();
    }
  });
}
