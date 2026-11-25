// The vendor's side: a key, and a bundle (the recipe, its signature, its
// artifacts) that can reach the site on a USB stick, with no network there.
//
//   node vendor.mjs key <keys dir>
//   node vendor.mjs bundle <recipe> <keys dir> <out dir> --source <service dir>
//        [--version v] [--channel release|candidate] [--for fix|instrument] [--from <git rev>] [--crash]
//
// The vendor's CI runs this: a release from what was merged, a candidate
// from a branch. The site installs only releases and only trials candidates.
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
  const opt1 = (k, d) => (opt.includes(k) ? opt[opt.indexOf(k) + 1] : d);
  recipe.channel = opt1('--channel', 'release');
  if (opt.includes('--from')) recipe.source = opt1('--from');
  if (opt.includes('--for')) recipe.candidate_for = opt1('--for');
  cpSync(opt1('--source', join(import.meta.dirname, 'service')), stage, { recursive: true, filter: (f) => !/(\/\.git|__pycache__|\/tests|\/fixtures)(\/|$)/.test(f) });
  const py = join(stage, 'server.py');
  writeFileSync(py, readFileSync(py, 'utf8').replace(/VERSION = "[^"]*"/, `VERSION = "${version}"`).replace('CRASH = False', `CRASH = ${opt.includes('--crash') ? 'True' : 'False'}`));
  execFileSync('tar', ['--sort=name', '--mtime=@0', '--owner=0', '--group=0', '-czf', join(out, 'service.tar.gz'), '-C', stage, '.']);
  rmSync(stage, { recursive: true });
  const bytes = readFileSync(join(out, 'service.tar.gz'));
  recipe.artifacts = [{ name: 'service.tar.gz', sha256: createHash('sha256').update(bytes).digest('hex'), bytes: bytes.length }];
  const body = Buffer.from(JSON.stringify(recipe, null, 2) + '\n');
  writeFileSync(join(out, 'recipe.json'), body);
  writeFileSync(join(out, 'recipe.sig'), sign(null, body, createPrivateKey(readFileSync(join(keys, 'vendor.key'), 'utf8'))).toString('base64') + '\n');
  console.log(`${recipe.channel} bundle ${recipe.service} ${version}${recipe.source ? ` from ${recipe.source}` : ''} in ${out}`);
} else { console.error('usage: vendor.mjs key|bundle …'); process.exit(2); }
