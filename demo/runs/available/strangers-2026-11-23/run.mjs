import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { once } from 'node:events';
import { join } from 'node:path';
import { createInterface } from 'node:readline';

const sceneFile = join(import.meta.dirname, 'scene.mjs');
const xenia = process.env.XENIA;
const world = process.env.PLAY_LEND_GIVE_WORLD;
const arm = process.env.AVAIL_ARM;
const [mode, ...args] = process.argv.slice(2);
if (!xenia || !world || !['plant', 'built'].includes(arm)) {
  throw new Error('Set XENIA, PLAY_LEND_GIVE_WORLD and AVAIL_ARM=plant|built');
}
if (mode === 'check') {
  const { launch } = await import(`${xenia}/src/browser.ts`);
  const world = spawn(process.execPath, [sceneFile, '--serve'], {
    env: { ...process.env, AVAIL_PERSON: 'hana' },
    stdio: ['pipe', 'pipe', 'inherit'],
  });
  const lines = createInterface({ input: world.stdout });
  try {
    const [url] = await Promise.race([
      once(lines, 'line'),
      once(world, 'exit').then(([code]) => { throw new Error(`scene exited ${code}`); }),
    ]);
    const { browser } = await launch('computer');
    try {
      const page = await browser.newPage();
      await page.goto(url);
      const hand = page.locator('#lent details[aria-label="coder"]');
      await hand.locator('summary').click();
      const handText = await hand.getByRole('link', { name: 'Run it with svrngs' }).locator('..').innerText();
      const managedText = await page.getByRole('button', { name: 'Stop notes' }).locator('..').innerText();
      console.log(`${arm}: coder: ${handText}; ${managedText.replace(/\s+/g, ' ')}`);
      assert.match(handText, /^Shared while you run it\./);
      assert.match(managedText, /Run by svrngs\./);
    } finally {
      await browser.close();
    }
  } finally {
    lines.close();
    world.stdin.end();
  }
} else {
  assert.deepEqual([mode, ...args], ['play', '--variant', 'max']);
  // Reuse the Nov 22 rehearsal's one-model resolver. This xenia revision
  // has no variant argument; set effort on its request to this provider only.
  const resolver = join(world, '../runs/availability-2026-11-22/world/isaura-where.sh');
  const stranger = JSON.parse(execFileSync('sh', [resolver, 'where', 'deepseek-flash'], { encoding: 'utf8' }));
  assert.equal(stranger.base_url, 'https://api.deepinfra.com/v1/openai');
  assert.equal(stranger.call, 'deepseek-ai/DeepSeek-V4.1-Flash-1017');
  if (!process.env[stranger.key]) throw new Error(`${stranger.key} is not set`);
  const fetch = globalThis.fetch;
  const requestUrl = `${stranger.base_url}/chat/completions`;
  globalThis.fetch = (url, init) =>
    url === requestUrl
      ? fetch(url, { ...init, body: JSON.stringify({ ...JSON.parse(init.body), reasoning_effort: 'max' }) })
      : fetch(url, init);
  const { scene } = await import(`${xenia}/src/run.ts`);
  console.log(`${arm}: ${stranger.call} at ${stranger.endpoint}, reasoning_effort=max`);
  for (const who of ['hana', 'ivo']) {
    process.env.AVAIL_PERSON = who;
    const { accounts } = await scene(sceneFile, stranger, join(import.meta.dirname, arm, who), () => {});
    for (const a of accounts) {
      console.log(`${a.as}: ${a.verdict.kind} (${a.verdict.reason}), ${a.said.length} chat reply(s)`);
      for (const reply of a.said) console.log(`${reply.channel}: ${reply.text}`);
    }
  }
}
