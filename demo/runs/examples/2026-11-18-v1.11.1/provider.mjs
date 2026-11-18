// A stand-in for a provider that speaks OpenAI's chat API: only
// `Authorization: Bearer $KEY` is answered; each chat streams a little
// Markdown and reports 64 tokens. Every request it receives is written to
// the file named first, its headers and body, to read what Spot sent on.
//
//   KEY=<key> node provider.mjs <seen.jsonl> <port>
import { appendFileSync } from 'node:fs';
import http from 'node:http';

const [seen, port] = process.argv.slice(2);
const sse = (res, o) => res.write(`data: ${JSON.stringify(o)}\n\n`);
http
  .createServer(async (req, res) => {
    let body = '';
    for await (const c of req) body += c;
    appendFileSync(seen, JSON.stringify({ method: req.method, url: req.url, headers: req.headers, body }) + '\n');
    if (req.headers.authorization !== `Bearer ${process.env.KEY}`) {
      res.writeHead(401, { 'content-type': 'application/json' });
      return res.end(JSON.stringify({ error: { message: 'Invalid API key' } }));
    }
    res.writeHead(200, { 'content-type': 'text/event-stream' });
    for (const piece of ['Hello, **Bo**.', ' This came from the provider', ' through *Ana’s* account.'])
      sse(res, { choices: [{ delta: { content: piece } }] });
    sse(res, { choices: [], usage: { prompt_tokens: 20, completion_tokens: 44, total_tokens: 64 } });
    res.end('data: [DONE]\n\n');
  })
  .listen(Number(port), '127.0.0.1');
