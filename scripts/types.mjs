// SPDX-License-Identifier: AGPL-3.0-or-later
// The pages' types: jsconfig.json's files checked as it says, by the
// TypeScript web/package.json pins, emitting nothing
// (ralph/next/js/CHARTER.md, decision 1). Each error is printed as tsc
// prints it, then the count; exit 0 holds, 1 broken.
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

/** Every error jsconfig.json's files hold, printed: 0 for none, else 1. */
function check() {
  const read = ts.readConfigFile(join(ROOT, 'jsconfig.json'), ts.sys.readFile);
  if (read.error) {
    console.log(ts.formatDiagnostics([read.error], ts.createCompilerHost({})));
    return 1;
  }
  if (!existsSync(GLUE.replace(/\.js$/, '.d.ts'))) {
    console.log(`types: ${GLUE.replace(/\.js$/, '.d.ts')} is not built: scripts/build-web.sh`);
    return 1;
  }
  const config = ts.parseJsonConfigFileContent(read.config, ts.sys, ROOT);
  const host = ts.createCompilerHost(config.options);
  host.resolveModuleNameLiterals = (literals, containing, redirected, options) =>
    literals.map((literal) =>
      ts.resolveModuleName(served(literal.text), containing, options, host, undefined, redirected),
    );
  const program = ts.createProgram({ rootNames: config.fileNames, options: config.options, host });
  const errors = [...config.errors, ...ts.getPreEmitDiagnostics(program)];
  if (errors.length > 0) {
    console.log(ts.formatDiagnostics(errors, host));
  }
  console.log(`types: ${errors.length} errors in ${config.fileNames.length} files`);
  return errors.length > 0 ? 1 : 0;
}
