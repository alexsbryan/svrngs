#!/usr/bin/env python3
"""Spot, lent as its README says (PLAN §1, Spotting 2).

Ana and Bo are programs on this host, each with a HOME and an SVRNGS_DIR of
their own, in one new group. Ana runs apps/spot/server.mjs against a
stand-in provider (provider.mjs) and lends it with `svrngs share`, pasting
`Bearer <key>` where it asks; Bo opens it from his own home page in
Chromium at a phone's size (bo.mjs) and chats. PREDICTIONS.md was committed
before this ran.
Every answer is written under data/responses/; one host, one run: not a rate.

    python3 drive.py            the whole run
    python3 drive.py stop       stop whatever is still running
"""
import http.client
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
DATA = HERE / "data"
RESP, HOMES, LOGS = DATA / "responses", DATA / "home", DATA / "logs"
# A copy, so a build of target/ while this runs changes nothing under it.
BIN = Path(os.environ.get("SVRNGS_BIN", REPO / "target/debug/svrngs"))
PROVIDER, SPOT, KEY = 8311, 8312, "spot-run-k9Wq"
LOG = "info,cli=debug,door=debug,lend=debug,share=debug"
procs = {}


def save(name, body):
    path = RESP / name
    path.parent.mkdir(parents=True, exist_ok=True)
    text = body if isinstance(body, str) else json.dumps(body, indent=2, ensure_ascii=False)
    path.write_text(re.sub(r"svrngs://join/\S+", "svrngs://join/<token>", text) + "\n")


def call(port, method, path, body=None, headers=None, host=None, timeout=120):
    h = dict(headers or {})
    if host:
        h["Host"] = host
    if isinstance(body, (dict, list)):
        body = json.dumps(body).encode()
        h.setdefault("Content-Type", "application/json")
    c = http.client.HTTPConnection("127.0.0.1", port, timeout=timeout)
    try:
        c.request(method, path, body=body, headers=h)
        r = c.getresponse()
        return {"status": r.status, "headers": dict(r.getheaders()), "body": r.read().decode(errors="replace")}
    finally:
        c.close()


def wait(cond, bound, every=0.5):
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


def env(who):
    return {**os.environ, "HOME": str(HOMES / who), "USER": who, "NO_COLOR": "1",
            "SVRNGS_DIR": str(HOMES / who / ".svrngs"), "RUST_LOG": LOG}


def start(name, argv, who=None):
    if who:
        (HOMES / who).mkdir(parents=True, exist_ok=True)
    out, err = open(LOGS / f"{name}.out", "w"), open(LOGS / f"{name}.err", "w")
    p = subprocess.Popen(argv, env=env(who) if who else os.environ, stdin=subprocess.DEVNULL,
                         stdout=out, stderr=err, start_new_session=True)
    (LOGS / f"{name}.pid").write_text(str(p.pid))
    procs[name] = p


def stop(name):
    pidfile = LOGS / f"{name}.pid"
    p = procs.pop(name, None)
    pid = p.pid if p else (int(pidfile.read_text()) if pidfile.exists() else None)
    if pid is not None:
        try:
            os.killpg(os.getpgid(pid), signal.SIGINT)
            if p:
                p.wait(20)
        except (ProcessLookupError, subprocess.TimeoutExpired):
            try:
                os.killpg(os.getpgid(pid), signal.SIGKILL)
            except ProcessLookupError:
                pass
    pidfile.unlink(missing_ok=True)


def api_port(who):
    return int((HOMES / who / ".svrngs/api").read_text().strip().rsplit(":", 1)[1])


def token(who):
    return (HOMES / who / ".svrngs/token").read_text().strip()


def record_api(who, path):
    r = call(api_port(who), "GET", path, headers={"Authorization": f"Bearer {token(who)}"})
    return json.loads(r["body"]) if r["status"] == 200 else None


def group(who):
    got = record_api(who, "/v1/groups")
    return got["groups"][0]["id"] if got and got.get("groups") else None


def main():
    if sys.argv[1:] == ["stop"]:
        for pidfile in sorted(LOGS.glob("*.pid")):
            stop(pidfile.stem)
        return
    shutil.rmtree(HOMES, ignore_errors=True)
    shutil.rmtree(RESP, ignore_errors=True)
    for d in (RESP, HOMES, LOGS):
        d.mkdir(parents=True, exist_ok=True)
    seen, provider_log = {}, LOGS / "provider-seen.jsonl"
    try:
        g = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True)
        n = subprocess.run(["node", "--version"], capture_output=True, text=True)
        save("versions.json", {"svrngs_commit": g.stdout.strip(), "node": n.stdout.strip()})
        os.environ["KEY"] = KEY
        start("provider", ["node", str(HERE / "provider.mjs"), str(provider_log), str(PROVIDER)])
        start("Ana", [str(BIN), "new", "Spot Run", "--as", "ana"], who="Ana")
        link = wait(lambda: re.search(r"svrngs://join/\S+", (LOGS / "Ana.out").read_text()), 120)
        if not link or not wait(lambda: group("Ana"), 120):
            raise RuntimeError("Ana made no group")
        start("Bo", [str(BIN), "join", link.group(0), "--as", "bo"], who="Bo")
        if not wait(lambda: group("Bo") == group("Ana"), 180):
            raise RuntimeError("Bo did not join Ana's group")

        # Ana, as apps/spot/README.md says: the server, then share, her key pasted.
        api = HOMES / "Ana/.svrngs/api"
        start("spot", ["node", str(REPO / "apps/spot/server.mjs"), "--port", str(SPOT),
                       "--provider", f"http://127.0.0.1:{PROVIDER}/v1/openai",
                       "--model", "deepseek-flash=deepseek-ai/DeepSeek-V4.1-Flash-1017",
                       "--until", "23:59", "--budget", "60", "--svrngs", str(api)], who="Ana")
        if not wait(lambda: "svrngs share" in (LOGS / "spot.out").read_text(), 30):
            raise RuntimeError("spot did not start")
        argv = [str(BIN), "share", str(SPOT), "--name", "deepseek-flash", "--program", "spot", "--group", group("Ana")]
        shared = subprocess.run(argv, env=env("Ana"), input=f"Bearer {KEY}\n", capture_output=True, text=True, timeout=120)
        save("1-share.txt", f"$ svrngs share {SPOT} --name deepseek-flash --program spot --group <group>\nexit {shared.returncode}\n{shared.stdout}{shared.stderr}")
        if shared.returncode != 0:
            raise RuntimeError("share failed")

        # Bo, at his home page, in Chromium at a phone's size.
        r = subprocess.run([str(BIN), "open", "--print"], env=env("Bo"), capture_output=True, text=True, timeout=60)
        address = re.search(r"http://\S+", r.stdout + r.stderr).group(0)
        got = subprocess.run(["node", str(HERE / "bo.mjs"), address, str(RESP), "Say hello to Bo.", "And again?"],
                             capture_output=True, text=True, timeout=300)
        bo = json.loads(got.stdout.strip().splitlines()[-1]) if got.stdout.strip() else {"error": got.stderr}
        save("2-bo.json", bo)
        time.sleep(1)
        sent = [json.loads(l) for l in provider_log.read_text().splitlines()] if provider_log.exists() else []
        save("3-provider-saw.json", [{"url": s["url"], "model": json.loads(s["body"]).get("model"),
                                      "authorization": "Bearer <the key>" if s["headers"].get("authorization") == f"Bearer {KEY}" else s["headers"].get("authorization"),
                                      "headers": sorted(s["headers"])} for s in sent])
        seen["1"] = (bo.get("title"), bo.get("listed"))
        seen["2"] = bo.get("answers", [{}])[0]
        seen["3"] = [(s["url"], json.loads(s["body"]).get("model"), s["headers"].get("authorization") == f"Bearer {KEY}",
                      [h for h in s["headers"] if "svrngs" in h or h == "cookie"]) for s in sent]
        seen["5"] = bo.get("keyInBrowser")
        seen["6"] = (bo.get("answers", [{}, {}])[1:2], len(sent))
    finally:
        for name in ["Bo", "spot", "Ana", "provider"]:
            stop(name)
        out = LOGS / "spot.out"
        if out.exists():
            save("4-ana-terminal.txt", out.read_text().replace(KEY, "<the key>"))
            seen["4"] = [l for l in out.read_text().splitlines() if l.startswith("spot: bo")]
        save("seen.json", seen)
    print(json.dumps(seen, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
