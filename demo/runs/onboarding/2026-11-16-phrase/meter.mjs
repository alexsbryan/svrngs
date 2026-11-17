// SPDX-License-Identifier: AGPL-3.0-or-later
// Metered Redis commands per resolver operation, as Upstash bills them: the
// EVAL counts 1 and each inner redis.call counts 1 (ONBOARDING.md §5). It runs
// the real createResolver of the checkout at <root> against its own local
// redis-server on a Unix socket, and answers what each operation cost.
//
//   node demo/runs/onboarding/2026-11-16-phrase/meter.mjs <root> <never-issued phrase>
//
// meter.txt holds the run before (699187e3, five words) and after (a07799fe).
import { spawn, spawnSync, execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';

const [root, stranger] = process.argv.slice(2);
const { createResolver } = await import(`${root}/resolver/handler.mjs`);
const { GENERATION, PREFIX } = await import(`${root}/resolver/store.mjs`);
const { body, mutation } = await import(`${root}/web/tests/resolver-fixture.mjs`);
const dir = mkdtempSync(join(tmpdir(), 'svrngs-meter-'));
const socket = join(dir, 'redis.sock');
const args = ['--port', '0', '--unixsocket', socket, '--dir', dir, '--save', ''];
const redis = spawn('redis-server', args, { stdio: 'ignore' });
const cli = (...a) =>
  JSON.parse(
    spawnSync('redis-cli', ['-s', socket, '--json', ...a.map(String)], { encoding: 'utf8' })
      .stdout || 'null',
  );
for (let i = 0; i < 100 && cli('PING') !== 'PONG'; i++)
  await new Promise((resolve) => setTimeout(resolve, 20));
cli('SET', `${PREFIX}:generation`, GENERATION);
const execute = promisify(execFile);
let metered;
async function fetch(_url, options) {
  const command = JSON.parse(options.body);
  const script = command[1].replaceAll('redis.call(', 'call(');
  command[1] = `local metered = 1
local function call(...) metered = metered + 1 return redis.call(...) end
local result = cjson.decode((function()\n${script}\nend)())
result.metered = metered
return cjson.encode(result)`;
  const { stdout } = await execute('redis-cli', ['-s', socket, '--json', ...command.map(String)]);
  const result = JSON.parse(JSON.parse(stdout));
  metered = result.metered;
  delete result.metered;
  return Response.json({ result: JSON.stringify(result) });
}
const handle = createResolver({ token: 'fixture', fetch });
async function ask(label, request) {
  metered = 0;
  const answer = await handle(request, '127.0.0.1');
  console.log(`${label}: ${answer.status}, ${metered} commands`);
  return answer;
}
const q = body();
const { phrase } = await (await ask('publish', mutation(q))).json();
await ask('lookup, open', new Request(`https://join.example/${phrase}`));
await ask('lookup, never issued', new Request(`https://join.example/${stranger}`));
await ask('lookup, cannot be read', new Request('https://join.example/acron'));
await ask('status', mutation({ ...q, action: 'status', token: null }));
await ask('close', mutation({ ...q, action: 'close', token: null }));
await ask('lookup, ended', new Request(`https://join.example/${phrase}`));
const ip = `${PREFIX}:quota:ip:${createHash('sha256').update('127.0.0.1').digest('hex')}`;
cli('SET', ip, JSON.stringify({ minute: [Math.floor(Date.now() / 60000), 30] }));
await ask('lookup, issued, refused', new Request(`https://join.example/${phrase}`));
await ask('lookup, never issued, refused', new Request(`https://join.example/${stranger}`));
redis.kill();
await new Promise((resolve) => redis.once('exit', resolve));
rmSync(dir, { recursive: true, force: true });
