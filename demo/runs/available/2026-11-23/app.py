"""The scratch app serves only its on-disk marker and a long response."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import sys
import time


class App(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path not in ("/marker", "/stream"):
            self.send_error(404)
            return
        marker = Path("marker").read_bytes()
        repeats = 2000 if self.path == "/stream" else 1
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(repeats * len(marker)))
        self.end_headers()
        try:
            for _ in range(repeats):
                self.wfile.write(marker)
                if repeats > 1:
                    time.sleep(0.05)
        except (BrokenPipeError, ConnectionResetError):
            pass


ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), App).serve_forever()
