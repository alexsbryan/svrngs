// The kinds there are, in one place, so the server and the canary carry the
// same ones (AGENTS.md 8): a kind is one file in kinds/ and its name here
// (kinds/README.md).

import { kindsOf } from './grapevine.js';

const KIND_FILES = ['ask', 'tool'];

export const KINDS = kindsOf(await Promise.all(KIND_FILES.map(async (k) => (await import(`./kinds/${k}.js`)).default)));
