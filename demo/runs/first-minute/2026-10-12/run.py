#!/usr/bin/env python3
"""The newcomer's first minute, split (ralph/next/bash/STATE.md,
bash-first-minute (a); Demo C's dry run, finding 8): three computers on
this host, each a svrngs of its own with debug traces — Sam the issuer,
Ana let in by Sam's link, then Alex — each of Sam and Ana lending one
port. From the moment Alex's `join` starts, Alex's `/v1/lends` is asked
every 250 ms until both lends are listed; each look and each computer's
log are kept in data/.

    W=<scratch> SVRNGS=<binary> python3 run.py [label]
"""
import json, os, re, shutil, subprocess, sys, threading, time, urllib.request
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

HERE = Path(__file__).resolve().parent
LABEL = sys.argv[1] if len(sys.argv) > 1 else "base"
OUT = HERE / "data" / LABEL
W = Path(os.environ["W"]); BIN = os.environ["SVRNGS"]
LOG = "info,node=debug,redeem=debug,cli=debug,surface=debug,svrngs_node::ring_sync=debug,transport=warn"
procs, events = {}, []


def say(event, **kw):
    events.append({"t": round(time.time(), 3), "event": event, **kw})


def data(who): return W / who / ".svrngs"


def env(who):
    e = {k: v for k, v in os.environ.items() if k != "SVRNGS_DIR"}
    return {**e, "HOME": str(W / who), "USER": who, "SVRNGS_DIR": str(data(who)), "RUST_LOG": LOG, "NO_COLOR": "1"}


def api(who, path):
    try:
        base = (data(who) / "api").read_text().strip().rstrip("/")
        with urllib.request.urlopen(base + path, timeout=10) as r:
            return json.loads(r.read())
    except Exception:
        return None


def wait(cond, bound, every=0.1):
    end = time.time() + bound
    while time.time() < end:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def serve(who, *args):
    data(who).mkdir(parents=True, exist_ok=True)
    out = open(W / f"{who}.out", "w")
    procs[who] = subprocess.Popen([BIN, *args], env=env(who), stdin=subprocess.DEVNULL, stdout=out,
                                  stderr=open(data(who) / "svrngs.log", "w"))


def share(who, port):
    run = subprocess.run([BIN, "share", str(port), "--kind", f"{who}-things"], env={**env(who), "RUST_LOG": "error"},
                         input="\n", capture_output=True, text=True)
    say("share", who=who, port=port, exit=run.returncode)


def lender(port):
    srv = HTTPServer(("127.0.0.1", port), SimpleHTTPRequestHandler)
    threading.Thread(target=srv.serve_forever, daemon=True).start()


def main():
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(parents=True)
    say("start", load=os.getloadavg()[0], cores=os.cpu_count())
    try:
        lender(18601); lender(18602)
        serve("sam", "new", "trip", "--as", "Sam")
        link = wait(lambda: re.search(r"svrngs://join/[A-Za-z0-9_-]+", (W / "sam.out").read_text()), 60)
        if not link: sys.exit("Sam's new printed no link")
        link = link[0]
        gid = wait(lambda: (api("sam", "/v1/groups") or {}).get("groups", [{}])[0].get("id"), 30)
        share("sam", 18601)
        serve("ana", "join", link, "--as", "ana")
        if not wait(lambda: "in trip" in (W / "ana.out").read_text().lower() or "you're in" in (W / "ana.out").read_text().lower(), 60):
            sys.exit(f"Ana not in: {(W / 'ana.out').read_text()}")
        share("ana", 18602)
        # Sam's and Ana's each listing the other's lend: the two standing
        # computers settled before Alex's clock starts.
        settled = wait(lambda: len((api("sam", f"/v1/lends?namespace={gid}") or {}).get("lent", [])) == 2
                       and len((api("ana", f"/v1/lends?namespace={gid}") or {}).get("lent", [])) == 2, 120, 0.5)
        say("settled", ok=bool(settled))
        time.sleep(2)
        t0 = time.time(); say("alex-join")
        serve("alex", "join", link, "--as", "alex")
        looks = []
        while time.time() - t0 < 120:
            t = time.time()
            got = api("alex", f"/v1/lends?namespace={gid}")
            if got is not None:
                look = {"ms": round((t - t0) * 1000), "took_ms": round((time.time() - t) * 1000),
                        "lent": sorted(f"{l['person']}/{l['name']}" for l in got.get("lent", [])),
                        "unanswered": got.get("unanswered")}
                looks.append(look)
                if len(look["lent"]) == 2: break
            time.sleep(0.25)
        say("alex-done", looks=len(looks))
        (OUT / "looks.json").write_text(json.dumps(looks, indent=1))
    finally:
        for p in procs.values(): p.terminate()
        for p in procs.values():
            try: p.wait(10)
            except Exception: p.kill()
        for who in ("sam", "ana", "alex"):
            for f, name in ((data(who) / "svrngs.log", f"{who}.log"), (W / f"{who}.out", f"{who}.out")):
                if f.exists():
                    text = re.sub(r"svrngs://join/[A-Za-z0-9_-]+", "svrngs://join/<the token>", f.read_text())
                    (OUT / name).write_text(text)
        (OUT / "events.jsonl").write_text("".join(json.dumps(e) + "\n" for e in events))


if __name__ == "__main__":
    main()
