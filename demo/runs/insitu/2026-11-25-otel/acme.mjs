// The vendor's side of the site, as an agent uses it: a command, or the same
// two tools over MCP (stdio). Both reach the clinic only through svrngs: the
// clinic lends its view to the vendor, and the vendor lends its candidates to
// the clinic, which fetches them itself. Neither side opens a port to the
// other.
//
//   node acme.mjs view <path>                       GET a path of the clinic's view
//   node acme.mjs deliver <branch> <version> [--for fix|instrument]
//   node acme.mjs --mcp                             the same, as MCP tools
//
// SVRNGS_DIR names the vendor computer's svrngs. `deliver` is the vendor's
// CI: the branch's tests, then a signed candidate published on the
// candidates lend. A candidate can only be trialed at the clinic.
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
import { svrngs } from '../../../../apps/pantry/svrngs.mjs';

const HERE = import.meta.dirname;
const REPO = join(HERE, 'out/vendor-repo'), CANDIDATES = join(HERE, 'out/candidates');

async function view(path) {
  const s = svrngs();
  let origin = null;
  for (const g of (await s.ask('/v1/groups')).groups) origin ??= (await s.ask(`/v1/lends?namespace=${g.id}`)).lent.find((l) => l.name === 'clinic-view')?.origin ?? null;
  if (!origin) return { verdict: 'never-ran', why: 'no group of this computer lends clinic-view now' };
  try {
    const r = await s.fetch(new URL(path.replace(/^\//, ''), origin).href, { signal: AbortSignal.timeout(20_000) });
    const text = await r.text();
    return r.ok ? JSON.parse(text) : { verdict: 'never-ran', status: r.status, why: text.slice(0, 300) };
  } catch (e) { return { verdict: 'never-ran', why: `the clinic didn't answer: ${e.message}` }; }
}

function deliver(branch, version, purpose = 'fix') {
  if (!/^[\w./-]+$/.test(branch) || !/^[\w.-]+$/.test(version)) return { verdict: 'never-ran', why: 'a branch and a version are words' };
  const w = join(HERE, 'out', `ci-${version}`);
  rmSync(w, { recursive: true, force: true });
  spawnSync('git', ['-C', REPO, 'worktree', 'prune']);
  const add = spawnSync('git', ['-C', REPO, 'worktree', 'add', '-q', '-f', w, branch], { encoding: 'utf8' });
  if (add.status) return { verdict: 'never-ran', why: add.stderr.trim() };
  try {
    const t = spawnSync('python3', ['-m', 'unittest', 'discover', '-s', 'tests'], { cwd: w, encoding: 'utf8' });
    const tests = t.stderr.trim().split('\n').slice(-3).join(' ');
    if (t.status) return { verdict: 'failed', why: 'the branch\'s tests fail, so nothing was built', tests };
    const rev = execFileSync('git', ['-C', w, 'rev-parse', '--short', 'HEAD'], { encoding: 'utf8' }).trim();
    const name = `candidate-${version}`;
    execFileSync('node', [join(HERE, 'vendor.mjs'), 'bundle', join(w, 'recipe.json'), join(HERE, 'keys'), join(CANDIDATES, name), '--source', w, '--version', version, '--channel', 'candidate', '--for', purpose, '--from', `${branch}@${rev}`]);
    mkdirSync(CANDIDATES, { recursive: true });
    writeFileSync(join(CANDIDATES, 'index.json'), JSON.stringify(readdirSync(CANDIDATES).filter((d) => d.startsWith('candidate-') && existsSync(join(CANDIDATES, d, 'recipe.sig')))));
    return { verdict: 'passed', tests, delivered: name, from: `${branch}@${rev}`, purpose, next: 'the clinic fetches it within seconds and trials it; read /trials' };
  } finally { spawnSync('git', ['-C', REPO, 'worktree', 'remove', '--force', w]); }
}

const TOOLS = [
  { name: 'view', description: "GET a path of the clinic's read-only view: / lists the routes (/site, /contract, /service/telemetry, /service/effects, /trials).", inputSchema: { type: 'object', properties: { path: { type: 'string' } }, required: ['path'] } },
  { name: 'deliver', description: "Test a branch of the vendor repo and publish it as a signed candidate the clinic trials beside production. purpose: fix, or instrument (it must change no outcome).", inputSchema: { type: 'object', properties: { branch: { type: 'string' }, version: { type: 'string' }, purpose: { enum: ['fix', 'instrument'] } }, required: ['branch', 'version'] } },
];

if (process.argv[2] === '--mcp') {
  const send = (m) => process.stdout.write(JSON.stringify({ jsonrpc: '2.0', ...m }) + '\n');
  for await (const line of createInterface({ input: process.stdin })) {
    if (!line.trim()) continue;
    const m = JSON.parse(line);
    if (m.method === 'initialize') send({ id: m.id, result: { protocolVersion: m.params?.protocolVersion ?? '2025-06-18', capabilities: { tools: {} }, serverInfo: { name: 'acme-site', version: '0.1.0' } } });
    else if (m.method === 'tools/list') send({ id: m.id, result: { tools: TOOLS } });
    else if (m.method === 'tools/call') {
      const a = m.params.arguments ?? {};
      const out = m.params.name === 'view' ? await view(a.path) : m.params.name === 'deliver' ? deliver(a.branch, a.version, a.purpose) : { verdict: 'never-ran', why: `no tool ${m.params.name}` };
      send({ id: m.id, result: { content: [{ type: 'text', text: JSON.stringify(out, null, 1) }], isError: out.verdict === 'never-ran' } });
    } else if (m.id != null) send({ id: m.id, error: { code: -32601, message: `no method ${m.method}` } });
  }
} else if (process.argv[2] === 'view') console.log(JSON.stringify(await view(process.argv[3] ?? '/'), null, 1));
else if (process.argv[2] === 'deliver') {
  const r = deliver(process.argv[3], process.argv[4], process.argv.includes('--for') ? process.argv[process.argv.indexOf('--for') + 1] : 'fix');
  console.log(JSON.stringify(r, null, 1));
  process.exitCode = r.verdict === 'passed' ? 0 : 1;
} else { console.error('usage: acme.mjs view <path> | deliver <branch> <version> [--for fix|instrument] | --mcp'); process.exitCode = 2; }
