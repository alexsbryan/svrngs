import http.server, sys, urllib.parse, pathlib
here = pathlib.Path(sys.argv[1])
class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        p = urllib.parse.urlparse(self.path)
        if p.path == '/log':
            print('PHONE:', urllib.parse.unquote(p.query), flush=True); body, ct = b'', 'text/plain'
        else:
            name = 'sw.js' if p.path == '/sw.js' else 'index.html'
            body = (here / name).read_bytes(); ct = 'text/javascript' if name.endswith('.js') else 'text/html'
        self.send_response(200); self.send_header('Content-Type', ct); self.send_header('Cache-Control', 'no-store')
        self.send_header('Content-Length', str(len(body))); self.end_headers(); self.wfile.write(body)
    def log_message(self, *a): pass
http.server.ThreadingHTTPServer(('127.0.0.1', 18097), H).serve_forever()
