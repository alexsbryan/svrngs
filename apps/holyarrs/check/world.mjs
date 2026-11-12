// Build both sides, then pin the actual product by hash before the world
// starts. No later Cargo profile may silently replace what the actors run.
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { chmodSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
function built(args, target) {
  const out = spawnSync('cargo', [...args, '--message-format=json'], {
    cwd: root,
    encoding: 'utf8',
    timeout: 600_000,
  });
  process.stderr.write(out.stderr ?? '');
  assert.equal(out.status, 0, `cargo ${args.join(' ')} succeeded`);
  const artifacts = out.stdout
    .split('\n')
    .filter((l) => l.startsWith('{'))
    .map((l) => JSON.parse(l));
  const executable = artifacts.findLast(
    (a) => a.reason === 'compiler-artifact' && a.target.name === target && a.executable,
  )?.executable;
  assert.ok(executable, `${target} has an actual executable artifact`);
  return executable;
}
const world = built(['test', '-p', 'svrngs', '--test', 'xenia', '--no-run'], 'xenia');
const product = built(['build', '-p', 'svrngs', '--bin', 'svrngs'], 'svrngs');
const bytes = readFileSync(product);
const hash = createHash('sha256').update(bytes).digest('hex');
const folder = resolve(root, 'target/newcomer-film/bin', hash);
mkdirSync(folder, { recursive: true });
const pinned = resolve(folder, 'svrngs');
if (!existsSync(pinned)) {
  writeFileSync(pinned, bytes, { mode: 0o755 });
  chmodSync(pinned, 0o755);
}
assert.equal(createHash('sha256').update(readFileSync(pinned)).digest('hex'), hash);
process.stderr.write(`newcomer product sha256 ${hash}\n`);
const child = spawn(world, ['film-newcomer'], {
  cwd: root,
  env: { ...process.env, SCENES_BINARY: pinned },
  stdio: 'inherit',
});
for (const signal of ['SIGTERM', 'SIGINT']) process.on(signal, () => child.kill(signal));
child.on('exit', (code) => {
  process.exitCode = code ?? 1;
});
