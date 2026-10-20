// SPDX-License-Identifier: AGPL-3.0-or-later
// A WebSocket's server half (RFC 6455), as much as the doc's own sockets
// use: the handshake, frames either way, a message in fragments, ping and
// close. Node's own http server hands over the upgraded socket; nothing is
// installed for it. Each socket arrives through svrngs's door, which
// splices the asker's bytes to this port once it answers 101 (svrngs
// crates/svrngs-node/src/door.rs, "A live socket"), so a phone's frames
// come from its lend's worker (svrngs web/lend.js `frame`), masked.

import { createHash } from 'node:crypto';

const GUID = '258EAFA5-E914-47DA-95CA-C5AB0DC85B11';

/** The biggest message taken: a whole doc's first sync fits many times over. */
export const MOST = 16 * 1024 * 1024;

/**
 * Answer `req`'s upgrade on `socket` and hand the open socket to `open`,
 * or refuse it, 400, when it asks for no WebSocket.
 */
export function accept(req, socket, head, open) {
  const key = req.headers['sec-websocket-key'];
  if (String(req.headers.upgrade).toLowerCase() !== 'websocket' || !key) {
    socket.end('HTTP/1.1 400 Bad Request\r\nconnection: close\r\ncontent-length: 0\r\n\r\n');
    return;
  }
  const answer = createHash('sha1').update(key + GUID).digest('base64');
  socket.write(`HTTP/1.1 101 Switching Protocols\r\nupgrade: websocket\r\nconnection: Upgrade\r\nsec-websocket-accept: ${answer}\r\n\r\n`);
  socket.setNoDelay(true);
  const peer = new Peer(socket);
  open(peer);
  socket.on('data', (d) => peer.feed(d));
  socket.on('close', () => peer.ended());
  socket.on('error', () => socket.destroy());
  if (head?.length) peer.feed(head);
}

/** One frame from this end: unmasked, as a server's are (§5.1). */
export function frame(op, data, mask = null) {
  const bytes = typeof data === 'string' ? Buffer.from(data) : Buffer.from(data.buffer ?? data, data.byteOffset ?? 0, data.byteLength ?? data.length);
  const n = bytes.length;
  const head = n < 126 ? [0x80 | op, n] : n < 65536 ? [0x80 | op, 126, n >> 8, n & 255] : [0x80 | op, 127, 0, 0, 0, 0, n >>> 24, (n >>> 16) & 255, (n >>> 8) & 255, n & 255];
  if (!mask) return Buffer.concat([Buffer.from(head), bytes]);
  head[1] |= 0x80;
  const body = Buffer.from(bytes);
  for (let i = 0; i < n; i++) body[i] ^= mask[i & 3];
  return Buffer.concat([Buffer.from(head), Buffer.from(mask), body]);
}

/**
 * Frames read from `bytes`, each `{op, fin, data}`, and what is left of a
 * frame not whole yet. Masked or not: the tests read the server's frames
 * with it too.
 */
export function frames(bytes) {
  const out = [];
  let buf = bytes;
  for (;;) {
    if (buf.length < 2) break;
    let n = buf[1] & 127;
    let at = 2;
    if (n === 126) {
      if (buf.length < 4) break;
      [n, at] = [buf.readUInt16BE(2), 4];
    } else if (n === 127) {
      if (buf.length < 10) break;
      [n, at] = [Number(buf.readBigUInt64BE(2)), 10];
    }
    if (n > MOST) {
      out.push({ op: 8, fin: true, data: Buffer.from([1009 >> 8, 1009 & 255]), over: n });
      return { out, rest: Buffer.alloc(0) };
    }
    const mask = buf[1] & 128 ? buf.subarray(at, at + 4) : null;
    const from = at + (mask ? 4 : 0);
    if (buf.length < from + n) break;
    const data = Buffer.from(buf.subarray(from, from + n));
    if (mask) for (let i = 0; i < n; i++) data[i] ^= mask[i & 3];
    out.push({ op: buf[0] & 15, fin: Boolean(buf[0] & 128), data });
    buf = buf.subarray(from + n);
  }
  return { out, rest: buf };
}

/** An open socket: `onmessage(data)`, a string or a Uint8Array; `onclose()`. */
class Peer {
  constructor(socket) {
    Object.assign(this, { socket, buf: Buffer.alloc(0), parts: [], open: true, onmessage: () => {}, onclose: () => {} });
  }

  feed(chunk) {
    const { out, rest } = frames(Buffer.concat([this.buf, chunk]));
    this.buf = rest;
    for (const f of out) {
      if (f.op === 8) {
        // Closed by the other end, or a message past MOST: said back, then ended.
        if (this.open) this.socket.end(frame(8, f.data.subarray(0, 2)));
        return this.ended();
      }
      if (f.op === 9) this.socket.write(frame(10, f.data));
      if (f.op === 1 || f.op === 2) this.parts = [f];
      if (f.op === 0) this.parts.push(f);
      if (f.fin && f.op <= 2 && this.parts.length) {
        const whole = Buffer.concat(this.parts.map((p) => p.data));
        const text = this.parts[0].op === 1;
        this.parts = [];
        this.onmessage(text ? whole.toString('utf8') : new Uint8Array(whole.buffer, whole.byteOffset, whole.length));
      }
    }
  }

  send(data) {
    if (this.open) this.socket.write(frame(typeof data === 'string' ? 1 : 2, data));
  }

  close(code = 1000) {
    if (!this.open) return;
    this.socket.end(frame(8, Buffer.from([code >> 8, code & 255])));
    this.ended();
  }

  ended() {
    if (!this.open) return;
    this.open = false;
    this.onclose();
  }
}
