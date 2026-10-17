#!/usr/bin/env python3
"""relay-n0-off's run (PREDICTIONS.md here): a group of three computers and
a phone's page on this host with n0 out of the path. Each computer is the
svrngs command with one relay, iroh's on 127.0.0.1, and n0_services off
(crates/svrngs/tests/n0_off.rs `the_command`); the relay is `the_relay`
there. Sam makes the group; Ana and Alex join by its link; Ana lends a
film; a photo of Ana's goes in the group's folder; Sam's and Alex's home
pages are read; Cy opens the link on a phone and plays Ana's film; then Sam
is stopped and a second photo of Ana's is waited for on Alex's disk. Every
remote address each process opened is sampled with lsof.

    W=<scratch, absent> python3 demo/runs/n0-off/2026-10-12/run.py

Data in data/ beside this file, the group's token never in it.
scripts/check.sh runs none of this."""
import gzip, hashlib, json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "data"; W = Path(os.environ["W"])
GROUP = "house"
LOG = ("info,cli=debug,transport=debug,folder=debug,door=debug,node=debug,rails=debug,redeem=debug,"
       "surface=debug,home=debug,lend=debug,origin=debug,svrngs_node=debug,svrngs=debug,iroh=debug,iroh_relay=debug")
N0_HOSTS = ["use1-1.relay.n0.iroh.link", "usw1-1.relay.n0.iroh.link", "euc1-1.relay.n0.iroh.link",
            "aps1-1.relay.n0.iroh.link", "use1-1.staging-relay.n0.iroh.link", "euc1-1.staging-relay.n0.iroh.link",
            "dns.iroh.link", "staging-dns.iroh.link"]
procs, pids, summary, hidden = {}, {}, {}, []
lock = threading.Lock()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}))
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(url, timeout=10):
    try:
        with urllib.request.urlopen(url, timeout=timeout) as r: return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e: return e.code, e.read().decode(errors="replace")
    except Exception as e: return None, str(e)


def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()


# --- each computer: a HOME of its own, the command with n0 off ---------------
def home(who): return W / who
def data(who): return home(who) / ".svrngs"
def folder(who): return home(who) / "svrngs" / GROUP
def base(who): return (data(who) / "api").read_text().strip().rstrip("/")
def home_page(who): return f"http://svrngs.localhost:{base(who).rsplit(':', 1)[1]}/"


def api(who, path):
    try: status, body = http(base(who) + path)
    except Exception: return None
    return body if status == 200 else None


def command(who, *args, stdin="", stay=False):
    """`svrngs <args>` as `who` types it; with `stay`, kept running."""
    env = {**{k: v for k, v in os.environ.items() if k != "SVRNGS_DIR"}, "HOME": str(home(who)), "USER": who,
           "NO_COLOR": "1", "N0OFF_RELAY": RELAY, "N0OFF_ARGS": "\n".join(args), "RUST_LOG": LOG}
    cmd = [BIN, "--ignored", "--exact", "the_command", "--nocapture"]
    if stay:
        procs[who] = subprocess.Popen(cmd, env=env, stdin=subprocess.DEVNULL, stdout=open(W / f"{who}.out", "w"),
                                      stderr=open(W / f"{who}.log", "w"))
        pids[procs[who].pid] = who
        say("started", who=who, args=list(args), pid=procs[who].pid)
        return procs[who]
    t = time.time()
    run = subprocess.run(cmd, env={**env, "RUST_LOG": "error"}, input=stdin, capture_output=True, text=True)
    said = "\n".join(l for l in (run.stdout + run.stderr).splitlines()
                     if not re.match(r"^(running \d+ test|test result:|$)", l))
    out = {"who": who, "args": list(args), "exit": run.returncode, "said": said.strip(), "ms": round((time.time() - t) * 1000)}
    say("command", **out)
    return out


def browser(mode, args, timeout=400):
    p = subprocess.Popen(["node", str(HERE / "n0.mjs"), mode, json.dumps(args)], cwd=REPO,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    with lock: pids[p.pid] = f"browser-{mode}"
    try: stdout, stderr = p.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        p.kill(); stdout, stderr = p.communicate()
    with lock: pids.pop(p.pid, None)
    try: return json.loads(stdout.strip().splitlines()[-1])
    except Exception: return {"stopped": f"n0.mjs printed nothing: {stderr[-400:]}"}


# --- every remote address each process opened -------------------------------
def descendants(pid):
    r = subprocess.run(["pgrep", "-P", str(pid)], capture_output=True, text=True)
    kids = [int(k) for k in r.stdout.split()]
    return kids + [d for k in kids for d in descendants(k)]


def sampler(stop):
    seen = {}
    while not stop.is_set():
        with lock: watched = dict(pids)
        for pid, who in list(watched.items()):
            if who.startswith("browser"):
                for d in descendants(pid): watched[d] = who
        if watched:
            r = subprocess.run(["lsof", "-i", "-n", "-P", "-a", "-p", ",".join(map(str, watched))],
                               capture_output=True, text=True)
            for line in r.stdout.splitlines()[1:]:
                f = line.split()
                if len(f) < 9: continue
                name = " ".join(f[8:])
                remote = name.split("->", 1)[1].split()[0] if "->" in name else None
                key = (watched.get(int(f[1]), "?"), f[0], f[7], remote or name.split()[0])
                if key not in seen:
                    seen[key] = time.time()
                    with open(OUT / "lsof.jsonl", "a") as out:
                        out.write(json.dumps({"t": round(time.time(), 3), "who": key[0], "command": key[1], "pid": int(f[1]),
                                              "proto": key[2], "remote": remote, "local": None if remote else key[3]}) + "\n")
        stop.wait(2)


def n0_addresses():
    found = {}
    for host in N0_HOSTS:
        ips = []
        for kind in ("A", "AAAA"):
            r = subprocess.run(["dig", "+short", kind, host], capture_output=True, text=True)
            ips += [l for l in r.stdout.split() if re.match(r"^[0-9a-f.:]+$", l) and (":" in l or l.count(".") == 3)]
        found[host] = ips
    return found


def stop_all():
    for who, p in procs.items():
        try: p.send_signal(signal.SIGCONT); p.terminate(); p.wait(20)
        except Exception as e: say("stop", who=who, error=str(e))


def main():
    global BIN, RELAY
    if W.exists(): sys.exit(f"{W} exists: give the run a scratch directory of its own")
    W.mkdir(parents=True); OUT.mkdir(exist_ok=True)
    head = subprocess.run(["git", "-C", REPO, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("start", build=head, load=os.getloadavg()[0], cores=os.cpu_count())
    stop = threading.Event()
    try:
        # 0. The harness, the page, the film, n0's addresses, the relay.
        built = subprocess.run(["cargo", "test", "-p", "svrngs", "--test", "n0_off", "--no-run", "--message-format=json"],
                               cwd=REPO, capture_output=True, text=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        BIN = next(json.loads(l)["executable"] for l in built.stdout.splitlines()
                   if '"executable"' in l and json.loads(l).get("executable") and "n0_off" in json.loads(l)["executable"])
        web = subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], capture_output=True, text=True)
        film = W / "film.mp4"
        mk = subprocess.run(["ffmpeg", "-loglevel", "error", "-f", "lavfi", "-i", "testsrc2=size=1280x720:rate=30",
                             "-f", "lavfi", "-i", "anoisesrc=d=30", "-t", "30", "-c:v", "libx264", "-pix_fmt", "yuv420p",
                             "-b:v", "4M", "-c:a", "aac", "-movflags", "+faststart", film], capture_output=True, text=True)
        summary["n0"] = n0_addresses()
        say("setup", build_web=web.returncode, film=mk.returncode, film_bytes=film.stat().st_size if film.exists() else None,
            n0=summary["n0"], harness=Path(BIN).name)
        if web.returncode or mk.returncode: sys.exit(f"setup failed: {web.stderr[-400:]} {mk.stderr[-400:]}")
        threading.Thread(target=sampler, args=(stop,), daemon=True).start()
        at = W / "relay-at"
        procs["relay"] = subprocess.Popen([BIN, "--ignored", "--exact", "the_relay", "--nocapture"],
                                          env={**os.environ, "N0OFF_RELAY_AT": str(at), "RUST_LOG": "iroh_relay=debug"},
                                          stdout=open(W / "relay.log", "w"), stderr=subprocess.STDOUT)
        pids[procs["relay"].pid] = "relay"
        RELAY = wait(lambda: at.exists() and at.read_text(), 30)
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        pids[procs["page"].pid] = "page"
        PAGE = f"http://localhost:{procs['page'].stdout.readline().split()[1]}/"
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        procs["film"] = subprocess.Popen(["node", HERE / "film.mjs", film], stdout=subprocess.PIPE, text=True)
        pids[procs["film"].pid] = "film"
        FILM_PORT = procs["film"].stdout.readline().split()[1]
        film_log = []
        threading.Thread(target=lambda: [film_log.append((round(time.time(), 3), l.strip())) for l in procs["film"].stdout],
                         daemon=True).start()
        say("served", relay=RELAY, page=PAGE, film_port=FILM_PORT)

        # 1. Sam makes the group; Ana and Alex join by its link.
        for who in ("sam", "ana", "alex"): home(who).mkdir()
        t = time.time()
        command("sam", "new", GROUP, "--as", "Sam", "--origin", PAGE, stay=True)
        m = wait(lambda: re.search(re.escape(PAGE) + r"#([A-Za-z0-9_-]+)", (W / "sam.out").read_text()), 90)
        if not m: sys.exit(f"Sam's new printed no link: {(W / 'sam.out').read_text()[-400:]} {(W / 'sam.log').read_text()[-800:]}")
        LINK = m[0]; hidden.append(m[1])
        client = re.search(r"svrngs://\S+", (W / "sam.out").read_text())[0]
        hidden.extend(re.findall(r"svrngs://join/([A-Za-z0-9_-]+)", client))
        summary["sam"] = {"link_ms": round((time.time() - t) * 1000)}
        wait(lambda: api("sam", "/v1/groups"), 60)
        GROUP_ID = api("sam", "/v1/groups")["groups"][0]["id"]
        summary["sam"]["folder"] = folder("sam").is_dir()
        say("sam", group=GROUP_ID, **summary["sam"])
        for who in ("ana", "alex"):
            t = time.time()
            command(who, "join", client, "--as", who, stay=True)
            said = wait(lambda: re.search(r"You're in .*", (W / f"{who}.out").read_text()), 60)
            summary[who] = {"joined_ms": round((time.time() - t) * 1000) if said else None, "said": said and said[0],
                            "folder": wait(lambda: folder(who).is_dir(), 10) or False}
            say("joined", who=who, **summary[who])
        t_in = time.time()

        # 2. Ana lends the film.
        summary["ana"]["share"] = command("ana", "share", FILM_PORT, "--kind", "film", "--name", "film", stdin="\n")
        t_share = time.time()

        # 3. A photo of Ana's in the group's folder: on Sam's disk, then Alex's.
        photo = folder("ana") / "trip-1.jpg"
        photo.write_bytes(os.urandom(2_000_000)); want = sha(photo); t = time.time()
        got = {}
        def arrived(who): return (folder(who) / photo.name).is_file() and sha(folder(who) / photo.name) == want
        for who in ("sam", "alex"):
            ok = wait(lambda: arrived(who), 150 - (time.time() - t), 0.25)
            got[who] = round((time.time() - t) * 1000) if ok else None
        summary["photo_1"] = got; say("photo-1", **got)

        # 4. The lends on Sam's home page and Alex's.
        for who in ("sam", "alex"):
            r = browser("home", {"url": home_page(who), "until": ["film"], "or": ["did not answer"], "within": 90_000,
                                 "t0": round(t_share * 1000)})
            summary.setdefault("home", {})[who] = r; say("home", who=who, at=r.get("at"), said=r.get("said"))

        # 5. Cy on a phone: the link, a name, the lends, Ana's film.
        r = browser("phone", {"link": LINK, "name": "Cy", "lend": "film", "t0": round(time.time() * 1000)})
        summary["phone"] = r; say("phone", **{k: v for k, v in r.items() if k != "logs"})
        summary["film_answers"] = len(film_log)

        # 6. Sam stopped: a second photo of Ana's, waited for on Alex's disk.
        procs["sam"].send_signal(signal.SIGSTOP); t = time.time()
        photo2 = folder("ana") / "trip-2.jpg"
        photo2.write_bytes(os.urandom(2_000_000)); want = sha(photo2)
        ok = wait(lambda: (folder("alex") / photo2.name).is_file() and sha(folder("alex") / photo2.name) == want, 150, 0.5)
        summary["photo_2_sam_stopped"] = {"alex_ms": round((time.time() - t) * 1000) if ok else None, "waited_s": 150}
        procs["sam"].send_signal(signal.SIGCONT); t = time.time()
        ok = ok or wait(lambda: (folder("alex") / photo2.name).is_file() and sha(folder("alex") / photo2.name) == want, 150, 0.5)
        summary["photo_2_sam_again"] = {"alex_ms": round((time.time() - t) * 1000) if ok else None}
        say("photo-2", stopped=summary["photo_2_sam_stopped"], again=summary["photo_2_sam_again"])

        # Who is who: the keys the record holds, and each computer's own.
        summary["people"] = api("sam", f"/v1/rail/people?namespace={GROUP_ID}")
        summary["film_log"] = film_log
        summary["since_in_s"] = round(time.time() - t_in)
    finally:
        stop_all(); stop.set(); time.sleep(2.5)
        for who in ("relay", "sam", "ana", "alex"):
            for suffix in ("log", "out"):
                src = W / f"{who}.{suffix}"
                if src.exists():
                    with open(src, "rb") as f, gzip.open(OUT / f"{who}.{suffix}.gz", "wb") as g:
                        g.write(redact(f.read().decode(errors="replace")).encode())
        summary["end"] = {"load": os.getloadavg()[0], "t": round(time.time(), 3)}
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1)))


main()
