// The vendor's side: a key, and a bundle (the recipe, its signature, its
// artifacts) that can reach the site on a USB stick, with no network there.
//
//   node vendor.mjs key <keys dir>
//   node vendor.mjs bundle <recipe> <keys dir> <out dir> [--version v] [--crash]
import { createHash, generateKeyPairSync, createPrivateKey, sign } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, rmSync, cpSync } from 'node:fs';
import { join } from 'node:path';

const [verb, ...a] = process.argv.slice(2);
if (verb === 'key') {
  mkdirSync(a[0], { recursive: true });
  const { publicKey, privateKey } = generateKeyPairSync('ed25519');
  writeFileSync(join(a[0], 'vendor.pub'), publicKey.export({ type: 'spki', format: 'pem' }));
  writeFileSync(join(a[0], 'vendor.key'), privateKey.export({ type: 'pkcs8', format: 'pem' }), { mode: 0o600 });
} else if (verb === 'bundle') {
  const [recipePath, keys, out, ...opt] = a;
  const recipe = JSON.parse(readFileSync(recipePath, 'utf8'));
  const version = opt.includes('--version') ? opt[opt.indexOf('--version') + 1] : recipe.version;
  recipe.version = version;
  rmSync(out, { recursive: true, force: true });
  const stage = join(out, '.stage');
  cpSync(join(import.meta.dirname, 'vendor/service'), stage, { recursive: true });
  const py = join(stage, 'server.py');
  writeFileSync(py, readFileSync(py, 'utf8').replace('VERSION = "0.1.0"', `VERSION = "${version}"`).replace('CRASH = False', `CRASH = ${opt.includes('--crash') ? 'True' : 'False'}`));
  execFileSync('tar', ['--sort=name', '--mtime=@0', '--owner=0', '--group=0', '-czf', join(out, 'service.tar.gz'), '-C', stage, '.']);
  rmSync(stage, { recursive: true });
  const bytes = readFileSync(join(out, 'service.tar.gz'));
  recipe.artifacts = [{ name: 'service.tar.gz', sha256: createHash('sha256').update(bytes).digest('hex'), bytes: bytes.length }];
  const body = Buffer.from(JSON.stringify(recipe, null, 2) + '\n');
  writeFileSync(join(out, 'recipe.json'), body);
  writeFileSync(join(out, 'recipe.sig'), sign(null, body, createPrivateKey(readFileSync(join(keys, 'vendor.key'), 'utf8'))).toString('base64') + '\n');
  console.log(`bundle ${recipe.service} ${version} in ${out}`);
} else { console.error('usage: vendor.mjs key|bundle …'); process.exit(2); }
