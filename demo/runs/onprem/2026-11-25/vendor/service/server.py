# A stand-in for the vendor's service. The site tool must not know or care
# what it does: it answers /health with its name and version, and nothing
# else. VERSION and CRASH are set by vendor.mjs when it builds the artifact.
import argparse, json, http.server, sys
VERSION = "0.1.0"
CRASH = False
a = argparse.ArgumentParser(); a.add_argument("--port", type=int, required=True); a.add_argument("--state", required=True)
args = a.parse_args()
if CRASH: sys.exit("stand-in: planted crash at start")
class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != "/health": self.send_error(404); return
        b = json.dumps({"service": "notes-to-forms", "version": VERSION}).encode()
        self.send_response(200); self.send_header("content-type", "application/json"); self.end_headers(); self.wfile.write(b)
    def log_message(self, *a): pass
http.server.ThreadingHTTPServer(("127.0.0.1", args.port), H).serve_forever()
