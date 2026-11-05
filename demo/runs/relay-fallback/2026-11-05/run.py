#!/usr/bin/env python3
"""relay-fallback's reproduction run (README.md here): the production
HTTPS/CSP failure, synthetic and cold. A plain-HTTP relay on 127.0.0.1
stands in for the Pi's, a fresh `svrngs new` keeper meets its group there
(cold), and a fresh Chromium opens the link on the built page served as
production serves it (web/index.html's own meta CSP, no loopback
allowance). The policy refuses the HTTP relay, so no name is asked and the
page says the computer did not answer. Then the same page with the tests'
loopback allowance, the control: the join form appears.

    BIN=<svrngs built at the commit> python3 demo/runs/relay-fallback/2026-11-05/run.py

The keeper's data is a scratch directory under data/, removed at the end;
the link's token is never written. scripts/check.sh runs none of this.
"""
import json, os, re, shutil, signal, socket, subprocess, sys, time, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "data"
W = OUT / "scratch"
BIN = os.environ.get("BIN", "target/debug/svrngs")
GROUP = "house"


def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def wait(cond, bound, every=0.25):
    t0 = time.time()
    while time.time() - t0 < bound:
        try:
            got = cond()
        except Exception:
            got = None
        if got:
            return got
        time.sleep(every)
    return None


def env_of(who, log="info"):
    return {
        **{k: v for k, v in os.environ.items() if k not in ("SVRNGS_DIR", "RUST_LOG")},
        "HOME": str(W / who),
        "SVRNGS_DIR": str(W / who / ".svrngs"),
        "NO_COLOR": "1",
        "RUST_LOG": log,
    }


def start(who, *args, log="info"):
    (W / who).mkdir(parents=True, exist_ok=True)
    out = open(OUT / f"{who}.out", "w")
    err = open(OUT / f"{who}.log", "w")
    p = subprocess.Popen([BIN, *args], env=env_of(who, log), stdin=subprocess.DEVNULL, stdout=out, stderr=err)
    return p, out


def stop(p):
    if p.poll() is None:
        p.send_signal(signal.SIGINT)
        try:
            p.wait(20)
        except subprocess.TimeoutExpired:
            p.kill()
            p.wait()


def ping(url):
    try:
        with urllib.request.urlopen(url + "ping", timeout=2) as r:
            return r.status == 200
    except Exception:
        return False


def browser(engine, site, token, label, allow=False):
    said = subprocess.run(
        [
            "node",
            str(HERE / "open.mjs"),
            json.dumps({"engine": engine, "site": site, "token": token, "allow": allow}),
        ],
        capture_output=True,
        text=True,
        timeout=180,
    )
    (OUT / f"open-{label}.stderr").write_text(said.stderr)
    line = said.stdout.strip().splitlines()[-1] if said.stdout.strip() else None
    seen = json.loads(line) if line else {"error": said.stderr}
    seen["exit"] = said.returncode
    return seen


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    shutil.rmtree(W, ignore_errors=True)
    if not Path(BIN).exists():
        sys.exit(f"no svrngs at {BIN}; build it first: cargo build --locked -p svrngs")
    site = os.environ.get("SITE", "target/web/site")
    if not Path(site, "index.html").exists():
        sys.exit(f"no page at {site}; build it first: scripts/build-web.sh")

    port = free_port()
    url = f"http://127.0.0.1:{port}/"
    relay, _ = start("relay", "relay", "--at", f"127.0.0.1:{port}", "--open")
    if not wait(lambda: ping(url), 30):
        stop(relay)
        sys.exit("the relay never answered /ping")

    keeper, out = start("keeper", "new", GROUP, "--as", "Sam", "--relay", url)
    link = wait(lambda: re.search(r"svrngs://join/([A-Za-z0-9_-]+)", (OUT / "keeper.out").read_text()), 60)
    if not link:
        stop(keeper)
        stop(relay)
        sys.exit("the keeper printed no link: " + (OUT / "keeper.out").read_text()[-400:])
    token = link.group(1)

    # The failure: the page's production policy refuses the HTTP relay.
    failure = browser("chromium", site, token, "production")
    # The control: the same page, the same plain relay, the policy given
    # the tests' loopback allowance only — it cannot prove the correction
    # (CHARTER.md), only that the failure above is the policy.
    control = browser("chromium", site, token, "control", allow=True)

    stop(keeper)
    stop(relay)
    (OUT / "repro.json").write_text(
        json.dumps({"relay": url, "failure": failure, "control": control}, indent=1)
    )
    print(
        json.dumps(
            {
                "relay": url,
                "failure": {k: failure.get(k) for k in ("status", "nameForm", "exit")},
                "control": {k: control.get(k) for k in ("status", "nameForm", "exit")},
            },
            indent=1,
        )
    )


if __name__ == "__main__":
    main()
