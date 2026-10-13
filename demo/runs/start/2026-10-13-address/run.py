"""The address after a restart, again (README.md, row start-address):
start-measure's (a) restart step — a friend's llama-server on Sam's
computer, lent, asked from Bo's by curl, Python's `openai` and Node's
`fetch` before and after Bo's svrngs starts again — and then Bo's port held
by another program as Bo's svrngs starts. Two svrngs processes on this
host, each with its own HOME and SVRNGS_DIR; start-measure's clients, by
path.

    PY=<venv with openai>/bin/python W=<scratch> python3 demo/runs/start/2026-10-13-address/run.py

Data in data/ beside this file: events.jsonl, summary.json, each node's log
and what each command said, llama-server's log. scripts/check.sh runs none
of this."""
import http.client, json, os, re, shutil, signal, socket, subprocess, sys, time, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]; MEASURE = HERE.parent / "2026-10-13"
OUT = HERE / "data"; W = Path(os.environ.get("W", "/tmp/sa-w"))
PY = os.environ.get("PY", "/tmp/sm-py/bin/python")
PORT, MODEL = 18089, "Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M"
LOG = "info,cli=debug,rails=debug,origin=debug,home=debug,surface=debug,transport=warn"
procs, summary = {}, {}


def say(event, **kw):
    line = json.dumps({"t": round(time.time(), 3), "event": event, **kw})
    print(line[:600], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def host(): return {"load": os.getloadavg(), "cores": os.cpu_count()}


def wait(cond, bound, every=0.25):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def data(who): return W / who / ".svrngs"
def env(who): return {**os.environ, "HOME": str(W / who), "USER": who, "SVRNGS_DIR": str(data(who)),
                      "NO_COLOR": "1", "RUST_LOG": LOG}
def kept(who): return (data(who) / "api").read_text()
def base(who): return kept(who).strip().rstrip("/")


def api(who, path):
    try:
        with urllib.request.urlopen(base(who) + path, timeout=5) as r: return json.loads(r.read())
    except Exception: return None


def up(who, *args):
    """svrngs on `who`'s computer, running; its words to data/<who>.out. The
    address file is never deleted: what is kept is what is measured."""
    data(who).mkdir(parents=True, exist_ok=True)
    procs[who] = subprocess.Popen([W / "svrngs", *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(OUT / f"{who}.out", "a"), stderr=open(OUT / f"{who}.log", "a"))
    if not wait(lambda: api(who, "/v1/groups") is not None, 60): sys.exit(f"{who}'s svrngs did not come up")
    say("up", who=who, args=[a if "join/" not in a else "<the link>" for a in args], api=base(who))


def stop(who):
    p = procs.pop(who)
    p.send_signal(signal.SIGTERM)
    try: p.wait(15)
    except Exception: p.kill(); p.wait()
    say("stopped", who=who, exit=p.returncode)


def command(who, *args):
    run = subprocess.run([W / "svrngs", *args], env={**env(who), "RUST_LOG": "error"}, capture_output=True,
                         text=True, stdin=subprocess.DEVNULL, timeout=120)
    said = (run.stdout + run.stderr).strip()
    say("command", who=who, args=[a if "join/" not in a else "<the link>" for a in args], exit=run.returncode, said=said)
    return run.returncode, said


def ask(origin, path, headers):
    """GET `path` at `origin` on this host, its Host the origin's: status and the body's first bytes."""
    h = origin.split("//")[1].rstrip("/")
    c = http.client.HTTPConnection("127.0.0.1", int(h.rsplit(":", 1)[1]), timeout=10)
    try:
        c.request("GET", path, headers={"Host": h, **headers})
        r = c.getresponse(); blob = r.read()
    except Exception as e:
        return {"status": None, "error": f"{type(e).__name__}: {e}"}
    return {"status": r.status, "refused": r.getheader("x-svrngs-refused"), "home_page": b"/home.js" in blob}


def lends(who):
    groups = (api(who, "/v1/groups") or {}).get("groups") or [{}]
    return api(who, f"/v1/lends?namespace={groups[0].get('id')}") or {}


def origin_of(who, name):
    return next((l["origin"] for l in lends(who).get("lent", []) if l["name"] == name), None)


def home(who): return f"http://svrngs.localhost:{base(who).rsplit(':', 1)[1]}/"


def said_home(text): return re.findall(r"^Your home page.*$", text, re.M)


# --- the clients, as Bo's programs: start-measure's -------------------------
CHAT = {"model": "any", "messages": [{"role": "user", "content": "Name three rivers in Spain."}],
        "max_tokens": 32, "temperature": 0, "seed": 1}


def curl(origin):
    url = origin.rstrip("/") + "/v1/chat/completions"
    out = {"origin": origin}
    for way, body in (("whole", CHAT), ("streamed", {**CHAT, "stream": True})):
        r = subprocess.run(["curl", "-sS", "-N", "--max-time", "60", "-w", "\n%{http_code} %{remote_ip}", "-H",
                            "content-type: application/json", "-d", json.dumps(body), url],
                           capture_output=True, text=True)
        tail = r.stdout.rsplit("\n", 1)[-1].split() if r.stdout else []
        out[way] = {"reached": r.returncode == 0, "exit": r.returncode, "error": r.stderr.strip()[:300] or None,
                    "status": tail[0] if tail else None, "remote_ip": tail[1] if len(tail) > 1 else None}
        if way == "streamed": out[way]["events"] = sum(1 for l in r.stdout.splitlines() if l.startswith("data:"))
    return out


def clients(origin, step):
    got = {"curl": curl(origin)}
    for name, argv in (("python", [PY, str(MEASURE / "client.py"), origin]),
                       ("node", ["node", str(MEASURE / "fetch.mjs"), origin])):
        r = subprocess.run(argv, capture_output=True, text=True, timeout=180)
        try: got[name] = json.loads(r.stdout.strip().splitlines()[-1])
        except Exception: got[name] = {"error": (r.stderr or r.stdout)[-400:]}
    for name, g in got.items(): say("client", step=step, client=name, **g)
    return got


def typed(url): return ask(url, "/", {"Sec-Fetch-Site": "none", "Sec-Fetch-Mode": "navigate"})


def main():
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir()
    shutil.copy(REPO / "target/debug/svrngs", W / "svrngs")
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    version = subprocess.run(["llama-server", "--version"], capture_output=True, text=True)
    say("start", host=host(), head=head, llama=(version.stdout + version.stderr).strip().splitlines()[:2])
    summary["start"] = host()
    try:
        procs["llama"] = subprocess.Popen(["llama-server", "-hf", MODEL, "--host", "127.0.0.1", "--port", str(PORT),
                                           "-c", "4096", "--parallel", "2"], stdin=subprocess.DEVNULL,
                                          stdout=open(OUT / "llama.log", "a"), stderr=subprocess.STDOUT)
        straight = f"http://127.0.0.1:{PORT}/"
        if not wait(lambda: ask(straight, "/health", {})["status"] == 200, 900, 1):
            sys.exit("llama-server did not come up")

        up("sam", "new", "Lab", "--as", "Sam")
        link = wait(lambda: re.search(r"svrngs://join/\S+", (OUT / "sam.out").read_text()), 60)[0]
        summary["bo_join"] = command("bo", "join", link, "--as", "Bo", "--no-run")
        up("bo")
        summary["share"] = command("sam", "share", str(PORT), "--name", "llama", "--kind", "llama", "--limit", "4",
                                   "--refuse", "/lora-adapters")
        if not wait(lambda: origin_of("bo", "llama"), 180, 1): sys.exit(f"Bo's computer never listed llama: {lends('bo')}")

        # Before: the addresses, the page and the clients.
        llama, page = origin_of("bo", "llama"), home("bo")
        before = {"kept": kept("bo"), "home": page, "llama": llama, "said": said_home((OUT / "bo.out").read_text())}
        summary["before"] = before; say("before", **before)
        summary["home_before"] = typed(page); say("home", step="before", **summary["home_before"])
        summary["clients_before"] = clients(llama, "before")

        # The restart, as launchd restarts it: SIGTERM, then the bare
        # command over the same data dir.
        said_so_far = (OUT / "bo.out").read_text()
        stop("bo")
        up("bo")
        wait(lambda: origin_of("bo", "llama"), 60, 1)
        after = {"kept": kept("bo"), "home": home("bo"), "llama": origin_of("bo", "llama"),
                 "said": said_home((OUT / "bo.out").read_text()[len(said_so_far):])}
        summary["after"] = after; say("after", **after)
        summary["same"] = {k: before[k] == after[k] for k in ("kept", "home", "llama", "said")}
        summary["home_after"] = typed(page); say("home", step="after, at the address set before", **summary["home_after"])
        summary["clients_after"] = clients(llama, "after, at the address set before")

        # A port held: another program on Bo's port as Bo's svrngs starts.
        stop("bo")
        port = int(base("bo").rsplit(":", 1)[1])
        held = socket.socket(); held.bind(("127.0.0.1", port)); held.listen()
        run = subprocess.run([W / "svrngs"], env={**env("bo"), "RUST_LOG": "error"}, capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, timeout=60)
        held.close()
        summary["held"] = {"port": port, "exit": run.returncode, "said": (run.stdout + run.stderr).strip(),
                           "kept_after": kept("bo")}
        say("held", **summary["held"])
    finally:
        for p in procs.values():
            p.terminate()
            try: p.wait(10)
            except Exception: p.kill()
        summary["end"] = host()
        (OUT / "summary.json").write_text(json.dumps(summary, indent=1))
        say("end", host=host())


if __name__ == "__main__":
    main()
