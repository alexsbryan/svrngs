// A stand-in svrngs for an app's tests, in the test's process: svrngs as
// it answers a program since V1.13.3 ("Never send the key"), over
// connections in memory, no listener and no port. Its data dir holds the
// `api` and `token` files svrngs writes, so an app's svrngs.mjs finds it
// as it finds svrngs.
//
//   const svrngs = standin((req, res) => { ... });
//   app({ svrngs: svrngs.client });
//
// `POST /v1/proof` proves it holds the token for that connection's two
// ends and gives the connection a nonce of its own. Every other request is
// let in only with `Authorization: Svrngs <code>` made for that nonce and
// that connection; Bearer, no code, or a code made for another connection
// is refused 401, as svrngs refuses each. What it lets in, `answer(req, res)`
// answers, on its API (`host` 127.0.0.1:4200) or at a lend's address
// (`<name>.localhost:4200`), as http.createServer's handler would.

import { randomBytes, timingSafeEqual } from 'node:crypto';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import net from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { duplexPair } from 'node:stream';

import { code, proof, svrngs } from './svrngs.mjs';

/** Where the stand-in says it listens, in its `api` file. */
export const PORT = 4200;
/** The token its data dir holds. */
export const TOKEN = 'the-stand-in-token';

const refuse = (res, kind, error) =>
  res.writeHead(401, { 'content-type': 'application/json' }).end(JSON.stringify({ error, kind }));

/** A connection's end as svrngs names it in a proof (svrngs.mjs `end`). */
const end = (address, port) =>
  net.isIPv6(address) ? `[${address}]:${port}` : `${address}:${port}`;

const made = new Set();
process.once('exit', () => made.forEach((dir) => rmSync(dir, { recursive: true, force: true })));

/** A data dir as svrngs writes one, `api` saying `at` and `token`, gone
 * when the test's process ends. */
export function dataDir(at = `http://127.0.0.1:${PORT}`, token = TOKEN) {
  const dir = mkdtempSync(join(tmpdir(), 'svrngs-standin-'));
  writeFileSync(join(dir, 'api'), `${at}\n`);
  writeFileSync(join(dir, 'token'), `${token}\n`);
  made.add(dir);
  return dir;
}

/**
 * node:net's `connect` for `server`, in memory: each connection a stream
 * pair, the program's end 127.0.0.1:<50001 on, one of its own in this
 * process> and the server's 127.0.0.1:PORT, as a socket names them. `opened(end)` is told each.
 */
let wired = 0;
export function wire(server, opened = () => {}) {
  return () => {
    const [mine, theirs] = duplexPair();
    const port = 50000 + ++wired;
    Object.assign(theirs, {
      remoteAddress: '127.0.0.1',
      remotePort: port,
      localAddress: '127.0.0.1',
      localPort: PORT,
    });
    Object.assign(mine, {
      localAddress: '127.0.0.1',
      localPort: port,
      remoteAddress: '127.0.0.1',
      remotePort: PORT,
    });
    // What a socket has that a stream doesn't, as http.Agent asks it.
    for (const f of ['setKeepAlive', 'setNoDelay', 'setTimeout', 'ref', 'unref'])
      mine[f] = () => mine;
    theirs.destroySoon = () => theirs.end(() => theirs.destroy());
    mine.on('close', () => theirs.destroy());
    theirs.on('close', () => mine.destroy());
    opened(theirs);
    server.emit('connection', theirs);
    return mine;
  };
}

/** A stand-in svrngs answering what it lets in with `answer`; `at`, a
 * real address, for a test that hands it real sockets (`serve`). */
export function standin(answer, { at } = {}) {
  const dir = dataDir(at);
  const nonces = new WeakMap();
  const sockets = new Set();
  const seen = { proofs: 0, connections: 0, asked: [], refused: [] };
  const ends = (s) => [end(s.remoteAddress, s.remotePort), end(s.localAddress, s.localPort)];

  const server = http.createServer((req, res) => {
    const [client, here] = ends(req.socket);
    if (req.method === 'POST' && req.url === '/v1/proof') {
      const chunks = [];
      req.on('data', (c) => chunks.push(c));
      req.on('end', () => {
        const { challenge } = JSON.parse(Buffer.concat(chunks).toString() || '{}');
        const nonce = randomBytes(32).toString('hex');
        nonces.set(req.socket, nonce);
        seen.proofs += 1;
        res.writeHead(200, { 'content-type': 'application/json' });
        res.end(JSON.stringify({ proof: proof(TOKEN, challenge, client, here), nonce }));
      });
      return;
    }
    const shown = req.headers.authorization ?? '';
    const nonce = nonces.get(req.socket);
    const wanted = nonce && Buffer.from(`Svrngs ${code(TOKEN, nonce, client, here)}`);
    const given = Buffer.from(shown);
    const asked = { method: req.method, url: req.url, host: req.headers.host, client };
    if (shown.startsWith('Bearer ')) {
      seen.refused.push({ ...asked, why: 'bearer' });
      return refuse(
        res,
        'no_bearer',
        "svrngs doesn't take its token from a program any more: ask through `svrngs ask <route>`, or apps/client/svrngs.mjs",
      );
    }
    if (!wanted || wanted.length !== given.length || !timingSafeEqual(wanted, given)) {
      seen.refused.push({ ...asked, why: nonce ? 'not this connection' : 'unproved' });
      return refuse(
        res,
        'unasked',
        'ask with the code made for this connection, after POST /v1/proof on it',
      );
    }
    seen.asked.push(asked);
    // As svrngs takes its own credential off before a lend's service is asked.
    delete req.headers.authorization;
    answer(req, res);
  });

  const opened = (socket) => {
    seen.connections += 1;
    sockets.add(socket);
    socket.on('close', () => sockets.delete(socket));
  };
  const connect = wire(server, opened);

  return {
    dir,
    seen,
    connect,
    client: svrngs(dir, { connect }),
    /** A real socket's connection, svrngs's own. */
    serve: (socket) => (opened(socket), server.emit('connection', socket)),
    /** Every connection closed by svrngs's end, as svrngs restarting closes them. */
    drop: () => sockets.forEach((s) => s.destroy()),
    close() {
      sockets.forEach((s) => s.destroy());
      rmSync(dir, { recursive: true, force: true });
    },
  };
}
