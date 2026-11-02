// The pages as they were before page-switchover, read from git at
// 19b63b2b, the commit before it: question.js with its DOM builders
// (`el`, `keeping`, `row`, `door`, `visiting`, `table`, `changes`, `acts`,
// `uses`), and web/index.html with the body v1 drew into, the oracles the
// pages' parts were held to byte for byte (parts.test.mjs and this
// directory's), so they still are. question.js is written under target/,
// a file a test process, since node runs each test file in a process of
// its own.
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';

const ROOT = new URL('../../..', import.meta.url).pathname;
const dir = `${ROOT}target/before`, at = `${dir}/question-${process.pid}.js`;
mkdirSync(dir, { recursive: true });
export const before = (file) => execFileSync('git', ['show', `19b63b2b:${file}`], { cwd: ROOT, encoding: 'utf8' });
writeFileSync(at, before('crates/svrngs-node/src/home/question.js'));
export const v1 = await import(`file://${at}`);
