#!/usr/bin/env python3
"""relay-fallbacks' run (PREDICTIONS.md here): a group whose keeper names two
`svrngs relay --at` relays on 127.0.0.1, A and B, beside the build's, n0's,
with three computers on the build as committed, each in a HOME of its own,
under no-direct.sb so they reach each other only through a relay. Ana lends
a page. Then: the relay Ana is homed at is stopped and started again; then
both are stopped and started again. Each lend fetch, each computer's home
and tier, from its trace, are timed from each stop and start.

    BIN=<svrngs built at the commit> W=<scratch, absent> python3 demo/runs/relay-fallbacks/2026-11-02/run.py

Data in data/ beside this file, the group's links never in it.
scripts/check.sh runs none of this."""
import datetime, gzip, hashlib, json, os, re, signal, socket, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "data"; W = Path(os.environ["W"]); BIN = os.environ["BIN"]; SB = HERE / "no-direct.sb"
GROUP = "house"
LOG = "info,cli=debug,tiers=debug,transport=debug,join=debug,group=debug,origin=debug,door=debug,iroh::net_report=debug"
procs, pids, summary, hidden, started = {}, {}, {}, [], {}
lock = threading.Lock()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}))
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def wait(cond, bound, every=0.25):
    t0 = time.time()
    while time.time() - t0 < bound:
        try: got = cond()
        except Exception: got = None
        if got: return got
        time.sleep(every)
    return None


def boxed(cmd): return ["sandbox-exec", "-f", str(SB), *map(str, cmd)]


def free_port():
    s = socket.socket(); s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]; s.close(); return p


def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def home(who): return W / who
def data(who): return home(who) / ".svrngs"
def env_of(who, log=LOG):
    e = {k: v for k, v in os.environ.items() if k not in ("SVRNGS_DIR", "RUST_LOG")}
    return {**e, "HOME": str(home(who)), "USER": who, "SVRNGS_DIR": str(data(who)), "NO_COLOR": "1", "RUST_LOG": log}


def log_of(who): return W / "logs" / f"{who}-{started[who]}.log"


def start(who, *args, log=LOG):
    home(who).mkdir(parents=True, exist_ok=True); (W / "logs").mkdir(exist_ok=True)
    n = started[who] = started.get(who, 0) + 1
    out, err = W / "logs" / f"{who}-{n}.out", W / "logs" / f"{who}-{n}.log"
    p = subprocess.Popen(boxed([BIN, *args]), env=env_of(who, log), stdin=subprocess.DEVNULL,
                         stdout=open(out, "w"), stderr=open(err, "w"))
    with lock: procs[who] = p; pids[p.pid] = who
    say("started", who=who, n=n, args=[redact(a) for a in args], pid=p.pid)
    return time.time(), out


def stop(who):
    p = procs.pop(who, None)
    if not p or p.poll() is not None: return None
    t = time.time(); p.send_signal(signal.SIGINT)
    try: p.wait(20); killed = False
    except subprocess.TimeoutExpired: p.kill(); p.wait(); killed = True
    with lock: pids.pop(p.pid, None)
    r = {"who": who, "exit": p.returncode, "ms": round((time.time() - t) * 1000), "killed": killed, "t": round(t, 3)}
    say("stopped", **r)
    return r


def http(url, headers=None, timeout=15):
    req = urllib.request.Request(url, headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: return r.status, r.read().decode(errors="replace")
    except urllib.error.HTTPError as e: return e.code, e.read().decode(errors="replace")
    except Exception as e: return None, str(e)


def base(who): return (data(who) / "api").read_text().strip().rstrip("/")


def api(who, path):
    try: token = (data(who) / "token").read_text().strip()
    except Exception: return None
    status, body = http(base(who) + path, {"Authorization": f"Bearer {token}"})
    if status != 200: return None
    try: return json.loads(body)
    except Exception: return None


def gid(who):
    g = api(who, "/v1/groups")
    return g and g.get("groups") and g["groups"][0]["id"]


def listed(who, name):
    r = api(who, f"/v1/lends?namespace={GID}")
    return r and {l.get("name"): l.get("origin") for l in r.get("lent", [])}.get(name)


def fetch(who, origin, words, timeout=15):
    host = re.match(r"https?://([^/]+)/", origin)[1]
    token = (data(who) / "token").read_text().strip()
    status, body = http(f"http://127.0.0.1:{base(who).rsplit(':', 1)[1]}/", {"Host": host, "Authorization": f"Bearer {token}"}, timeout)
    return status == 200 and words in body


def joined(out): return re.search(r"You.re in .*", out.read_text())


def link_in(out):
    m = re.search(r"svrngs://join/([A-Za-z0-9_-]+)", out.read_text())
    if m and m[1] not in hidden: hidden.append(m[1])
    return m and m[0]


def app(name, words):
    d = W / "apps" / name; d.mkdir(parents=True, exist_ok=True)
    (d / "index.html").write_text(f"<!doctype html><title>{name}</title><h1>{words}</h1>\n")
    port = free_port()
    p = subprocess.Popen(boxed(["python3", "-m", "http.server", port, "--bind", "127.0.0.1", "--directory", d]),
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    procs[f"app-{name}"] = p
    with lock: pids[p.pid] = f"app-{name}"
    wait(lambda: http(f"http://127.0.0.1:{port}/", timeout=2)[0] == 200, 10)
    return port


def share(who, port, name):
    t = time.time()
    r = subprocess.run(boxed([BIN, "share", port, "--kind", "page", "--name", name]),
                       env=env_of(who, "cli=debug"), input="\n", capture_output=True, text=True, timeout=60)
    out = {"who": who, "exit": r.returncode, "ms": round((time.time() - t) * 1000), "said": r.stdout.strip()[-300:]}
    say("share", **out)
    return out


# --- the instrument: no direct path, checked against plants -----------------
def instrument(control_port):
    send = ("import socket,sys\nh=sys.argv[1];f=socket.AF_INET6 if ':' in h else socket.AF_INET\n"
            "try:\n socket.socket(f,socket.SOCK_DGRAM).sendto(b'x',(h,9));print('sent')\nexcept OSError as e:print(repr(e))")
    own = ["127.0.0.1", "::1"] + re.findall(r"inet6? ((?:\d+\.){3}\d+|[0-9a-f:]+) ", subprocess.run(["ifconfig"], capture_output=True, text=True).stdout)
    own = sorted({a for a in own if not a.startswith("fe80")})
    probes = {f"udp to {a}": subprocess.run(boxed(["python3", "-I", "-c", send, a]), capture_output=True, text=True).stdout.strip() for a in own}
    probes["udp to 1.1.1.1 (control)"] = subprocess.run(boxed(["python3", "-I", "-c", send, "1.1.1.1"]), capture_output=True, text=True).stdout.strip()
    probes["tcp to loopback (control)"] = str(subprocess.run(boxed(["curl", "-sS", "-m5", "-o", "/dev/null", "-w", "%{http_code}", f"http://127.0.0.1:{control_port}/"]), capture_output=True, text=True).stdout)
    probes["https to n0 (control)"] = str(subprocess.run(boxed(["curl", "-sS", "-m10", "-o", "/dev/null", "-w", "%{http_code}", "https://usw1-1.relay.n0.iroh.link/ping"]), capture_output=True, text=True).stdout)
    held = {k: (v == "sent" if "control" in k and k.startswith("udp") else v.startswith("200") if "control" in k else "not permitted" in v) for k, v in probes.items()}
    summary["instrument"] = {"probes": probes, "held": held}
    say("instrument", **summary["instrument"])
    if not all(held.values()): sys.exit(f"the instrument does not hold: {held}")


# --- reading the traces -----------------------------------------------------
ANSI = re.compile(r"\x1b\[[0-9;]*m")
STAMP = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+)Z")


def at(line):
    m = STAMP.match(line)
    return m and datetime.datetime.strptime(m[1][:26], "%Y-%m-%dT%H:%M:%S.%f").replace(tzinfo=datetime.timezone.utc).timestamp()


def lines_of(path): return [ANSI.sub("", l) for l in path.read_text(errors="replace").splitlines()] if path.exists() else []


def homes(who):
    """Each (time, relay) this computer's home moved to (svrngs-node tiers.rs `moved`)."""
    out = []
    for l in lines_of(log_of(who)):
        m = re.search(r"meets its groups at another relay now.*? from=\S+ to=(?:Some\(\"([^\"]+)\"\)|None)", l)
        if m and at(l): out.append((at(l), m[1]))
    return out


def tiers(who):
    out = []
    for l in lines_of(log_of(who)):
        m = re.search(r"meets its groups at another tier now from=(\w+) to=(\w+)", l)
        if m and at(l): out.append((at(l), m[2]))
    return out


def home_now(who):
    h = homes(who)
    return h[-1][1] if h else None


def after(events, since, want=None):
    """ms from `since` to the first of `events` at or after it (that is `want`), else None."""
    for t, v in events:
        if t >= since and (want is None or v == want): return round((t - since) * 1000)
    return None


def watch_lend(who, origin, words, since, bound, every=0.5):
    """ms from `since` of the first fetch on `who`'s computer that works,
    each fetch kept as (ms it began, worked, ms it took)."""
    seen = []
    while time.time() - since < bound:
        t = time.time(); ok = fetch(who, origin, words, timeout=12)
        seen.append((round((t - since) * 1000), ok, round((time.time() - t) * 1000)))
        if ok: break
        time.sleep(every)
    return {"works_ms": next((ms for ms, ok, _ in seen if ok), None), "fetches": seen}


def main():
    global GID
    if W.exists(): sys.exit(f"{W} exists: give the run a scratch directory of its own")
    W.mkdir(parents=True); OUT.mkdir(exist_ok=True)
    root = Path(BIN).parents[2]
    built = subprocess.run(["git", "-C", root, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(["git", "-C", root, "status", "--porcelain", "--untracked-files=no"], capture_output=True, text=True).stdout.strip()
    summary["start"] = {"build": built, "build_tree_clean": not dirty, "bin_sha256": sha(BIN), "load": os.getloadavg()[0], "cores": os.cpu_count()}
    say("start", **summary["start"])
    people = ("sam", "ana", "alex")
    try:
        notes = app("notes", "Ana's notes, through a relay")
        instrument(notes)

        ports = {"A": free_port(), "B": free_port()}
        url = {k: f"http://127.0.0.1:{p}/" for k, p in ports.items()}
        def relay_up(k):
            t, out = start(f"relay{k}", "relay", "--at", f"127.0.0.1:{ports[k]}", "--open", log="relay=debug")
            said = wait(lambda: "Relaying at" in out.read_text(), 15)
            return {"ms": round((time.time() - t) * 1000) if said else None, "t": round(t, 3)}
        summary["relays"] = {k: {"url": url[k], "up": [relay_up(k)]} for k in url}
        say("relays", **summary["relays"])

        t, out = start("sam", "new", GROUP, "--as", "Sam", "--relay", url["A"].rstrip("/"), "--relay", url["B"].rstrip("/"))
        link = wait(lambda: link_in(out), 60)
        summary["sam"] = {"link_ms": round((time.time() - t) * 1000) if link else None}
        say("made", **summary["sam"])
        if not link: sys.exit(f"no link: {redact(out.read_text())[-400:]}")
        GID = wait(lambda: gid("sam"), 30)
        for who, name in (("ana", "Ana"), ("alex", "Alex")):
            t, o = start(who, "join", link, "--as", name)
            said = wait(lambda: joined(o), 60)
            summary[who] = {"joined_ms": round((time.time() - t) * 1000) if said else None}
            say("joined", who=who, **summary[who])

        summary["share"] = share("ana", notes, "notes")
        t = time.time()
        origin = {w: wait(lambda: listed(w, "notes"), 90) for w in ("sam", "alex")}
        summary["listed_ms"] = round((time.time() - t) * 1000)
        if not all(origin.values()): sys.exit(f"notes never listed: {origin}")
        summary["before"] = {w: watch_lend(w, origin[w], "Ana's notes", time.time(), 60)["works_ms"] for w in origin}
        wait(lambda: all(home_now(w) for w in people), 60)
        time.sleep(30)  # settled: past each computer's first ask and iroh's next report (take 2 read Alex's home 2 s after a move)
        summary["homes_before"] = {w: home_now(w) for w in people}
        summary["tiers_before"] = {w: tiers(w) and tiers(w)[-1][1] for w in people}
        say("before", homes=summary["homes_before"], tiers=summary["tiers_before"], works=summary["before"])

        # 1. The relay Ana is homed at stopped, mid-session.
        x = next(k for k in url if url[k] == summary["homes_before"]["ana"])
        y = "B" if x == "A" else "A"
        killed = stop(f"relay{x}")["t"]
        def watch_both():
            got = {}
            ts = [threading.Thread(target=lambda w=w: got.__setitem__(w, watch_lend(w, origin[w], "Ana's notes", killed, 180))) for w in ("sam", "alex")]
            for th in ts: th.start()
            for th in ts: th.join()
            return got
        lend = watch_both()
        wait(lambda: all(home_now(w) == url[y] for w in people), 90 - (time.time() - killed), 1)
        summary["one_stopped"] = {"stopped": x, "lend": lend,
                                  "home_moved_ms": {w: after(homes(w), killed, url[y]) for w in people},
                                  "homes": {w: home_now(w) for w in people},
                                  "tiers": {w: [v for t_, v in tiers(w) if t_ >= killed] for w in people}}
        say("one-stopped", **summary["one_stopped"])

        # 2. It starts again at its address: does home return?
        summary["relays"][x]["up"].append(relay_up(x))
        back = summary["relays"][x]["up"][-1]["t"]
        time.sleep(90)
        summary["one_back"] = {"returned_ms": {w: after(homes(w), back, url[x]) for w in people},
                               "homes": {w: home_now(w) for w in people},
                               "lend_works": {w: fetch(w, origin[w], "Ana's notes") for w in ("sam", "alex")}}
        say("one-back", **summary["one_back"])

        # 3. Both stopped: the build's relay, n0's, takes the group.
        both = time.time()
        summary["both_stopped"] = {"stopped": [stop("relayA"), stop("relayB")]}
        killed = both
        lend = watch_both()
        wait(lambda: all(home_now(w) and "n0.iroh.link" in home_now(w) for w in people), 90 - (time.time() - killed), 1)
        summary["both_stopped"].update({"lend": lend,
                                        "tier_ms": {w: after(tiers(w), killed, "Unanswered") for w in people},
                                        "home_n0_ms": {w: next((round((t_ - killed) * 1000) for t_, v in homes(w) if t_ >= killed and v and "n0.iroh.link" in v), None) for w in people},
                                        "homes": {w: home_now(w) for w in people}})
        say("both-stopped", **summary["both_stopped"])

        # 4. Both start again: the group's own take it back.
        summary["relays"]["A"]["up"].append(relay_up("A")); summary["relays"]["B"]["up"].append(relay_up("B"))
        back = summary["relays"]["B"]["up"][-1]["t"]
        wait(lambda: all(home_now(w) in url.values() for w in people), 90, 1)
        time.sleep(5)
        summary["both_back"] = {"tier_ms": {w: after(tiers(w), back, "Own") for w in people},
                                "home_own_ms": {w: next((round((t_ - back) * 1000) for t_, v in homes(w) if t_ >= back and v in url.values()), None) for w in people},
                                "homes": {w: home_now(w) for w in people},
                                "lend_works": {w: fetch(w, origin[w], "Ana's notes") for w in ("sam", "alex")}}
        say("both-back", **summary["both_back"])
    finally:
        summary["stopped_end"] = [stop(w) for w in list(procs)]
        summary["traces"] = {}
        for f in sorted((W / "logs").glob("*")):
            text = redact(f.read_text(errors="replace"))
            if f.suffix == ".log":
                ls = text.splitlines()
                held = [re.search(r"^(\S+).*?peer=(\w+) alpn=(\w+) known=(\[.*?\])", l) for l in ls if "iroh holding these addresses" in l]
                summary["traces"][f.stem] = {"lines": len(ls), "warn_error": sum(1 for l in ls if " WARN " in l or " ERROR " in l),
                                             "asks": sum(1 for l in ls if "whether any of the groups' own relays answers" in l),
                                             "dials_held": [m.groups() for m in held if m]}
            with gzip.open(OUT / f"{f.name}.gz", "wt") as g: g.write(text)
        summary["end"] = {"load": os.getloadavg()[0], "t": round(time.time(), 3)}
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, default=str)))
        say("end")


main()
