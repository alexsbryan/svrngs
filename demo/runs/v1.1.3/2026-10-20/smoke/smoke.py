#!/usr/bin/env python3
"""V1.1.3's smoke, from the Mac: the release's Linux svrngs on the Linux
machine as three computers (computers.py), and a phone here in WebKit, then
in Chromium (phone.mjs), through the release's page. Cy's computer, then
ada's, the link's, are stopped as a closed lid leaves them, and started
again after each phone. A computer's tab at cy's shelf, on ada's computer
through a tunnel, is read while cy's is stopped (desktop.mjs).

  smoke.py --bin <linux svrngs here> --site <the release's page> --run <dir>

Between the shelf and the away page, the lid twice: cy's computer stopped,
a socket asked for; started again, a socket asked for each second until
one opens (V1.1.3 (d)); then stopped again for the away page.

"""
import argparse, json, subprocess, sys, time
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("--bin", required=True); ap.add_argument("--site", required=True)
ap.add_argument("--run", required=True)
ap.add_argument("--host", default="alexbryan@fedora")
a = ap.parse_args()
run = Path(a.run); (run / "shots").mkdir(parents=True, exist_ok=True)
here = Path(__file__).resolve().parent
W = f"v113smoke/w-{int(time.time())}"

def ssh(cmd, **kw):
    return subprocess.run(["ssh", a.host, cmd], capture_output=True, text=True, **kw)

def log(**kw):
    line = json.dumps({"t": round(time.time(), 3), **kw})
    print(line, flush=True)
    with open(run / "smoke.jsonl", "a") as f:
        f.write(line + "\n")

ssh(f"mkdir -p {W}")
subprocess.run(["scp", "-q", str(here / "computers.py"), a.bin, f"{a.host}:{W}/"], check=True)
up = ssh(f"cd {W} && python3 computers.py up --bin ./svrngs --w .", timeout=400)
log(step="up", out=up.stdout[-3000:], err=up.stderr[-1000:])
state = json.loads(ssh(f"cat {W}/state.json").stdout)
token, api = state["token"], state["ada_api"]
def origins(x):
    if isinstance(x, dict):
        if "origin" in x:
            yield x
        for v in x.values():
            yield from origins(v)
    elif isinstance(x, list):
        for v in x:
            yield from origins(v)
shelf = next((l["origin"] for l in origins(state["lends"]) if "shelf" in json.dumps(l)), None)
log(step="state", api=api, shelf=shelf, lends=state["lends"])

for engine, name in (("webkit", "bo"), ("chromium", "di")):
    p = subprocess.Popen(["node", str(here / "phone.mjs"), json.dumps({"engine": engine, "site": a.site, "token": token, "name": name, "shots": str(run / "shots")})],
                         stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    def until(word):
        for line in p.stderr:
            log(engine=engine, phone=line.strip())
            if line.startswith(f"ready {word}"):
                return True
        return False
    if until("shelf"):
        log(engine=engine, step="freeze cy", out=ssh(f"cd {W} && python3 computers.py freeze cy --w .").stdout.strip())
        p.stdin.write("go\n"); p.stdin.flush()
    if until("thaw"):
        log(engine=engine, step="thaw cy", out=ssh(f"cd {W} && python3 computers.py thaw cy --w .").stdout.strip())
        p.stdin.write("go\n"); p.stdin.flush()
    if until("shelf2"):
        log(engine=engine, step="freeze cy", out=ssh(f"cd {W} && python3 computers.py freeze cy --w .").stdout.strip())
        if engine == "webkit" and shelf:
            port = api.rsplit(":", 1)[1]
            tunnel = subprocess.Popen(["ssh", "-N", "-L", f"{port}:127.0.0.1:{port}", a.host])
            time.sleep(3)
            d = subprocess.run(["node", str(here / "desktop.mjs"), json.dumps({"url": shelf, "shot": str(run / "shots" / "computer-away.png")})],
                               capture_output=True, text=True, timeout=120)
            log(step="computer-away", out=d.stdout.strip(), err=d.stderr[-500:])
            tunnel.terminate()
        p.stdin.write("go\n"); p.stdin.flush()
    if until("away"):
        log(engine=engine, step="freeze ada", out=ssh(f"cd {W} && python3 computers.py freeze ada --w .").stdout.strip())
        p.stdin.write("go\n"); p.stdin.flush()
    out, err = p.communicate(timeout=600)
    (run / f"phone-{engine}.json").write_text(out)
    log(engine=engine, step="phone-done", err=err[-1500:])
    for who in ("cy", "ada"):
        ssh(f"cd {W} && python3 computers.py thaw {who} --w .")
    time.sleep(15)

ssh(f"cd {W} && python3 computers.py down --w .")
for f in ("events.jsonl", "ada.log", "cy.log", "lee.log", "llama.log"):
    subprocess.run(["scp", "-q", f"{a.host}:{W}/{f}", str(run / f)])
log(step="done", w=W)
