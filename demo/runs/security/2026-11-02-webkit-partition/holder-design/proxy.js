// A tiny HTTP CONNECT proxy: every CONNECT host:443 is tunnelled to the local
// HTTPS server, so any hostname reaches it without DNS or /etc/hosts.
const http = require('http');
const net = require('net');

const PROXY_PORT = +(process.env.PROXY_PORT || 8888);
const HTTPS_PORT = +(process.env.HTTPS_PORT || 8443);
const seen = [];
const proxy = http.createServer((req, res) => { seen.push('PLAIN ' + req.url); res.statusCode = 502; res.end(); });
proxy.on('connect', (req, sock, head) => {
  seen.push('CONNECT ' + req.url);
  const up = net.connect(HTTPS_PORT, '127.0.0.1', () => {
    sock.write('HTTP/1.1 200 Connection Established\r\n\r\n');
    if (head && head.length) up.write(head);
    up.pipe(sock); sock.pipe(up);
  });
  up.on('error', () => sock.destroy());
  sock.on('error', () => up.destroy());
});
proxy.listen(PROXY_PORT, '127.0.0.1');
module.exports = { PROXY_PORT, seen };
