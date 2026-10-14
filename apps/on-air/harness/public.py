#!/usr/bin/env python3
"""The public directory, as `python3 -m http.server --bind 127.0.0.1` serves
it, with one change: a listen backlog of 128, not socketserver's 5. Chromium
opens the page's thirty-odd modules at once, and past five waiting
connections macOS resets the rest (net::ERR_CONNECTION_RESET, the smoke run
in this row's commit). Prints `port <n>` once it listens.

    python3 public.py <dir>
"""
import functools, http.server, sys


class Server(http.server.ThreadingHTTPServer):
    request_queue_size = 128


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a): pass


with Server(("127.0.0.1", 0), functools.partial(Quiet, directory=sys.argv[1])) as s:
    print(f"port {s.server_address[1]}", flush=True)
    s.serve_forever()
