// Each computer's copy of a group's record, folded by the app's own fold
// (apps/events/events.js) for run.py here: reads a JSON list of
// `{computer, group, log}`, `log` as GET /v1/rail/log answered it, and
// prints the same list with each copy's acts and what they come to.
//
//   node fold.mjs <copies.json>
import { readFileSync } from 'node:fs';
import { actsOf, fold } from '../../../../apps/events/events.js';

const copies = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const out = copies.map(({ computer, group, log }) => {
  try {
    const acts = actsOf(log);
    return { computer, group, complete: log.complete, ops: log.ops.length, acts, ...fold(acts) };
  } catch (e) {
    return { computer, group, unread: e.message };
  }
});
console.log(JSON.stringify(out));
