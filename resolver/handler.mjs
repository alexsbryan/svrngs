// SPDX-License-Identifier: AGPL-3.0-or-later
import { createHash, createPublicKey, randomInt, verify } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { isIP } from 'node:net';
import { GENERATION, PREFIX, SCRIPT } from './store.mjs';

const words = readFileSync(new URL('./words.txt', import.meta.url), 'utf8')
  .trim()
  .split(/\s+/);
const wordset = new Set(words);
if (words.length !== 1295 || wordset.size !== 1295 || words.some((w) => !/^[a-z]{3,5}$/.test(w))) {
  throw new Error('resolver: invalid pinned wordlist');
}
const HEX = /^[0-9a-f]{64}$/;
const unavailable =
  'joining is temporarily unavailable. Please wait at least a minute, then try again. If it keeps happening, ask the person who invited you to check joining. If you have already joined, open svrngs on your computer to continue.';
const ended =
  'This invitation has ended. Ask the person who invited you for a new address. If you have already joined, open svrngs on your computer to continue.';
const hash = (text) => createHash('sha256').update(text).digest('hex');
export const phrase = () =>
  Array.from({ length: 5 }, () => words[randomInt(words.length)]).join('-');
const validPhrase = (text) =>
  text.split('-').length === 5 && text.split('-').every((w) => wordset.has(w));

// Bound streams while reading, not after response.text() has allocated them.
async function bounded(stream, signal) {
  if (!stream) return '';
  const reader = stream.getReader();
  const cancel = () => {
    void reader.cancel().catch(() => {});
  };
  signal.addEventListener('abort', cancel, { once: true });
  const chunks = [];
  let length = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (signal.aborted) throw new Error('timeout');
      // A BOM is signed data too; TextDecoder otherwise strips it.
      if (done)
        return new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(
          Buffer.concat(chunks),
        );
      length += value.length;
      if (length > 8192) throw new Error('body_limit');
      chunks.push(Buffer.from(value));
    }
  } finally {
    signal.removeEventListener('abort', cancel);
    await reader.cancel();
  }
}
function answer(value, status = 200) {
  if (Buffer.byteLength(JSON.stringify(value)) > 8192) throw new Error('response_limit');
  return Response.json(value, {
    status,
    headers: {
      'cache-control': 'no-store',
      'referrer-policy': 'no-referrer',
      'x-content-type-options': 'nosniff',
      ...(status === 503 ? { 'retry-after': '60' } : {}),
    },
  });
}

// Navigation uses the canonical page's UI and policy; no cross-origin fetch
// or invitation in an HTTP request to the distribution host is needed.
async function navigation(response) {
  const body = await response.json();
  const headers = new Headers(response.headers);
  if (response.ok && body.state === 'open') {
    const page = new URL(body.page);
    if (
      page.protocol !== 'https:' ||
      page.username ||
      page.password ||
      page.search ||
      page.hash ||
      !/^[A-Za-z0-9_-]+$/.test(body.token)
    )
      throw new Error('secret');
    page.hash = `${body.token}&native`;
    headers.set('location', page.href);
    return new Response(null, { status: 303, headers });
  }
  headers.set('content-type', 'text/html; charset=utf-8');
  headers.set(
    'content-security-policy',
    "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'",
  );
  const message = response.ok && body.state === 'ended' ? ended : unavailable;
  return new Response(
    `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Join your group</title><h1>Join your group</h1><p>${message}</p></html>`,
    {
      status: response.status,
      headers,
    },
  );
}

/** Concrete REST transport; the hosting edge must supply its trusted IP.
 * No listener, deployment wiring, forwarded-header fallback or store reseeding.
 * @param {{token: string, url?: string, fetch?: typeof globalThis.fetch, debug?: (event: string) => void}} config
 */
export function createResolver(config) {
  const url = config.url ?? 'https://true-pheasant-225442.upstash.io';
  const fetcher = config.fetch ?? globalThis.fetch;
  const debug =
    config.debug ??
    ((event) => {
      if (process.env.SVRNGS_DEBUG) console.debug(`resolver:${event}`);
    });
  return async function handle(request, ip) {
    const html = request.method === 'GET' && request.headers.get('accept')?.includes('text/html');
    const abort = new AbortController();
    const timer = setTimeout(() => abort.abort(), 8000);
    try {
      // Promise.race bounds the whole request, including stalled inbound streams.
      const timeout = new Promise((_, reject) =>
        abort.signal.addEventListener('abort', () => reject(new Error('timeout')), { once: true }),
      );
      const response = await Promise.race([run(request, ip, abort.signal), timeout]);
      return html ? await navigation(response) : response;
    } catch (error) {
      const known = [
        'timeout',
        'configuration',
        'body_limit',
        'response_limit',
        'signature_shape',
        'signature',
        'body',
        'secret_response_limit',
        'route',
        'provider_status',
        'provider_result',
        'store_refusal',
        'secret',
        'close_not_acknowledged',
      ];
      debug(
        `unavailable:${known.includes(error?.message) ? error.message : 'transport_or_decode'}`,
      );
      const response = answer({ error: 'unavailable', message: unavailable }, 503);
      return html ? navigation(response) : response;
    } finally {
      clearTimeout(timer);
      abort.abort();
    }
  };

  async function run(request, ip, signal) {
    if (!config.token || !isIP(ip) || !url.startsWith('https://')) throw new Error('configuration');
    const ipKey = `ip:${hash(isIP(ip) === 6 ? new URL(`http://[${ip}]/`).hostname : ip)}`;
    const at = new URL(request.url);
    let q;
    let subject;
    if (request.method === 'GET' && !at.search && validPhrase(at.pathname.slice(1))) {
      q = { action: 'lookup', phrase: at.pathname.slice(1) };
      subject = ipKey;
    } else if (request.method === 'POST' && at.pathname === '/' && !at.search) {
      const raw = await bounded(request.body, signal);
      const owner = request.headers.get('x-svrngs-key') ?? '';
      const signature = request.headers.get('x-svrngs-signature') ?? '';
      if (!HEX.test(owner) || !/^[0-9a-f]{128}$/.test(signature))
        throw new Error('signature_shape');
      const key = createPublicKey({
        key: Buffer.from(`302a300506032b6570032100${owner}`, 'hex'),
        format: 'der',
        type: 'spki',
      });
      if (
        !verify(
          null,
          Buffer.from(`svrngs-app:onboarding:${raw}`),
          key,
          Buffer.from(signature, 'hex'),
        )
      )
        throw new Error('signature');
      const body = JSON.parse(raw);
      if (
        Object.keys(body).sort().join(',') !== 'action,group,id,page,token,until' ||
        !['publish', 'close', 'status'].includes(body.action) ||
        !HEX.test(body.id) ||
        typeof body.group !== 'string' ||
        body.group.length < 1 ||
        body.group.length > 256 ||
        !Number.isSafeInteger(body.until) ||
        body.until < 1 ||
        !(
          body.page === null ||
          (typeof body.page === 'string' && new URL(body.page).protocol === 'https:')
        ) ||
        !(body.action === 'publish'
          ? typeof body.token === 'string' && body.token.length > 0
          : body.token === null)
      )
        throw new Error('body');
      q = {
        action: body.action,
        idkey: hash(`${owner}:${body.id}`),
        until: body.until,
        binding: hash(JSON.stringify([owner, body.id, body.group, body.until, body.page])),
        digest: body.action === 'publish' ? hash(body.token) : '',
        phrase: phrase(),
        secret: JSON.stringify({ token: body.token, page: body.page, until: body.until }),
        id: body.id,
      };
      if (
        Buffer.byteLength(
          JSON.stringify({
            result: JSON.stringify({
              state: 'open',
              phrase: 'xxxxx-xxxxx-xxxxx-xxxxx-xxxxx',
              secret: q.secret,
            }),
          }),
        ) > 8192
      )
        throw new Error('secret_response_limit');
      subject = body.action === 'status' ? ipKey : `owner:${owner}`;
    } else {
      throw new Error('route');
    }
    const date = new Date();
    q.monthStart = Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1) / 1000;
    q.monthEnd = Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 1) / 1000;
    const command = [
      'EVAL',
      SCRIPT,
      '5',
      `${PREFIX}:generation`,
      `${PREFIX}:reservations`,
      `${PREFIX}:active`,
      `${PREFIX}:quotas`,
      `${PREFIX}:quota:${subject}`,
      GENERATION,
      JSON.stringify(q),
    ];
    const response = await fetcher(url, {
      method: 'POST',
      redirect: 'error',
      signal,
      headers: { authorization: `Bearer ${config.token}`, 'content-type': 'application/json' },
      body: JSON.stringify(command),
    });
    if (!response.ok) {
      await response.body?.cancel();
      throw new Error('provider_status');
    }
    const envelope = JSON.parse(await bounded(response.body, signal));
    if (envelope.error || typeof envelope.result !== 'string') throw new Error('provider_result');
    const result = JSON.parse(envelope.result);
    if (result.state === 'unavailable') {
      const reasons = [
        'generation',
        'clock',
        'binding',
        'digest',
        'lifetime',
        'active_limit',
        'collision',
        'reservation_limit',
        'memory_limit',
        'lookup_limit',
        'mutation_limit',
        'publication_limit',
        'close_headroom',
        'pending',
        'missing_secret',
      ];
      debug(`store:${reasons.includes(result.reason) ? result.reason : 'refused'}`);
    }
    debug(
      `${q.action}:${['open', 'closed', 'ended', 'unavailable'].includes(result.state) ? result.state : 'invalid'}`,
    );
    if (result.state === 'ended' && (q.action === 'lookup' || q.action === 'status'))
      return answer({ state: 'ended', message: ended });
    if (result.state === 'closed' && q.action === 'close')
      return answer({ id: q.id, closed: true });
    if (result.state !== 'open' || !validPhrase(result.phrase)) throw new Error('store_refusal');
    if (q.action === 'lookup') {
      const secret = JSON.parse(result.secret);
      if (typeof secret.token !== 'string' || !Number.isSafeInteger(secret.until))
        throw new Error('secret');
      return answer({ state: 'open', ...secret });
    }
    if (q.action === 'close') throw new Error('close_not_acknowledged');
    return answer({ id: q.id, phrase: result.phrase, state: 'open' });
  }
}
