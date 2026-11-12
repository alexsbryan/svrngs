// The consumer owns its scratch HTTPS transport; Xenia owns the browser and
// the independent stranger. Only declared scene origins remain reachable.
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { proxied } from '../../../web/tests/page.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../../..');
const scenes = resolve(root, 'demo/xenia/holyarrs');
const xenia = process.env.XENIA_ROOT ?? resolve(root, '../xenia');
const runHome = resolve(root, 'target/newcomer-film/xenia');
mkdirSync(runHome, { recursive: true });
const proxy = await proxied();
const child = spawn(
  process.execPath,
  [
    resolve(xenia, 'src/run.ts'),
    'run',
    scenes,
    '--runs',
    process.argv[2] ?? '1',
    '--model',
    'deepseek-flash',
    '--reasoning-effort',
    'max',
    '--people',
    process.argv[3] ?? resolve(scenes, 'people'),
    '--findings',
    resolve(scenes, 'findings'),
    '--source',
    root,
    '--debug',
  ],
  {
    cwd: runHome,
    env: {
      ...process.env,
      XENIA_BROWSER_PROXY: proxy,
      XENIA_ALLOW_TEST_TLS: 'true',
      PATH: `${dirname(process.execPath)}:${process.env.PATH}`,
    },
    stdio: 'inherit',
  },
);
for (const signal of ['SIGTERM', 'SIGINT']) process.on(signal, () => child.kill(signal));
child.on('exit', (code) => {
  process.exitCode = code ?? 1;
});
