#!/usr/bin/env node
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, resolve } from 'node:path';

const repo = resolve(import.meta.dirname, '../../../..');
const world = process.env.PLAY_LEND_GIVE_WORLD;
const arm = process.env.AVAIL_ARM;
const who = process.env.AVAIL_PERSON;
if (!world || !['plant', 'built'].includes(arm) || !['hana', 'ivo'].includes(who)) {
  throw new Error('Set PLAY_LEND_GIVE_WORLD, AVAIL_ARM=plant|built, AVAIL_PERSON=hana|ivo');
}
const me = who[0].toUpperCase() + who.slice(1);
const home = join(repo, 'crates/svrngs-node/src/home');
const house = '1'.repeat(64);
// Real response shapes: home.rs `lent`, plan/words.rs `owned` and `run_form`.
const [hand, managed] = ['Shared while you run it.', 'Run by svrngs.'];
const [handText, managedText] = arm === 'plant' ? [managed, hand] : [hand, managed];
const run = 'Run it with svrngs';
const lend = (name, kind) => ({ name, kind, groups: [house], limit: null, using: 0 });
const replies = {
  '/lent': {
    lent: [lend('coder', 'llama-server'), lend('notes', 'notes')],
    signing: [],
    readers: {},
    headers: {},
    sharing: { coder: [handText, run] },
  },
  '/run': {
    run,
    fields: [
      ['name', 'Name'],
      ['port', 'Port'],
      ['folder', 'Folder'],
      ['command', 'The command you start it with'],
    ],
    starting: 'Starting…',
    rows: [{ name: 'notes', port: 8099, text: managedText, action: 'stop', button: 'Stop notes' }],
  },
};
const fixture = await readFile(join(world, 'fixture.js'), 'utf8');
const setup = `<script>window.XENIA = ${JSON.stringify({ arm: 'A', scenario: 'a', me, small: true })};</script>
<script src="/__fixture.js"></script>`;
const overlay = `
(() => {
  const base = window.fetch;
  const replies = ${JSON.stringify(replies)};
  window.fetch = (input, init) => {
    const path = new URL(String(input), location.href).pathname;
    if ((init?.method ?? 'GET') !== 'GET') {
      window.__did('attempted change: ' + path);
      return Promise.resolve(Response.json({ error: 'This rehearsal does not change any shares or programs.' }, { status: 409 }));
    }
    return Object.hasOwn(replies, path) ? Promise.resolve(Response.json(replies[path])) : base(input, init);
  };
})();`;
const types = {
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.svg': 'image/svg+xml',
  '.html': 'text/html; charset=utf-8',
  '.woff2': 'font/woff2',
};
const server = createServer(async (req, res) => {
  const path = new URL(req.url, 'http://127.0.0.1').pathname;
  if (path === '/__fixture.js') {
    res.writeHead(200, { 'content-type': types['.js'] }).end(fixture + overlay);
    return;
  }
  if (path === '/__did') {
    for await (const part of req) console.error(`${arm}/${who}: ${part}`);
    res.writeHead(204).end();
    return;
  }
  const file =
    path === '/'
      ? join(home, 'index.html')
      : path === '/favicon.svg'
        ? join(repo, 'docs/brand/favicon.svg')
        : join(path.startsWith('/vendor/') ? join(repo, 'web') : home, path);
  try {
    const bytes = await readFile(file);
    const body =
      path === '/' ? bytes.toString().replace('<script type="module"', `${setup}\n<script type="module"`) : bytes;
    res.writeHead(200, { 'content-type': types[extname(file)] ?? 'application/octet-stream' }).end(body);
  } catch {
    res.writeHead(404).end();
  }
});
await new Promise((done) => server.listen(0, '127.0.0.1', done));
const origin = `http://127.0.0.1:${server.address().port}`;
const url = `${origin}/?view=computer`;
if (process.argv[2] === '--serve') console.log(url);
else
  console.log(
    JSON.stringify({
      begins: 'on their laptop, with svrngs’s This computer page open. Cy uses coder and notes in Maple St house. A housemate set the two shares up, and they haven’t used this page before',
      cast: [{ as: me, person: join(world, `people/${who}.md`), device: 'computer', url }],
      screens: [{ name: 'This computer', url: 'view=computer' }],
      channels: [
        {
          name: 'the house chat',
          members: [me],
          opening: [
            {
              from: 'Cy',
              text: 'After you restart your laptop tonight and sign back in, which of coder and notes can I open tomorrow without you doing anything else? Just tell me here; no need to change anything.',
            },
          ],
        },
      ],
      explore: { purpose: 'Find out when the apps you share will be available to a friend.' },
      reach: [origin],
    }),
  );
process.stdin.resume();
process.stdin.on('end', () => server.close());
