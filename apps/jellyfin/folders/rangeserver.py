# Range-capable static file server for the .strm experiment.
# Logs ONE JSON line per request at completion: start/end time, client IP,
# User-Agent, method, path, Range, status, bytes actually written to the
# socket (a client abort mid-body is counted up to the abort), plus the
# fetch-metadata headers that identify a browser media request.
# usage: rangeserver.py ROOT PORT LOGFILE
import json, os, sys, time, threading
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler

ROOT, PORT, LOGPATH = sys.argv[1], int(sys.argv[2]), sys.argv[3]
LOG = open(LOGPATH, "a", buffering=1)
LOCK = threading.Lock()
CHUNK = 64 * 1024
TYPES = {".mp4": "video/mp4", ".mkv": "video/x-matroska"}


class H(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *a):
        pass

    def emit(self, t0, status, sent, rng_out=None, aborted=False):
        h = self.headers
        rec = {
            "t_start": round(t0, 3), "t_end": round(time.time(), 3),
            "ip": self.client_address[0], "ua": h.get("User-Agent"),
            "method": self.command, "path": self.path, "range": h.get("Range"),
            "status": status, "bytes": sent, "content_range": rng_out,
            "aborted": aborted, "sec_fetch_dest": h.get("Sec-Fetch-Dest"),
            "sec_fetch_mode": h.get("Sec-Fetch-Mode"), "origin": h.get("Origin"),
            "referer": h.get("Referer"),
        }
        with LOCK:
            LOG.write(json.dumps(rec) + "\n")

    def cors(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "Range, Content-Type")
        self.send_header("Access-Control-Allow-Methods", "GET, HEAD, OPTIONS")
        self.send_header("Access-Control-Expose-Headers",
                         "Content-Range, Content-Length, Accept-Ranges")

    def do_OPTIONS(self):
        t0 = time.time()
        self.send_response(204)
        self.cors()
        self.send_header("Content-Length", "0")
        self.end_headers()
        self.emit(t0, 204, 0)

    def do_HEAD(self):
        self.do_GET(head=True)

    def do_GET(self, head=False):
        t0 = time.time()
        p = os.path.join(ROOT, self.path.split("?")[0].lstrip("/"))
        if not os.path.isfile(p):
            self.send_response(404)
            self.cors()
            self.send_header("Content-Length", "0")
            self.end_headers()
            self.emit(t0, 404, 0)
            return
        size = os.path.getsize(p)
        rng = self.headers.get("Range")
        start, end, code, cr = 0, size - 1, 200, None
        if rng and rng.startswith("bytes="):
            a, b = rng[6:].split(",")[0].strip().split("-")
            if a:
                start = int(a)
                end = min(int(b), size - 1) if b else size - 1
            else:
                start = max(0, size - int(b))
            if start >= size:
                self.send_response(416)
                self.cors()
                self.send_header("Content-Range", f"bytes */{size}")
                self.send_header("Content-Length", "0")
                self.end_headers()
                self.emit(t0, 416, 0)
                return
            code, cr = 206, f"bytes {start}-{end}/{size}"
        n = end - start + 1
        self.send_response(code)
        self.cors()
        self.send_header("Accept-Ranges", "bytes")
        self.send_header("Content-Type", TYPES.get(os.path.splitext(p)[1], "application/octet-stream"))
        self.send_header("Content-Length", str(n))
        if cr:
            self.send_header("Content-Range", cr)
        self.end_headers()
        sent, aborted = 0, False
        if not head:
            try:
                with open(p, "rb") as f:
                    f.seek(start)
                    left = n
                    while left > 0:
                        buf = f.read(min(CHUNK, left))
                        if not buf:
                            break
                        self.wfile.write(buf)
                        sent += len(buf)
                        left -= len(buf)
            except (BrokenPipeError, ConnectionResetError, TimeoutError, OSError):
                aborted = True
                self.close_connection = True
        self.emit(t0, code, sent, cr, aborted)


if __name__ == "__main__":
    srv = ThreadingHTTPServer(("0.0.0.0", PORT), H)
    srv.daemon_threads = True
    print(f"serving {ROOT} on 0.0.0.0:{PORT}, log {LOGPATH}", flush=True)
    srv.serve_forever()
