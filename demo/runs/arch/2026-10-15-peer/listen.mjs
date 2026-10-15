import http from 'node:http';
const s = http.createServer((req, res) => { let b=''; req.on('data', d => b += d); req.on('end', () => { console.error(`SEEN ${req.method} ${req.url} origin=${req.headers.origin} body=${b}`); res.end('ok'); }); });
s.listen(30999, '127.0.0.1');
setTimeout(() => process.exit(0), 60000);
