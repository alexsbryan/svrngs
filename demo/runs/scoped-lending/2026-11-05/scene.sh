#!/bin/sh
# The real-node House/Trip denial and withdrawal scene (SCENE.md here;
# docs/LENDING_SCOPE.md "Proof before landing" 1, 3, 4). Separate processes
# of target/debug/svrngs through a `svrngs relay --at 127.0.0.1:<port> --open`.
#
#   W=<scratch> [BIN=<binary>] [OUT=<dir>] sh demo/runs/scoped-lending/2026-11-05/scene.sh
#
# Each keeper has a HOME and SVRNGS_DIR of its own under $W and an environment
# built from nothing. Every process runs under sandbox-exec (scene.sb, written
# below): outbound network is refused but for TCP to loopback, so n0 cannot
# carry anything, and no UDP leaves, so no QUIC path forms between keepers and
# everything between them goes through the local relay. Writes to the real
# ~/.svrngs and ~/svrngs are refused too. It stops at the first failed check. The tokens and links it sees are never
# written to its output. scripts/check.sh runs none of this.
exec python3 -I -c '
import sys
sys.argv = sys.argv[1:]
src = open(sys.argv[0]).read().split("#---PY---\n", 1)[1]
exec(compile(src, sys.argv[0], "exec"), {"__name__": "__main__", "__file__": sys.argv[0]})
' "$0" "$@"
#---PY---
import http.client, json, os, re, shutil, signal, socket, subprocess, sys, threading, time, urllib.parse
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
BIN = Path(os.environ.get("BIN", REPO / "target/debug/svrngs"))
W = Path(os.environ["W"])
OUT = Path(os.environ.get("OUT", HERE / "logs"))
LOG = "door=debug,tiers=debug,join=debug"
REAL = Path(os.path.expanduser("~"))
PY = "/usr/bin/python3"
procs, outs, hidden, pids, facts, ups = {}, {}, [], [], {}, []


def redact(text):
    for h in hidden:
        text = text.replace(h, "<secret>")
    return re.sub(r"svrngs://join/\S+", "svrngs://join/<link>", text)


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}, ensure_ascii=False, default=str))
    print(line[:900], flush=True)
    with open(OUT / "events.jsonl", "a") as f:
        f.write(line + "\n")


class Failed(Exception):
    pass


def check(item, what, ok, **evidence):
    say("check", item=item, what=what, ok=bool(ok), **evidence)
    facts.setdefault("checks", []).append({"item": item, "what": what, "ok": bool(ok)})
    if not ok:
        raise Failed(f"item {item}: {what}")


def wait(cond, bound, every=0.25):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got:
            return got
        time.sleep(every)
    return None


def env(who, log=LOG):
    home = W / who
    return {"HOME": str(home), "USER": who, "LOGNAME": who, "SVRNGS_DIR": str(home / ".svrngs"),
            "PATH": "/usr/bin:/bin", "NO_COLOR": "1", "RUST_LOG": log, "TMPDIR": str(W / "tmp")}


def wrapped(args):
    return ["/usr/bin/sandbox-exec", "-f", str(W / "scene.sb"), str(BIN), *args]


def start(who, *args, log=LOG):
    (W / who).mkdir(parents=True, exist_ok=True)
    n = len(list(W.glob(f"{who}.*.out"))) + 1
    outs[who] = W / f"{who}.{n}.out"
    procs[who] = subprocess.Popen(wrapped(args), env=env(who, log), stdin=subprocess.DEVNULL,
                                  stdout=open(outs[who], "w"), stderr=open(W / f"{who}.log", "a"))
    pids.append(procs[who].pid)
    say("svrngs", who=who, pid=procs[who].pid, args=[a if not a.startswith(("svrngs://", "http://localhost")) else "<link>" for a in args])


def stop(who):
    p = procs.pop(who)
    p.send_signal(signal.SIGINT)
    try:
        p.wait(20)
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
    say("stopped", who=who, pid=p.pid, exit=p.returncode)


def said(who):
    return outs[who].read_text() if who in outs and outs[who].exists() else ""


def verb(who, *args, stdin="\n"):
    """A command run to its end, as a person types it beside a running svrngs."""
    t0 = time.time()
    r = subprocess.run(wrapped(args), env=env(who, "error"), input=stdin, capture_output=True, text=True)
    return r.returncode, (r.stdout + r.stderr).strip(), t0, time.time()


def data(who):
    return W / who / ".svrngs"


def api_port(who):
    return urllib.parse.urlsplit((data(who) / "api").read_text().strip()).port


def token(who):
    t = (data(who) / "token").read_text().strip()
    if t not in hidden:
        hidden.append(t)
    return t


def ask(who, path, host=None, method="GET", body=None, extra=None, timeout=30):
    """(status, body text, refused kind) of one request to who's own svrngs, as a program there asks."""
    h = {"Authorization": f"Bearer {token(who)}"}
    if host:
        h["Host"] = host
    if body is not None:
        h["Content-Type"] = "application/json"
    h.update(extra or {})
    try:
        c = http.client.HTTPConnection("127.0.0.1", api_port(who), timeout=timeout)
        c.request(method, path, body=body, headers=h)
        r = c.getresponse()
        text = r.read().decode(errors="replace")
        return r.status, text, r.getheader("x-svrngs-refused")
    except Exception as e:
        return None, repr(e), None


def api(who, path):
    s, b, _ = ask(who, path, timeout=20)
    try:
        return json.loads(b) if s == 200 else None
    except ValueError:
        return None


def up(who, bound=60):
    if not wait(lambda: api(who, "/v1/groups"), bound):
        raise Failed(f"{who}'s svrngs did not answer: {said(who)[-400:]}")


def link_of(who, bound=90):
    got = wait(lambda: re.search(r"svrngs://join/(\S+)", said(who)), bound)
    if not got:
        raise Failed(f"{who} printed no link: {said(who)[-600:]}")
    hidden.append(got[1])
    return got[0]


def joined(who, bound=120):
    if not wait(lambda: "You're in" in said(who), bound):
        raise Failed(f"{who} did not join: {said(who)[-400:]} {(W / (who + '.log')).read_text()[-800:]}")


def gid(who, name):
    for g in (api(who, "/v1/groups") or {}).get("groups", []):
        if g["name"] == name:
            return g["id"]
    return None


def listed(who, group):
    """The lends who's computer lists in the group, by lender and name, with where each opens."""
    got = api(who, f"/v1/lends?namespace={group}") or {}
    return {(l["person"], l["name"]): l["origin"] for l in got.get("lent", [])}, got


def host_of(origin):
    return urllib.parse.urlsplit(origin).netloc


class Upstream:
    def __init__(self, name):
        self.name = name
        self.log = W / f"upstream-{name}.jsonl"
        self.p = subprocess.Popen([PY, "-I", "-u", str(W / "upstream.py"), name, str(self.log)], stdout=subprocess.PIPE,
                                  stdin=subprocess.DEVNULL, text=True, env={"PATH": "/usr/bin:/bin", "HOME": str(W)})
        pids.append(self.p.pid)
        ups.append(self)
        self.port = int(self.p.stdout.readline())
        say("upstream", name=name, pid=self.p.pid, port=self.port)

    def requests(self):
        return [json.loads(l) for l in self.log.read_text().splitlines() if '"request"' in l]

    def count(self):
        return len(self.requests())


UPSTREAM = r"""
import http.server, json, sys, time, urllib.parse
name, path = sys.argv[1], sys.argv[2]
out = open(path, "a", buffering=1)
def log(**kw):
    out.write(json.dumps({"t": round(time.time(), 3), **kw}) + "\n")
class H(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    def log_message(self, *a): pass
    def do_GET(self):
        u = urllib.parse.urlsplit(self.path)
        told = {k.lower(): v for k, v in self.headers.items() if k.lower().startswith("x-svrngs") and "pass" not in k.lower()}
        log(kind="request", path=self.path, peer=self.client_address[1], told=told)
        if u.path == "/stream":
            n = int(urllib.parse.parse_qs(u.query).get("lines", ["300"])[0])
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Transfer-Encoding", "chunked")
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            try:
                for i in range(n):
                    b = f"{name} line {i}\n".encode()
                    self.wfile.write(b"%x\r\n%s\r\n" % (len(b), b))
                    self.wfile.flush()
                    time.sleep(0.2)
                self.wfile.write(b"0\r\n\r\n")
                self.wfile.flush()
                log(kind="stream-done", path=self.path, lines=n)
            except OSError as e:
                log(kind="stream-cut", path=self.path, at_line=i, error=type(e).__name__)
                self.close_connection = True
            return
        b = f"answer from {name} at {self.path}\n".encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(len(b)))
        self.end_headers()
        self.wfile.write(b)
s = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
s.daemon_threads = True
print(s.server_address[1], flush=True)
s.serve_forever()
"""


class Stream(threading.Thread):
    """One streaming GET through who's own svrngs, read off the raw socket: when each piece arrived, and how it ended."""

    def __init__(self, who, host, path):
        super().__init__(daemon=True)
        self.who, self.host, self.path = who, host, path
        self.times, self.buf, self.how, self.end, self.head_at = [], b"", None, None, None
        self.first = threading.Event()

    def run(self):
        s = socket.create_connection(("127.0.0.1", api_port(self.who)), timeout=120)
        s.sendall((f"GET {self.path} HTTP/1.1\r\nHost: {self.host}\r\nAuthorization: Bearer {token(self.who)}\r\nConnection: close\r\n\r\n").encode())
        try:
            while True:
                b = s.recv(65536)
                now = time.time()
                if not b:
                    self.how = "eof"
                    break
                self.buf += b
                self.times.append(now)
                if b" line " in self.buf:
                    self.first.set()
        except OSError as e:
            now = time.time()
            self.how = type(e).__name__
        self.end = now
        s.close()

    def lines(self):
        return self.buf.count(b" line ")

    def clean(self):
        return self.buf.endswith(b"0\r\n\r\n")

    def status(self):
        return self.buf.split(b"\r\n", 1)[0].decode(errors="replace")


def poll_answers(who, host, stop_flag, into):
    while not stop_flag.is_set():
        t = time.time()
        s, b, r = ask(who, "/", host, timeout=10)
        into.append({"t": round(t, 3), "ms": round((time.time() - t) * 1000), "status": s, "refused": r})
        time.sleep(0.25)


def appended(who, group, n):
    s, b, _ = ask(who, f"/v1/rail/append?namespace={group}", method="POST", body=json.dumps({"op": "record", "payload": {"scene": n}}))
    return json.loads(b)["id"] if s == 200 else None


def holds(who, group, id_):
    s, b, _ = ask(who, f"/v1/rail/log?namespace={group}")
    return s == 200 and id_ in b


def sync_ms(frm, to, group, n, bound=20):
    t = time.time()
    id_ = appended(frm, group, n)
    if not id_:
        return None
    return round((time.time() - t) * 1000) if wait(lambda: holds(to, group, id_), bound, 0.05) else None


def sha(path):
    return subprocess.run(["/usr/bin/shasum", "-a", "256", str(path)], capture_output=True, text=True).stdout.split()[0]


def real_svrngs():
    p = REAL / "svrngs"
    return sorted(x.name for x in p.iterdir()) if p.is_dir() else []


def share(port, name, *groups):
    g = [a for x in groups for a in ("--group", x)]
    return verb("Lender", "share", str(port), "--name", name, "--kind", "probe", *g)


def scene():
    facts["head"] = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
    facts["binary_sha256"] = sha(BIN)
    before = real_svrngs()
    n0 = subprocess.run(["/usr/bin/sandbox-exec", "-f", str(W / "scene.sb"), "/usr/bin/curl", "-sS", "--max-time", "6", "-o", "/dev/null",
                         "https://use1-1.relay.n0.iroh.link/ping"], capture_output=True, text=True)
    check("rules", "under the profile a process cannot reach n0", n0.returncode != 0, curl_exit=n0.returncode, said=n0.stderr.strip())
    say("start", head=facts["head"], binary_sha256=facts["binary_sha256"], real_home_folders=len(before),
        load=round(os.getloadavg()[0], 1))

    main, porch, atlas, attic = (Upstream(n) for n in ("hearth", "porch", "atlas", "attic"))
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        rport = s.getsockname()[1]
    relay = f"http://127.0.0.1:{rport}/"
    start("Relay", "relay", "--at", f"127.0.0.1:{rport}", "--open", log="relay=debug,info")
    if not wait(lambda: subprocess.run(["/bin/sh", str(REPO / "scripts/relay-answers.sh"), relay], capture_output=True, env={"PATH": "/usr/bin:/bin"}).returncode == 0, 30):
        raise Failed("the relay did not answer")
    say("relay", url=relay)

    # House, then Trip, by the verbs a person types: `new` and `link` each need
    # the data dir to itself, so svrngs stops between them.
    start("Lender", "new", "House", "--as", "Lender", "--relay", relay)
    link_of("Lender"); up("Lender"); stop("Lender")
    start("Lender", "new", "Trip", "--as", "Lender", "--relay", relay)
    trip_link = link_of("Lender"); up("Lender")
    start("Theo", "join", trip_link, "--as", "Theo")
    joined("Theo"); up("Theo")
    stop("Lender")
    start("Lender", "link", "House")
    house_link = link_of("Lender"); up("Lender")
    start("Hana", "join", house_link, "--as", "Hana")
    joined("Hana"); up("Hana")
    house, trip = gid("Lender", "House"), gid("Lender", "Trip")
    facts.update(house=house, trip=trip)
    for who in ("Relay", "Lender", "Hana", "Theo"):
        seen = subprocess.run(["/bin/ps", "-wwE", "-o", "command=", "-p", str(procs[who].pid)], capture_output=True, text=True).stdout
        homes = re.findall(r"(?:^| )HOME=(\S+)", seen)
        check("rules", f"{who}'s HOME is its own", homes == [str(W / who)], pid=procs[who].pid, homes=homes)
    say("groups", house=house, trip=trip, hana=api("Hana", "/v1/groups"), theo=api("Theo", "/v1/groups"))

    for up_, name, group in ((main, "hearth", "House"), (porch, "porch", "House"), (atlas, "atlas", "Trip"), (attic, "attic", "House")):
        code, text, _, _ = share(up_.port, name, group)
        say("share", name=name, group=group, exit=code, said=text[-400:])
        check("setup", f"share {name} to {group}", code == 0)
    decl = data("Lender") / "lends" / "hearth.json"
    say("declaration", file=decl.name, bytes=decl.read_text())

    # 1. Hana lists hearth and gets the upstream's answer through it.
    got = wait(lambda: (lambda l: l if ("Lender", "hearth") in l else None)(listed("Hana", house)[0]), 60, 0.5)
    check(1, "Hana lists hearth", got, listed=sorted(n for _, n in (got or {})))
    hearth, hporch, hattic = (host_of(got[("Lender", n)]) for n in ("hearth", "porch", "attic"))
    n0 = main.count()
    s, b, r = ask("Hana", "/", hearth)
    check(1, "Hana gets the upstream's answer through hearth", s == 200 and "answer from hearth" in b and main.count() == n0 + 1,
          status=s, body=b, upstream=main.requests()[-1])

    # 2. Theo: not listed, and the copied address refused with no upstream contact.
    got = wait(lambda: (lambda l: l if ("Lender", "atlas") in l else None)(listed("Theo", trip)[0]), 60, 0.5)
    check(2, "Theo lists atlas in Trip, so the Lender answered him, and not hearth",
          got and not any(n in ("hearth", "porch", "attic") for _, n in got), listed=sorted(n for _, n in (got or {})))
    s, b, _ = ask("Theo", f"/v1/lends?namespace={house}")
    check(2, "Theo cannot ask House's lends", s != 200 or "hearth" not in b, status=s, body=b[:300])
    n0 = main.count()
    tries = [("copied address /", "/", {}),
             ("copied address /stream", "/stream?lines=3", {}),
             ("forged headers", "/", {"X-Svrngs-User": "Hana", "X-Svrngs-Group": house, "X-Svrngs-Person": "Hana"})]
    for name, path, extra in tries:
        s, b, r = ask("Theo", path, hearth, extra=extra)
        check(2, f"Theo refused: {name}", s not in (None, 200) and "answer from" not in b, status=s, refused=r, body=b[:300])
    time.sleep(0.5)
    check(2, "the upstream saw none of Theo's attempts", main.count() == n0, upstream_requests=main.count())

    # 3. Trip's lend works for Theo: the denial is scope, not reachability.
    n0 = atlas.count()
    s, b, r = ask("Theo", "/", host_of(got[("Lender", "atlas")]))
    check(3, "Theo gets atlas through Trip", s == 200 and "answer from atlas" in b and atlas.count() == n0 + 1, status=s, body=b, upstream=atlas.requests()[-1])

    # 4. A third group on the Lender changes no declaration and reaches nothing.
    before_sha = {p.name: sha(p) for p in sorted((data("Lender") / "lends").glob("*.json"))}
    before_bytes = decl.read_bytes()
    start("Ren", "new", "Book", "--as", "Ren", "--relay", relay)
    book_link = link_of("Ren"); up("Ren")
    code, text, _, _ = verb("Lender", "join", book_link, "--as", "Lender")
    say("join", who="Lender", exit=code, said=text[-300:])
    check(4, "Lender joins Book", code == 0)
    book = wait(lambda: gid("Lender", "Book"), 30)
    facts["book"] = book
    check(4, "Book is held on the Lender", book, groups=[g["name"] for g in api("Lender", "/v1/groups")["groups"]])
    after_sha = {p.name: sha(p) for p in sorted((data("Lender") / "lends").glob("*.json"))}
    check(4, "the declaration's bytes are unchanged", before_sha == after_sha and decl.read_bytes() == before_bytes,
          before=before_sha["hearth.json"], after=after_sha["hearth.json"])
    time.sleep(3)
    l, whole = listed("Ren", book)
    check(4, "Ren lists none of the Lender's lends in Book", not any(p == "Lender" for p, _ in l),
          listed=sorted(l), unanswered=whole.get("unanswered"), refused=whole.get("refused"), unheard=whole.get("unheard"), older=whole.get("older"))
    n0 = main.count()
    s, b, r = ask("Ren", "/", hearth)
    time.sleep(0.5)
    check(4, "Ren's copied address is refused, no upstream contact", s not in (None, 200) and main.count() == n0, status=s, refused=r, body=b[:300])

    # 5. Withdrawal, three runs, each a different phase of the door's sweep.
    runs = []
    for i in range(3):
        if i:
            code, text, _, _ = share(main.port, "hearth", "House")
            check(5, f"run {i + 1}: hearth lent to House again", code == 0, said=text[-200:])
        if not wait(lambda: ask("Hana", "/", hearth)[0] == 200, 20, 0.2):
            raise Failed(f"run {i + 1}: Hana cannot reach hearth before the cut")
        flag, polls = threading.Event(), []
        poller = threading.Thread(target=poll_answers, args=("Hana", hporch, flag, polls), daemon=True)
        poller.start()
        cut = Stream("Hana", hearth, "/stream?lines=300")
        other = Stream("Hana", hporch, f"/stream?lines={40}")
        other.start(); cut.start()
        check(5, f"run {i + 1}: the stream is running", cut.first.wait(20), status=cut.status())
        time.sleep(1.6 + 0.37 * i)
        before_ms = sync_ms("Lender", "Hana", house, f"before-{i}")
        lines_before = cut.lines()
        code, text, t0, t1 = verb("Lender", "unshare", "hearth", "--group", "House")
        during_ms = sync_ms("Hana", "Lender", house, f"during-{i}")
        cut.join(15)
        after_cut = time.time()
        back = sync_ms("Lender", "Hana", house, f"after-{i}")
        status, body, refused = ask("Hana", "/", hearth)
        other.join(20)
        flag.set(); poller.join(5)
        window = [p for p in polls if p["t"] >= t0 - 1]
        run = {"run": i + 1, "unshare_exit": code, "unshare_said": text[-200:], "unshare_ms": round((t1 - t0) * 1000),
               "return_to_end_s": round(cut.end - t1, 3) if cut.end else None, "ended": cut.how, "clean_end": cut.clean(),
               "lines_at_unshare": lines_before, "lines_total": cut.lines(), "last_data_after_return_s": round(cut.times[-1] - t1, 3),
               "fresh_request": {"status": status, "refused": refused, "body": body[:200]},
               "porch_stream": {"how": other.how, "clean": other.clean(), "lines": other.lines(), "of": 40},
               "porch_polls": {"n": len(window), "answered": sum(p["status"] == 200 for p in window), "worst_ms": max(p["ms"] for p in window) if window else None},
               "sync_ms": {"lender_to_hana_before": before_ms, "hana_to_lender_during": during_ms, "lender_to_hana_after": back},
               "upstream_end": [json.loads(l) for l in main.log.read_text().splitlines() if '"stream-' in l][-1:]}
        runs.append(run)
        say("withdrawal", **run)
        check(5, f"run {i + 1}: unshare exit 0", code == 0)
        check(5, f"run {i + 1}: the stream ended, not to its natural end", cut.end and not cut.clean() and cut.lines() < 300)
        check(5, f"run {i + 1}: ended within the 2 s bound", cut.end - t1 <= 2.0, seconds=run["return_to_end_s"])
        check(5, f"run {i + 1}: a fresh request is refused", status not in (None, 200), status=status, refused=refused)
        check(5, f"run {i + 1}: porch's stream ran to its end", other.clean() and other.lines() == 40, **run["porch_stream"])
        check(5, f"run {i + 1}: every porch request answered", window and all(p["status"] == 200 for p in window), **run["porch_polls"])
        check(5, f"run {i + 1}: sync both ways through the cut", before_ms is not None and during_ms is not None and back is not None, **run["sync_ms"])
    facts["withdrawal"] = runs

    # 6. Restart: the selection persists, Theo still denied.
    code, text, _, _ = share(main.port, "hearth", "House")
    check(6, "hearth lent to House again", code == 0, said=text[-200:])
    wait(lambda: ask("Hana", "/", hearth)[0] == 200, 20, 0.2)
    b6 = sha(decl); bytes6 = decl.read_text()
    stop("Lender"); start("Lender"); up("Lender")
    check(6, "the declaration is the same bytes after a restart", sha(decl) == b6 and house in decl.read_text(), sha=b6, bytes=decl.read_text())
    ok = wait(lambda: ask("Hana", "/", hearth)[0] == 200, 60, 0.5)
    check(6, "Hana still gets hearth", ok)
    l = wait(lambda: (lambda m: m if ("Lender", "atlas") in m else None)(listed("Theo", trip)[0]), 60, 0.5)
    check(6, "Theo lists atlas and not hearth", l and not any(n == "hearth" for _, n in l), listed=sorted(n for _, n in (l or {})))
    n0 = main.count()
    s, b, r = ask("Theo", "/", hearth)
    time.sleep(0.5)
    check(6, "Theo still refused, no upstream contact", s not in (None, 200) and main.count() == n0, status=s, refused=r, body=b[:300])

    # 7. Legacy: `groups` taken out of attic's file while svrngs is stopped.
    s, b, _ = ask("Hana", "/", hattic)
    check(7, "attic answers Hana while it is lent to House", s == 200 and "answer from attic" in b, status=s)
    stop("Lender")
    afile = data("Lender") / "lends" / "attic.json"
    parsed = json.loads(afile.read_text())
    say("legacy-selected", bytes=afile.read_text())
    parsed.pop("groups")
    afile.write_text(json.dumps(parsed, indent=2) + "\n")
    legacy_before = afile.read_text()
    say("legacy-before", sha=sha(afile), bytes=legacy_before)
    check(7, "no groups key before start", '"groups"' not in legacy_before)
    start("Lender"); up("Lender")
    wait(lambda: '"groups"' in afile.read_text(), 20, 0.1)
    legacy_after = afile.read_text()
    say("legacy-after", sha=sha(afile), bytes=legacy_after)
    check(7, "stamped groups: null on start", json.loads(legacy_after).get("groups", "absent") is None and re.search(r'"groups":\s*null', legacy_after))
    wait(lambda: ("Lender", "hearth") in listed("Hana", house)[0], 60, 0.5)
    wait(lambda: ("Lender", "atlas") in listed("Theo", trip)[0], 60, 0.5)
    lists = {"Hana/House": sorted(listed("Hana", house)[0]), "Theo/Trip": sorted(listed("Theo", trip)[0]), "Ren/Book": sorted(listed("Ren", book)[0])}
    check(7, "no group lists attic", not any(n == "attic" for v in lists.values() for _, n in v), lists=lists)
    n0 = attic.count()
    s1, b1, r1 = ask("Hana", "/", hattic)
    time.sleep(0.5)
    check(7, "Hana's copied address to attic is refused, no upstream contact", s1 not in (None, 200) and attic.count() == n0, status=s1, refused=r1, body=b1[:300])
    code, text, _, _ = verb("Lender", "unshare", "attic", "--group", "House")
    check(7, "unshare --group on a legacy lend is refused and says so", code != 0, exit=code, said=text[-300:])

    log = (W / "Lender.log").read_text(errors="replace")
    relayed = {"true": len(re.findall(r"relayed=Some\(true\)", log)), "false": len(re.findall(r"relayed=Some\(false\)", log))}
    given = sorted(set(re.findall(r"a relay given to this computer's endpoint relay=(\S+)", re.sub(r"\x1b\[[0-9;]*m", "", log))))
    facts["relayed"] = relayed
    plain = re.sub(r"\x1b\[[0-9;]*m", "", log).splitlines()
    denied = [l for l in plain if "door: refused — the dialer stands in no group this lend is lent to" in l]
    keys = {w: api(w, "/v1/groups")["key"] for w in ("Theo", "Ren", "Hana")}
    count = {w: sum(f'actor="{k}"' in l and "lend=hearth" in l for l in denied) for w, k in keys.items()}
    cuts = [l for l in plain if "door: an answer cut" in l]
    check("rules", "the door traced each refusal of hearth", count["Theo"] == 4 and count["Ren"] == 1, count=count)
    check("rules", "the door traced each cut", len(cuts) == 3, cuts=[l[:120] for l in cuts])
    check("rules", "every member's use the door traced came through the relay", relayed["true"] and not relayed["false"], **relayed)
    check("rules", "the only relay ever given to the Lender's endpoint is the local one", given == [relay], given=given)
    after = real_svrngs()
    check("rules", "no new folder in the real ~/svrngs", set(after) <= set(before), new=sorted(set(after) - set(before)), count=len(after))
    facts["verdict"] = "pass"


def trim():
    """The lines that prove an item, from each keeper's trace."""
    keep = re.compile(r"door: (refused|let in|guessed|an answer cut|the lend's service answered)|tiers: (this computer meets|a relay (given|taken))|join: |relay: (admitted|plain|starting)|WARN|ERROR")
    noise = re.compile(r"lend=porch")
    for who in ("Lender", "Hana", "Theo", "Ren", "Relay"):
        p = W / f"{who}.log"
        if p.exists():
            shutil.copy(p, W / f"{who}.full.log")
            lines = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in p.read_text(errors="replace").splitlines()]
            lines = [l for l in lines if keep.search(l) and not noise.search(l)]
            (OUT / f"{who}.log").write_text(redact("\n".join(lines[-1500:])) + "\n")
    for p in W.glob("upstream-*.jsonl"):
        (OUT / p.name).write_text(redact(p.read_text()))


def main_():
    shutil.rmtree(W, ignore_errors=True)
    (W / "tmp").mkdir(parents=True)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "events.jsonl").write_text("")
    (W / "upstream.py").write_text(UPSTREAM)
    (W / "scene.sb").write_text(f'''(version 1)
(allow default)
(deny network-outbound (remote ip "*:*"))
(allow network-outbound (remote tcp "localhost:*"))
(deny file-write* (subpath "{REAL}/.svrngs") (subpath "{REAL}/svrngs"))
''')
    code = 1
    try:
        scene()
        code = 0
    except Failed as e:
        facts["verdict"] = f"FAILED: {e}"
        say("failed", why=str(e))
    finally:
        for who in reversed(list(procs)):
            stop(who)
        for u in ups:
            u.p.terminate()
            u.p.wait(10)
        alive = [p for p in pids if subprocess.run(["/bin/kill", "-0", str(p)], capture_output=True).returncode == 0]
        facts["pids"] = pids
        facts["left_running"] = alive
        trim()
        (OUT / "facts.json").write_text(redact(json.dumps(facts, indent=1, ensure_ascii=False, default=str)) + "\n")
        print(json.dumps({k: facts.get(k) for k in ("verdict", "left_running")}), flush=True)
    sys.exit(code)


main_()
