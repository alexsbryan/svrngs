// Shared original signing vectors and request builders, not a store substitute.
import assert from 'node:assert/strict';
import { createHash, createPrivateKey, createPublicKey, sign } from 'node:crypto';

export const ownerKey = createPrivateKey({
  key: Buffer.from(`302e020100300506032b657004220420${'01'.repeat(32)}`, 'hex'),
  format: 'der',
  type: 'pkcs8',
});
export const owner = createPublicKey(ownerKey)
  .export({ format: 'der', type: 'spki' })
  .subarray(-32)
  .toString('hex');
export const otherKey = createPrivateKey({
  key: Buffer.from(`302e020100300506032b657004220420${'02'.repeat(32)}`, 'hex'),
  format: 'der',
  type: 'pkcs8',
});
export const hash = (text) => createHash('sha256').update(text).digest('hex');
export const fixed = 'acid-acorn-acre-acts-afar';
export const seconds = () => Math.floor(Date.now() / 1000);
export const body = (changes = {}) => ({
  action: 'publish',
  id: 'ab'.repeat(32),
  group: 'house',
  token: 'ordinary-invitation',
  until: seconds() + 3600,
  page: 'https://svrnme.sh',
  ...changes,
});
export function mutation(value, key = ownerKey, domain = 'onboarding') {
  const raw = typeof value === 'string' ? value : JSON.stringify(value);
  const publicKey = createPublicKey(key)
    .export({ format: 'der', type: 'spki' })
    .subarray(-32)
    .toString('hex');
  return new Request('https://join.example/', {
    method: 'POST',
    headers: {
      'x-svrngs-key': publicKey,
      'x-svrngs-signature': sign(null, Buffer.from(`svrngs-app:${domain}:${raw}`), key).toString(
        'hex',
      ),
    },
    body: raw,
  });
}
export async function refused(response) {
  assert.equal(response.status, 503);
  assert.equal(response.headers.get('retry-after'), '60');
  const text = await response.text();
  assert.match(text, /joining is temporarily unavailable/);
  assert.match(text, /at least a minute/);
  assert.doesNotMatch(text, /invitation has ended/);
}
