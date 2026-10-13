#!/usr/bin/env python3
"""relay-park (c)'s run: the command as shipped, n0's relays and lookup its
default (`cli::SHIPPED`), makes a group whose link a phone's page opens.

On this host, for each engine, Chromium and WebKit as an iPhone
(web/tests/page.mjs): web/tests/join.mjs serves the page built by
scripts/build-web.sh on `localhost`; `svrngs new house --as Sam --origin
<that page>` from this commit's binary, over a data dir and $HOME of its
own; the link it prints read for the relay it carries (svrngs-core
link.rs: `format || issuer[32] || secret[16] || relay?`, base64url); then
the page told the link and the name Bo, and what it said read back. Every
remote address the svrngs process opened is sampled each second (`lsof -i
-n -P -p <pid>`), and n0's relays' names resolved, to say which were n0's.

Before any of it, each of n0's relays is asked its /ping
(scripts/relay-answers.sh): if none answers, the run is could-not-judge.

  python3 demo/runs/park/2026-10-13/run.py

Writes data/: n0.json, <engine>.json, <engine>.svrngs.out and .err, and
summary.json. Exit 0 whatever the verdict; the verdict is in summary.json.
"""
import base64
import json
import os
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
DATA = HERE / "data"
LOG = "cli=debug,transport=debug,relay=debug"
NAME = "Bo"
GROUP = "house"


def say(event, **kw):
    print(json.dumps({"t": time.strftime("%H:%M:%S"), "event": event, **kw}), flush=True)


def n0_relays(err_text):
    """n0's relays as the binary listed them: its `reach` trace's relay_urls."""
    for line in err_text.splitlines():
        if "the relays this computer meets its groups at" in line and "relay_urls=" in line:
            raw = line.split("relay_urls=", 1)[1].split("]", 1)[0].lstrip("[")
            return [u.strip().strip('"') for u in raw.split(",") if u.strip()]
    return []


def carried(link):
    """The relay a link's token carries, else None."""
    token = link.split("#", 1)[1]
    raw = base64.urlsafe_b64decode(token + "=" * (-len(token) % 4))
    rest = raw[1 + 32 + 16:]
    return rest.decode() if rest else None


def addresses(urls):
    """Each relay's host and the addresses it resolves to now."""
    out = {}
    for u in urls:
        host = u.split("//", 1)[1].split("/", 1)[0].split(":")[0]
        try:
            out[host] = sorted({a[4][0] for a in socket.getaddrinfo(host, 443)})
        except OSError as e:
            out[host] = f"unresolved: {e}"
    return out


def sampler(pid, samples, stop):
    while not stop.is_set():
        r = subprocess.run(["lsof", "-i", "-n", "-P", "-a", "-p", str(pid)], capture_output=True, text=True)
        remote = sorted({l.split()[8].split("->", 1)[1] for l in r.stdout.splitlines()[1:]
                         if len(l.split()) > 8 and "->" in l.split()[8]})
        samples.append({"t": time.time(), "remote": remote})
        stop.wait(1)


def engine_run(engine, binary, site):
    work = Path(tempfile.mkdtemp(prefix=f"park-{engine}-"))
    home = work / "home"
    home.mkdir()
    env = {**os.environ, "HOME": str(home), "SVRNGS_DIR": str(home / ".svrngs"), "RUST_LOG": LOG}
    r = {"engine": engine}
    phone = subprocess.Popen(["node", str(REPO / "web/tests/join.mjs"), json.dumps({"engine": engine, "site": str(site)})],
                             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    origin = json.loads(phone.stdout.readline())["origin"]
    r["origin"] = origin
    out_path, err_path = DATA / f"{engine}.svrngs.out", DATA / f"{engine}.svrngs.err"
    t0 = time.time()
    sam = subprocess.Popen([str(binary), "new", GROUP, "--as", "Sam", "--origin", origin], env=env,
                           stdout=open(out_path, "w"), stderr=open(err_path, "w"))
    link = None
    while time.time() - t0 < 60 and sam.poll() is None and link is None:
        for line in out_path.read_text().splitlines():
            if line.strip().startswith(origin) and "#" in line:
                link = line.strip()
        time.sleep(0.2)
    r["link_ms"] = round((time.time() - t0) * 1000) if link else None
    r["link"] = link
    if not link:
        r["stopped"] = f"no link on {origin} in 60 s; svrngs exited {sam.poll()}"
        sam.send_signal(signal.SIGINT)
        phone.kill()
        return r
    r["carried"] = carried(link)
    samples, stop = [], threading.Event()
    watch = threading.Thread(target=sampler, args=(sam.pid, samples, stop))
    watch.start()
    t1 = time.time()
    phone.stdin.write(json.dumps({"link": link, "name": NAME}) + "\n")
    phone.stdin.flush()
    try:
        seen, phone_err = phone.communicate(timeout=300)
    except subprocess.TimeoutExpired:
        phone.kill()
        seen, phone_err = phone.communicate()
    r["page_ms"] = round((time.time() - t1) * 1000)
    r["page_exit"] = phone.returncode
    stop.set()
    watch.join()
    sam.send_signal(signal.SIGINT)
    try:
        sam.wait(timeout=20)
    except subprocess.TimeoutExpired:
        sam.kill()
    r["svrngs_exit"] = sam.returncode
    last = [l for l in seen.splitlines() if l.startswith("{")]
    page = json.loads(last[-1]) if last else {}
    r["page"] = {k: page.get(k) for k in ("first", "again", "alone")}
    r["page"]["lines"] = page.get("lines")
    r["page_err"] = phone_err[-4000:]
    r["samples"] = samples
    err = err_path.read_text()
    r["relays"] = n0_relays(err)
    r["link_trace"] = [l for l in err.splitlines() if "the link carries" in l or "no relay — the link" in l]
    r["reach_trace"] = [l for l in err.splitlines() if "the relays this computer meets its groups at" in l]
    return r


def main():
    DATA.mkdir(exist_ok=True)
    rev = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("build", rev=rev)
    subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True,
                   env={**os.environ, "CARGO_INCREMENTAL": "0"})
    binary = REPO / "target/debug/svrngs"
    site = Path(tempfile.mkdtemp(prefix="park-site-")) / "site"
    subprocess.run([str(REPO / "scripts/build-web.sh"), "--out", str(site)], cwd=REPO, check=True)
    shipped = [l for l in (REPO / "crates/svrngs/src/cli.rs").read_text().splitlines() if l.startswith("pub const SHIPPED")]
    # n0's relays as the shipped binary lists them, from its own trace.
    probe = Path(tempfile.mkdtemp(prefix="park-probe-"))
    listed = subprocess.run([str(binary), "help"], env={**os.environ, "HOME": str(probe), "SVRNGS_DIR": str(probe / ".svrngs"),
                            "RUST_LOG": LOG}, capture_output=True, text=True)
    relays = n0_relays(listed.stderr)
    answered = {u: subprocess.run([str(REPO / "scripts/relay-answers.sh"), u], capture_output=True, text=True).returncode == 0
                for u in relays}
    n0 = {"rev": rev, "shipped": shipped, "relays": relays, "answered": answered, "resolved": addresses(relays)}
    (DATA / "n0.json").write_text(json.dumps(n0, indent=1) + "\n")
    say("n0", **n0)
    summary = {"rev": rev, "shipped": shipped}
    if not relays or not any(answered.values()):
        summary["verdict"] = "could-not-judge"
        summary["why"] = "none of n0's relays answered its /ping" if relays else "the binary listed no n0 relay"
        (DATA / "summary.json").write_text(json.dumps(summary, indent=1) + "\n")
        say("verdict", **summary)
        return
    n0_ips = {ip for ips in n0["resolved"].values() if isinstance(ips, list) for ip in ips}
    for engine in ["chromium", "webkit"]:
        say("engine", engine=engine)
        r = engine_run(engine, binary, site)
        (DATA / f"{engine}.json").write_text(json.dumps(r, indent=1) + "\n")
        remote = sorted({a for s in r.get("samples", []) for a in s["remote"]})
        hits = sorted({a for a in remote if a.rsplit(":", 1)[0].strip("[]") in n0_ips})
        first = (r.get("page") or {}).get("first") or {}
        alone = (r.get("page") or {}).get("alone") or {}
        listed_ok = first.get("status") == f"You’re in {GROUP}, as {NAME}" and alone.get("status") == first.get("status")
        verdict = ("passed" if r.get("carried") in relays and listed_ok else "failed")
        summary[engine] = {
            "carried": r.get("carried"), "carried_is_n0": r.get("carried") in relays, "link_ms": r.get("link_ms"),
            "page_ms": r.get("page_ms"), "first_status": first.get("status"), "alone_status": alone.get("status"),
            "said": first.get("said"), "samples": len(r.get("samples", [])), "remote": remote, "n0_hits": hits,
            "verdict": verdict, "stopped": r.get("stopped"),
        }
        say("result", engine=engine, **summary[engine])
    verdicts = {summary[e]["verdict"] for e in ("chromium", "webkit")}
    summary["verdict"] = "passed" if verdicts == {"passed"} else "failed"
    (DATA / "summary.json").write_text(json.dumps(summary, indent=1, ensure_ascii=False) + "\n")
    say("verdict", verdict=summary["verdict"])


if __name__ == "__main__":
    sys.exit(main())
