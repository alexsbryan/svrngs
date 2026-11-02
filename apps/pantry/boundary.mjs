// Credential-bearing requests have one authority: a caller's target never supplies it.
import { createHash, timingSafeEqual } from 'node:crypto';

export function authorized(headers, key) {
  if (!key || headers.origin !== undefined || headers['sec-fetch-site'] !== undefined) return false;
  const shown = headers.authorization;
  if (typeof shown !== 'string' || !shown.startsWith('Bearer ')) return false;
  const digest = (s) => createHash('sha256').update(s).digest();
  return timingSafeEqual(digest(shown.slice(7)), digest(key));
}

export function destination(base, path) {
  if (typeof path !== 'string' || !path.startsWith('/') || path.startsWith('//') || path.includes('\\') || [...path].some((c) => c.charCodeAt(0) <= 32 || c.charCodeAt(0) === 127)) {
    throw new Error('a request names a path on its selected lend, never another authority');
  }
  const origin = new URL(base);
  const target = new URL(path, origin);
  if (origin.protocol !== 'http:' || target.origin !== origin.origin || origin.username || origin.password || target.hash) {
    throw new Error('a credential-bearing request stays at the selected lend');
  }
  return target;
}
