#!/usr/bin/env python3
"""u7-removal (f)'s run (README here): Sam, Ana and Bo, three svrngs data dirs
on this host through iroh's public relays, and Cy's phone (phone.mjs), in one
group, Trip; Sam and Ana each lend a page; the three computers share a folder,
Trip. Ana asks to remove Bo, then Sam removes Bo, and each lend, Bo's computer,
the folder, Bo's own removal, Bo's home page and Cy's phone are read, each
phase as the README orders them. Data in OUT (else run/); W is the scratch
dir (else /tmp/u7-run). svrngs and the page are built first. scripts/check.sh
runs none of this."""
import json, os, re, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / os.environ.get("OUT", "run"); W = Path(os.environ.get("W", "/tmp/u7-run")); B = REPO / "target/debug/svrngs"
LOG = "info,door=debug,removal=debug,node=debug,origin=debug,folder=info,transport=warn"
nodes, servers, PHASE, T0 = {}, [], ["setup"], [None]


def say(event, **kw):
    line = {"t": round(time.time(), 3), "phase": PHASE[0], "event": event, **kw}
    if T0[0] is not None: line["since_cut_s"] = round(line["t"] - T0[0], 3)
    print(json.dumps(line), flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(json.dumps(line) + "\n")


def phase(name): PHASE[0] = name; say("phase")
def data(who): return W / "data" / who
def disk(who): return W / "disk" / who / "Trip"
def env(who): return {**os.environ, "SVRNGS_DIR": str(data(who)), "NO_COLOR": "1", "RUST_LOG": LOG}


def up(who, *args):
    (data(who) / "api").unlink(missing_ok=True)
    nodes[who] = subprocess.Popen([B, *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(data(who) / "out", "a"), stderr=open(W / f"{who}.log", "a"))
    if not wait(lambda: api(who, "/v1/groups"), 60): sys.exit(f"{who}'s node did not come up")
    say("up", who=who, args=[a if "join/" not in a else "<the link>" for a in args])


def command(who, *args):
    """`svrngs <args>` as who types it beside their running node: its words, exit and time."""
    t = time.time()
    run = subprocess.run([B, *args], env={**env(who), "RUST_LOG": "error"}, capture_output=True, text=True,
                         stdin=subprocess.DEVNULL)
    out = {"who": who, "args": list(args), "exit": run.returncode, "said": (run.stdout + run.stderr).strip(),
           "ms": round((time.time() - t) * 1000)}
    say("command", **out)
    return out


def get(url, host=None, body=None):
    """(status, body as text or JSON) of a request to 127.0.0.1, or (None, why)."""
    req = urllib.request.Request(url, data=body and json.dumps(body).encode(),
                                 headers={"content-type": "application/json", **({"Host": host} if host else {})})
    try:
        with urllib.request.urlopen(req, timeout=10) as r: status, blob = r.status, r.read()
    except urllib.error.HTTPError as e: status, blob = e.code, e.read()
    except Exception as e: return None, str(e)
    try: return status, json.loads(blob)
    except Exception: return status, blob.decode(errors="replace")[:300]


def base(who): return (data(who) / "api").read_text().strip().rstrip("/")


def api(who, path):
    try:
        status, body = get(base(who) + path)
        return body if status == 200 else None
    except Exception: return None


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        if cond(): return round(time.time() - t0, 2)
        time.sleep(every)
    return None


def lend(who, name, words):
    """A plain file server with one page, lent by who as `name`."""
    site = W / "sites" / name; site.mkdir(parents=True, exist_ok=True)
    (site / "index.html").write_text(f"<!doctype html><title>{name}</title><p>{words}</p>\n")
    said = W / f"{name}-server.log"
    servers.append(subprocess.Popen([sys.executable, "-u", "-m", "http.server", "0", "--bind", "127.0.0.1",
                                     "--directory", str(site)], stdout=open(said, "w"), stderr=subprocess.STDOUT))
    wait(lambda: re.search(r"port (\d+)", said.read_text()), 10)
    port = int(re.search(r"port (\d+)", said.read_text())[1])
    command(who, "share", str(port), "--name", name, "--kind", name, "--limit", "8")


def lends_for(who):
    """Each lend who's computer lists, by name: its origin there."""
    body = api(who, f"/v1/lends?namespace={GROUP}") or {}
    return {l["name"]: l["origin"] for l in body.get("lent", [])}


def ask_lend(who, origin):
    """who's computer asks the lend at origin, as a program on it: (status, kind, words)."""
    host = origin.split("//")[1].rstrip("/")
    port = host.rsplit(":", 1)[1]
    status, body = get(f"http://127.0.0.1:{port}/", host=host)
    if isinstance(body, dict): return status, body.get("kind"), body.get("error")
    return status, None, (body or "")[:120]


def holds(who, name): return (disk(who) / name).is_file()


def write(who, name):
    blob = os.urandom(1 << 20); (disk(who) / name).write_bytes(blob)
    say("wrote", who=who, path=name, size=len(blob))


class Phone:
    def __init__(self, site):
        self.p = subprocess.Popen(["node", str(HERE / "phone.mjs"), str(site)], cwd=REPO, stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=open(W / "phone.err", "a"), text=True)
        self.origin = json.loads(self.p.stdout.readline())["origin"]

    def do(self, **cmd):
        self.p.stdin.write(json.dumps(cmd) + "\n"); self.p.stdin.flush()
        line = self.p.stdout.readline()
        out = json.loads(line) if line else {"stopped": "phone.mjs printed nothing"}
        say("phone", **{k: v for k, v in out.items() if k != "t"})
        return out

    def quit(self):
        try: self.p.stdin.write('{"do":"quit"}\n'); self.p.stdin.flush(); self.p.wait(30)
        except Exception: self.p.kill()


def poll_lends(origins, bound):
    """Bo's computer asks each lend every 200 ms for `bound` s: each status change, and the first `removed`."""
    first, last = {}, {}
    def one(name, origin):
        while time.time() - T0[0] < bound:
            status, kind, words = ask_lend("bo", origin)
            if (status, kind) != last.get(name):
                last[name] = (status, kind); say("lend", who="bo", lend=name, status=status, kind=kind, words=words)
            if kind == "removed" and name not in first: first[name] = round(time.time() - T0[0], 3)
            time.sleep(0.2)
    threads = [threading.Thread(target=one, args=(n, o)) for n, o in origins.items()]
    [t.start() for t in threads]; [t.join() for t in threads]
    return first


def acts_of(who_dir, actor):
    """Each act in who's journal signed by actor: (seq, op)."""
    out = []
    for log in (who_dir / "rings").rglob("ring_oplog.jsonl"):
        for line in log.read_text().splitlines():
            if f'"actor":"{actor}"' in line:
                m = re.search(r'"seq":(\d+)', line); op = re.search(r'"op":"(\w+)"', line)
                out.append((int(m[1]) if m else None, op[1] if op else None))
    return sorted(out, key=lambda a: a[0] or 0)


def main():
    global GROUP
    subprocess.run(["cargo", "build", "-q", "-p", "svrngs"], cwd=REPO, check=True)
    subprocess.run(["rm", "-rf", str(W), str(OUT)]); OUT.mkdir(parents=True)
    for w in ("sam", "ana", "bo"): data(w).mkdir(parents=True); disk(w).mkdir(parents=True)
    subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
    say("load", uptime=subprocess.run(["uptime"], capture_output=True, text=True).stdout.strip(),
        head=subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip())
    phone = Phone(W / "site")
    summary = {}
    try:
        # 1. Setup.
        up("sam", "new", "Trip", "--as", "Sam", "--origin", phone.origin)
        wait(lambda: "svrngs://join/" in (data("sam") / "out").read_text(), 60)
        said = (data("sam") / "out").read_text()
        link = re.search(r"svrngs://join/\S+", said)[0]
        page_link = re.search(re.escape(phone.origin) + r"#\S+", said)[0]
        GROUP = api("sam", "/v1/groups")["groups"][0]["id"]; say("group", id=GROUP)
        for w in ("ana", "bo"):
            up(w, "join", link, "--as", w.title())
            if wait(lambda: "You're in" in (data(w) / "out").read_text(), 60) is None: sys.exit(f"{w} did not join")
        lend("sam", "films", "Sam's films"); lend("ana", "music", "Ana's music")
        for w in ("sam", "ana", "bo"): command(w, "share", "--folder", str(disk(w)), "--group", "Trip")
        phone.do(do="join", link=page_link, name="Cy")
        # Bo's computer opens both lends: each computer learns the others at its rounds.
        t = wait(lambda: len(o := lends_for("bo")) == 2 and all(ask_lend("bo", u)[0] == 200 for u in o.values()), 240, 2)
        origins = lends_for("bo"); say("bo-opens-both", within_s=t, origins=origins)
        if t is None: sys.exit("Bo's computer never opened both lends")
        write("sam", "before.bin")
        t = wait(lambda: holds("ana", "before.bin") and holds("bo", "before.bin"), 120)
        say("before-everywhere", within_s=t)
        summary["setup"] = {"bo_opens_both": origins, "before_everywhere_s": t,
                            "cy_before": phone.do(do="open", lend="films")}

        # 2. Ana asks.
        phase("ask")
        summary["ask"] = command("ana", "remove", "Bo", "--group", "Trip")
        seen = []
        t_end = time.time() + 10
        while time.time() < t_end:
            seen += [(n, ask_lend("bo", o)[0]) for n, o in origins.items()]; time.sleep(0.5)
        summary["ask"]["bo_after"] = sorted({f"{n} {s}" for n, s in seen})
        say("after-ask", seen=summary["ask"]["bo_after"])

        # 3. Sam removes Bo.
        phase("cut")
        T0[0] = time.time(); say("zero")
        groups = {}
        def told():
            body = api("bo", "/v1/groups") or {}
            g = next((g for g in body.get("groups", []) if g["id"] == GROUP), None)
            if g and g.get("removed") and "s" not in groups: groups.update(s=round(time.time() - T0[0], 3), body=g)
            return "s" in groups
        watcher = threading.Thread(target=lambda: wait(told, 90))
        watcher.start()
        cut = command("sam", "remove", "Bo", "--group", "Trip")
        refused = poll_lends(origins, 90)
        watcher.join()
        summary["cut"] = {"command": cut, "refused_removed_s": refused, "bo_told": groups}
        say("cut-measured", **summary["cut"])

        # 4. The folder.
        phase("folder")
        before_bo = sorted(p.name for p in disk("bo").iterdir())
        w0 = time.time(); write("sam", "after.bin")
        t_ana = wait(lambda: holds("ana", "after.bin"), 60)
        say("after-at-ana", within_s=t_ana, since_cut=round(time.time() - T0[0], 2))
        time.sleep(45)
        summary["folder"] = {"ana_s": t_ana, "written_since_cut_s": round(w0 - T0[0], 2),
                             "bo_holds_after": holds("bo", "after.bin"), "bo_before": before_bo,
                             "bo_now": sorted(p.name for p in disk("bo").iterdir()),
                             "bo_keeps_before": holds("bo", "before.bin")}
        say("folder-measured", **summary["folder"])

        # 5. Bo's own removal.
        phase("bo-removes")
        # Bo's key, from Sam's journal: the Admit naming Bo.
        bo_key = next(re.search(r'"key":"([0-9a-f]{64})"', l)[1] for j in (data("sam") / "rings").rglob("ring_oplog.jsonl")
                      for l in j.read_text().splitlines() if '"op":"admit"' in l and '"person":"Bo"' in l)
        acts_before = acts_of(data("bo"), bo_key)
        tries = [command("bo", "remove", "Sam", "--group", "Trip"), command("bo", "remove", "Ana", "--group", "Trip")]
        time.sleep(5)
        summary["bo_removes"] = {
            "tries": tries, "bo_key": bo_key,
            "bo_acts_before": acts_before, "bo_acts_after": acts_of(data("bo"), bo_key),
            "standing": {w: sorted(p["person"] for p in (api(w, f"/v1/rail/people?namespace={GROUP}") or {}).get("people", []))
                         for w in ("sam", "ana")},
        }
        say("bo-removes-measured", **summary["bo_removes"])

        # 6. What each sees.
        phase("seen")
        home = f"http://svrngs.localhost:{base('bo').rsplit(':', 1)[1]}/"
        run = subprocess.run(["node", "web/tests/home.mjs", json.dumps({"engine": "chromium", "ask": home, "read": "#home section p"})],
                             cwd=REPO, capture_output=True, text=True)
        try: bo_home = json.loads(run.stdout.strip().splitlines()[-1])
        except Exception: bo_home = {"stopped": run.stderr[-500:]}
        say("bo-home", **bo_home)
        summary["seen"] = {"bo_home": bo_home, "cy_after": phone.do(do="open", lend="films"),
                           "cy_music": phone.do(do="open", lend="music")}
    finally:
        (OUT / "summary.json").write_text(json.dumps(summary, indent=1) + "\n")
        phone.quit()
        for w, p in list(nodes.items()): p.send_signal(signal.SIGTERM); p.wait()
        for s in servers: s.kill()
        for w in ("sam", "ana", "bo"):
            keep = [l for l in (W / f"{w}.log").read_text().splitlines()
                    if re.search(r"removal|refused|removed|cut|folder: |round membership", l)]
            (OUT / f"{w}.log").write_text("\n".join(keep) + "\n")
            (OUT / f"{w}.out").write_text((data(w) / "out").read_text())


if __name__ == "__main__":
    main()
