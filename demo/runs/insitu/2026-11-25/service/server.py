"""Acme Notes-to-Forms: watches an inbox of session notes and writes each
one's form to the outbox. Its telemetry is made for an agent to diagnose
from: every event says what happened and the input's shape (sizes,
encodings, the error's class), never a note's words, a file's name or a
client's id. Inputs are named by a hash of their file name.

  server.py --port P --state DIR --inbox DIR      run
  server.py --replay DIR --out DIR                parse every note once, write out/result.json
"""
import argparse, collections, hashlib, http.server, json, os, sys, threading, time
import notes

VERSION = "0.2.0"
CRASH = False

a = argparse.ArgumentParser()
a.add_argument("--port", type=int); a.add_argument("--state"); a.add_argument("--inbox")
a.add_argument("--replay"); a.add_argument("--out")
args = a.parse_args()
if CRASH:
    sys.exit("stand-in: planted crash at start")

ref = lambda name: hashlib.sha256(name.encode()).hexdigest()[:12]


def one(path):
    raw = open(path, "rb").read()
    ev = {"input": ref(os.path.basename(path)), "shape": notes.shape(raw)}
    try:
        r = notes.parse(raw)
        ev.update(outcome="parsed" if not r["missing"] else "incomplete", missing=r["missing"])
        return ev, r
    except Exception as e:  # the class and where, never the message: it can quote the input
        ev.update(outcome="error", error_class=type(e).__name__, at=f"{e.__traceback__.tb_next.tb_frame.f_code.co_name if e.__traceback__.tb_next else 'parse'}")
        return ev, None


if args.replay:
    os.makedirs(args.out, exist_ok=True)
    events = [one(os.path.join(args.replay, f))[0] for f in sorted(os.listdir(args.replay))]
    json.dump({"version": VERSION, "events": events, "counts": collections.Counter(e["outcome"] for e in events)},
              open(os.path.join(args.out, "result.json"), "w"), indent=1)
    sys.exit(0)

events = collections.deque(maxlen=500)
seen = set()
counts = collections.Counter()
started = time.time()
outbox = os.path.join(args.state, "outbox")
os.makedirs(outbox, exist_ok=True)


def watch():
    while True:
        try:
            names = sorted(os.listdir(args.inbox))
        except OSError as e:
            if not events or events[-1].get("outcome") != "inbox-unreadable":
                events.append({"at": round(time.time(), 1), "outcome": "inbox-unreadable", "error_class": type(e).__name__})
            time.sleep(1); continue
        for f in names:
            if f in seen:
                continue
            seen.add(f)
            ev, r = one(os.path.join(args.inbox, f))
            ev["at"] = round(time.time(), 1)
            events.append(ev); counts[ev["outcome"]] += 1
            if r:
                json.dump(r, open(os.path.join(outbox, ev["input"] + ".json"), "w"))
        time.sleep(1)


threading.Thread(target=watch, daemon=True).start()


class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = {
            "/health": lambda: {"service": "notes-to-forms", "version": VERSION},
            "/telemetry/state": lambda: {"version": VERSION, "up_s": round(time.time() - started), "counts": counts, "inbox_seen": len(seen)},
            "/telemetry/events": lambda: list(events),
        }.get(self.path)
        if not body:
            self.send_error(404); return
        b = json.dumps(body()).encode()
        self.send_response(200); self.send_header("content-type", "application/json"); self.end_headers(); self.wfile.write(b)

    def log_message(self, *a): pass


http.server.ThreadingHTTPServer(("127.0.0.1", args.port), H).serve_forever()
