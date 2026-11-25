// Judges the attempts in <out>/attempts.jsonl against the fixture's
// qualification (SUPPORT_LINK, "Fixture and qualification"), with the
// origin's log as the witness of what was served.
//
//   node qualify.mjs <out dir>
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const out = process.argv[2];
const lines = (f) => readFileSync(join(out, f), 'utf8').trim().split('\n').filter(Boolean).map((l) => JSON.parse(l));
const origin = lines('origin.log');
for (const a of lines('attempts.jsonl')) {
  const sw = a.events.filter((e) => e.e === 'rendition');
  const at760 = sw.filter((e) => e.media <= 760).at(-1)?.now ?? null;
  const bad = a.events.filter((e) => e.e === 'request' && e.segment === 381);
  const errs = a.events.filter((e) => e.e === 'error');
  console.log(JSON.stringify({
    id: a.id, kind: a.kind, cap: a.cap, verdict: a.verdict, why: a.why, evidence: a.evidence,
    rendition_at_12_40: at760, first_1080p_media_s: sw.find((e) => e.now === '1080p')?.media ?? null,
    segment_381_requests: bad.map((e) => e.rendition), errors: errs.map((e) => `${e.details}${e.status ? ` ${e.status}` : ''} ${e.rendition} #${e.segment} fatal=${e.fatal} action=${e.action}`),
    qualifies: a.kind === 'viewing' ? at760 === '1080p' && bad.some((e) => e.rendition === '1080p') && a.verdict === 'failed' && (a.evidence?.worst_gap_s ?? 0) >= 2 : undefined,
  }, null, 1));
}
console.log('origin 404s:', origin.filter((r) => r.status === 404).map((r) => r.path).join(' ') || 'none');
