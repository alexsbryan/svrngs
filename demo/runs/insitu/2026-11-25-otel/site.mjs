// The site tool: installs a vendor's signed bundle on this machine, with no
// shell for anyone. It knows nothing about what it installs: a recipe is a
// list of steps from a closed set, each with its own check, and needs from a
// closed set of read-only probes. It runs as the machine's admin.
//
//   node site.mjs trust <vendor key file>        pin the vendor's key, once
//   node site.mjs plan  <bundle> [--set k=v]... [--auto]
//   node site.mjs apply <bundle> --token <t> [--set k=v]... [--auto]
//   node site.mjs status <service>
//   node site.mjs allow-trials <service>          a standing yes for sandboxed trials
//   node site.mjs trial <candidate bundle>       the candidate and production, same inputs
//   node site.mjs watch <delivery dir>           trial each candidate dropped there
//   node site.mjs view <service> [--port 8620]   the read-only view for the vendor's agent
//
// A bundle's channel is release or candidate. Only a release is applied.
// A candidate is only trialed: it and the installed release each replay the
// service's real inputs once, sandboxed (no network, inputs read-only, a
// throwaway output), and the verdict compares them input by input. A trial
// can't change production or send anything anywhere, which is why the admin
// may say yes to trials once, and to each release every time.
//
// `plan` changes nothing. It says what it found and what it would do, and
// prints a token for exactly that. `apply` re-plans and refuses unless the
// token matches, so what runs is what the admin saw. Every probe, step and
// check ends in one of four verdicts, and every verdict but passed says why.
// Probes report only facts about the machine (sizes, ports, names, states),
// never file contents or service logs, so a diagnostic view of them (a support
// link) can't carry the data the service holds.
import { createHash, createPublicKey, verify } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { appendFileSync, existsSync, mkdirSync, readFileSync, renameSync, rmSync, statSync, statfsSync, writeFileSync, chmodSync } from 'node:fs';
import http from 'node:http';
import net from 'node:net';
import { dirname, join } from 'node:path';
import { randomBytes } from 'node:crypto';
import { contract, project } from './project.mjs';

const HOME = process.env.ONPREM_DIR ?? '/var/lib/onprem';
const TRUST = join(HOME, 'trusted-vendor.pub');
const LIB = '/usr/local/lib/onprem'; // the site's own pieces, installed by trust
const OTLP_PORT = 4318;
const V = Object.freeze({ passed: 'passed', failed: 'failed', unjudged: 'could-not-judge', never: 'never-ran' });
const sha = (b) => createHash('sha256').update(b).digest('hex');
const run = (cmd, args) => spawnSync(cmd, args, { encoding: 'utf8' });
const debug = process.env.ONPREM_DEBUG ? (...a) => console.error('debug:', ...a) : () => {};

// --- The recipe: parse, don't validate -------------------------------------

const STEPS = ['user', 'dir', 'unpack', 'telemetry', 'gateway', 'unit', 'health'];
const UNITS = ['telemetry', 'gateway', 'unit']; // the steps that run something, restarted when anything before them changed
const PROBES = ['memory', 'disk_free', 'port_free', 'systemd', 'disk_encrypted'];

function readBundle(dir) {
  const bytes = readFileSync(join(dir, 'recipe.json'));
  if (!existsSync(TRUST)) throw refusal(`no vendor is trusted on this machine yet: run \`node site.mjs trust <vendor key file>\` with the key the vendor gave you outside this bundle`);
  const key = createPublicKey(readFileSync(TRUST, 'utf8'));
  const sig = Buffer.from(readFileSync(join(dir, 'recipe.sig'), 'utf8').trim(), 'base64');
  const signed = verify(null, bytes, key, sig);
  debug('signature', signed ? 'holds' : 'does not hold', 'for', sha(bytes).slice(0, 12));
  if (!signed) throw refusal('this bundle\'s recipe is not signed by the vendor this machine trusts; nothing was read further');
  const r = JSON.parse(bytes);
  for (const s of r.steps) if (!STEPS.includes(s.do)) throw refusal(`the recipe asks for a step "${s.do}" this tool doesn't have; it has ${STEPS.join(', ')}`);
  for (const a of r.artifacts) {
    const got = existsSync(join(dir, a.name)) ? sha(readFileSync(join(dir, a.name))) : null;
    if (got !== a.sha256) throw refusal(`${a.name} in this bundle ${got ? `isn't the file the vendor signed (sha256 ${got.slice(0, 12)}…, signed ${a.sha256.slice(0, 12)}…)` : 'is missing'}; nothing was done`);
  }
  for (const n of r.needs) if (!PROBES.includes(n.probe)) throw refusal(`the recipe asks for a probe "${n.probe}" this tool doesn't have; it has ${PROBES.join(', ')}`);
  return { recipe: r, bytes, dir };
}

function params(recipe, sets) {
  const out = {};
  for (const [k, p] of Object.entries(recipe.params)) {
    const v = sets[k] ?? p.default;
    if (p.type === 'port' && !p.choices.includes(Number(v))) throw refusal(`port must be one of ${p.choices.join(', ')}, the vendor's choices; not ${v}`);
    if (p.type === 'dir' && !/^\/[\w./-]+$/.test(v)) throw refusal(`${k} must be an absolute path; not ${v}`);
    if (p.type === 'url' && !/^https?:\/\/[\w.-]+(:\d+)?(\/[\w./-]*)?$/.test(v)) throw refusal(`${k} must be an http or https address; not ${v}`);
    out[k] = p.type === 'port' ? Number(v) : v;
  }
  out.version = recipe.version;
  // Where the service sends its telemetry and its effects: the site's own
  // collector and gateway, never the outside.
  out.otlp = `http://127.0.0.1:${OTLP_PORT}`;
  for (const name of Object.keys(recipe.effects ?? {})) out[`effect.${name}`] = `http://127.0.0.1:${out.gateway_port}/${name}`;
  for (const k of Object.keys(sets)) if (!(k in recipe.params)) throw refusal(`the recipe has no setting ${k}; it has ${Object.keys(recipe.params).join(', ')}`);
  return out;
}

const fill = (v, p) => (typeof v === 'string' ? v.replace(/\{([\w.]+)\}/g, (_, k) => String(p[k])) : Array.isArray(v) ? v.map((x) => fill(x, p)) : v);

// --- Probes: read-only facts about this machine ---------------------------

const nearest = (p) => { while (!existsSync(p)) p = dirname(p); return p; };

async function probe(n, p, b) {
  const at = fill(n.at, p);
  switch (n.probe) {
    case 'memory': {
      const kb = Number(/MemTotal:\s+(\d+)/.exec(readFileSync('/proc/meminfo', 'utf8'))?.[1]);
      if (!kb) return { verdict: V.unjudged, why: '/proc/meminfo names no MemTotal', facts: {} };
      const gb = +(kb / 1048576).toFixed(1);
      return gb >= n.min_gb ? { verdict: V.passed, facts: { memory_gb: gb } } : { verdict: V.failed, why: `${gb} GB of memory; the service needs ${n.min_gb}`, facts: { memory_gb: gb } };
    }
    case 'disk_free': {
      const s = statfsSync(nearest(at));
      const gb = +((s.bavail * s.bsize) / 2 ** 30).toFixed(1);
      return gb >= n.min_gb ? { verdict: V.passed, facts: { at, free_gb: gb } } : { verdict: V.failed, why: `${gb} GB free at ${at}; the service needs ${n.min_gb}`, facts: { at, free_gb: gb } };
    }
    case 'port_free': {
      const port = Number(fill(n.port, p));
      if (b.recipe.steps.some((x) => x.do === 'unit' && portOfUnit(x.name) === port)) return { verdict: V.passed, facts: { port, held_by: 'this service' } };
      const free = await new Promise((ok) => { const s = net.createServer().once('error', () => ok(false)).listen(port, '0.0.0.0', () => s.close(() => ok(true))); });
      if (free) return { verdict: V.passed, facts: { port } };
      const who = run('ss', ['-Htlnp', `sport = :${port}`]).stdout.match(/users:\(\("([^"]+)"/)?.[1] ?? 'a program ss could not name';
      return { verdict: V.failed, why: `port ${port} is taken by ${who}`, facts: { port, held_by: who } };
    }
    case 'systemd':
      return existsSync('/run/systemd/system') ? { verdict: V.passed, facts: { init: 'systemd' } } : { verdict: V.failed, why: 'systemd is not this machine\'s init, so the service can\'t be kept running', facts: {} };
    case 'disk_encrypted': {
      const src = run('findmnt', ['-no', 'SOURCE', '-T', nearest(at)]).stdout.trim();
      const tree = src ? run('lsblk', ['-sno', 'TYPE', src.replace(/\[.*$/, '')]) : null;
      if (!tree || tree.status !== 0) return { verdict: V.unjudged, why: `can't tell which block device holds ${at} (it is on ${src || 'nothing findmnt names'})`, facts: { at, source: src || null } };
      const crypt = tree.stdout.split('\n').includes('crypt');
      return crypt ? { verdict: V.passed, facts: { at, encrypted: true } } : { verdict: V.failed, why: `${at} is on a disk that isn't encrypted`, facts: { at, encrypted: false } };
    }
  }
}


// --- Steps: each with its own check, so a rerun does only what's undone ----

function stepCheck(s, p, b) {
  switch (s.do) {
    case 'user': return run('id', ['-u', s.name]).status === 0;
    case 'dir': {
      const path = fill(s.path, p);
      if (!existsSync(path)) return false;
      const st = statSync(path);
      const owner = run('id', ['-u', s.owner]).stdout.trim();
      return (st.mode & 0o777).toString(8) === s.mode.slice(1) && String(st.uid) === owner;
    }
    case 'unpack': {
      const mark = join(fill(s.to, p), '.onprem-artifact');
      return existsSync(mark) && readFileSync(mark, 'utf8').trim() === artifact(b, s.artifact).sha256;
    }
    case 'telemetry':
    case 'gateway':
    case 'unit': {
      const u = unitOf(s, b, p);
      const file = `/etc/systemd/system/${u.name}.service`;
      const config = configOf(s, b, p);
      return existsSync(file) && readFileSync(file, 'utf8') === unitText(u, p) && (!config || (existsSync(config.path) && readFileSync(config.path, 'utf8') === config.text)) && run('systemctl', ['is-active', '--quiet', u.name]).status === 0;
    }
    case 'health': return null; // always run: it is the install's last word
  }
}

const artifact = (b, name) => {
  const a = b.recipe.artifacts.find((x) => x.name === name);
  if (!a) throw refusal(`the recipe names no artifact ${name}`);
  return a;
};

// The site's own units, made from a closed step: what runs is the site's,
// with only the vendor's names and addresses filled in.
function unitOf(s, b, p) {
  const svc = b.recipe.service;
  if (s.do === 'telemetry') return { name: `onprem-otel-${svc}`, user: 'root', says: `${svc}'s telemetry, kept on this machine`, exec: ['/usr/local/bin/otelcol-contrib', '--config', `/etc/onprem/otel-${svc}.yaml`], writes: [join(HOME, 'telemetry', svc)] };
  if (s.do === 'gateway') return { name: `onprem-gateway-${svc}`, user: 'root', says: `${svc}'s declared effects, each one recorded`, exec: ['/usr/bin/node', join(LIB, 'gateway.mjs'), '--port', String(p.gateway_port), ...Object.entries(b.recipe.effects).flatMap(([n, e]) => ['--route', `${n}=${fill(e.upstream, p)}`]), '--record', join(HOME, 'effects', `${svc}.jsonl`)], writes: [join(HOME, 'effects')] };
  return s;
}

function configOf(s, b, p) {
  if (s.do !== 'telemetry') return null;
  const svc = b.recipe.service;
  return { path: `/etc/onprem/otel-${svc}.yaml`, text: `# The site's collector for ${svc}: it takes OTLP on loopback and keeps it here.
receivers:
  otlp:
    protocols:
      http:
        endpoint: 127.0.0.1:${OTLP_PORT}
exporters:
  file:
    path: ${join(HOME, 'telemetry', svc, 'raw.jsonl')}
    flush_interval: 1s
service:
  telemetry:
    metrics:
      level: none
  pipelines:
    traces: { receivers: [otlp], exporters: [file] }
    logs: { receivers: [otlp], exporters: [file] }
` };
}

const unitText = (s, p) => `# Written by the site tool from ${s.name}'s signed recipe. Rerun the tool rather than editing this.
[Unit]
Description=${s.says}
After=network.target

[Service]
User=${s.user}
${Object.entries(s.env ?? {}).map(([k, v]) => `Environment=${k}=${fill(v, p)}\n`).join('')}ExecStart=${fill(s.exec, p).map((a) => (/\s/.test(a) ? `"${a}"` : a)).join(' ')}
Restart=on-failure
RestartSec=2
NoNewPrivileges=yes
ProtectSystem=strict
ReadWritePaths=${fill(s.writes, p).join(' ')}
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
`;

async function waitPort(port, within = 10_000) {
  const until = Date.now() + within;
  while (Date.now() < until) {
    const open = await new Promise((ok) => { const c = net.connect(port, '127.0.0.1').once('connect', () => { c.end(); ok(true); }).once('error', () => ok(false)); });
    if (open) return;
    await new Promise((ok) => setTimeout(ok, 200));
  }
  throw new Error(`nothing answered on port ${port} within ${within / 1000} s`);
}

async function doStep(s, p, b, changed) {
  switch (s.do) {
    case 'user':
      execFileSync('useradd', ['--system', '--no-create-home', '--shell', '/usr/sbin/nologin', s.name]);
      return { verdict: V.passed, did: `made the account ${s.name}, which can't sign in` };
    case 'dir': {
      const path = fill(s.path, p);
      mkdirSync(path, { recursive: true });
      chmodSync(path, Number.parseInt(s.mode, 8));
      execFileSync('chown', [`${s.owner}:${s.owner}`, path]);
      return { verdict: V.passed, did: `made ${path}, ${s.mode}, owned by ${s.owner}` };
    }
    case 'unpack': {
      const a = artifact(b, s.artifact);
      const file = join(b.dir, a.name);
      const got = sha(readFileSync(file));
      if (got !== a.sha256) return { verdict: V.failed, why: `${a.name} in the bundle isn't the file the vendor signed (sha256 ${got.slice(0, 12)}…, signed ${a.sha256.slice(0, 12)}…); nothing was unpacked` };
      const to = fill(s.to, p), next = `${to}.next`;
      rmSync(next, { recursive: true, force: true });
      mkdirSync(next, { recursive: true });
      execFileSync('tar', ['-xzf', file, '-C', next]);
      writeFileSync(join(next, '.onprem-artifact'), a.sha256 + '\n');
      rmSync(`${to}.prev`, { recursive: true, force: true });
      if (existsSync(to)) renameSync(to, `${to}.prev`);
      renameSync(next, to);
      return { verdict: V.passed, did: `unpacked ${a.name} (${a.sha256.slice(0, 12)}…) to ${to}` };
    }
    case 'telemetry':
    case 'gateway':
    case 'unit': {
      const u = unitOf(s, b, p);
      const config = configOf(s, b, p);
      if (config) { mkdirSync(dirname(config.path), { recursive: true }); writeFileSync(config.path, config.text); }
      for (const w of fill(u.writes, p)) mkdirSync(w, { recursive: true, mode: 0o700 });
      const file = `/etc/systemd/system/${u.name}.service`;
      writeFileSync(file, unitText(u, p));
      execFileSync('systemctl', ['daemon-reload']);
      execFileSync('systemctl', ['enable', u.name], { stdio: 'ignore' });
      execFileSync('systemctl', [changed ? 'restart' : 'start', u.name]);
      if (s.do === 'telemetry') await waitPort(OTLP_PORT);
      if (s.do === 'gateway') await waitPort(p.gateway_port);
      return { verdict: V.passed, did: `${changed ? 'restarted' : 'started'} ${u.name}, kept running by systemd and started at boot` };
    }
    case 'health': {
      const url = fill(s.url, p);
      const until = Date.now() + s.within_s * 1000;
      let last = 'no answer yet';
      const want = Object.fromEntries(Object.entries(s.expect).map(([k, v]) => [k, fill(v, p)]));
      while (Date.now() < until) {
        try {
          const r = await fetch(url, { signal: AbortSignal.timeout(2000) });
          const body = await r.json();
          const off = Object.entries(want).filter(([k, v]) => body[k] !== v);
          if (r.ok && !off.length) return { verdict: V.passed, did: `${url} answered ${JSON.stringify(want)}` };
          last = `${r.status}, ${off.map(([k, v]) => `${k} ${JSON.stringify(body[k])} not ${JSON.stringify(v)}`).join('; ') || 'not ok'}`;
        } catch (e) { last = e.cause?.code ?? e.message; }
        await new Promise((ok) => setTimeout(ok, 500));
      }
      const unit = b.recipe.steps.find((x) => x.do === 'unit')?.name;
      const f = unit ? Object.fromEntries(run('systemctl', ['show', unit, '-p', 'ActiveState,SubState,ExecMainStatus,NRestarts']).stdout.trim().split('\n').map((l) => l.split('='))) : null;
      const state = f ? `${f.ActiveState} (${f.SubState}), last exit status ${f.ExecMainStatus}, restarted ${f.NRestarts} times` : 'no unit';
      // The unit's state and exit status, never its log: a service's log may hold what it serves.
      return { verdict: V.failed, why: `${url} didn't answer as the recipe expects within ${s.within_s} s (last: ${last}); the unit is ${state}` };
    }
  }
}

// --- Plan and apply ---------------------------------------------------------

async function plan(b, sets, auto) {
  let p = params(b.recipe, sets);
  let needs = await Promise.all(b.recipe.needs.map(async (n) => ({ n, ...(await probe(n, p, b)) })));
  // --auto: where a need fails on a setting the vendor gave choices for, take
  // the first choice that passes, and say so.
  const chose = [];
  if (auto) for (const [k, def] of Object.entries(b.recipe.params)) {
    if (def.type !== 'port' || k in sets) continue;
    const blocked = needs.some((x) => x.verdict === V.failed && x.n.port === `{${k}}`);
    if (!blocked) continue;
    for (const c of def.choices) {
      const tryP = { ...p, [k]: c };
      const again = await Promise.all(b.recipe.needs.filter((n) => n.port === `{${k}}`).map((n) => probe(n, tryP, b)));
      if (again.every((x) => x.verdict === V.passed)) { chose.push(`${k}=${c} (${p[k]} was taken)`); p = tryP; break; }
    }
    needs = await Promise.all(b.recipe.needs.map(async (n) => ({ n, ...(await probe(n, p, b)) })));
  }
  const steps = [];
  for (const s of b.recipe.steps) {
    // A unit runs what the steps before it put in place, so it is undone
    // (restarted) whenever any of them is.
    const before = steps.some((x) => x.done === false);
    steps.push({ s, done: UNITS.includes(s.do) && before ? false : stepCheck(s, p, b) });
  }
  const blockers = needs.filter((x) => x.verdict !== V.passed && x.n.severity !== 'warn');
  const facts = { recipe: sha(b.bytes), params: p, needs: needs.map((x) => [x.n.probe, x.verdict, x.facts]), steps: steps.map((x) => [x.s.do, x.done]) };
  return { p, needs, steps, blockers, chose, token: sha(JSON.stringify(facts)).slice(0, 16) };
}

function portOfUnit(name) {
  const file = `/etc/systemd/system/${name}.service`;
  if (!existsSync(file) || run('systemctl', ['is-active', '--quiet', name]).status !== 0) return null;
  return Number(/--port\s+(\d+)/.exec(readFileSync(file, 'utf8'))?.[1]) || null;
}

function say(b, pl) {
  const r = b.recipe;
  console.log(`${r.says}, version ${r.version}, signed by the vendor this machine trusts.`);
  console.log(`Settings: ${Object.entries(pl.p).map(([k, v]) => `${k} ${v}`).join(', ')}${pl.chose.length ? `  (chosen: ${pl.chose.join('; ')})` : ''}`);
  console.log('\nWhat this machine has:');
  for (const x of pl.needs) console.log(`  ${x.verdict === V.passed ? 'ok  ' : x.n.severity === 'warn' ? 'warn' : 'STOP'} ${x.n.says}${x.why ? `: ${x.why}` : ''}`);
  console.log('\nWhat it would do:');
  for (const x of pl.steps) console.log(`  ${x.done === true ? 'done' : x.done === null ? 'check' : '    '} ${x.s.says}`);
  if (pl.blockers.length) {
    console.log(`\nNothing can be installed until ${pl.blockers.length === 1 ? 'this is' : 'these are'} fixed.`);
    for (const x of pl.blockers) {
      const k = Object.keys(r.params).find((k) => x.n.port === `{${k}}`);
      if (k) console.log(`  The vendor allows ${k} to be ${r.params[k].choices.join(', ')}: rerun with --auto to take the first free one, or --set ${k}=<one>.`);
    }
    return 1;
  }
  const todo = pl.steps.filter((x) => x.done === false).length;
  console.log(`\n${todo === 0 ? 'Nothing to change; apply would only check it answers' : todo === 1 ? 'One step to run' : `${todo} steps to run`}. To run exactly this, and nothing if anything above changes first:\n  node site.mjs apply ${b.dir} --token ${pl.token}${argvSets()}`);
  return 0;
}

const argvSets = () => process.argv.slice(2).filter((a, i, all) => a === '--auto' || a === '--set' || all[i - 1] === '--set').map((a) => ` ${a}`).join('');

async function apply(b, sets, auto, token) {
  if (b.recipe.channel !== 'release') throw refusal(`this bundle is a ${b.recipe.channel ?? 'unlabelled'} build, not a release: it can only be trialed here (\`node site.mjs trial ${b.dir}\`), never installed`);
  const pl = await plan(b, sets, auto);
  if (pl.token !== token) {
    debug('token', token, 'but the plan now is', pl.token);
    console.log('What you approved has changed since you saw it, so nothing ran. Here is what this machine has now:\n');
    say(b, pl);
    return 1;
  }
  if (pl.blockers.length) return say(b, pl);
  mkdirSync(HOME, { recursive: true });
  const journal = join(HOME, `${b.recipe.service}.journal`);
  const note = (row) => appendFileSync(journal, JSON.stringify({ at: new Date().toISOString(), version: b.recipe.version, ...row }) + '\n');
  note({ applying: pl.token, params: pl.p });
  let changed = false;
  for (const { s, done } of pl.steps) {
    if (done === true) { console.log(`done already  ${s.says}`); continue; }
    let r;
    try { r = await doStep(s, pl.p, b, changed); } catch (e) { r = { verdict: V.failed, why: `${e.message.split('\n')[0]}` }; }
    if (r.verdict === V.passed && s.do !== 'health') {
      const holds = stepCheck(s, pl.p, b);
      if (holds !== true) r = { verdict: V.failed, why: `the step ran, but its own check doesn't hold afterwards` };
    }
    note({ step: s.do, says: s.says, ...r });
    console.log(`${r.verdict.padEnd(12)}  ${s.says}${r.why ? `: ${r.why}` : ''}`);
    if (r.verdict !== V.passed) {
      const rest = pl.steps.slice(pl.steps.findIndex((x) => x.s === s) + 1);
      for (const x of rest) { note({ step: x.s.do, says: x.s.says, verdict: V.never, why: 'an earlier step failed' }); console.log(`${V.never.padEnd(12)}  ${x.s.says}`); }
      console.log(`\nStopped. What ran is in ${journal}; rerunning plan picks up from here.`);
      return 1;
    }
    changed = true;
  }
  writeFileSync(applied(b.recipe.service), JSON.stringify({ recipe: b.recipe, params: pl.p, token: pl.token }, null, 1));
  console.log(`\nInstalled ${b.recipe.service} ${b.recipe.version}. The journal is ${journal}.`);
  return 0;
}

// --- Trials: a candidate beside production, on the same inputs and state ----

const applied = (service) => join(HOME, `${service}.applied.json`);
const readApplied = (service) => {
  if (!existsSync(applied(service))) throw refusal(`no release of ${service} is installed here, so there is nothing to compare a candidate with`);
  return JSON.parse(readFileSync(applied(service), 'utf8'));
};
const shapeKey = () => readFileSync(join(HOME, 'shape.key'), 'utf8').trim();
const lines = (f) => (existsSync(f) ? readFileSync(f, 'utf8').split('\n').filter(Boolean) : []);
const projected = (file, recipe, limit = 400) => {
  const c = contract(recipe.telemetry), key = shapeKey();
  return lines(file).flatMap((l) => { try { return project(l, c, key); } catch { return [{ kind: 'unreadable' }]; } }).slice(-limit);
};

// A trial's sandbox must have no network, and the site proves it rather than
// trusting systemd to have made one: where it can't make a network
// namespace, systemd says "proceeding without" and runs the unit with the
// network (2026-11-25: 353 claims reached the clearinghouse from trials that
// way). So the same properties run `ip -o link` first, and anything but
// loopback refuses the trial. Each arm checks again before it runs.
const SANDBOX = (user, writes) => ['-p', `User=${user}`, '-p', 'PrivateNetwork=yes', '-p', 'ProtectSystem=strict', '-p', 'ProtectHome=yes', '-p', 'PrivateTmp=yes', '-p', 'NoNewPrivileges=yes', ...(writes ? ['-p', `ReadWritePaths=${writes}`] : []), '-p', 'RuntimeMaxSec=180'];
function isolation(user) {
  const r = run('systemd-run', ['--wait', '--collect', '--pipe', '--quiet', ...SANDBOX(user), '--', '/usr/sbin/ip', '-o', 'link']);
  const links = r.stdout.split('\n').filter(Boolean).map((l) => l.split(':')[1]?.trim().split('@')[0]);
  debug('isolation', r.status, links);
  if (r.status !== 0) return `the sandbox didn't run (${(r.stderr.trim().split('\n').at(-1) || `exit ${r.status}`)})`;
  if (links.length !== 1 || links[0] !== 'lo') return `a sandboxed trial here would have the network (${links.join(', ')}): this machine can't make a network namespace for it`;
  return null;
}
// A trial's own ports: if isolation ever failed, it still couldn't reach
// production's collector or gateway.
const TRIAL_OTLP = 14318, TRIAL_GATEWAY = 18640;

// One arm of a trial: the service's replay, in one sandbox with its own
// collector and the gateway answering from what production recorded. The
// sandbox has no network, so nothing it does reaches the outside.
function arm(dir, app, prod, inputs) {
  const svc = prod.recipe.service, t = prod.recipe.trial, user = prod.recipe.steps.find((s) => s.do === 'unit').user;
  const out = join(dir, 'out'), state = join(dir, 'state');
  mkdirSync(out, { recursive: true }); mkdirSync(state, { recursive: true });
  for (const f of fill(t.state ?? [], prod.params)) if (existsSync(f)) writeFileSync(join(state, f.split('/').at(-1)), readFileSync(f));
  writeFileSync(join(dir, 'recorded.jsonl'), lines(join(HOME, 'effects', `${svc}.jsonl`)).join('\n') + '\n');
  writeFileSync(join(dir, 'otel.yaml'), configOf({ do: 'telemetry' }, prod, prod.params).text.replace(join(HOME, 'telemetry', svc, 'raw.jsonl'), join(out, 'telemetry.jsonl')).replace(`127.0.0.1:${OTLP_PORT}`, `127.0.0.1:${TRIAL_OTLP}`));
  const p = { ...prod.params, app, inputs, out, state, otlp: `http://127.0.0.1:${TRIAL_OTLP}` };
  for (const n of Object.keys(prod.recipe.effects ?? {})) p[`effect.${n}`] = `http://127.0.0.1:${TRIAL_GATEWAY}/${n}`;
  const env = Object.entries(prod.recipe.steps.find((s) => s.do === 'unit').env ?? {}).map(([k, v]) => `${k}=${fill(v, p)}`);
  const q = (a) => `'${String(a).replace(/'/g, "'\\''")}'`;
  writeFileSync(join(dir, 'arm.sh'), `#!/bin/sh
[ "$(/usr/sbin/ip -o link | grep -vc ': lo:')" = 0 ] || { echo network > ${q(join(out, 'not-isolated'))}; exit 97; }
/usr/local/bin/otelcol-contrib --config ${q(join(dir, 'otel.yaml'))} >/dev/null 2>&1 & O=$!
/usr/bin/node ${q(join(LIB, 'gateway.mjs'))} --port ${TRIAL_GATEWAY} ${Object.keys(prod.recipe.effects ?? {}).map((n) => `--route ${n}=`).join(' ')} --replay ${q(join(dir, 'recorded.jsonl'))} --unrecorded ${q(join(out, 'unrecorded.jsonl'))} & G=$!
for i in $(seq 50); do (exec 3<>/dev/tcp/127.0.0.1/${TRIAL_OTLP}) 2>/dev/null && break; sleep 0.1; done
env ${env.map(q).join(' ')} ${fill(t.exec, p).map(q).join(' ')}; R=$?
sleep 2; kill $G $O; wait
exit $R
`, { mode: 0o755 });
  execFileSync('chown', ['-R', `${user}:${user}`, dir]);
  const r = run('systemd-run', ['--wait', '--collect', '--quiet', ...SANDBOX(user, dir), '--', '/bin/bash', join(dir, 'arm.sh')]);
  const file = join(out, t.result);
  if (existsSync(join(out, 'not-isolated'))) return { unjudged: 'its sandbox had the network, so it stopped before running anything' };
  const res = existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : { unjudged: `the replay wrote no ${t.result} (systemd-run exited ${r.status})` };
  res.telemetry = projected(join(out, 'telemetry.jsonl'), prod.recipe, 300);
  res.unrecorded = lines(join(out, 'unrecorded.jsonl')).length;
  return res;
}

const RANK = { submitted: 3, held: 2, rejected: 1, error: 0 };

function trial(b) {
  if (b.recipe.channel !== 'candidate') throw refusal(`only a candidate is trialed; this is a ${b.recipe.channel ?? 'unlabelled'} build`);
  const svc = b.recipe.service;
  if (!existsSync(join(HOME, `${svc}.trials-allowed`))) throw refusal(`trials of ${svc} aren't allowed on this machine: \`node site.mjs allow-trials ${svc}\` says yes to them, once`);
  const prod = readApplied(svc);
  const user = prod.recipe.steps.find((s) => s.do === 'unit').user;
  const leak = isolation(user);
  if (leak) {
    const r = { id: `${new Date().toISOString().replace(/[:.]/g, '-')}-${sha(b.bytes).slice(0, 8)}`, at: new Date().toISOString(), candidate: { version: b.recipe.version, from: b.recipe.source ?? null }, verdict: V.never, why: `no trial ran: ${leak}` };
    mkdirSync(join(HOME, 'trials'), { recursive: true });
    writeFileSync(join(HOME, 'trials', `${r.id}.json`), JSON.stringify(r, null, 1));
    return r;
  }
  if (JSON.stringify(b.recipe.trial) !== JSON.stringify(prod.recipe.trial)) throw refusal('the candidate replays differently from the installed release, so their results can\'t be compared');
  const id = `${new Date().toISOString().replace(/[:.]/g, '-')}-${sha(b.bytes).slice(0, 8)}`;
  const dir = join(HOME, 'trials', id);
  const unpack = prod.recipe.steps.find((s) => s.do === 'unpack');
  const capp = join(dir, 'candidate-app');
  mkdirSync(capp, { recursive: true });
  execFileSync('tar', ['-xzf', join(b.dir, artifact(b, unpack.artifact).name), '-C', capp]);
  const inputs = fill(prod.recipe.trial.inputs, prod.params);
  const n = (existsSync(inputs) ? run('ls', ['-1', inputs]).stdout.trim().split('\n').filter(Boolean).length : 0);
  const base = arm(join(dir, 'release'), fill(unpack.to, prod.params), prod, inputs);
  const cand = arm(join(dir, 'candidate'), capp, prod, inputs);
  rmSync(dir, { recursive: true, force: true });
  const purpose = b.recipe.candidate_for === 'instrument' ? 'instrument' : 'fix';
  const res = { id, at: new Date().toISOString(), purpose, candidate: { version: b.recipe.version, recipe: sha(b.bytes).slice(0, 12), from: b.recipe.source ?? null }, release: prod.recipe.version, inputs: n };
  // A candidate's own contract is shown, never used: its telemetry is
  // projected by the release's, so a candidate can't widen what passes.
  if (JSON.stringify(b.recipe.telemetry) !== JSON.stringify(prod.recipe.telemetry)) res.contract_change = { proposed: b.recipe.telemetry, note: 'shown for review; this trial projected by the installed contract' };
  if (base.unjudged || cand.unjudged) Object.assign(res, { verdict: V.unjudged, why: [base.unjudged && `the release: ${base.unjudged}`, cand.unjudged && `the candidate: ${cand.unjudged}`].filter(Boolean).join('; ') });
  else if (!n) Object.assign(res, { verdict: V.unjudged, why: `there are no inputs at ${inputs} to replay` });
  else {
    const by = (r) => Object.fromEntries(r.events.map((e) => [e.input, e]));
    const B = by(base), C = by(cand);
    const moves = {};
    const per = Object.keys(B).map((k) => {
      const from = B[k].outcome, to = C[k]?.outcome ?? 'absent';
      moves[`${from}→${to}`] = (moves[`${from}→${to}`] ?? 0) + 1;
      return { input: k, from, to, error_class: C[k]?.error_class ?? null };
    });
    const regressed = per.filter((x) => x.to !== 'unjudged' && (RANK[x.to] ?? -1) < RANK[x.from]);
    const improved = per.filter((x) => x.to !== 'unjudged' && (RANK[x.to] ?? -1) > RANK[x.from]);
    const unjudged = per.filter((x) => x.to === 'unjudged');
    Object.assign(res, { moves, regressed, improved, unjudged, changed: per.filter((x) => x.from !== x.to), effects_unrecorded: { release: base.unrecorded, candidate: cand.unrecorded }, telemetry: { release: base.telemetry, candidate: cand.telemetry } });
    if (purpose === 'instrument') Object.assign(res, per.every((x) => x.from === x.to) ? { verdict: V.passed, note: 'it observed and changed no outcome' } : { verdict: V.failed, why: `an instrumentation candidate changed ${per.filter((x) => x.from !== x.to).length} outcome(s)` });
    else if (regressed.length) Object.assign(res, { verdict: V.failed, why: `${regressed.length} input(s) do worse than under the release` });
    else if (improved.length) Object.assign(res, { verdict: V.passed, ...(unjudged.length ? { partly: `${unjudged.length} input(s) now send what was never sent, so the replay can't say how the outside answers them` } : {}) });
    else if (unjudged.length) Object.assign(res, { verdict: V.unjudged, why: `every changed input now sends what was never sent, so the replay can't say how the outside answers` });
    else Object.assign(res, { verdict: V.failed, why: 'it improves no input' });
  }
  mkdirSync(join(HOME, 'trials'), { recursive: true });
  writeFileSync(join(HOME, 'trials', `${id}.json`), JSON.stringify(res, null, 1));
  return res;
}

// Candidates come from the vendor's own lend, which this machine dials out
// to: nothing reaches in. Each is checked like any bundle before its trial.
async function watchLend(name) {
  const { svrngs } = await import(join(LIB, 'svrngs.mjs'));
  const s = svrngs();
  const done = new Set(), into = join(HOME, 'delivered');
  console.log(`watching the lend ${name} for candidates`);
  for (;;) {
    try {
      let origin = null;
      for (const g of (await s.ask('/v1/groups')).groups) origin ??= (await s.ask(`/v1/lends?namespace=${g.id}`)).lent.find((l) => l.name === name)?.origin ?? null;
      if (!origin) throw new Error(`no group of this machine lends ${name} now`);
      const get = async (path) => { const r = await s.fetch(new URL(path, origin).href, { signal: AbortSignal.timeout(15_000) }); if (!r.ok) throw new Error(`${path}: ${r.status}`); return Buffer.from(await r.arrayBuffer()); };
      for (const c of JSON.parse(await get('index.json'))) {
        if (done.has(c) || !/^[\w.-]+$/.test(c)) continue;
        done.add(c);
        const d = join(into, c);
        mkdirSync(d, { recursive: true });
        for (const f of ['recipe.json', 'recipe.sig']) writeFileSync(join(d, f), await get(`${c}/${f}`));
        for (const a of JSON.parse(readFileSync(join(d, 'recipe.json'))).artifacts) if (/^[\w.-]+$/.test(a.name)) writeFileSync(join(d, a.name), await get(`${c}/${a.name}`));
        let r;
        try { r = trial(readBundle(d)); } catch (e) { r = { verdict: V.never, why: e instanceof Refusal ? e.message : `the trial broke: ${e.message}`, at: new Date().toISOString(), bundle: c }; mkdirSync(join(HOME, 'trials'), { recursive: true }); writeFileSync(join(HOME, 'trials', `${r.at.replace(/[:.]/g, '-')}-${c}.json`), JSON.stringify(r, null, 1)); }
        console.log(`${c}: ${r.verdict}${r.why ? `: ${r.why}` : ''}`);
      }
    } catch (e) { debug('watch', e.message); }
    await new Promise((ok) => setTimeout(ok, 2000));
  }
}

// --- The view: what the vendor's agent reads, and only reads ---------------

function view(service, port) {
  const prod = () => readApplied(service);
  const q = (u, k, d) => Math.min(Number(u.searchParams.get(k) ?? d) || d, 1000);
  const routes = {
    '/': () => ({ service, routes: Object.keys(routes), says: 'Read-only. Facts about this machine, the service\'s telemetry as this site projects it, its effects, and trial results. No string passes as written unless the site can tell it carries no data.' }),
    '/site': () => {
      const p = prod();
      return { release: p.recipe.version, params: Object.fromEntries(Object.entries(p.params).filter(([k]) => !k.startsWith('effect.'))), trials_allowed: existsSync(join(HOME, `${service}.trials-allowed`)), journal: lines(join(HOME, `${service}.journal`)).slice(-12).map((l) => JSON.parse(l)) };
    },
    '/contract': () => ({ installed: prod().recipe.telemetry }),
    '/service/telemetry': (u) => projected(join(HOME, 'telemetry', service, 'raw.jsonl'), prod().recipe, q(u, 'limit', 200)),
    '/service/effects': (u) => { const key = shapeKey(); return lines(join(HOME, 'effects', `${service}.jsonl`)).slice(-q(u, 'limit', 100)).map((l) => { const r = JSON.parse(l); return { at: r.at, effect: r.effect, method: r.method, path: r.path, status: r.status, request: r.req_hash.slice(0, 12), answer: r.resp == null ? null : project(JSON.stringify({ resourceLogs: [{ scopeLogs: [{ logRecords: [{ body: { stringValue: r.resp } }] }] }] }), contract({}), key)[0].body }; }); },
    '/trials': () => (existsSync(join(HOME, 'trials')) ? run('ls', ['-1', join(HOME, 'trials')]).stdout.trim().split('\n').filter((f) => f.endsWith('.json')).map((f) => JSON.parse(readFileSync(join(HOME, 'trials', f), 'utf8'))) : []),
  };
  http.createServer(async (req, res) => {
    const say = (code, v) => res.writeHead(code, { 'content-type': 'application/json' }).end(JSON.stringify(v, null, 1));
    if (req.method !== 'GET') return say(405, { error: 'this view only reads' });
    const u = new URL(req.url, 'http://x');
    const route = routes[u.pathname];
    if (!route) return say(404, { error: 'no such view', routes: Object.keys(routes) });
    try { say(200, await route(u)); } catch (e) { say(e instanceof Refusal ? 409 : 500, { error: e instanceof Refusal ? e.message : 'the view could not read this' }); }
  }).listen(port, '127.0.0.1', () => console.log(`view of ${service} on 127.0.0.1:${port}`));
}

// --- The command -------------------------------------------------------------

class Refusal extends Error {}
const refusal = (m) => new Refusal(m);

const [verb, target, ...rest] = process.argv.slice(2);
const sets = {};
let auto = false, token = null;
for (let i = 0; i < rest.length; i++) {
  if (rest[i] === '--set') { const [k, ...v] = rest[++i].split('='); sets[k] = v.join('='); }
  else if (rest[i] === '--auto') auto = true;
  else if (rest[i] === '--token') token = rest[++i];
  else if (rest[i] === '--port') sets.__port = rest[++i];
  else { console.error(`site: no option ${rest[i]}`); process.exit(2); }
}
try {
  if (verb === 'trust') {
    const pem = readFileSync(target, 'utf8');
    const fp = sha(createPublicKey(pem).export({ type: 'spki', format: 'der' })).slice(0, 16);
    mkdirSync(HOME, { recursive: true });
    writeFileSync(TRUST, pem, { mode: 0o644 });
    // The site's own pieces, where its units run them from, and its key for
    // shaping values (each site's own, so shapes match within a site only).
    mkdirSync(LIB, { recursive: true });
    for (const f of ['site.mjs', 'project.mjs', 'gateway.mjs', 'svrngs.mjs']) writeFileSync(join(LIB, f), readFileSync(join(import.meta.dirname, f)), { mode: 0o644 });
    if (!existsSync(join(HOME, 'shape.key'))) writeFileSync(join(HOME, 'shape.key'), randomBytes(32).toString('hex'), { mode: 0o600 });
    console.log(`This machine now trusts the vendor key ${fp}. Check it against what the vendor told you.`);
  } else if (verb === 'plan') process.exitCode = say(readBundle(target), await plan(readBundle(target), sets, auto));
  else if (verb === 'apply') {
    if (!token) throw refusal('apply runs only the plan you saw: give it the --token that plan printed');
    process.exitCode = await apply(readBundle(target), sets, auto, token);
  } else if (verb === 'status') {
    const j = join(HOME, `${target}.journal`);
    console.log(existsSync(j) ? readFileSync(j, 'utf8').trim().split('\n').slice(-8).join('\n') : `${V.never}: nothing of ${target} was ever applied here`);
  } else if (verb === 'allow-trials') {
    mkdirSync(HOME, { recursive: true });
    writeFileSync(join(HOME, `${target}.trials-allowed`), new Date().toISOString() + '\n');
    console.log(`Trials of ${target} candidates may run here from now on: each replays this machine's inputs once, sandboxed, with no network and nothing kept but its verdict. Installing one still asks you. Remove ${join(HOME, `${target}.trials-allowed`)} to stop.`);
  } else if (verb === 'trial') {
    const r = trial(readBundle(target));
    console.log(JSON.stringify(r, null, 1));
    process.exitCode = r.verdict === V.passed ? 0 : 1;
  } else if (verb === 'watch') await watchLend(target);
  else if (verb === 'view') view(target, Number(sets.__port ?? 8620));
  else { console.error('usage: site.mjs trust|plan|apply|status|allow-trials|trial|watch|view …'); process.exitCode = 2; }
} catch (e) {
  if (!(e instanceof Refusal)) throw e;
  console.log(`Refused: ${e.message}`);
  process.exitCode = 1;
}
