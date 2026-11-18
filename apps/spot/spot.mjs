// SPDX-License-Identifier: AGPL-3.0-or-later
// spot: lend your group a provider's models for a while, your key kept.
// `node apps/spot/spot.mjs [config]`, the config's comments say the rest.
//
// The key goes to `svrngs share` once, as the lend's secret; the door adds it
// to each request and displaces any a friend sends
// (crates/svrngs-node/src/door.rs:384), so the forwarder below holds none.
// It serves its chat page (page.html, page.js, markdown.js: a friend's, on a phone or a computer),
// what that page shows (/spot.json), and two OpenAI paths for the models
// named, sending the provider only authorization, content-type and accept:
// no X-Svrngs-*, no cookie, nobody's name.
import { spawnSync, spawn } from 'node:child_process';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline';
import { Readable } from 'node:stream';

const say = (line) => console.log(`spot: ${line}`);
const fail = (line) => {
  console.error(`spot: ${line}`);
  process.exit(1);
};
const n = (x) => x.toLocaleString('en-US');

// The config: the subset of TOML its file uses; any other line is refused by number.
function parse(text, file) {
  const out = {};
  let table = out;
  text.split('\n').forEach((raw, i) => {
    const line = raw.trim();
    if (line === '' || line.startsWith('#')) return;
    const section = line.match(/^\[([\w-]+)\]\s*(#.*)?$/);
    if (section) return void (table = out[section[1]] = {});
    const pair = line.match(/^"?([\w.-]+)"?\s*=\s*("(?:[^"\\]|\\.)*"|[\d_]+)\s*(#.*)?$/);
    if (!pair) fail(`${file}:${i + 1} is not \`key = "text"\` or \`key = 123\`: ${line}`);
    table[pair[1]] = pair[2].startsWith('"')
      ? JSON.parse(pair[2])
      : Number(pair[2].replaceAll('_', ''));
  });
  return out;
}

function endOf(until) {
  if (until === undefined) return null;
  const hm = String(until).match(/^(\d{1,2}):(\d{2})$/);
  const end = hm ? new Date() : new Date(until);
  if (hm) {
    end.setHours(Number(hm[1]), Number(hm[2]), 0, 0);
    if (end <= Date.now()) end.setDate(end.getDate() + 1);
  }
  if (isNaN(end)) fail(`until = ${JSON.stringify(until)} is not "HH:MM" or a date and time`);
  if (end <= Date.now()) fail(`until = ${JSON.stringify(until)} has already passed`);
  return end;
}

async function askHidden(question) {
  if (!process.stdin.isTTY)
    fail(`no key: set the variable the config's key_env names, or run spot in a terminal`);
  process.stdout.write(question);
  // stty is the same on macOS and Linux; echo comes back even if Ctrl-C ends the wait.
  process.once('exit', () => spawnSync('stty', ['echo'], { stdio: 'inherit' }));
  spawnSync('stty', ['-echo'], { stdio: 'inherit' });
  const rl = readline.createInterface({ input: process.stdin });
  const line = await new Promise((done) => rl.once('line', done));
  rl.close();
  spawnSync('stty', ['echo'], { stdio: 'inherit' });
  process.stdout.write('\n');
  return line.trim();
}

// svrngs's data dir, as svrngs and pantry find it on any computer:
// $SVRNGS_DIR, else ~/.svrngs (crates/svrngs/src/cli.rs:675, apps/pantry/pantry.mjs:42).
const dataDir = process.env.SVRNGS_DIR || path.join(os.homedir(), '.svrngs');

// The svrngs command: on PATH, else where its installer puts it on macOS and
// Linux alike, ~/.local/bin, which a Mac's terminal often does not search
// (docs/guide/models.md: "`svrngs` means `~/.local/bin/svrngs` if your
// terminal doesn't find it"). $SVRNGS_BIN names another.
function findSvrngs() {
  if (process.env.SVRNGS_BIN) return process.env.SVRNGS_BIN;
  const runs = (f) => {
    try {
      fs.accessSync(f, fs.constants.X_OK);
      return fs.statSync(f).isFile();
    } catch {
      return false;
    }
  };
  const onPath = (process.env.PATH ?? '')
    .split(path.delimiter)
    .filter(Boolean)
    .map((dir) => path.join(dir, 'svrngs'));
  const found = [...onPath, path.join(os.homedir(), '.local', 'bin', 'svrngs')].find(runs);
  if (!found) fail('no svrngs here: not on PATH, nor at ~/.local/bin/svrngs; install it first');
  return found;
}

// The config is the lender's, beside svrngs's own files on their computer:
// the template beside this file is copied there the first time, never read
// as the config itself.
const ownConfig = path.join(dataDir, 'spot.toml');
const file = path.resolve(process.argv[2] ?? ownConfig);
if (!fs.existsSync(file)) {
  if (process.argv[2]) fail(`no config at ${file}`);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.copyFileSync(new URL('spot.example.toml', import.meta.url), file);
  fail(`wrote your config, ${file}, from the template: say your group in it, then run spot again`);
}
const config = parse(fs.readFileSync(file, 'utf8'), file);
for (const key of ['group', 'name'])
  if (config[key] === undefined || config[key] === '') fail(`${file} says no \`${key}\``);
if (!config.provider?.url) fail(`${file} says no [provider] url`);
const models = new Map(Object.entries(config.models ?? {}));
if (models.size === 0) fail(`${file} lends no [models]`);
const upstream = config.provider.url.replace(/\/$/, '');
const end = endOf(config.until);
const budget = config.budget ?? null;
const svrngs = findSvrngs();
// Each night's log, by this computer's date: who used how many tokens.
const logDir = path.join(dataDir, 'spot');
fs.mkdirSync(logDir, { recursive: true, mode: 0o700 });
const today = new Date();
const day = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
const logPath = path.join(logDir, `${config.name}-${day}.jsonl`);

const envKey = config.provider.key_env && process.env[config.provider.key_env];
const key = envKey || (await askHidden(`Your ${new URL(upstream).hostname} key (not shown): `));
if (!key) fail('no key given');
const bearer = /^bearer /i.test(key) ? key : `Bearer ${key}`;

// Before lending: does the provider take the key, and does it serve each model?
const listed = await fetch(`${upstream}/models`, { headers: { authorization: bearer } }).catch(
  (e) => fail(`${upstream} did not answer: ${e.cause?.code ?? e.message}`),
);
if (listed.status === 401 || listed.status === 403)
  fail(`${new URL(upstream).hostname} refused the key (${listed.status})`);
if (!listed.ok) fail(`${upstream}/models answered ${listed.status}`);
const served = new Set(((await listed.json()).data ?? []).map((m) => m.id));
for (const [alias, id] of models)
  if (!served.has(id)) fail(`${new URL(upstream).hostname} does not list ${id} (for ${alias})`);

// The page's files, read once: path → [type, bytes].
const html = 'text/html; charset=utf-8';
const js = 'text/javascript; charset=utf-8';
const files = new Map(
  [
    ['/', 'page.html', html],
    ['/index.html', 'page.html', html],
    ['/page.js', 'page.js', js],
    ['/markdown.js', 'markdown.js', js],
  ].map(([at, file, type]) => [at, [type, fs.readFileSync(new URL(file, import.meta.url))]]),
);
const used = new Map(); // person → tokens the provider reported
let spent = 0;
const log = (row) =>
  fs.appendFileSync(logPath, JSON.stringify({ at: new Date().toISOString(), ...row }) + '\n');
const refuse = (res, who, status, why) => {
  log({ who, refused: status, why });
  console.log(`  ${who}: refused, ${why}`);
  res.writeHead(status, { 'content-type': 'application/json' });
  res.end(JSON.stringify({ error: { message: why, type: 'spot_refused' } }));
};

const server = http.createServer(async (req, res) => {
  const who = req.headers['x-svrngs-person'] ?? '(not through svrngs)';
  const route = req.url.split('?')[0];
  if (req.method === 'GET' && files.has(route)) {
    const [type, bytes] = files.get(route);
    res.writeHead(200, { 'content-type': type, 'cache-control': 'no-cache' });
    return res.end(bytes);
  }
  // What the page shows: what is lent, until when, what is left, what this person used.
  if (req.method === 'GET' && route === '/spot.json') {
    const ended =
      end && Date.now() > end
        ? `This lend ended at ${end.toLocaleTimeString()}.`
        : budget !== null && spent >= budget
          ? `This lend's ${n(budget)} tokens are spent.`
          : null;
    res.writeHead(200, { 'content-type': 'application/json', 'cache-control': 'no-store' });
    return res.end(
      JSON.stringify({
        name: config.name,
        models: [...models.keys()],
        provider: new URL(upstream).hostname,
        until: end?.toISOString() ?? null,
        budget,
        spent,
        yours: used.get(who) ?? 0,
        ended,
      }),
    );
  }
  if (req.method === 'GET' && route === '/v1/models') {
    res.writeHead(200, { 'content-type': 'application/json' });
    const data = [...models.keys()].map((id) => ({ id, object: 'model', owned_by: config.name }));
    return res.end(JSON.stringify({ object: 'list', data }));
  }
  if (!(req.method === 'POST' && route === '/v1/chat/completions'))
    return refuse(
      res,
      who,
      404,
      `only the page, GET /v1/models and POST /v1/chat/completions are lent, not ${req.method} ${route}`,
    );
  if (end && Date.now() > end)
    return refuse(res, who, 403, `this lend ended at ${end.toLocaleTimeString()}`);
  if (budget !== null && spent >= budget)
    return refuse(res, who, 403, `tonight's ${n(budget)} tokens are spent`);
  if (!req.headers.authorization)
    return refuse(res, who, 401, 'no key arrived: it reaches spot only through svrngs');
  const chunks = [];
  let size = 0;
  for await (const c of req) {
    if ((size += c.length) > 16 << 20) return refuse(res, who, 413, 'a request over 16 MiB');
    chunks.push(c);
  }
  let body;
  try {
    body = JSON.parse(Buffer.concat(chunks));
  } catch {
    return refuse(res, who, 400, 'the body is not JSON');
  }
  const alias = body.model;
  if (!models.has(alias))
    return refuse(
      res,
      who,
      400,
      `${JSON.stringify(alias)} is not lent; lent: ${[...models.keys()].join(', ')}`,
    );
  body.model = models.get(alias);
  if (body.stream) body.stream_options = { ...body.stream_options, include_usage: true };
  let answer;
  try {
    answer = await fetch(`${upstream}/chat/completions`, {
      method: 'POST',
      headers: {
        authorization: req.headers.authorization,
        'content-type': 'application/json',
        accept: req.headers.accept ?? '*/*',
      },
      body: JSON.stringify(body),
    });
  } catch (e) {
    return refuse(res, who, 502, `the provider did not answer: ${e.cause?.code ?? e.message}`);
  }
  res.writeHead(answer.status, {
    'content-type': answer.headers.get('content-type') ?? 'application/json',
  });
  let tail = '';
  const stream = Readable.fromWeb(answer.body);
  stream.on('data', (c) => {
    tail = (tail + c).slice(-8192);
    res.write(c);
  });
  stream.on('error', (e) => {
    log({ who, model: alias, status: answer.status, broke: e.message });
    res.end();
  });
  stream.on('end', () => {
    res.end();
    let usage = null;
    try {
      usage = JSON.parse([...tail.matchAll(/"usage"\s*:\s*(\{[^{}]*\})/g)].pop()?.[1]);
    } catch {}
    const tokens = usage?.total_tokens;
    if (tokens) {
      spent += tokens;
      used.set(who, (used.get(who) ?? 0) + tokens);
    }
    log({ who, model: alias, status: answer.status, tokens: tokens ?? 'not reported', spent });
    const what = answer.ok
      ? `${tokens ? n(tokens) : 'unreported'} tokens`
      : `the provider answered ${answer.status}`;
    console.log(`  ${who}: ${alias}, ${what}${budget ? ` (${n(spent)} of ${n(budget)})` : ''}`);
  });
});

let stopping = false;
function stop(why) {
  if (stopping) return;
  stopping = true;
  server.close();
  const un = spawnSync(svrngs, ['unshare', config.name, '--group', config.group], {
    stdio: 'inherit',
  });
  const who = [...used].map(([p, t]) => `${p} ${n(t)}`).join(', ') || 'nobody';
  say(
    `${why}. Used: ${who}${budget ? `, ${n(spent)} of ${n(budget)} tokens` : ''}. The log: ${logPath}`,
  );
  if (un.status !== 0) say(`svrngs unshare did not finish: run \`svrngs unshare ${config.name}\``);
  process.exit(un.status === 0 ? 0 : 1);
}

server.on('error', (e) =>
  fail(
    `port ${config.port}: ${e.code === 'EADDRINUSE' ? 'something else already listens there' : e.message}`,
  ),
);
// A port the config names, else any free one: several spots can run at once.
server.listen(config.port ?? 0, '127.0.0.1', () => {
  const share = spawn(
    svrngs,
    [
      'share',
      String(server.address().port),
      '--group',
      config.group,
      '--program',
      'spot',
      '--name',
      config.name,
    ],
    {
      stdio: ['pipe', 'inherit', 'inherit'],
    },
  );
  share.stdin.end(`${bearer}\n`);
  share.on('error', (e) => fail(`could not run ${svrngs}: ${e.message}`));
  share.on('exit', (code) => {
    if (code !== 0) fail(`svrngs share did not lend it (exit ${code})`);
    log({
      lent: config.name,
      group: config.group,
      models: Object.fromEntries(models),
      until: end?.toISOString() ?? null,
      budget,
    });
    const left = end
      ? ` until ${end.toLocaleString([], { weekday: 'short', hour: '2-digit', minute: '2-digit' })}`
      : '';
    say(
      `lending ${[...models.keys()].join(', ')} to ${config.group}${left}${budget ? `, ${n(budget)} tokens` : ''}. Ctrl-C stops it.`,
    );
    say(
      `friends open ${config.name} from their svrngs home page, on a phone or a computer; programs reach it through pantry`,
    );
    if (end) setTimeout(() => stop(`stopped at ${end.toLocaleTimeString()}`), end - Date.now());
  });
});
process.on('SIGINT', () => stop('stopped'));
process.on('SIGTERM', () => stop('stopped'));
