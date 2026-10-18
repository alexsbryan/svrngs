#!/usr/bin/env python3
"""One machine's part of the 25-computer run: several svrngs computers on
this machine, each a process with a HOME and SVRNGS_DIR of its own under
the work dir W, as demo/runs/first-minute/2026-10-12/run.py runs three (its
env at :32-33, api() at :36, serve and share at :57-64). Stdlib only; the
same file runs on the Mac and on Linux. It never touches this user's own
~/.svrngs or ~/svrngs: every computer's HOME is W/<name>.

Keepers, each running until `stop` (SIGTERM), each writing its pid to
W/keeper-<role>.pid and the svrngs it started to W/children-<role>.json:

    fleet.py found   --bin B --w W --plan PLAN.json --group NAME
        the founder: `svrngs new <group> --as <name>`, its greeter lent;
        writes W/founded.json {link, gid, port, founder} once it holds both
    fleet.py start   --bin B --w W --plan PLAN.json --machine M --link L --gid G
        every non-founder person whose machine is M, each at its `at`
        (epoch seconds on THIS machine's clock): greeter up, `svrngs join
        <link> --as <name>`, in, then `svrngs share` of the greeter
    fleet.py sampler --bin B --w W [--secs N]
        back-to-back windows of N s: each svrngs process's CPU and RSS,
        appended to W/samples.jsonl

One-shots, each printing one JSON object:

    fleet.py wall   --w W --gid G --who FOUNDER --expect N --ceiling S --linger S
        the founder's home page's reads, back to back: /v1/lends then
        /v1/rail/people, each starting when the last lands; every read a
        line of W/wall.jsonl; ends `linger` s after it first reads all N
        settled, or at `ceiling` s
    fleet.py probe  --w W --gid G --plan PLAN.json [--visits N] [--seed S]
    fleet.py paths  --w W
    fleet.py logs   --w W            a scrubbed .tar.gz of the logs, on stdout
    fleet.py stop   --w W
    fleet.py clean  --w W
    fleet.py clock

Every wait has a ceiling; one that runs out is said, never waited on
further (AGENTS.md 6). Every step is a JSON line in W/events.jsonl,
stamped with this machine's clock.
"""
import argparse
import http.client
import io
import json
import os
import random
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import threading
import time
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import greeter  # noqa: E402

MARK = ".twentyfive"
# Debug where the run reads its timings, and iroh's one event naming the
# network path it picked to each remote (iroh-1.0.2
# src/socket/remote_map/remote_state.rs:661-667, `iroh::_events::path::selected`).
LOG = os.environ.get("TWENTYFIVE_LOG", "info,node=debug,redeem=debug,cli=debug,surface=debug,"
                     "svrngs_node::ring_sync=debug,transport=warn,iroh::_events::path::selected=debug")
FOUND_WITHIN = 60   # s: `new` prints its link and holds its group, or the run never ran
JOIN_WITHIN = 120   # s: a join not in by then is said not in
SHARE_WITHIN = 30   # s: `share` declares a file and returns (V1.1 cli.rs:818, svrngs running here)
TOKEN = re.compile(r"svrngs://join/[A-Za-z0-9_-]+")

lock = threading.Lock()


def now():
    return round(time.time(), 3)


class Events:
    def __init__(self, path):
        self.f = open(path, "a", buffering=1)

    def say(self, event, **kw):
        line = json.dumps({"t": now(), "event": event, **kw})
        with lock:
            self.f.write(line + "\n")


def home(w, who): return Path(w) / who
def data(w, who): return home(w, who) / ".svrngs"


def env(w, who, log=LOG):
    e = {k: v for k, v in os.environ.items() if k not in ("SVRNGS_DIR", "RUST_LOG")}
    return {**e, "HOME": str(home(w, who)), "USER": who, "SVRNGS_DIR": str(data(w, who)),
            "RUST_LOG": log, "NO_COLOR": "1"}


def api_base(w, who):
    try:
        return (data(w, who) / "api").read_text().strip().rstrip("/")
    except OSError:
        return None


def get(base, path, timeout=10):
    """GET base+path: (ms, status, json), or (ms, None, why). An absence is
    said, never read as empty."""
    t = time.time()
    try:
        with urllib.request.urlopen(base + path, timeout=timeout) as r:
            body = r.read()
            return round((time.time() - t) * 1000), r.status, json.loads(body)
    except urllib.error.HTTPError as e:
        return round((time.time() - t) * 1000), e.code, e.read()[:300].decode(errors="replace")
    except Exception as e:
        return round((time.time() - t) * 1000), None, str(e)


def holds(w, who, gid):
    base = api_base(w, who)
    if not base:
        return False
    _, status, body = get(base, "/v1/groups", timeout=5)
    return status == 200 and any(g.get("id") == gid for g in body.get("groups", []))


def slug(who):
    return re.sub(r"[^a-z0-9]", "", who.lower())


def keeper(w, role):
    w = Path(w)
    w.mkdir(parents=True, exist_ok=True)
    (w / MARK).write_text("made by fleet.py\n")
    (w / f"keeper-{role}.pid").write_text(str(os.getpid()))
    stopping = threading.Event()
    signal.signal(signal.SIGTERM, lambda *_: stopping.set())
    signal.signal(signal.SIGINT, lambda *_: stopping.set())
    return stopping


class Children:
    def __init__(self, w, role):
        self.path, self.pids = Path(w) / f"children-{role}.json", {}

    def add(self, who, pid):
        with lock:
            self.pids[who] = pid
            self.path.write_text(json.dumps(self.pids))

    def stop(self, ev):
        """SIGTERM, so each svrngs closes its endpoint (node.rs
        `asked_to_stop`); SIGKILL whatever is left after 15 s."""
        with lock:
            pids = list(self.pids.values())
        for pid in pids:
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        end = time.time() + 15
        while time.time() < end and any(alive(p) for p in pids):
            time.sleep(0.2)
        for pid in pids:
            if alive(pid):
                os.kill(pid, signal.SIGKILL)
                ev.say("killed", pid=pid)


def alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    try:  # a child of ours that exited is a zombie until reaped
        return os.waitpid(pid, os.WNOHANG) == (0, 0)
    except ChildProcessError:
        return True


def svrngs(binary, w, who, *args, ev):
    out = open(Path(w) / f"{who}.out", "w")
    err = open(Path(w) / f"{who}.log", "w")
    return subprocess.Popen([binary, *args], env=env(w, who), stdin=subprocess.DEVNULL, stdout=out, stderr=err)


def lend(binary, w, who, port, ev):
    """The greeter lent as `hello-<name>`, of kind `hello`: a port svrngs
    does not know, so `share` asks for a password, answered with Enter
    alone (V1.1 cli.rs:1383, `prompt_secret`)."""
    name = f"hello-{slug(who)}"
    try:
        run = subprocess.run([binary, "share", str(port), "--name", name, "--kind", "hello"],
                             env=env(w, who, "error"), input="\n", capture_output=True, text=True,
                             timeout=SHARE_WITHIN)
        ev.say("shared", who=who, lend=name, port=port, code=run.returncode,
               said=(run.stdout + run.stderr).strip()[-400:])
        return run.returncode == 0
    except subprocess.TimeoutExpired:
        ev.say("share-not-done", who=who, within_s=SHARE_WITHIN)
        return False


def found(a):
    stopping = keeper(a.w, "found")
    ev = Events(Path(a.w) / "events.jsonl")
    plan = json.loads(Path(a.plan).read_text())
    who = next(p["name"] for p in plan["people"] if p.get("founder"))
    home(a.w, who).mkdir(parents=True, exist_ok=True)
    kids = Children(a.w, "found")
    srv = greeter.serve(0, who)
    port = srv.server_address[1]
    ev.say("greeter-up", who=who, port=port)
    ev.say("new-start", who=who, group=a.group, load=os.getloadavg()[0], cores=os.cpu_count())
    proc = svrngs(a.bin, a.w, who, "new", a.group, "--as", who, ev=ev)
    kids.add(who, proc.pid)
    end, link, gid = time.time() + FOUND_WITHIN, None, None
    while time.time() < end and proc.poll() is None and not (link and gid):
        link = link or (m[0] if (m := TOKEN.search((Path(a.w) / f"{who}.out").read_text())) else None)
        base = api_base(a.w, who)
        if base and not gid:
            _, status, body = get(base, "/v1/groups", timeout=5)
            if status == 200 and body.get("groups"):
                gid = body["groups"][0].get("id")
        time.sleep(0.1)
    if not (link and gid):
        ev.say("found-not-done", who=who, link=bool(link), gid=bool(gid), exited=proc.poll())
        (Path(a.w) / "founded.json").write_text(json.dumps({"failed": True, "founder": who}))
        kids.stop(ev)
        return
    ev.say("founded", who=who, gid=gid)
    lent = lend(a.bin, a.w, who, port, ev)
    api = api_base(a.w, who)
    (Path(a.w) / "founded.json").write_text(json.dumps(
        {"founder": who, "link": link, "gid": gid, "lent": lent, "port": int(api.rsplit(":", 1)[1])}))
    stopping.wait()
    ev.say("found-stopping")
    kids.stop(ev)
    ev.say("found-stopped")


def person_run(a, p, ev, kids):
    who = p["name"]
    delay = p["at"] - time.time()
    if delay > 0:
        time.sleep(delay)
    home(a.w, who).mkdir(parents=True, exist_ok=True)
    srv = greeter.serve(0, who)
    port = srv.server_address[1]
    ev.say("greeter-up", who=who, port=port)
    ev.say("join-start", who=who, late_ms=round((time.time() - p["at"]) * 1000))
    proc = svrngs(a.bin, a.w, who, "join", a.link, "--as", who, ev=ev)
    kids.add(who, proc.pid)
    end = time.time() + JOIN_WITHIN
    while time.time() < end:
        if proc.poll() is not None:
            ev.say("join-exited", who=who, code=proc.returncode)
            return
        if holds(a.w, who, a.gid):
            ev.say("joined", who=who)
            lend(a.bin, a.w, who, port, ev)
            return
        time.sleep(0.1)
    ev.say("join-not-in", who=who, within_s=JOIN_WITHIN)


def start(a):
    stopping = keeper(a.w, "start")
    ev = Events(Path(a.w) / "events.jsonl")
    plan = [p for p in json.loads(Path(a.plan).read_text())["people"]
            if p["machine"] == a.machine and not p.get("founder")]
    ev.say("fleet-start", machine=a.machine, people=[p["name"] for p in plan],
           load=os.getloadavg()[0], cores=os.cpu_count())
    kids = Children(a.w, "start")
    for p in plan:
        threading.Thread(target=person_run, args=(a, p, ev, kids), daemon=True).start()
    stopping.wait()
    ev.say("fleet-stopping", children=len(kids.pids))
    kids.stop(ev)
    ev.say("fleet-stopped")


def seconds(cpu):
    """ps's cumulative CPU time as macOS (`m:ss.cc`, `h:mm:ss.cc`) and
    procps (`[d-]hh:mm:ss`) write it, in seconds."""
    days = 0
    if "-" in cpu:
        d, cpu = cpu.split("-", 1)
        days = int(d)
    s = 0.0
    for x in cpu.split(":"):
        s = s * 60 + float(x)
    return days * 86400 + s


def ps(binary):
    """Every process of `binary` on this machine: CPU seconds, RSS KiB, and
    whose it is, by its `--as`."""
    cols = ["ps", "-axo", "pid=,time=,rss=,command="] if sys.platform == "darwin" else ["ps", "-eo", "pid=,time=,rss=,args="]
    found_ = {}
    for line in subprocess.run(cols, capture_output=True, text=True).stdout.splitlines():
        bits = line.split(None, 3)
        if len(bits) == 4 and bits[3].startswith(binary + " ") and "--as" in bits[3]:
            m = re.search(r"--as (\S+)", bits[3])
            found_[int(bits[0])] = {"cpu_s": seconds(bits[1]), "rss_kib": int(bits[2]), "who": m[1]}
    return found_


def sampler(a):
    stopping = keeper(a.w, "sampler")
    out = open(Path(a.w) / "samples.jsonl", "a", buffering=1)
    t0, first = time.time(), ps(a.bin)
    while not stopping.wait(a.secs):
        t1, second = time.time(), ps(a.bin)
        rows = [{"who": s["who"], "cpu_pct": round(100 * (s["cpu_s"] - first[pid]["cpu_s"]) / (t1 - t0), 1),
                 "rss_mib": round(s["rss_kib"] / 1024, 1)} for pid, s in second.items() if pid in first]
        out.write(json.dumps({"t0": round(t0, 3), "t1": round(t1, 3), "load": os.getloadavg()[0],
                              "cores": os.cpu_count(), "rows": sorted(rows, key=lambda r: r["who"])}) + "\n")
        t0, first = t1, second


def wall(a):
    """The founder's home page, read as home.js reads it (V1.1
    home/home.js `show`: /v1/rail/people, then /v1/lends), back to back.
    Lit is the door's rule, copied as an instrument's reading and cited,
    not decided here: a person with a computer, not in `unanswered`
    (V1.1 home/question.js:115)."""
    base = api_base(a.w, a.who)
    out = open(Path(a.w) / "wall.jsonl", "a", buffering=1)
    start_, settled, reads, keys = time.time(), None, 0, {}
    while True:
        t = time.time()
        if t - start_ > a.ceiling or (settled and t - settled > a.linger):
            break
        pms, pstatus, people = get(base, f"/v1/rail/people?namespace={a.gid}")
        lms, lstatus, lends = get(base, f"/v1/lends?namespace={a.gid}", timeout=15)
        reads += 1
        line = {"t": round(t, 3), "people_ms": pms, "lends_ms": lms, "people_status": pstatus, "lends_status": lstatus}
        if pstatus == 200 and lstatus == 200:
            rows = people.get("people") or []
            computers = sorted({r["person"] for r in rows if r.get("device") == "computer"})
            unl = {k: lends.get(k) or [] for k in ("unanswered", "unheard", "refused", "older")}
            lit = [p for p in computers if p not in unl["unanswered"]]
            listed = sorted({l.get("person") for l in lends.get("lent", [])})
            line.update(people=sorted({r["person"] for r in rows}), computers=computers, lit=lit,
                        listed=listed, lent=len(lends.get("lent", [])), **unl)
            now_keys = {r["key"]: r["person"] for r in rows if r.get("key")}
            if now_keys != keys:  # each key's person, said when it changes, for paths' short ids
                keys = now_keys
                line["keys"] = keys
            whole = (len(computers) == a.expect and len(lit) == a.expect and len(listed) == a.expect
                     and not any(unl.values()))
            if whole and not settled:
                settled = t
                line["settled"] = True
        else:
            line["error"] = {"people": people if pstatus != 200 else None, "lends": lends if lstatus != 200 else None}
        out.write(json.dumps(line) + "\n")
        # A pause from one read's start to the next; 0 is back to back. The
        # first linux-burst run read back to back, ~700 reads a second, with
        # the founder at 180% CPU and 442 MiB: --every tells reads from members.
        if a.every:
            time.sleep(max(0.0, t + a.every - time.time()))
    print(json.dumps({"reads": reads, "settled_at": settled and round(settled, 3), "ended": now(),
                      "ceiling_hit": not settled}))


def visit(port, origin, timeout=10):
    """GET <origin>who as a program on this computer: loopback, the lend's
    Host, no Sec-Fetch-Site, which the door reads as a program's own
    (V1.1 crates/svrngs-node/src/origin.rs:79-84, `app_of`; :141, the
    label from Host)."""
    host = origin.split("://", 1)[1].split("/", 1)[0]
    t = time.time()
    try:
        c = http.client.HTTPConnection("127.0.0.1", port, timeout=timeout)
        c.request("GET", "/who", headers={"Host": host})
        r = c.getresponse()
        body = r.read()
        ms = round((time.time() - t) * 1000)
        try:
            return ms, r.status, json.loads(body)
        except ValueError:
            return ms, r.status, {"raw": body[:200].decode(errors="replace")}
    except Exception as e:
        return round((time.time() - t) * 1000), None, {"error": str(e)}


def people_here(w):
    return sorted(d.name for d in Path(w).iterdir() if (d / ".svrngs" / "api").exists())


def probe(a):
    """Each computer here: its own /v1/lends (its row of the matrix) and
    /v1/rail/people; then N visits to others' lends, one on another
    machine when there is one, each checking the greeting's owner and that
    the door named this computer's person."""
    machine_of = {p["name"]: p["machine"] for p in json.loads(Path(a.plan).read_text())["people"]}
    rng = random.Random(a.seed)
    rows = []
    for who in people_here(a.w):
        base = api_base(a.w, who)
        ms, status, body = get(base, f"/v1/lends?namespace={a.gid}", timeout=15)
        row = {"who": who, "t": now(), "lends_ms": ms, "status": status}
        lent = body.get("lent", []) if status == 200 else []
        if status == 200:
            row.update(lent=sorted(f"{l.get('person')}/{l.get('name')}" for l in lent),
                       **{k: body.get(k) for k in ("unanswered", "unheard", "refused", "older")})
        else:
            row["error"] = body
        _, pstatus, people = get(base, f"/v1/rail/people?namespace={a.gid}")
        row["people"] = sorted({r["person"] for r in people.get("people") or []}) if pstatus == 200 else None
        if a.visits:
            others = [l for l in lent if l.get("person") != who and l.get("origin")]
            far = [l for l in others if machine_of.get(l.get("person")) not in (None, machine_of.get(who))]
            picked = rng.sample(far, 1) if far else []
            rest = [l for l in others if l not in picked]
            picked += rng.sample(rest, min(len(rest), max(0, a.visits - len(picked))))
            port = int(base.rsplit(":", 1)[1])
            row["visits"] = []
            for l in picked:
                vms, vstatus, got = visit(port, l["origin"])
                said = (got.get("headers") or {}).get("x-svrngs-person")
                row["visits"].append({"to": l.get("person"), "far": l in far, "ms": vms, "status": vstatus,
                                      "owner": got.get("owner"), "person_said": said,
                                      "right": vstatus == 200 and said == who and got.get("owner") == l.get("person"),
                                      "headers": got.get("headers"), "error": got.get("error") or got.get("raw")})
        rows.append(row)
    print(json.dumps({"t": now(), "rows": rows}))


def paths(a):
    """iroh's path::selected events in each computer's log: to whom (iroh's
    short id), over which network path, as iroh names it."""
    # The path as iroh-1.0.2 writes it (src/socket/transports.rs:1126-1137):
    # `Ip(<local>-><remote>)` or `Relay(<url>, <short id>)`.
    pat = re.compile(r"remote=(\S+).*?network_path=(.+?)(?: prev_network_path=|$)")
    ansi = re.compile(r"\x1b\[[0-9;]*m")
    rows = []
    for log in sorted(Path(a.w).glob("*.log")):
        for line in log.read_text(errors="replace").splitlines():
            line = ansi.sub("", line)
            if "path::selected" in line and (m := pat.search(line)):
                rows.append({"who": log.stem, "remote": m[1].strip(","), "path": m[2].strip().strip(",")})
    print(json.dumps({"rows": rows}))


def logs(a):
    """Every computer's log and output, the link's token taken out, as one
    .tar.gz on stdout."""
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        for f in sorted(list(Path(a.w).glob("*.log")) + list(Path(a.w).glob("*.out")) + list(Path(a.w).glob("*.jsonl"))):
            text = TOKEN.sub("svrngs://join/<the token>", f.read_text(errors="replace")).encode()
            info = tarfile.TarInfo(f.name)
            info.size, info.mtime = len(text), int(f.stat().st_mtime)
            tar.addfile(info, io.BytesIO(text))
    sys.stdout.buffer.write(buf.getvalue())


def stop(a):
    w = Path(a.w)
    said = {}
    for pidfile in sorted(w.glob("keeper-*.pid")):
        role = pidfile.stem.split("-", 1)[1]
        try:
            pid = int(pidfile.read_text())
            os.kill(pid, signal.SIGTERM)
        except (OSError, ValueError):
            said[role] = "not running"
            continue
        end = time.time() + 30
        while time.time() < end:
            try:
                os.kill(pid, 0)
                time.sleep(0.2)
            except ProcessLookupError:
                break
        said[role] = "stopped"
    killed = []
    for kids in w.glob("children-*.json"):
        try:
            for who, pid in json.loads(kids.read_text()).items():
                try:
                    os.kill(pid, signal.SIGKILL)
                    killed.append(who)
                except ProcessLookupError:
                    pass
        except (OSError, ValueError):
            pass
    print(json.dumps({"keepers": said, "killed_after": killed}))


def clean(a):
    w = Path(a.w)
    if not (w / MARK).exists():
        sys.exit(f"{w} holds no {MARK}: not a fleet's work dir, so nothing was removed")
    shutil.rmtree(w)
    print(json.dumps({"removed": str(w)}))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("verb", choices=["found", "start", "sampler", "wall", "probe", "paths", "logs", "stop", "clean", "clock"])
    for flag in ("--bin", "--w", "--link", "--gid", "--plan", "--machine", "--group", "--who"):
        ap.add_argument(flag)
    ap.add_argument("--visits", type=int, default=3)
    ap.add_argument("--seed", type=int, default=25)
    ap.add_argument("--secs", type=float, default=10)
    ap.add_argument("--expect", type=int)
    ap.add_argument("--ceiling", type=float)
    ap.add_argument("--linger", type=float)
    ap.add_argument("--every", type=float, default=0.0)
    a = ap.parse_args()
    if a.verb == "clock":
        print(json.dumps({"t": time.time()}))
        return
    {"found": found, "start": start, "sampler": sampler, "wall": wall, "probe": probe, "paths": paths,
     "logs": logs, "stop": stop, "clean": clean}[a.verb](a)


if __name__ == "__main__":
    main()
