// SPDX-License-Identifier: AGPL-3.0-or-later
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createResolver } from '../../resolver/handler.mjs';
import { GENERATION, PREFIX } from '../../resolver/store.mjs';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawn, spawnSync, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { before, after } from 'node:test';
import {
  ownerKey,
  owner,
  otherKey,
  hash,
  fixed,
  seconds,
  body,
  mutation,
  refused,
} from './resolver-fixture.mjs';

const directory = mkdtempSync(join(tmpdir(), 'svrngs-resolver-'));
const socket = join(directory, 'redis.sock');
let redis;
let commands = [];
let fault;
let candidate;
let readOverride;
const execute = promisify(execFile);
const plant = process.env.RESOLVER_PLANT;
const stateKey = `${PREFIX}:reservations`;
function cli(...args) {
  const child = spawnSync('redis-cli', ['-s', socket, '--json', ...args.map(String)], {
    encoding: 'utf8',
    timeout: 5000,
  });
  if (child.status !== 0) throw new Error(child.stderr);
  const value = JSON.parse(child.stdout);
  if (value?.error) throw new Error(value.error);
  return value;
}
before(async () => {
  redis = spawn(
    'redis-server',
    [
      '--port',
      '0',
      '--unixsocket',
      socket,
      '--dir',
      directory,
      '--appendonly',
      'yes',
      '--appendfsync',
      'always',
      '--save',
      '',
      '--maxmemory-policy',
      'noeviction',
    ],
    { stdio: 'ignore' },
  );
  for (let attempt = 0; attempt < 100; attempt++) {
    try {
      if (cli('PING') === 'PONG') return;
    } catch {
      /* socket not ready */
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  throw new Error('Redis did not start');
});
after(async () => {
  if (redis?.exitCode === null) {
    const stopped = new Promise((resolve) => redis.once('exit', resolve));
    redis.kill();
    await stopped;
  }
  rmSync(directory, { recursive: true, force: true });
});
function reset() {
  cli('FLUSHDB');
  cli('SET', `${PREFIX}:generation`, GENERATION);
  fault = undefined;
  candidate = undefined;
  readOverride = undefined;
  commands = [];
}
function instrument(source) {
  if (plant === 'expiry') source = source.replace(", q.secret, 'EXAT', r['until'])", ', q.secret)');
  if (plant === 'partial')
    source = source.replace(
      "if r.state ~= 'open' then return refuse('pending') end",
      "if r.state ~= 'open' and r.state ~= 'pending' then return refuse('pending') end",
    );
  if (plant === 'reset')
    source = source.replace(
      "if redis.call('GET', KEYS[1]) ~= ARGV[1] then return refuse('generation') end",
      '',
    );
  if (plant === 'reuse')
    source = source.replace(
      "if redis.call('HGET', KEYS[2], 'p:'..q.phrase) then return refuse('collision') end",
      '',
    );
  if (plant === 'reopen')
    source = source.replace(
      'local fresh =',
      "if r and q.action == 'publish' then r.state = 'pending' end\nlocal fresh =",
    );
  if (plant === 'limits')
    source = source
      .replace('n >= cap', 'n >= 1000000000')
      .replace('active >= activeCap', 'active >= 1000000000')
      .replace('KEYS[2]) >= 100000', 'KEYS[2]) >= 1000000000')
      .replace('used >= 100000000', 'used >= 1000000000');
  source = source
    .replaceAll('redis.call(', 'call(')
    .replace('state=state,', 'commands=metered, state=state,')
    .replace("state='unavailable',", "commands=metered, state='unavailable',");
  const which = fault?.which ?? 0;
  const when = fault?.when ?? 'before';
  return `local metered = 1
local writes = 0
local function call(command, ...)
  metered = metered + 1
  ${readOverride ? `if command == '${readOverride.command}' then return cjson.decode(${JSON.stringify(JSON.stringify(readOverride.value))}) end` : ''}
  local write = command == 'SET' or command == 'HSET' or command == 'ZADD' or command == 'DEL' or command == 'ZREM'
  if write then writes = writes + 1 end
  if write and writes == ${which} and '${when}' == 'before' then error('injected_before_write') end
  local result = redis.call(command, ...)
  if write and writes == ${which} and '${when}' == 'after' then error('injected_after_write') end
  return result
end\n${source}`;
}
async function localFetch(_url, options) {
  const command = JSON.parse(options.body);
  const q = JSON.parse(command.at(-1));
  if (candidate && q.action === 'publish') q.phrase = candidate;
  command[1] = instrument(command[1]);
  command[command.length - 1] = JSON.stringify(q);
  try {
    const { stdout } = await execute(
      'redis-cli',
      ['-s', socket, '--json', ...command.map(String)],
      { timeout: 5000 },
    );
    const result = JSON.parse(stdout);
    if (result?.error) throw new Error(result.error);
    commands.push(JSON.parse(result).commands);
    if (fault?.lost) throw new Error('lost reply');
    return Response.json({ result });
  } catch (error) {
    if (fault?.lost) throw error;
    return Response.json({ error: error.message });
  }
}
const handle = createResolver({ token: 'fixture', fetch: localFetch });
const publish = (value, key) => handle(mutation(value, key), '127.0.0.1');
const lookup = (name, ip = '127.0.0.1') => handle(new Request(`https://join.example/${name}`), ip);

test('authenticated lifecycle, immutable binding, closure and no reopen', async () => {
  reset();
  const q = body();
  const first = await (await publish(q)).json();
  assert.equal(first.id, q.id);
  assert.equal((await (await lookup(first.phrase)).json()).token, q.token);
  assert.equal((await (await publish(q)).json()).phrase, first.phrase);
  for (const changes of [
    { token: 'other' },
    { until: q.until + 1 },
    { group: 'other' },
    { page: 'https://elsewhere.example' },
  ])
    await refused(await publish({ ...q, ...changes }));
  await refused(await handle(mutation(q, ownerKey, 'other-app'), '127.0.0.1'));
  const close = { ...q, action: 'close', token: null };
  assert.equal((await (await publish(close)).json()).closed, true);
  assert.equal((await (await publish(close)).json()).closed, true);
  await refused(await publish(q));
  const ended = await (await lookup(first.phrase)).json();
  assert.equal(ended.state, 'ended');
  assert.match(ended.message, /open svrngs on your computer/);
  assert.equal(cli('EXISTS', `${stateKey}:secret:${hash(`${owner}:${q.id}`)}`), 0);
  assert.equal(cli('TTL', stateKey), -1);
  const newer = body({ id: 'cd'.repeat(32) });
  const newPhrase = (await (await publish(newer)).json()).phrase;
  await publish(close);
  assert.equal((await (await lookup(newPhrase)).json()).token, newer.token);
  assert.ok(Math.max(...commands) <= 16, `metered commands: ${commands}`);
  console.log(`lifecycle command maximum: ${Math.max(...commands)}`);
});

test('old phrase cannot name a later owner and close-before-publish is final', async () => {
  reset();
  candidate = fixed;
  const q = body();
  assert.equal((await publish(q)).status, 200);
  await publish({ ...q, action: 'close', token: null });
  await refused(await publish(body({ id: 'cc'.repeat(32), token: 'other-invitation' }), otherKey));
  assert.equal((await (await lookup(fixed)).json()).state, 'ended');
  const unseen = body({ id: 'dd'.repeat(32) });
  await publish({ ...unseen, action: 'close', token: null });
  candidate = undefined;
  await refused(await publish(unseen));
});

test('missing and mismatched generation refuse every operation without writes', async () => {
  for (const generation of [null, 'wrong-generation']) {
    reset();
    const q = body();
    const first = await (await publish(q)).json();
    if (generation === null) cli('DEL', `${PREFIX}:generation`);
    else cli('SET', `${PREFIX}:generation`, generation);
    const snapshot = cli('DUMP', stateKey);
    for (const action of ['publish', 'close', 'status'])
      await refused(await publish({ ...q, action, token: action === 'publish' ? q.token : null }));
    await refused(await lookup(first.phrase));
    assert.equal(cli('DUMP', stateKey), snapshot);
    assert.equal(cli('GET', `${PREFIX}:generation`), generation);
  }
});

test('expiry erases actual secret while reservation permanently refuses reuse', async () => {
  reset();
  candidate = fixed;
  const q = body({ until: seconds() + 2 });
  assert.equal((await publish(q)).status, 200);
  await new Promise((resolve) => setTimeout(resolve, 2100));
  assert.equal(cli('EXISTS', `${stateKey}:secret:${hash(`${owner}:${q.id}`)}`), 0);
  const answer = await (await lookup(fixed)).json();
  assert.equal(answer.state, 'ended');
  assert.match(answer.message, /open svrngs/);
  await refused(await publish(body({ id: 'ee'.repeat(32) })));
  assert.equal(cli('TTL', stateKey), -1);
});

test('every write boundary is uncertain-safe and reconciles the same operation', async () => {
  for (const action of ['publish', 'close']) {
    const writes = action === 'publish' ? 6 : 5;
    for (const when of ['before', 'after'])
      for (let which = 1; which <= writes; which++) {
        reset();
        candidate = fixed;
        const q = body();
        if (action === 'close') assert.equal((await publish(q)).status, 200);
        fault = { when, which };
        await refused(await publish(action === 'publish' ? q : { ...q, action, token: null }));
        fault = undefined;
        const observed = await lookup(fixed);
        const value = await observed.json();
        if (
          action === 'publish' &&
          (which > 3 || (which === 3 && when === 'after')) &&
          !(when === 'after' && which === 6)
        ) {
          assert.equal(
            observed.status,
            503,
            `${action}/${when}/${which}: pending must be invisible`,
          );
        }
        if (
          action === 'close' &&
          (which === 3 || which === 4 || (when === 'before' && which === 5)) &&
          !(which === 3 && when === 'before')
        ) {
          assert.equal(observed.status, 503, 'partial close is unavailable, not ended');
        }
        assert.ok(observed.status === 503 || value.state === 'ended' || value.state === 'open');
        const retried = await publish(action === 'publish' ? q : { ...q, action, token: null });
        assert.equal(retried.status, 200, `${action}/${when}/${which}: recover`);
        const final = await (await lookup(fixed)).json();
        assert.equal(final.state, action === 'publish' ? 'open' : 'ended');
        assert.ok(Math.max(...commands) <= 16);
      }
  }
  for (const action of ['publish', 'close']) {
    reset();
    candidate = fixed;
    const q = body();
    if (action === 'close') await publish(q);
    fault = { lost: true };
    await refused(await publish(action === 'publish' ? q : { ...q, action, token: null }));
    fault = undefined;
    assert.equal(
      (await publish(action === 'publish' ? q : { ...q, action, token: null })).status,
      200,
    );
    assert.equal(
      (await (await lookup(fixed)).json()).state,
      action === 'publish' ? 'open' : 'ended',
    );
  }
  console.log('before/after all 11 lifecycle writes and lost commit replies: passed');
});

test('distributed guessing and owner limits include misses, retries and status', async () => {
  const clock = seconds();
  const date = new Date();
  const month = Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1) / 1000;
  const minute = Math.floor(clock / 60);
  const hour = Math.floor(clock / 3600);
  const day = Math.floor(clock / 86400);
  const ipQuota = `${PREFIX}:quota:ip:${hash('127.0.0.1')}`;
  const ownerQuota = `${PREFIX}:quota:owner:${owner}`;
  const globalQuota = `${PREFIX}:quotas`;
  for (const [key, name, bucket, cap] of [
    [ipQuota, 'minute', minute, 30],
    [ipQuota, 'day', day, 300],
    [globalQuota, 'lookupDay', day, 1000],
    [globalQuota, 'lookupMonth', month, 20000],
  ]) {
    reset();
    readOverride = { command: 'TIME', value: [String(clock), '0'] };
    cli('SET', key, JSON.stringify({ [name]: [bucket, cap - 1] }));
    assert.equal((await lookup(fixed)).status, 200, `${name}: last allowed miss`);
    // A new handler instance still shares Redis counters.
    const another = createResolver({ token: 'fixture', fetch: localFetch });
    await refused(await another(new Request(`https://join.example/${fixed}`), '127.0.0.1'));
    assert.equal(cli('EXISTS', stateKey), 0);
  }
  for (const [key, name, bucket, cap] of [
    [ownerQuota, 'minute', minute, 10],
    [ownerQuota, 'hour', hour, 60],
    [globalQuota, 'mutationMonth', month, 3000],
  ]) {
    reset();
    readOverride = { command: 'TIME', value: [String(clock), '0'] };
    const q = body({ action: 'close', token: null });
    cli('SET', key, JSON.stringify({ [name]: [bucket, cap - 1] }));
    assert.equal((await publish(q)).status, 200);
    const snapshot = cli('DUMP', stateKey);
    await refused(await publish(q));
    assert.equal(cli('DUMP', stateKey), snapshot);
  }
  for (const [key, cap] of [
    [ownerQuota, 5],
    [globalQuota, 100],
  ]) {
    reset();
    readOverride = { command: 'TIME', value: [String(clock), '0'] };
    const q = body();
    cli('SET', key, JSON.stringify({ newDay: [day, cap - 1] }));
    assert.equal((await publish(q)).status, 200);
    await refused(await publish(body({ id: 'cc'.repeat(32) })));
    assert.equal(
      (await publish({ ...q, action: 'close', token: null })).status,
      200,
      'new-publication quota leaves close available',
    );
  }
  reset();
  readOverride = { command: 'TIME', value: [String(clock), '0'] };
  const q = body();
  await publish(q);
  cli('SET', ipQuota, JSON.stringify({ minute: [minute, 30] }));
  await refused(await publish({ ...q, action: 'status', token: null }));
  console.log('all 9 approved request ceilings plus status counting: passed');
});

test('new creation leaves two close attempts per active secret near monthly ceiling', async () => {
  reset();
  const date = new Date();
  const month = Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1) / 1000;
  cli('SET', `${PREFIX}:quotas`, JSON.stringify({ mutationMonth: [month, 2800] }));
  await refused(await publish(body()));
  assert.equal((await publish(body({ action: 'close', token: null }))).status, 200);
});

test('active, reservation and memory caps fail before publication writes', async () => {
  for (const override of [
    { command: 'ZCOUNT', value: 100 },
    { command: 'HLEN', value: 100000 },
    { command: 'INFO', value: 'used_memory:100000000\r\n' },
  ]) {
    reset();
    readOverride = override;
    await refused(await publish(body()));
    assert.equal(cli('EXISTS', stateKey), 0);
    assert.equal(cli('EXISTS', `${stateKey}:secret:${hash(`${owner}:${body().id}`)}`), 0);
  }
  reset();
  for (let i = 0; i < 100; i++) cli('ZADD', `${PREFIX}:active`, seconds() + 600, `fixture-${i}`);
  await refused(await publish(body()));
  cli('ZADD', `${PREFIX}:active`, seconds() - 1, 'fixture-0');
  assert.equal(
    (await publish(body())).status,
    200,
    'expired score makes room without waiting for TTL sweep',
  );
  reset();
  candidate = fixed;
  fault = { which: 3, when: 'after' };
  const q = body();
  await refused(await publish(q));
  fault = undefined;
  for (let i = 0; i < 100; i++) cli('ZADD', `${PREFIX}:active`, seconds() + 600, `fixture-${i}`);
  await refused(await publish(q));
  assert.equal((await lookup(fixed)).status, 503, 'pending recovery cannot bypass active cap');
});

test('concurrent allocation and publish-close races keep one immutable owner', async () => {
  reset();
  candidate = fixed;
  const q = body();
  const races = await Promise.all([publish(q), publish(body({ id: 'cc'.repeat(32) }), otherKey)]);
  assert.deepEqual(races.map((r) => r.status).sort(), [200, 503]);
  reset();
  const original = await (await publish(q)).json();
  const close = { ...q, action: 'close', token: null };
  const replies = await Promise.all([publish(q), publish(close)]);
  assert.equal(replies[1].status, 200);
  assert.equal((await (await lookup(original.phrase)).json()).state, 'ended');
  await refused(await publish(q));
});

test('pending secret reserves capacity before the capability write', async () => {
  reset();
  for (let i = 0; i < 99; i++) cli('ZADD', `${PREFIX}:active`, seconds() + 600, `fixture-${i}`);
  const q = body();
  fault = { which: 4, when: 'after' };
  await refused(await publish(q));
  fault = undefined;
  assert.equal(
    cli('ZCOUNT', `${PREFIX}:active`, seconds() + 1, '+inf'),
    100,
    'tentative capability must already occupy a slot',
  );
  await refused(await publish(body({ id: 'cc'.repeat(32) })));
  assert.equal((await publish(q)).status, 200, 'reserved owner can reconcile even at capacity');
  assert.ok(Math.max(...commands) <= 16);
});

test('whole-response timeout cancels stalled streams and cannot later publish', async () => {
  let canceled = false;
  let attempts = 0;
  const stalled = createResolver({
    token: 'fixture',
    fetch: async () => {
      attempts++;
      return new Response(
        new ReadableStream({
          cancel() {
            canceled = true;
          },
        }),
      );
    },
  });
  await refused(await stalled(new Request(`https://join.example/${fixed}`), '127.0.0.1'));
  assert.equal(attempts, 1);
  assert.equal(canceled, true);
});

test('AOF restart retains closed reservations, never the discarded secret', async () => {
  reset();
  const q = body();
  const first = await (await publish(q)).json();
  await publish({ ...q, action: 'close', token: null });
  const stopped = new Promise((resolve) => redis.once('exit', resolve));
  const args = redis.spawnargs.slice(1);
  redis.kill('SIGKILL');
  await stopped;
  redis = spawn('redis-server', args, { stdio: 'ignore' });
  let ready = false;
  for (let attempt = 0; attempt < 100; attempt++) {
    try {
      if (cli('PING') === 'PONG') {
        ready = true;
        break;
      }
    } catch {
      /* AOF recovery in progress */
    }
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  assert.equal(ready, true);
  await refused(await publish(q));
  assert.equal((await (await lookup(first.phrase)).json()).state, 'ended');
  assert.equal(cli('TTL', stateKey), -1);
  assert.equal(cli('EXISTS', `${stateKey}:secret:${hash(`${owner}:${q.id}`)}`), 0);
});
