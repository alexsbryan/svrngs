// A cold connection may draw the form before its module has arrived. The
// controls must not promise an action which would submit the bare HTML form.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import { launch } from '../../web/tests/page.mjs';

test('Movie Night keeps its form inert until the actual module binds it', async () => {
  let release;
  const held = new Promise((done) => {
    release = done;
  });
  const server = createServer(async (req, res) => {
    const path = new URL(req.url, 'http://localhost').pathname;
    if (path === '/svrngs-reads') {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end('{"lends":[],"unanswered":[]}');
      return;
    }
    if (path === '/index.js') await held;
    const file = path === '/' ? 'index.html' : path.slice(1);
    try {
      const bytes = await readFile(new URL(file, import.meta.url));
      res.writeHead(200, {
        'content-type': file.endsWith('.js')
          ? 'text/javascript'
          : file.endsWith('.css')
            ? 'text/css'
            : 'text/html',
      });
      res.end(bytes);
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  const { context, close } = await launch('chromium');
  try {
    const page = await context.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}/`, { waitUntil: 'commit' });
    await page.getByRole('heading', { name: 'Movie Night' }).waitFor();
    assert.equal(
      await page.locator('#q').isDisabled(),
      true,
      'cold query cannot be entered before its action exists',
    );
    assert.equal(
      await page.locator('#search button').isDisabled(),
      true,
      'cold submit cannot reload the page and lose the query',
    );
    release();
    const search = page.getByRole('button', { name: 'Search', exact: true });
    await search.waitFor();
    await page.waitForFunction(
      () =>
        !document.getElementById('q').disabled &&
        !document.querySelector('#search button').disabled,
    );
    await page.fill('#q', 'A film');
    await search.click();
    await page.getByText('No library is open to Movie Night here.', { exact: true }).waitFor();
    assert.equal(await page.locator('#q').inputValue(), 'A film', 'the bound form keeps the query');
  } finally {
    release();
    await close();
    server.closeAllConnections();
    server.close();
  }
});
