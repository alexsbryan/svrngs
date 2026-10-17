"""The llama-server run (README.md): a stock llama-server on Sam's computer, lent
with one `svrngs share`, asked from Bo's computer — its page in Chromium, its
OpenAI-compatible API whole and streamed — beside the same asked straight at
the server. Two svrngs nodes on this host, each with its own SVRNGS_DIR, as
demo/runs/u7/2026-10-11/run.py runs them.

    W=<scratch> python3 demo/runs/llama/2026-10-11/run.py

Data in run/ beside this file: events.jsonl, summary.json, each node's log
and llama-server's. scripts/check.sh runs none of this."""
import http.client, json, os, re, shutil, statistics, subprocess, sys, time, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "run"; W = Path(os.environ.get("W", "/tmp/llama-run"))
PORT, MODEL = 18089, "Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M"
LOG = "info,door=debug,origin=debug,transport=warn"
procs, summary = {}, {}


def say(event, **kw):
    line = json.dumps({"t": round(time.time(), 3), "event": event, **kw})
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def host(): return {"load": os.getloadavg(), "cores": os.cpu_count()}


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def data(who): return W / "data" / who
def env(who): return {**os.environ, "SVRNGS_DIR": str(data(who)), "NO_COLOR": "1", "RUST_LOG": LOG}
def base(who): return (data(who) / "api").read_text().strip().rstrip("/")


def api(who, path):
    try:
        with urllib.request.urlopen(base(who) + path, timeout=5) as r: return json.loads(r.read())
    except Exception: return None


def up(who, *args):
    data(who).mkdir(parents=True, exist_ok=True)
    procs[who] = subprocess.Popen([W / "svrngs", *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(data(who) / "out", "a"), stderr=open(OUT / f"{who}.log", "a"))
    if not wait(lambda: api(who, "/v1/groups"), 60): sys.exit(f"{who}'s node did not come up")
    say("up", who=who, args=[a if "join/" not in a else "<the link>" for a in args])


def command(who, *args):
    run = subprocess.run([W / "svrngs", *args], env={**env(who), "RUST_LOG": "error"}, capture_output=True,
                         text=True, stdin=subprocess.DEVNULL)
    say("command", who=who, args=list(args), exit=run.returncode, said=(run.stdout + run.stderr).strip())
    return run.returncode


def ask(at, method, path, body=None, stream=False):
    """One request: at is ("127.0.0.1", port, Host or None). Its status, the
    times of its first and last byte (or event, streamed), the events, a body."""
    ip, port, h = at
    c = http.client.HTTPConnection(ip, port, timeout=60)
    headers = {"Content-Type": "application/json", **({"Host": h} if h else {})}
    t0 = time.perf_counter()
    c.request(method, path, body=json.dumps(body) if body else None, headers=headers)
    r = c.getresponse()
    if not stream:
        blob = r.read(); t1 = time.perf_counter()
        try: parsed = json.loads(blob)
        except Exception: parsed = blob.decode(errors="replace")[:200]
        return {"status": r.status, "ms": round((t1 - t0) * 1000, 1), "body": parsed}
    events = []
    while True:
        line = r.readline()
        if not line: break
        if line.startswith(b"data:"): events.append(round((time.perf_counter() - t0) * 1000, 1))
    return {"status": r.status, "events": len(events), "first_ms": events[0] if events else None,
            "last_ms": events[-1] if events else None, "span_ms": round(events[-1] - events[0], 1) if events else None}


CHAT = {"model": "any", "messages": [{"role": "user", "content": "Name three rivers in Spain."}],
        "max_tokens": 32, "temperature": 0, "seed": 1}


def main():
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir()
    shutil.copy(REPO / "target/debug/svrngs", W / "svrngs")
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("start", host=host(), head=head, svrngs=str(REPO / "target/debug/svrngs"))
    try:
        procs["llama"] = subprocess.Popen(["llama-server", "-hf", MODEL, "--host", "127.0.0.1", "--port", str(PORT),
                                           "-c", "4096", "--parallel", "2"], stdin=subprocess.DEVNULL,
                                          stdout=open(OUT / "llama.log", "a"), stderr=subprocess.STDOUT)
        straight = ("127.0.0.1", PORT, None)
        if not wait(lambda: ask(straight, "GET", "/health")["status"] == 200 if _alive(PORT) else None, 900, 1):
            sys.exit("llama-server did not come up")
        say("llama", up=True)

        up("sam", "new", "Lab", "--as", "Sam")
        said = wait(lambda: re.search(r"svrngs://join/\S+", (data("sam") / "out").read_text()), 60)
        up("bo", "join", said[0], "--as", "Bo")
        if not wait(lambda: "You're in" in (data("bo") / "out").read_text(), 60): sys.exit("bo did not join")
        command("sam", "share", str(PORT), "--name", "llama", "--kind", "llama", "--limit", "4",
                "--refuse", "/slots", "--refuse", "/lora-adapters")
        group = api("sam", "/v1/groups")["groups"][0]["id"]
        origin = wait(lambda: next((l["origin"] for l in (api("bo", f"/v1/lends?namespace={group}") or {}).get("lent", [])
                                    if l["name"] == "llama"), None), 60)
        if not origin: sys.exit("Bo's computer never listed Sam's llama")
        h = origin.split("//")[1].rstrip("/"); lent = ("127.0.0.1", int(h.rsplit(":", 1)[1]), h)
        say("lend", origin=origin)

        # 1. The page, in Chromium on Bo's computer.
        b = subprocess.run(["node", str(HERE / "browser.mjs"), origin], capture_output=True, text=True, timeout=120)
        summary["page"] = json.loads(b.stdout.strip().splitlines()[-1]) if b.stdout.strip() else {"error": b.stderr[-400:]}
        say("page", **summary["page"])

        # 2. The API whole, five each way, alternating.
        whole = {"straight": [], "lent": []}
        for _ in range(5):
            for name, at in (("straight", straight), ("lent", lent)):
                r = ask(at, "POST", "/v1/chat/completions", CHAT); whole[name].append(r); say("whole", way=name, **{k: v for k, v in r.items() if k != "body"})
        summary["whole"] = {n: {"statuses": sorted({r["status"] for r in rs}), "median_ms": statistics.median(r["ms"] for r in rs),
                                "each_ms": [r["ms"] for r in rs]} for n, rs in whole.items()}
        summary["whole"]["lent_body"] = whole["lent"][0]["body"]

        # 3. The API streamed, five each way.
        streamed = {"straight": [], "lent": []}
        for _ in range(5):
            for name, at in (("straight", straight), ("lent", lent)):
                r = ask(at, "POST", "/v1/chat/completions", {**CHAT, "stream": True}, stream=True)
                streamed[name].append(r); say("streamed", way=name, **r)
        summary["streamed"] = {n: {"statuses": sorted({r["status"] for r in rs}), "events": [r["events"] for r in rs],
                                   "first_ms_median": statistics.median(r["first_ms"] for r in rs),
                                   "span_ms_median": statistics.median(r["span_ms"] for r in rs)} for n, rs in streamed.items()}

        # 4. What the lend keeps back.
        summary["slots"] = {"straight": ask(straight, "GET", "/slots")["status"], "lent": ask(lent, "GET", "/slots")}
        say("slots", **summary["slots"])
    finally:
        for p in procs.values():
            p.terminate()
            try: p.wait(10)
            except Exception: p.kill()
        summary["end"] = host()
        (OUT / "summary.json").write_text(json.dumps(summary, indent=1))
        say("end", host=host())


def _alive(port):
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=2): return True
    except Exception: return False


if __name__ == "__main__":
    main()
