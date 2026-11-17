// SPDX-License-Identifier: AGPL-3.0-or-later
import { createHash, createPublicKey, randomInt, verify } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { isIP } from 'node:net';
import { GENERATION, PREFIX, SCRIPT } from './store.mjs';

// EFF's short list, which the five-word phrases issued before this resolver drew from.
const eff = readFileSync(new URL('./words.txt', import.meta.url), 'utf8')
  .trim()
  .split(/\s+/);
if (eff.length !== 1295 || new Set(eff).size !== 1295 || eff.some((w) => !/^[a-z]{3,5}$/.test(w)))
  throw new Error('resolver: invalid pinned wordlist');
// One letter changed, added or dropped: a Levenshtein distance of at most 1.
const near = (a, b) => {
  if (a.length > b.length) [a, b] = [b, a];
  let i = 0;
  while (i < a.length && a[i] === b[i]) i++;
  return b.length - a.length < 2 && a.slice(i + (a.length === b.length)) === b.slice(i + 1);
};
// Two neighboring letters exchanged: two Levenshtein edits, one slip of the hand.
const swapped = (a, b) =>
  [...a].some((c, i) => i + 1 < a.length && a.slice(0, i) + a[i + 1] + c + a.slice(i + 2) === b);
// Phrases draw from that list pruned so no two words are near: shortest first,
// then alphabetically, a word is kept unless a kept word is near it (818 of
// 1,295). A letter changed, added or dropped can then never spell another word.
export const words = [];
for (const w of eff.toSorted((a, b) => a.length - b.length || (a < b ? -1 : 1)))
  if (!words.some((kept) => near(kept, w))) words.push(w);
// Three words and a four-digit number, such as cedar-otter-robin-4821.
const [WORDS, DIGITS] = [3, 4];
export const SPACE = words.length ** WORDS * 10 ** DIGITS;
export const phrase = () =>
  Array.from({ length: WORDS }, () => words[randomInt(words.length)]).join('-') +
  `-${String(randomInt(10 ** DIGITS)).padStart(DIGITS, '0')}`;
const HEX = /^[0-9a-f]{64}$/;
const shape =
  'An invitation address is three words and a four-digit number, like cedar-otter-robin-4821. Check yours with whoever gave it to you.';
const unread = (word) =>
  `“${word}” isn't one of the words. Check the address with whoever gave it to you.`;
const missing =
  'No invitation has this address. Check each word and the number with whoever gave it to you.';
const unavailable =
  'Joining is temporarily unavailable. Wait at least a minute, then try again, or ask the person who invited you to send the link in a chat or show its QR code, which work without this address. If you have already joined, open svrngs on your computer to continue.';
const ended =
  'This invitation has ended. Ask the person who invited you for a new address. If you have already joined, open svrngs on your computer to continue.';
const misses = (reason) =>
  `Too many addresses from here didn't match an invitation. Check yours with whoever gave it to you, and try again ${reason === 'miss_day' ? 'tomorrow' : 'in an hour'}, or ask them to send the link in a chat or show its QR code, which work without this address.`;

// The network a budget is kept for: IPv4 by its address, IPv6 by its /56, one
// household's usual allocation, so a host cannot rotate through its /64. A
// mapped IPv4 address is kept whole, not as the one /56 all of them share.
function network(ip) {
  const host = isIP(ip) === 6 ? new URL(`http://[${ip}]/`).hostname.slice(1, -1) : ip;
  if (isIP(ip) === 4 || host.startsWith('::ffff:')) return host;
  const [left, right = []] = host.split('::').map((half) => (half ? half.split(':') : []));
  const groups = [...left, ...Array(8 - left.length - right.length).fill('0'), ...right];
  // The first three groups and the high byte of the fourth: 56 bits.
  return `${groups.slice(0, 3).join(':')}:${parseInt(groups[3], 16) >> 8}/56`;
}
const hash = (text) => createHash('sha256').update(text).digest('hex');

// The address as typed, read as the one phrase it can mean, or what is wrong
// with it, before any store call: any case; any run of other characters
// between parts; the number glued to a word; one slip in a word, read as the
// only word it can be. Each request is still one lookup of one phrase against
// the same budgets, so forgiveness gives a guesser nothing.
export function read(path) {
  const text = path.toLowerCase().replace(/%[0-9a-f]{2}/g, '-');
  const typed = text.match(/[a-z]+/g) ?? [];
  const number = text.replace(/[^0-9]/g, '');
  // Five of EFF's words: a phrase issued before this resolver, read as it was.
  if (typed.length === 5 && !number && typed.every((w) => eff.includes(w)))
    return { phrase: typed.join('-') };
  if (path.length > 64 || typed.length !== WORDS || number.length !== DIGITS)
    return { wrong: shape };
  const meant = typed.map((part) =>
    words.includes(part) ? [part] : words.filter((w) => near(w, part) || swapped(w, part)),
  );
  const unknown = meant.findIndex((found) => found.length !== 1);
  if (unknown >= 0) return { wrong: unread(typed[unknown]) };
  return { phrase: [...meant.flat(), number].join('-') };
}
// The handler's own refusals. A debug event names their code, never a foreign
// error's text, which can quote a phrase or token.
class Refusal extends Error {}
// The store's refusal reasons, read from the script that gives them.
const reasons = new Set(Array.from(SCRIPT.matchAll(/refuse\('(\w+)'\)/g), ([, reason]) => reason));

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
      if (signal.aborted) throw new Refusal('timeout');
      // A BOM is signed data too; TextDecoder otherwise strips it.
      if (done)
        return new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(
          Buffer.concat(chunks),
        );
      length += value.length;
      if (length > 8192) throw new Refusal('body_limit');
      chunks.push(Buffer.from(value));
    }
  } finally {
    signal.removeEventListener('abort', cancel);
    await reader.cancel();
  }
}
function answer(value, status = 200) {
  if (Buffer.byteLength(JSON.stringify(value)) > 8192) throw new Refusal('response_limit');
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
      throw new Refusal('secret');
    page.hash = `${body.token}&native`;
    headers.set('location', page.href);
    return new Response(null, { status: 303, headers });
  }
  headers.set('content-type', 'text/html; charset=utf-8');
  headers.set(
    'content-security-policy',
    "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'",
  );
  // Fixed text; the one quoted word read() names is letters only.
  const message = body.message ?? unavailable;
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
        abort.signal.addEventListener('abort', () => reject(new Refusal('timeout')), {
          once: true,
        }),
      );
      const response = await Promise.race([run(request, ip, abort.signal), timeout]);
      return html ? await navigation(response) : response;
    } catch (error) {
      debug(`unavailable:${error instanceof Refusal ? error.message : 'transport_or_decode'}`);
      const response = answer({ error: 'unavailable', message: unavailable }, 503);
      return html ? navigation(response) : response;
    } finally {
      clearTimeout(timer);
      abort.abort();
    }
  };

  async function run(request, ip, signal) {
    if (!config.token || !isIP(ip) || !url.startsWith('https://'))
      throw new Refusal('configuration');
    const ipKey = `ip:${hash(network(ip))}`;
    const at = new URL(request.url);
    let q;
    let subject;
    if (request.method === 'GET') {
      const typed = read(at.pathname.slice(1));
      debug(`read:${typed.wrong ? 'wrong' : 'phrase'}`);
      if (typed.wrong) return answer({ state: 'wrong', message: typed.wrong }, 404);
      q = { action: 'lookup', phrase: typed.phrase };
      subject = ipKey;
    } else if (request.method === 'POST' && at.pathname === '/' && !at.search) {
      const raw = await bounded(request.body, signal);
      const owner = request.headers.get('x-svrngs-key') ?? '';
      const signature = request.headers.get('x-svrngs-signature') ?? '';
      if (!HEX.test(owner) || !/^[0-9a-f]{128}$/.test(signature))
        throw new Refusal('signature_shape');
      const key = createPublicKey({
        key: Buffer.from(`302a300506032b6570032100${owner}`, 'hex'),
        format: 'der',
        type: 'spki',
      });
      const signed = Buffer.from(`svrngs-app:onboarding:${raw}`);
      if (!verify(null, signed, key, Buffer.from(signature, 'hex'))) throw new Refusal('signature');
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
        throw new Refusal('body');
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
      // The largest reply a lookup of this secret can bring, under the 8 KiB read.
      const largest = { state: 'open', phrase: 'xxxxx-xxxxx-xxxxx-xxxxx-xxxxx', secret: q.secret };
      if (Buffer.byteLength(JSON.stringify({ result: JSON.stringify(largest) })) > 8192)
        throw new Refusal('secret_response_limit');
      subject = body.action === 'status' ? ipKey : `owner:${owner}`;
    } else {
      throw new Refusal('route');
    }
    const date = new Date();
    q.monthStart = Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1) / 1000;
    q.monthEnd = Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 1) / 1000;
    const keys = ['generation', 'reservations', 'active', 'quotas', `quota:${subject}`];
    const named = keys.map((key) => `${PREFIX}:${key}`);
    const command = ['EVAL', SCRIPT, String(keys.length), ...named, GENERATION, JSON.stringify(q)];
    const response = await fetcher(url, {
      method: 'POST',
      redirect: 'error',
      signal,
      headers: { authorization: `Bearer ${config.token}`, 'content-type': 'application/json' },
      body: JSON.stringify(command),
    });
    if (!response.ok) {
      await response.body?.cancel();
      throw new Refusal('provider_status');
    }
    const envelope = JSON.parse(await bounded(response.body, signal));
    if (envelope.error || typeof envelope.result !== 'string') throw new Refusal('provider_result');
    const result = JSON.parse(envelope.result);
    if (result.state === 'unavailable')
      debug(`store:${reasons.has(result.reason) ? result.reason : 'refused'}`);
    debug(
      `${q.action}:${['open', 'closed', 'ended', 'missing', 'unavailable'].includes(result.state) ? result.state : 'invalid'}`,
    );
    if (result.state === 'ended' && (q.action === 'lookup' || q.action === 'status'))
      return answer({ state: 'ended', message: ended });
    if (result.state === 'missing' && q.action === 'lookup')
      return answer({ state: 'missing', message: missing }, 404);
    if (result.reason === 'miss_hour' || result.reason === 'miss_day')
      return answer({ error: 'misses', message: misses(result.reason) }, 429);
    if (result.state === 'closed' && q.action === 'close')
      return answer({ id: q.id, closed: true });
    if (result.state !== 'open' || read(result.phrase).phrase !== result.phrase)
      throw new Refusal('store_refusal');
    if (q.action === 'lookup') {
      const secret = JSON.parse(result.secret);
      if (typeof secret.token !== 'string' || !Number.isSafeInteger(secret.until))
        throw new Refusal('secret');
      return answer({ state: 'open', ...secret });
    }
    if (q.action === 'close') throw new Refusal('close_not_acknowledged');
    return answer({ id: q.id, phrase: result.phrase, state: 'open' });
  }
}
