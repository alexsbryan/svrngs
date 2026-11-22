// The pantry's address, P's own (docs/seams.tsv): `serve` takes the port it
// took last time, so the address a person pasted into their interface stays
// across a restart, and refuses one another program holds by name. What the
// pantry answers there is pantry.test.mjs's, asked in process.

import assert from 'node:assert/strict';
import net from 'node:net';
import { mkdtempSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { serve } from './pantry.mjs';

// This computer's own data dir is never read: the pantry's svrngs is one
// of an empty data dir, never asked here.
process.env.SVRNGS_DIR = mkdtempSync(join(process.cwd(), 'target', 'pantry-home-'));

// The address a person pasted into their interface stays across a restart.
test('serve answers at the same address across a restart and refuses a held one by name', async (t) => {
  const state = mkdtempSync(join(process.cwd(), 'target', 'pantry-test-'));
  // The record is never asked: what is claimed here is the address alone.
  t.after(() => rmSync(state, { recursive: true, force: true }));

  const first = await serve({ state });
  const port = first.port;
  await first.close();

  const again = await serve({ state });
  t.after(() => again.close());
  assert.equal(again.port, port, 'the address moved across a restart');
  await again.close();

  const held = net.createServer();
  await new Promise((resolve) => held.listen(port, '127.0.0.1', resolve));
  t.after(() => held.close());
  await assert.rejects(
    serve({ state }),
    new RegExp(`holds 127\\.0\\.0\\.1:${port}`),
    'silently moved to another port',
  );
  await new Promise((resolve) => held.close(resolve));

  const free = await new Promise((resolve) => {
    const probe = net.createServer();
    probe.listen(0, '127.0.0.1', () => {
      const chosen = probe.address().port;
      probe.close(() => resolve(chosen));
    });
  });
  const named = await serve({ state, port: free });
  assert.equal(named.port, free, '--port names another');
  await named.close();
  const kept = await serve({ state });
  t.after(() => kept.close());
  assert.equal(kept.port, free, 'and what --port named stays the address');
});
