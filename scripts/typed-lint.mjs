// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' lint by their types (ralph/next/js/CHARTER.md, "Answered" 4):
// the rules in .oxlintrc.json that read type information, which a plain
// `oxlint` skips without a word, run by oxlint's type-aware mode with
// oxlint-tsgolint as web/package.json pins it. Each diagnostic is printed
// as oxlint prints it; exit 0 holds, 1 broken.
//
// tsgolint finds a module by the file its specifier names and takes no
// resolver, so each page's import of a URL (`?v=__VERSION__`,
// `/svrngs-session.js`) would be untyped, and a rule judging a call to it
// blind: a floating promise from an imported function went unnamed
// (js-lint (d)'s probe). So both pages' trees are copied under
// target/typed-lint/, each such specifier rewritten to the file
// scripts/types.mjs `served` resolves it to, and linted there.
//
// tsgolint types a file by the tsconfig.json found beside it, whatever
// `--tsconfig` names: given the workers' config by that flag, the service
// workers' `ServiceWorkerGlobalScope` was still an error type (the same
// probe). So each program has its own copy, whose tsconfig.json is that
// program's config: the pages' (jsconfig.json) and the service workers'
// (jsconfig.workers.json), each file linted in the copy of the config
// that names it, as `types` checks it. The copies are never served, and
// nothing that ships is changed. A diagnostic names a file by its path in
// the repository and its line there; on an import line, its column is the
// copy's.
//
// Every other JavaScript file here (scripts/, web/tests/, apps/) is in
// neither program and has no types to read: each type-aware rule the
// config turns on, by name or by its category, is never-ran there, and
// `lint` says nothing of it.

import { spawnSync } from 'node:child_process';
import { cpSync, existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, join, relative } from 'node:path';
import { filesOf, GLUE, ROOT, served } from './types.mjs';

const COPIES = join(ROOT, 'target/typed-lint');
const TREES = ['web', 'crates/svrngs-node/src/home'];
// Each program: the config that names its files, and the copy it is linted in.
const PROGRAMS = [
  { config: 'jsconfig.json', copy: join(COPIES, 'pages') },
  { config: 'jsconfig.workers.json', copy: join(COPIES, 'workers') },
];
const BIN = join(ROOT, 'web/node_modules/.bin');
// A specifier a page names by its URL: one stamped with the build's version,
// or the node's session module.
const URL_SPECIFIER = /(['"])([^'"\n]+\?v=__VERSION__|\/svrngs-session\.js)\1/g;

process.exitCode = lint();

/** Every type-aware diagnostic in both programs' files, printed: 0 for none, else 1. */
function lint() {
  if (!existsSync(join(BIN, 'tsgolint'))) {
    console.log('typed lint: oxlint-tsgolint is not installed here: npm ci --prefix web');
    return 1;
  }
  if (!existsSync(GLUE.replace(/\.js$/, '.d.ts'))) {
    console.log(`typed lint: ${GLUE.replace(/\.js$/, '.d.ts')} is not built: scripts/build-web.sh`);
    return 1;
  }
  rmSync(COPIES, { recursive: true, force: true });
  let status = 0;
  for (const { config, copy } of PROGRAMS) {
    status = Math.max(status, program(config, copy));
  }
  return status;
}

/**
 * The files `config` names, copied to `copy` with that config as its
 * tsconfig.json, and linted there by their types: 0 for no diagnostic, else 1.
 * @param {string} config
 * @param {string} copy
 */
function program(config, copy) {
  for (const tree of TREES) {
    cpSync(join(ROOT, tree), join(copy, tree), { recursive: true, filter: copied });
  }
  // jsconfig.json beside the workers' config, which extends it.
  for (const file of ['.oxlintrc.json', 'jsconfig.json']) {
    cpSync(join(ROOT, file), join(copy, file));
  }
  cpSync(join(ROOT, config), join(copy, 'tsconfig.json'));
  const files = filesOf(config)
    .filter((file) => file.endsWith('.js'))
    .map((file) => relative(ROOT, file));
  for (const file of files) {
    rewrite(join(copy, file), copy);
  }
  const run = spawnSync(join(BIN, 'oxlint'), ['--type-aware', ...files], {
    cwd: copy,
    encoding: 'utf8',
    env: { ...process.env, OXLINT_TSGOLINT_PATH: join(BIN, 'tsgolint') },
  });
  // oxlint says nothing of a file that passes, so the count is said here.
  console.log(`typed lint, ${config}: ${files.length} files, exit ${run.status}`);
  process.stdout.write(`${run.stdout}${run.stderr}`);
  return run.status === 0 ? 0 : 1;
}

/**
 * Whether `path` goes into a copy: the trees but their tests, their
 * installed packages and the phone page's Rust.
 * @param {string} path
 */
function copied(path) {
  const inside = relative(join(ROOT, 'web'), path);
  return !['node_modules', 'tests', 'src', 'target'].some(
    (left) => inside === left || inside.startsWith(`${left}/`),
  );
}

/**
 * The copied file at `at` with each URL specifier naming the file it is
 * served from, in `copy`.
 * @param {string} at
 * @param {string} copy
 */
function rewrite(at, copy) {
  const text = readFileSync(at, 'utf8').replace(URL_SPECIFIER, (_, quote, specifier) => {
    return `${quote}${inCopy(served(specifier), at, copy)}${quote}`;
  });
  writeFileSync(at, text);
}

/**
 * The specifier `to` as the copied file `from` names it: a file in a tree
 * at its place in `copy`, the built wasm glue where it was built.
 * @param {string} to
 * @param {string} from
 * @param {string} copy
 */
function inCopy(to, from, copy) {
  if (!isAbsolute(to) || !TREES.some((tree) => to.startsWith(join(ROOT, tree)))) {
    return to;
  }
  const path = relative(dirname(from), join(copy, relative(ROOT, to)));
  return path.startsWith('.') ? path : `./${path}`;
}
