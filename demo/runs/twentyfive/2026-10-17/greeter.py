#!/usr/bin/env python3
"""The app every person in the run lends: it greets whoever opens it by the
name svrngs's door hands it, X-Svrngs-Person (DESIGN §3, "What an app
gets"). `/` answers `Hi, <person>`; `/who` answers every x-svrngs-* header
it got, as JSON, so a check can read who the door said asked.

    python3 greeter.py <port> <owner>

Stdlib only. Also importable: `serve(port, owner)` starts it in a thread.
"""
import json
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def handler(owner):
    class Greet(BaseHTTPRequestHandler):
        def do_GET(self):
            got = {k.lower(): v for k, v in self.headers.items() if k.lower().startswith("x-svrngs-")}
            person = got.get("x-svrngs-person")
            if self.path.startswith("/who"):
                body = json.dumps({"owner": owner, "headers": got}).encode()
                kind = "application/json"
            else:
                said = f"Hi, {person}" if person else "Hi. svrngs did not say who you are."
                body = f"<!doctype html><title>{owner}'s hello</title><h1>{said}</h1><p>From {owner}'s computer.</p>".encode()
                kind = "text/html; charset=utf-8"
            self.send_response(200)
            self.send_header("content-type", kind)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    return Greet


def serve(port, owner):
    srv = ThreadingHTTPServer(("127.0.0.1", port), handler(owner))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


if __name__ == "__main__":
    port, owner = int(sys.argv[1]), sys.argv[2]
    ThreadingHTTPServer(("127.0.0.1", port), handler(owner)).serve_forever()
