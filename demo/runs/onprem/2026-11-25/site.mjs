// The site tool: installs a vendor's signed bundle on this machine, with no
// shell for anyone. It knows nothing about what it installs: a recipe is a
// list of steps from a closed set, each with its own check, and needs from a
// closed set of read-only probes. It runs as the machine's admin.
//
//   node site.mjs trust <vendor key file>        pin the vendor's key, once
//   node site.mjs plan  <bundle> [--set k=v]... [--auto]
//   node site.mjs apply <bundle> --token <t> [--set k=v]... [--auto]
//   node site.mjs status <service>
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
import net from 'node:net';
import { dirname, join } from 'node:path';

const HOME = process.env.ONPREM_DIR ?? '/var/lib/onprem';
const TRUST = join(HOME, 'trusted-vendor.pub');
const V = Object.freeze({ passed: 'passed', failed: 'failed', unjudged: 'could-not-judge', never: 'never-ran' });
const sha = (b) => createHash('sha256').update(b).digest('hex');
const run = (cmd, args) => spawnSync(cmd, args, { encoding: 'utf8' });
const debug = process.env.ONPREM_DEBUG ? (...a) => console.error('debug:', ...a) : () => {};

// --- The recipe: parse, don't validate -------------------------------------

const STEPS = ['user', 'dir', 'unpack', 'unit', 'health'];
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
    out[k] = p.type === 'port' ? Number(v) : v;
  }
  out.version = recipe.version;
  for (const k of Object.keys(sets)) if (!(k in recipe.params)) throw refusal(`the recipe has no setting ${k}; it has ${Object.keys(recipe.params).join(', ')}`);
  return out;
}

const fill = (v, p) => (typeof v === 'string' ? v.replace(/\{(\w+)\}/g, (_, k) => String(p[k])) : Array.isArray(v) ? v.map((x) => fill(x, p)) : v);

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
    case 'unit': {
      const file = `/etc/systemd/system/${s.name}.service`;
      return existsSync(file) && readFileSync(file, 'utf8') === unitText(s, p) && run('systemctl', ['is-active', '--quiet', s.name]).status === 0;
    }
    case 'health': return null; // always run: it is the install's last word
  }
}

const artifact = (b, name) => {
  const a = b.recipe.artifacts.find((x) => x.name === name);
  if (!a) throw refusal(`the recipe names no artifact ${name}`);
  return a;
};

const unitText = (s, p) => `# Written by the site tool from ${s.name}'s signed recipe. Rerun the tool rather than editing this.
[Unit]
Description=${s.says}
After=network.target

[Service]
User=${s.user}
ExecStart=${fill(s.exec, p).map((a) => (/\s/.test(a) ? `"${a}"` : a)).join(' ')}
Restart=on-failure
RestartSec=2
NoNewPrivileges=yes
ProtectSystem=strict
ReadWritePaths=${fill(s.writes, p).join(' ')}
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
`;

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
    case 'unit': {
      const file = `/etc/systemd/system/${s.name}.service`;
      writeFileSync(file, unitText(s, p));
      execFileSync('systemctl', ['daemon-reload']);
      execFileSync('systemctl', ['enable', s.name], { stdio: 'ignore' });
      execFileSync('systemctl', [changed ? 'restart' : 'start', s.name]);
      return { verdict: V.passed, did: `${changed ? 'restarted' : 'started'} ${s.name}, kept running by systemd and started at boot` };
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
    steps.push({ s, done: s.do === 'unit' && before ? false : stepCheck(s, p, b) });
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
  console.log(`\nInstalled ${b.recipe.service} ${b.recipe.version}. The journal is ${journal}.`);
  return 0;
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
  else { console.error(`site: no option ${rest[i]}`); process.exit(2); }
}
try {
  if (verb === 'trust') {
    const pem = readFileSync(target, 'utf8');
    const fp = sha(createPublicKey(pem).export({ type: 'spki', format: 'der' })).slice(0, 16);
    mkdirSync(HOME, { recursive: true });
    writeFileSync(TRUST, pem, { mode: 0o644 });
    console.log(`This machine now trusts the vendor key ${fp}. Check it against what the vendor told you.`);
  } else if (verb === 'plan') process.exitCode = say(readBundle(target), await plan(readBundle(target), sets, auto));
  else if (verb === 'apply') {
    if (!token) throw refusal('apply runs only the plan you saw: give it the --token that plan printed');
    process.exitCode = await apply(readBundle(target), sets, auto, token);
  } else if (verb === 'status') {
    const j = join(HOME, `${target}.journal`);
    console.log(existsSync(j) ? readFileSync(j, 'utf8').trim().split('\n').slice(-8).join('\n') : `${V.never}: nothing of ${target} was ever applied here`);
  } else { console.error('usage: site.mjs trust|plan|apply|status …'); process.exitCode = 2; }
} catch (e) {
  if (!(e instanceof Refusal)) throw e;
  console.log(`Refused: ${e.message}`);
  process.exitCode = 1;
}
