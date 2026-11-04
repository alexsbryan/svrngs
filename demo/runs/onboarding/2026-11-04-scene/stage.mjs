// SPDX-License-Identifier: AGPL-3.0-or-later
// Scene-only HTTPS transport, not a public deployment. The handler owns policy.
import { execFileSync } from 'node:child_process';
import { readFile, realpath, stat } from 'node:fs/promises';
import { createServer } from 'node:https';
import { resolve, sep } from 'node:path';
import { Readable } from 'node:stream';
import { pathToFileURL } from 'node:url';
import { createResolver } from '../../../../resolver/handler.mjs';

const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.sh': 'text/plain', '.gz': 'application/gzip' };

export async function staging({ site, key, cert, host, pagePort, resolverPort, handle }) {
  const root = await realpath(site);
  const tls = { key: await readFile(key), cert: await readFile(cert) };
  function server(route) {
    const instance = createServer(tls, async (req, res) => {
      try {
        const response = await route(req);
        res.writeHead(response.status, Object.fromEntries(response.headers));
        if (response.body) Readable.fromWeb(response.body).pipe(res);
        else res.end();
      } catch {
        // No request URL, token, credential or provider body enters the log.
        res.writeHead(503, { 'content-type': 'text/plain', 'cache-control': 'no-store', 'retry-after': '60' });
        res.end('joining is temporarily unavailable. Please wait at least a minute, then try again.');
        console.error('staging: request unavailable');
      }
    });
    instance.requestTimeout = 10000;
    instance.headersTimeout = 10000;
    return instance;
  }
  const page = server(async (req) => {
    if (req.method !== 'GET' && req.method !== 'HEAD') return new Response(null, { status: 405 });
    const pathname = decodeURIComponent(new URL(req.url, 'https://staging.invalid').pathname);
    const file = await realpath(resolve(root, `.${pathname === '/' ? '/index.html' : pathname}`));
    if (!file.startsWith(root + sep) || !(await stat(file)).isFile()) return new Response(null, { status: 404 });
    const extension = Object.keys(types).find((suffix) => file.endsWith(suffix));
    return new Response(req.method === 'HEAD' ? null : await readFile(file), {
      headers: { 'content-type': types[extension] ?? 'application/octet-stream', 'cache-control': 'no-store', 'referrer-policy': 'no-referrer', 'x-content-type-options': 'nosniff' },
    });
  });
  const resolver = server(async (req) => {
    // The socket is the trusted ingress; forwarding headers authorize nothing.
    const request = new Request(`https://staging.invalid${req.url}`, {
      method: req.method,
      headers: req.headers,
      ...(req.method === 'GET' || req.method === 'HEAD' ? {} : { body: Readable.toWeb(req), duplex: 'half' }),
    });
    return handle(request, req.socket.remoteAddress);
  });
  const listen = (server, port) => new Promise((yes, no) => {
    server.once('error', no);
    server.listen(port, host, yes);
  });
  try {
    await listen(page, pagePort);
    await listen(resolver, resolverPort);
  } catch (error) {
    page.close();
    resolver.close();
    throw error;
  }
  return { page, resolver, close: () => Promise.all([page, resolver].map((server) => new Promise((done) => { server.close(done); server.closeAllConnections(); }))) };
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  // All arguments are nonsecret. Certificate keys and the provider key stay in files/memory.
  const [site, key, cert, host, pagePort, resolverPort] = process.argv.slice(2);
  if (!site || !key || !cert || !host || !pagePort || !resolverPort) throw new Error('usage: node stage.mjs SITE TLS_KEY TLS_CERT BIND_IP PAGE_PORT RESOLVER_PORT');
  const token = execFileSync('security', ['find-generic-password', '-s', 'svrngs-upstash', '-a', 'alexsbryan', '-w'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  const servers = await staging({ site, key, cert, host, pagePort: Number(pagePort), resolverPort: Number(resolverPort), handle: createResolver({ token }) });
  console.log('staging: page and live resolver listening; stop with Ctrl-C; no request logging');
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, async () => { await servers.close(); });
}
