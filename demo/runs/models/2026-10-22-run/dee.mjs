#!/usr/bin/env node
// Dee's adapter, written in the run, under 50 lines (row models-run (a),
// bar C3): OpenAI in, OpenAI out, in front of Bo's pantry. It teaches every
// answer one line of its own and names the model it really uses. The pantry
// learns nothing of it.
import http from 'node:http';

const upstream = process.argv[2] || 'http://127.0.0.1:8280';
const port = Number(process.argv[3] || 8282);

function collect(req) {
  return new Promise((resolve) => {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => resolve(Buffer.concat(chunks)));
  });
}

const server = http.createServer(async (req, res) => {
  if (req.method === 'GET' && req.url === '/v1/models') {
    res.writeHead(200, { 'content-type': 'application/json' });
    return res.end(JSON.stringify({ object: 'list', data: [{ id: 'dee/teach', object: 'model' }] }));
  }
  const raw = await collect(req);
  let asked = {};
  try { asked = JSON.parse(raw.toString('utf8')); } catch { /* the pantry answers 400 */ }
  const body = JSON.stringify({
    ...asked,
    model: 'ana/coder',
    messages: [{ role: 'system', content: 'Answer in one short sentence, as Dee taught.' }, ...(asked.messages || [])],
  });
  const at = new URL(req.url, upstream);
  const headers = { ...req.headers, host: at.host, 'content-length': Buffer.byteLength(body) };
  delete headers.connection;
  const target = http.request({ hostname: at.hostname, port: at.port, path: at.pathname, method: req.method, headers }, (answer) => {
    res.writeHead(answer.statusCode, answer.headers);
    answer.pipe(res);
  });
  target.on('error', (e) => {
    res.writeHead(502, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ error: `Dee's adapter could not reach the pantry: ${e.message}` }));
  });
  target.end(body);
});

server.listen(port, '127.0.0.1', () => console.log(`Dee's adapter at http://127.0.0.1:${port}/v1`));
