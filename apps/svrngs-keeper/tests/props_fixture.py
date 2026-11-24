"""A listener that reports only a model path, with a different / for readiness."""
from http.server import BaseHTTPRequestHandler, HTTPServer
import sys


class Service(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.end_headers()
        if self.path == '/props':
            self.wfile.write(b'''{"model_path":"Sam's model.gguf"}''')


with HTTPServer(('127.0.0.1', int(sys.argv[1])), Service) as server:
    print(server.server_port, flush=True)
    server.serve_forever()
