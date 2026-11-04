// SPDX-License-Identifier: AGPL-3.0-or-later
// Bounded local transport proof, never a simulated people scene.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { request } from 'node:https';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createResolver } from '../../../../resolver/handler.mjs';
import { staging } from './stage.mjs';

const site = process.argv[2];
assert.ok(site, 'usage: node verify.mjs SITE [--plant-checksum]');
const temporary = await mkdtemp(join(tmpdir(), 'svrngs-scene-'));
let servers;
try {
  const key = join(temporary, 'key.pem');
  const cert = join(temporary, 'cert.pem');
  execFileSync('openssl', ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-keyout', key, '-out', cert, '-days', '1', '-subj', '/CN=localhost', '-addext', 'subjectAltName=IP:127.0.0.1'], { stdio: 'ignore' });
  const ca = await readFile(cert);
  let ingress;
  let bytes;
  let live = false;
  const token = execFileSync('security', ['find-generic-password', '-s', 'svrngs-upstash', '-a', 'alexsbryan', '-w'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  const handle = createResolver({ token });
  servers = await staging({ site, key, cert, host: '127.0.0.1', pagePort: 0, resolverPort: 0, handle: async (req, ip) => {
    ingress = ip;
    if (live) return handle(req, ip);
    bytes = Buffer.from(await req.arrayBuffer()).toString('utf8');
    return new Response(null, { status: 204 });
  } });
  function get(server, path, { method = 'GET', headers = {}, body } = {}) {
    return new Promise((yes, no) => {
      const req = request({ host: '127.0.0.1', port: server.address().port, path, method, ca, headers }, (res) => {
        const chunks = [];
        res.on('data', (chunk) => chunks.push(chunk));
        res.on('end', () => yes({ status: res.statusCode, headers: res.headers, body: Buffer.concat(chunks) }));
      });
      req.on('error', no);
      req.setTimeout(15000, () => req.destroy(new Error('probe timeout')));
      req.end(body);
    });
  }
  const raw = '\uFEFF{"name":"Zoë"}';
  assert.equal((await get(servers.resolver, '/', { method: 'POST', headers: { 'x-forwarded-for': '203.0.113.7' }, body: raw })).status, 204);
  assert.equal(ingress, '127.0.0.1');
  assert.equal(bytes, raw);
  console.log('passed: HTTPS adapter preserves exact body and uses socket IP, ignoring forwarded IP');
  const shell = await get(servers.page, '/');
  assert.equal(shell.status, 200);
  assert.match(shell.body.toString(), /Content-Security-Policy/i);
  assert.ok(!shell.body.toString().includes('__VERSION__'));
  assert.deepEqual((await get(servers.page, '/install.sh')).body, await readFile('web/install.sh'));
  assert.equal((await get(servers.page, '/wasm/svrngs_web_bg.wasm')).status, 200);
  const sums = (await get(servers.page, '/get/SHA256SUMS')).body.toString().trim().split('\n');
  assert.equal(sums.length, 8, 'all Mac/Linux svrngs and Give downloads');
  for (const line of sums) {
    const [, expected, name] = /^(\w{64})\s+(.+)$/.exec(line);
    assert.match(name, /^(svrngs|give)-(darwin|linux)-(aarch64|x86_64)\.tar\.gz$/);
    const archive = await get(servers.page, `/get/${name}`);
    assert.equal(archive.status, 200);
    const digest = createHash('sha256').update(archive.body).digest('hex');
    assert.equal(digest, process.argv.includes('--plant-checksum') ? '0'.repeat(64) : expected, `download checksum: ${name}`);
  }
  console.log('passed: built canonical shell, installer, wasm and all matching download checksums served over HTTPS');
  await writeFile(join(temporary, 'outside.txt'), 'not served');
  const escape = await get(servers.page, '/%2e%2e/%2e%2e/etc/passwd');
  assert.notEqual(escape.status, 200);
  console.log('passed: absent/outside static paths do not expose files');
  live = true;
  const absent = await get(servers.resolver, '/acid-acid-acid-acid-acid', { headers: { accept: 'text/html' } });
  // No publication or admission: one bounded lookup through the actual live guard.
  assert.equal(absent.status, 200, 'live lookup must be judged, not provider unavailability');
  assert.match(absent.body.toString(), /invitation has ended/);
  assert.equal(absent.headers['cache-control'], 'no-store');
  console.log('passed: local HTTPS resolver reaches live Free store with its generation guard; absent-address guidance; no publication');
} finally {
  if (servers) await servers.close();
  await rm(temporary, { recursive: true, force: true });
}
