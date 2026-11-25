// A stand-in for the clinic's clearinghouse, outside the clinic. Strict, as
// payers are: minutes a whole number, a date ISO, a plan named. Its errors
// echo the claim back, client id included, as real ones do. Its log is the
// operator's witness of every claim that reached it.
import { appendFileSync } from 'node:fs';
import http from 'node:http';
let n = 0;
http.createServer((req, res) => {
  let b = '';
  req.on('data', (c) => (b += c)).on('end', () => {
    let claim = {}, why = null, code = null;
    try { claim = JSON.parse(b); } catch { why = 'not JSON'; code = 'BAD_JSON'; }
    if (!why && !/^\d+$/.test(claim.minutes ?? '')) [why, code] = ['minutes must be a whole number', 'MINUTES_FORMAT'];
    else if (!why && !/^\d{4}-\d{2}-\d{2}$/.test(claim.date ?? '')) [why, code] = ['date must be YYYY-MM-DD', 'DATE_FORMAT'];
    else if (!why && !claim.plan) [why, code] = ['no plan', 'PLAN_MISSING'];
    const row = { at: new Date().toISOString(), path: req.url, client_id: claim.client_id ?? null, status: why ? 422 : 202, code };
    appendFileSync('/log/claims.jsonl', JSON.stringify(row) + '\n');
    if (why) return res.writeHead(422, { 'content-type': 'application/json' }).end(JSON.stringify({ code, error: why, echo: claim }));
    res.writeHead(202, { 'content-type': 'application/json' }).end(JSON.stringify({ claim: `CLM-${String(++n).padStart(5, '0')}` }));
  });
}).listen(8700, '0.0.0.0', () => console.log('clearinghouse on 8700'));
