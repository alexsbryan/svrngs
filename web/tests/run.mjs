// One catalog for isolated Node laws, Node resource integration and apps' tests.
import { spawnSync } from 'node:child_process';
import { readdirSync, realpathSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

// A stuck wait holds the loop, and node:test without a timeout waits forever
// (a timer-held law ran to the shell's cut with every other law done,
// demo/runs/test-audit/2026-11-16): this ends a hang by failing its test by
// name. 120 s is 60x the slowest law of 2026-11-16 (the pages' twin) and
// 15x the whole apps step, so it never races real work. No web/tests file
// is named here: touched.py reads a name in a file's text as an edge.
const TEST_TIMEOUT_MS = 120_000;

const ROOT = new URL('../../', import.meta.url);
const TIERS = ['unit', 'integration', 'apps'];

export function select(tier, paths) {
  if (!TIERS.includes(tier)) throw new Error(`never-ran: unknown test tier ${tier}`);
  const files = paths
    .filter((file) => {
      if (!file.endsWith('.test.mjs')) return false;
      if (tier === 'apps') return file.startsWith('apps/');
      return (
        (file.startsWith('web/tests/') || file.startsWith('resolver/')) &&
        file.endsWith('.integration.test.mjs') === (tier === 'integration')
      );
    })
    .sort();
  if (files.length === 0) throw new Error(`never-ran: no ${tier} test files`);
  return files;
}

export function catalog(tier) {
  const dirs = tier === 'apps' ? ['apps/'] : ['web/tests/', 'resolver/'];
  return select(
    tier,
    dirs.flatMap((dir) =>
      readdirSync(new URL(dir, ROOT), { recursive: true }).map((file) => dir + file),
    ),
  );
}

function main(args) {
  const [tier = 'unit', option] = args;
  if (args.length > 2 || (option && option !== '--list'))
    throw new Error('never-ran: run.mjs <unit|integration|apps> [--list]');
  const files = catalog(tier);
  if (option === '--list') {
    console.log(files.join('\n'));
    return 0;
  }
  console.log(`${tier}: ${files.length} test files`);
  const runArgs = ['--test', `--test-timeout=${TEST_TIMEOUT_MS}`, ...files];
  const result = spawnSync(process.execPath, runArgs, { cwd: ROOT, stdio: 'inherit' });
  if (result.error) throw new Error(`never-ran: ${result.error.message}`);
  if (result.signal) throw new Error(`could-not-judge: tests ended by ${result.signal}`);
  if (result.status === null) throw new Error('could-not-judge: no test exit status');
  return result.status;
}

if (process.argv[1] && import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 3;
  }
}
