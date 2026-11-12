// Integration: a rehearsal must draw the production page and actual stylesheet.
// Plant: serve() labels CSS as application/octet-stream; Safari refuses it.
import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { launch, serve } from './page.mjs';

test('the page host serves CSS as CSS and the phone applies it', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'svrngs-page-style-'));
  await writeFile(
    join(dir, 'index.html'),
    `<!doctype html><meta http-equiv="Content-Security-Policy" content="connect-src 'self' wss: https:;"><link rel="stylesheet" href="look.css"><h1>Friends</h1>`,
  );
  await writeFile(join(dir, 'look.css'), 'h1 { font-size: 37px; }');
  const server = await serve(dir);
  let close;
  try {
    const at = `http://127.0.0.1:${server.address().port}/`;
    const css = await fetch(`${at}look.css`);
    assert.match(css.headers.get('content-type'), /^text\/css\b/);
    const launched = await launch('webkit');
    close = launched.close;
    const page = await launched.context.newPage();
    await page.goto(at);
    assert.equal(await page.locator('h1').evaluate((el) => getComputedStyle(el).fontSize), '37px');
  } finally {
    await close?.();
    server.closeAllConnections();
    await new Promise((done) => server.close(done));
    await rm(dir, { recursive: true });
  }
});
