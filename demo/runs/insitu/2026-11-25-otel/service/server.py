"""Acme Notes-to-Forms: each session note in the inbox becomes a claim,
submitted to the clinic's clearinghouse. Instrumented with OpenTelemetry;
it exports to OTEL_EXPORTER_OTLP_ENDPOINT, wherever that points.

  server.py --port P --state DIR --inbox DIR      run
  server.py --replay DIR --state DIR --out DIR    each note once, then out/result.json
"""
import argparse, hashlib, http.server, json, logging, os, sys, threading, time, urllib.request, urllib.error
from opentelemetry import trace
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk._logs import LoggerProvider, LoggingHandler
from opentelemetry.sdk._logs.export import BatchLogRecordProcessor
from opentelemetry.exporter.otlp.proto.http._log_exporter import OTLPLogExporter
from opentelemetry._logs import set_logger_provider
import notes, clients

VERSION = "0.3.0"
CRASH = False

a = argparse.ArgumentParser()
for f in ("--port", "--state", "--inbox", "--replay", "--out"):
    a.add_argument(f)
args = a.parse_args()
if CRASH:
    sys.exit("stand-in: planted crash at start")

resource = Resource.create({"service.name": "notes-to-forms", "service.version": VERSION})
tp = TracerProvider(resource=resource); tp.add_span_processor(BatchSpanProcessor(OTLPSpanExporter(), schedule_delay_millis=1000))
trace.set_tracer_provider(tp)
lp = LoggerProvider(resource=resource); lp.add_log_record_processor(BatchLogRecordProcessor(OTLPLogExporter(), schedule_delay_millis=1000))
set_logger_provider(lp)
log = logging.getLogger("notes-to-forms"); log.setLevel(logging.INFO); log.addHandler(LoggingHandler(logger_provider=lp))
tracer = trace.get_tracer("notes-to-forms")
CLEARINGHOUSE = os.environ.get("CLEARINGHOUSE_URL", "")
ref = lambda name: hashlib.sha256(name.encode()).hexdigest()[:12]


def submit(claim):
    req = urllib.request.Request(f"{CLEARINGHOUSE}/claims", data=json.dumps(claim, sort_keys=True).encode(), headers={"content-type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()


def one(path):
    name = os.path.basename(path)
    with tracer.start_as_current_span("note") as span:
        span.set_attribute("note.file", name)
        span.set_attribute("input.ref", ref(name))
        try:
            with tracer.start_as_current_span("parse"):
                form = notes.parse(open(path, "rb").read())
            with tracer.start_as_current_span("lookup"):
                plan = clients.plan_of(os.path.join(args.state, "clients.db"), form["client_id"])
            with tracer.start_as_current_span("submit") as s:
                status, body = submit({**form, **plan})
                s.set_attribute("http.response.status_code", status)
            if status == 599:
                outcome = "unjudged"
            elif status >= 400:
                outcome = "rejected"
                log.warning("claim rejected for %s: %s", form["client_id"], body)
            else:
                outcome = "submitted"
            span.set_attribute("outcome", outcome)
            return {"input": ref(name), "outcome": outcome}
        except Exception as e:
            span.record_exception(e)
            span.set_status(trace.StatusCode.ERROR)
            span.set_attribute("outcome", "error")
            log.error("note %s failed: %s", name, e)
            return {"input": ref(name), "outcome": "error", "error_class": type(e).__name__}


if args.replay:
    os.makedirs(args.out, exist_ok=True)
    events = [one(os.path.join(args.replay, f)) for f in sorted(os.listdir(args.replay))]
    json.dump({"version": VERSION, "events": events}, open(os.path.join(args.out, "result.json"), "w"), indent=1)
    tp.force_flush(); lp.force_flush()
    sys.exit(0)

seen = set()


def watch():
    while True:
        try:
            names = sorted(os.listdir(args.inbox))
        except OSError as e:
            log.error("inbox unreadable: %s", type(e).__name__); time.sleep(5); continue
        for f in names:
            if f not in seen:
                seen.add(f); one(os.path.join(args.inbox, f))
        time.sleep(1)


threading.Thread(target=watch, daemon=True).start()


class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != "/health":
            self.send_error(404); return
        b = json.dumps({"service": "notes-to-forms", "version": VERSION}).encode()
        self.send_response(200); self.send_header("content-type", "application/json"); self.end_headers(); self.wfile.write(b)

    def log_message(self, *a): pass


http.server.ThreadingHTTPServer(("127.0.0.1", int(args.port)), H).serve_forever()
