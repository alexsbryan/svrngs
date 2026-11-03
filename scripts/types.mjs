// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' types: jsconfig.json's files checked as it says, by the
// TypeScript web/package.json pins, emitting nothing
// (ralph/next/js/CHARTER.md, decision 1). Each error is printed as tsc
// prints it, then the count; exit 0 holds, 1 broken.
//
// A file that carries `@ts-nocheck` is one tsc skips whole, so each is
// refused by name: every file the configs name is typed (the js-types
// row's close).
//
// The service workers (web/sw.js, web/lend.js and
// crates/svrngs-node/src/home/session-worker.js) run in a worker's global,
// not a window's, so jsconfig.workers.json checks them in a program of
// their own, with the worker's lib for the DOM's. Each program reports the
// errors in the files its config names: a module both import, such as
// store.js, is the pages' program's to report, and what of it the workers
// call is checked at their calls.
//
// tsc alone cannot run this check: it looks for a module by the file its
// specifier names, and a page names each module by a URL, as the browser
// loads it. Three of those URLs name no file in the repository, so this
// script resolves them as the servers do, and every other specifier as
// jsconfig.json says:
// - `?v=__VERSION__` is the build's version, stamped into every URL
//   (web/tests/specifiers.test.mjs), and no part of the file's name;
// - `/svrngs-session.js` is the node's, served from home/session.js
//   (crates/svrngs-node/src/session.rs);
// - `wasm/svrngs_web.js` is the page's wasm glue, built with its
//   declaration by scripts/build-web.sh into target/web/site/wasm/. That
//   declaration is the Rust crate's exports as wasm-bindgen writes them, so
//   it is never written by hand. Not built, this check fails and says so.

import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const require = createRequire(join(ROOT, 'web', 'package.json'));
const ts = require('typescript');

const VERSION = /\?v=__VERSION__$/;
const SESSION = join(ROOT, 'crates/svrngs-node/src/home/session.js');
const GLUE = join(ROOT, 'target/web/site/wasm/svrngs_web.js');

/** The file a specifier's URL is served from, as a specifier tsc can resolve. */
function served(specifier) {
  const url = specifier.replace(VERSION, '');
  if (url === '/svrngs-session.js') {
    return SESSION;
  }
  if (/(^|\/)wasm\/svrngs_web\.js$/.test(url)) {
    return GLUE;
  }
  return url;
}

// The exit status is set, never `process.exit`ed, so a long list of errors
// reaches a pipe whole before node ends.
process.exitCode = check();

/** Every error both programs' files hold, printed: 0 for none, else 1. */
function check() {
  if (!existsSync(GLUE.replace(/\.js$/, '.d.ts'))) {
    console.log(`types: ${GLUE.replace(/\.js$/, '.d.ts')} is not built: scripts/build-web.sh`);
    return 1;
  }
  let errors = 0;
  const files = new Set();
  for (const name of ['jsconfig.json', 'jsconfig.workers.json']) {
    const checked = program(name);
    errors += checked.errors;
    for (const file of checked.files) files.add(file);
  }
  console.log(`types: ${errors} errors in ${files.size} files`);
  return errors > 0 ? 1 : 0;
}

/**
 * The errors in the files config `name` names, printed, and those files.
 * @param {string} name
 */
function program(name) {
  const read = ts.readConfigFile(join(ROOT, name), ts.sys.readFile);
  if (read.error) {
    console.log(ts.formatDiagnostics([read.error], ts.createCompilerHost({})));
    return { errors: 1, files: [] };
  }
  const config = ts.parseJsonConfigFileContent(read.config, ts.sys, ROOT);
  const host = ts.createCompilerHost(config.options);
  host.resolveModuleNameLiterals = (literals, containing, redirected, options) =>
    literals.map((literal) =>
      ts.resolveModuleName(served(literal.text), containing, options, host, undefined, redirected),
    );
  const own = new Set(config.fileNames);
  const built = ts.createProgram({ rootNames: config.fileNames, options: config.options, host });
  const errors = [...config.errors, ...ts.getPreEmitDiagnostics(built)].filter(
    (e) => e.file === undefined || own.has(e.file.fileName),
  );
  if (errors.length > 0) {
    console.log(ts.formatDiagnostics(errors, host));
  }
  const skipped = config.fileNames.filter((file) =>
    /@ts-nocheck/.test(ts.sys.readFile(file) ?? ''),
  );
  for (const file of skipped) {
    console.log(`${file}: carries @ts-nocheck, so tsc skips its types`);
  }
  return { errors: errors.length + skipped.length, files: config.fileNames };
}
