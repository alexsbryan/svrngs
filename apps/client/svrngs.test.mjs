// svrngs.mjs, the one way a program asks svrngs ("Never send the key",
// svrngs V1.13.3): its proof and its code held to svrngs's vectors; the
// token never sent, to svrngs or to anything squatting svrngs's address;
// each request on the connection its code was made for, proved again
// when that connection is gone; and every copy in apps/ this file's bytes.
// Each against a stand-in svrngs in memory (standin.mjs), and the
// connection's real ends over a real socket in one test, the seam's own.

import assert from 'node:assert/strict';
import { readFileSync, readdirSync, rmSync } from 'node:fs';
import http from 'node:http';
import net from 'node:net';
import { join } from 'node:path';
import { mock, test } from 'node:test';

import { PORT, TOKEN, dataDir, standin, wire } from './standin.mjs';
import { PROVE_WITHIN, code, proof, svrngs } from './svrngs.mjs';

const APPS = new URL('..', import.meta.url).pathname;
const json = (res, status, value) =>
  res.writeHead(status, { 'content-type': 'application/json' }).end(JSON.stringify(value));
const groups = (req, res) =>
  json(res, 200, { groups: [{ id: 'g1', name: 'The House' }], asked: req.url });

// The vectors of NEVER_SEND_THE_KEY.md, which svrngs-node's api_tests.rs and
// Give hold too: token `test-token`.
test("the proof and the ask code are svrngs's, by its vectors", () => {
  const v4 = ['127.0.0.1:50000', '127.0.0.1:40000'];
  const v6 = ['[::1]:50001', '[::1]:40000'];
  assert.equal(
    proof('test-token', '00'.repeat(32), ...v4),
    'e5a8107be851bcf50a977799f3dd752fb8a7db2607128372ab6936aa42fa046f',
  );
  assert.equal(
    proof('test-token', 'ab'.repeat(32), ...v6),
    'bdaab3ee1e00b484f65ee220d690a8b5d462ecff25a6d9f5ac898148dec38472',
  );
  assert.equal(
    code('test-token', '11'.repeat(32), ...v4),
    '12948174292b9641bc74ea49e12918bd39931f81cddef77ae9c04d46e6585245',
  );
  assert.equal(
    code('test-token', 'cd'.repeat(32), ...v6),
    'd806666da2d6c78fa07dc51fe301f57596fed866e930680a54e5bbc9de104f1c',
  );
  assert.notEqual(proof('t', '11'.repeat(32), ...v4), code('t', '11'.repeat(32), ...v4));
});

test('asks on one proved connection, as you, never with the token', async (t) => {
  const stand = standin(groups);
  t.after(stand.close);
  assert.deepEqual((await stand.client.ask('/v1/groups')).groups, [
    { id: 'g1', name: 'The House' },
  ]);
  const posted = await stand.client.ask('/v1/rail/append?namespace=g1', { op: 'record' });
  assert.equal(posted.asked, '/v1/rail/append?namespace=g1');
  assert.equal(stand.seen.connections, 1, 'a connection per ask, where one serves both');
  assert.equal(stand.seen.proofs, 1);
  assert.deepEqual(stand.seen.refused, []);
  assert.deepEqual(
    stand.seen.asked.map((a) => [a.url, a.host]),
    [
      ['/v1/groups', `127.0.0.1:${PORT}`],
      ['/v1/rail/append?namespace=g1', `127.0.0.1:${PORT}`],
    ],
  );
});

test("a lend's address on this computer is asked on svrngs's connection, by its name", async (t) => {
  const stand = standin((req, res) => json(res, 200, { host: req.headers.host }));
  t.after(stand.close);
  const at = `http://ana-coder.localhost:${PORT}/v1/models`;
  assert.deepEqual(await stand.client.ask(at), { host: `ana-coder.localhost:${PORT}` });
  for (const elsewhere of [
    'https://example.com/v1/models',
    `http://ana.localhost:${PORT + 1}/`,
    '//x/y',
  ])
    await assert.rejects(stand.client.ask(elsewhere), /asks svrngs alone/, elsewhere);
  await assert.rejects(
    stand.client.fetch('/v1/groups', { headers: { authorization: `Bearer ${TOKEN}` } }),
    /takes no Authorization from a program/,
  );
  assert.equal(stand.seen.asked.length, 1, 'nothing but the lend was asked');
});

// The plant: a connection pooled past the one its code was made for (an
// http.Agent that opens another when the first is gone) sends the code on
// a connection svrngs never proved itself on, and svrngs refuses it.
test('a request whose connection is gone is proved again on a new one, never sent on it unproved', async (t) => {
  const stand = standin(groups);
  t.after(stand.close);
  await stand.client.ask('/v1/groups');
  stand.drop();
  await new Promise((r) => setImmediate(r));
  assert.equal((await stand.client.ask('/v1/groups')).groups.length, 1);
  assert.deepEqual(stand.seen.refused, [], 'a request went on a connection unproved');
  assert.equal(stand.seen.connections, 2);
  assert.equal(stand.seen.proofs, 2, 'each connection proved before its ask');
});

// The plant: a read, then a program asking svrngs's proof itself on the
// connection this file proved, then a read on that connection, whose code
// was made for the nonce that proof replaced. The stand-in proves at any
// host, as svrngs does at a `.localhost` address naming no lend.
test("a program's own proof is refused in plain words, and the next ask is asked as before", async (t) => {
  const stand = standin(groups);
  t.after(stand.close);
  const challenge = { challenge: '00'.repeat(32) };
  await stand.client.ask('/v1/groups');
  const own = stand.client.ask('/v1/proof', challenge);
  await own.catch(() => {});
  assert.equal((await stand.client.ask('/v1/groups')).groups.length, 1);
  await assert.rejects(own, /svrngs\.mjs proves svrngs itself.*leave \/v1\/proof to it/);
  const there = `http://127.0.0.1:${PORT}/v1/proof?again`;
  await assert.rejects(stand.client.ask(there, challenge), /leave/);
  assert.equal(stand.seen.proofs, 1, 'a proof asked but this file');
  await stand.client.ask(`http://not-a-lend.localhost:${PORT}/v1/proof`, challenge);
  assert.equal((await stand.client.ask('/v1/groups')).groups.length, 1);
  assert.deepEqual(stand.seen.refused, []);
  assert.deepEqual(
    [stand.seen.connections, stand.seen.proofs],
    [2, 3],
    'proved again, on a new one',
  );
});

// The plant: an answer with no body, 205 among them, as svrngs or a lend
// may give; the next ask on the same connection.
test('an empty answer reads as empty, and its connection serves the next ask', async (t) => {
  const stand = standin((req, res) =>
    req.url === '/v1/groups' ? groups(req, res) : res.writeHead(Number(req.url.slice(1))).end(),
  );
  t.after(stand.close);
  for (const status of [205, 204]) assert.equal(await stand.client.ask(`/${status}`, {}), null);
  assert.equal((await stand.client.ask('/v1/groups')).groups.length, 1);
  assert.deepEqual([stand.seen.connections, stand.seen.proofs], [1, 1]);
});

test('svrngs refusing says so, with its words and status', async (t) => {
  const stand = standin((req, res) => json(res, 403, { error: 'not yours', kind: 'not_yours' }));
  t.after(stand.close);
  await assert.rejects(stand.client.ask('/v1/rail/append?namespace=g', {}), (e) => {
    assert.equal(e.message, 'svrngs refused /v1/rail/append: 403 not yours');
    assert.equal(e.status, 403);
    assert.equal(e.answer.kind, 'not_yours');
    return true;
  });
});

/** What a program squatting svrngs's address is sent, all of it, when a
 * program asks svrngs at the data dir holding TOKEN; and the refusal. */
async function squatted(t, answer) {
  const got = [];
  const squat = http.createServer(answer);
  const dir = dataDir();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const connect = wire(squat, (s) => s.on('data', (c) => got.push(c)));
  const refused = await svrngs(dir, { connect })
    .ask('/v1/groups')
    .then(
      () => null,
      (e) => e,
    );
  const sent = Buffer.concat(got).toString('latin1');
  return { refused, sent };
}

const noToken = (sent) => {
  assert.ok(!sent.includes(TOKEN), 'the squatter was sent the token');
  assert.ok(!/authorization/i.test(sent), 'the squatter was sent an Authorization');
  assert.equal(sent.match(/^[A-Z]+ \S+ HTTP\/1\.1/gm)?.join(), 'POST /v1/proof HTTP/1.1');
};

test("a squatter at svrngs's address is sent a challenge alone, and refused in plain words", async (t) => {
  const { refused, sent } = await squatted(t, (req, res) =>
    req
      .resume()
      .on('end', () => json(res, 200, { proof: 'ab'.repeat(32), nonce: 'cd'.repeat(32) })),
  );
  assert.equal(refused?.withheld, 'unproven');
  assert.match(
    refused.message,
    /^Something other than your svrngs answers at http:\/\/127\.0\.0\.1:4200, where svrngs listens; it was sent none of your token\./,
  );
  noToken(sent);
});

test('a relay carrying the challenge to svrngs on a connection of its own proves nothing on this one', async (t) => {
  const stand = standin(groups);
  t.after(stand.close);
  const relay = await squatted(t, (req, res) => {
    const { method, url: path, headers } = req;
    const carried = { method, path, headers, createConnection: stand.connect };
    req.pipe(
      http.request(carried, (there) => there.pipe(res.writeHead(there.statusCode, there.headers))),
    );
  });
  assert.equal(relay.refused?.withheld, 'unproven');
  assert.equal(stand.seen.proofs, 1, "svrngs proved itself, for the relay's connection");
  noToken(relay.sent);
});

test('an older svrngs, or a squatter saying 404, an unbounded answer, or none, is sent nothing', async (t) => {
  const older = await squatted(t, (req, res) => json(res, 404, { error: 'no route' }));
  assert.equal(older.refused?.withheld, 'older');
  assert.match(
    older.refused.message,
    /older than this program.*sent none of your token.*svrngs restart/,
  );
  noToken(older.sent);

  // A svrngs that proves but gives no nonce: V1.13.3's proof, before it
  // stopped taking the token.
  const nonceless = await squatted(t, (req, res) => {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => {
      const { challenge } = JSON.parse(Buffer.concat(chunks).toString());
      const [client, server] = [
        `${req.socket.remoteAddress}:${req.socket.remotePort}`,
        `127.0.0.1:${PORT}`,
      ];
      json(res, 200, { proof: proof(TOKEN, challenge, client, server) });
    });
  });
  assert.equal(nonceless.refused?.withheld, 'older');
  noToken(nonceless.sent);

  const flood = await squatted(t, (req, res) => {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(`{"proof": "${'a'.repeat(64)}", "pad": "${'x'.repeat(8192)}"}`);
  });
  assert.equal(flood.refused?.withheld, 'unproven');
  noToken(flood.sent);

  mock.timers.enable({ apis: ['setTimeout'] });
  t.after(() => mock.timers.reset());
  const silent = squatted(t, (req) => req.resume());
  await new Promise((r) => setImmediate(r));
  mock.timers.tick(PROVE_WITHIN);
  const { refused, sent } = await silent;
  assert.equal(refused?.withheld, 'unanswered');
  assert.match(refused.message, /within 10 seconds; it was sent none of your token/);
  noToken(sent);
});

test("svrngs not running is said, naming what isn't there", async (t) => {
  const dir = dataDir();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  rmSync(join(dir, 'api'));
  await assert.rejects(svrngs(dir).ask('/v1/groups'), (e) => {
    assert.equal(e.withheld, 'away');
    assert.match(
      e.message,
      new RegExp(`svrngs isn't running on this computer: ${join(dir, 'api')}`),
    );
    return true;
  });
  const free = await new Promise((done) => {
    const probe = net.createServer().listen(0, '127.0.0.1', () => {
      const { port } = probe.address();
      probe.close(() => done(port));
    });
  });
  const gone = dataDir(`http://127.0.0.1:${free}`);
  t.after(() => rmSync(gone, { recursive: true, force: true }));
  await assert.rejects(svrngs(gone).ask('/v1/groups'), (e) => {
    assert.equal(e.withheld, 'away');
    assert.match(e.message, new RegExp(`nothing answers at http://127.0.0.1:${free}`));
    return true;
  });
});

// The seam's own: the ends a proof names, as a real socket gives them,
// on both loopbacks.
for (const host of ['127.0.0.1', '::1']) {
  test(`over a real socket on ${host}, the proof's ends are the connection's`, async (t) => {
    const listener = net.createServer();
    const bound = await new Promise((done) => {
      listener.once('error', () => done(false));
      listener.listen(0, host, () => done(true));
    });
    if (!bound) return t.skip(`could-not-judge: this computer has no ${host} to listen on`);
    t.after(() => listener.close());
    const at = net.isIPv6(host) ? `[${host}]` : host;
    const stand = standin(groups, { at: `http://${at}:${listener.address().port}` });
    t.after(stand.close);
    listener.on('connection', stand.serve);
    const client = svrngs(stand.dir);
    assert.equal((await client.ask('/v1/groups')).groups.length, 1);
    assert.equal((await client.ask('/v1/groups')).groups.length, 1);
    assert.deepEqual(stand.seen.refused, []);
    assert.equal(stand.seen.proofs, 1);
  });
}

// One decider: each app that is copied whole (pantry's template, the
// scenes' copies of events and grapevine) carries this file beside its
// code, as a person's own program does, byte for byte.
test('every svrngs.mjs in apps/ is this one', () => {
  const home = readFileSync(join(APPS, 'client/svrngs.mjs'));
  const copies = readdirSync(APPS, { recursive: true })
    .filter(
      (f) =>
        /(^|\/)svrngs\.mjs$/.test(f) && !f.includes('node_modules') && f !== 'client/svrngs.mjs',
    )
    .sort();
  assert.ok(copies.length > 0, 'never-ran: no copy of svrngs.mjs in apps/');
  const drifted = copies.filter((f) => !readFileSync(join(APPS, f)).equals(home));
  assert.deepEqual(drifted, [], 'copy apps/client/svrngs.mjs over each');
});
