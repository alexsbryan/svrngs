#!/usr/bin/env python3
"""Headers the door adds, lent from the home page (PLAN §1, Spotting 1).

Ana and Bo are programs on this host, each with a HOME and an SVRNGS_DIR of
their own, in one new group. Ana runs the real llama-server with --api-key
and lends it through her home page's own routes, as the page sends them
(POST /running, then POST /lent); Bo calls it as a program, as
docs/guide/models.md says. PREDICTIONS.md was committed before this ran.
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
LLAMA = Path("/usr/bin/llama-server")
MODEL = Path.home() / ".cache/llama.cpp/Qwen_Qwen2.5-0.5B-Instruct-GGUF_qwen2.5-0.5b-instruct-q4_k_m.gguf"
PORT, KEY = 8301, "night-k7Qz"
LOG = "info,cli=debug,door=debug,home=debug,lend=debug,share=debug"
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


def ana(act, step, value=""):
    """Ana's own home page in Chromium (ana.mjs), opened as `svrngs open
    --print` says, for one act of hers; what her row and /lent say."""
    r = subprocess.run([str(BIN), "open", "--print"], env=env("Ana"), capture_output=True, text=True, timeout=60)
    address = re.search(r"http://\S+", r.stdout + r.stderr)
    if not address:
        raise RuntimeError(f"open --print gave no address: {r.stdout}{r.stderr}")
    shot = RESP / f"{step}.png"
    got = subprocess.run(["node", str(HERE / "ana.mjs"), address.group(0), act, str(shot)],
                         env={**os.environ, "VALUE": value}, capture_output=True, text=True, timeout=240)
    try:
        seen = json.loads(got.stdout.strip().splitlines()[-1])
    except Exception:
        seen = {"error": got.stdout + got.stderr}
    save(f"{step}.json", seen)
    return seen


def bo_chats(step):
    """Bo's program: the lend's address, his own token, one chat."""
    lent = wait(lambda: {l["name"]: l for l in record_api("Bo", f"/v1/lends?namespace={group('Bo')}")
                         ["lent"]}.get("coder"), 120)
    if not lent:
        raise RuntimeError("Bo sees no coder")
    host = re.match(r"http://([^/]+)/", lent["origin"]).group(1)
    body = {"messages": [{"role": "user", "content": "Reply with the single word: lent"}], "max_tokens": 8}
    r = call(int(host.rsplit(":", 1)[1]), "POST", "/v1/chat/completions", body=body,
             headers={"Authorization": f"Bearer {token('Bo')}"}, host=host)
    save(f"{step}.json", r)
    return r


def main():
    if sys.argv[1:] == ["stop"]:
        for pidfile in sorted(LOGS.glob("*.pid")):
            stop(pidfile.stem)
        return
    shutil.rmtree(HOMES, ignore_errors=True)
    shutil.rmtree(RESP, ignore_errors=True)
    for d in (RESP, HOMES, LOGS):
        d.mkdir(parents=True, exist_ok=True)
    seen = {}
    try:
        v = subprocess.run([str(LLAMA), "--version"], capture_output=True, text=True)
        g = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True)
        save("versions.json", {"llama_server": (v.stdout + v.stderr).strip().splitlines()[0],
                               "svrngs_commit": g.stdout.strip(), "model": MODEL.name})
        start("llama", [str(LLAMA), "-m", str(MODEL), "--host", "127.0.0.1", "--port", str(PORT),
                        "--api-key", KEY, "-c", "2048"])
        if not wait(lambda: call(PORT, "GET", "/health")["status"] == 200, 180):
            raise RuntimeError("llama-server did not start")
        start("Ana", [str(BIN), "new", "Header Run", "--as", "ana"], who="Ana")
        link = wait(lambda: re.search(r"svrngs://join/\S+", (LOGS / "Ana.out").read_text()), 120)
        if not link or not wait(lambda: group("Ana"), 120):
            raise RuntimeError("Ana made no group")
        start("Bo", [str(BIN), "join", link.group(0), "--as", "bo"], who="Bo")
        if not wait(lambda: group("Bo") == group("Ana"), 180):
            raise RuntimeError("Bo did not join Ana's group")
        gid = group("Ana")

        # Ana lends it from her home page: looked for, named, ticked, Lend twice.
        lent = ana("lend", "1-ana-lends")
        if lent.get("error"):
            raise RuntimeError(f"Ana's page did not lend it: {lent['error']}")
        seen["1"] = bo_chats("2-bo-before")["status"]
        seen["2"] = lent.get("lent", {}).get("headers", {}).get("coder")

        refused = ana("refuse", "3-ana-refused", value="ana")
        seen["6"] = (refused.get("error"), refused.get("lent", {}).get("headers", {}).get("coder"))
        added = ana("add", "4-ana-adds", value=f"Bearer {KEY}")
        seen["3"] = (added.get("error"), added.get("lent", {}).get("headers", {}).get("coder"),
                     KEY in json.dumps(added))

        after = bo_chats("5-bo-after")
        seen["4"] = after["status"]
        seen["5"] = KEY in json.dumps(after)
        stopped = ana("stop", "6-ana-stops")
        seen["7"] = (stopped.get("error"), bo_chats("7-bo-stopped")["status"])
        kept = sorted(p.name for p in (HOMES / "Ana/.svrngs/secrets/lends/coder").glob("*")) if (HOMES / "Ana/.svrngs/secrets/lends/coder").exists() else []
        seen["kept_after_stop"] = kept
    finally:
        save("seen.json", seen)
        for name in ["Bo", "Ana", "llama"]:
            stop(name)
        for who in ["Ana", "Bo"]:
            err = LOGS / f"{who}.err"
            if err.exists():
                lines = [l for l in err.read_text(errors="replace").splitlines()
                         if re.search(r"target=\"?(door|home|lend|share)|door:|home:|lend:|share:", l)]
                save(f"log-{who}.txt", "\n".join(lines[-120:]).replace(KEY, "<the key>") if KEY in "\n".join(lines) else "\n".join(lines[-120:]))
    print(json.dumps(seen, indent=2))


if __name__ == "__main__":
    main()
