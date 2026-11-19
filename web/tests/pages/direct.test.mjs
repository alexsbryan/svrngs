// The direct connections line on the home page (webrtc-network;
// crates/svrngs-node/src/home/page/direct.js), with no browser: what the
// button says its press does, so the line is never a press whose effect
// its words do not name.

import assert from 'node:assert/strict';
import { register } from 'node:module';
import { test } from 'node:test';

register('./layout.mjs', import.meta.url);
const { words } = await import('../../../crates/svrngs-node/src/home/page/direct.js?v=__VERSION__');

test('the button says what its press does', () => {
  assert.equal(words(false), 'Turn direct connections on');
  assert.equal(words(true), 'Turn direct connections off');
});
