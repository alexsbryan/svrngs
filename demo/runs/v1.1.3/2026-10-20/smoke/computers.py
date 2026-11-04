#!/usr/bin/env python3
"""V1.1.3's smoke, the computers' half, on the Linux machine: three svrngs
computers, each a process with a HOME and SVRNGS_DIR of its own under W
(never ~/.svrngs or ~/svrngs). Ada starts House and its link; cy lends a
page, `shelf`; lee lends llama-server, `llama`: ~/dev/llama.cpp's build
(1464c62), which answers `GET /` 415 without gzip as the story run met
(demo/runs/start/2026-10-13-story/ 9), with Qwen2.5-0.5B-Instruct as that
run's. Every step is a line in W/events.jsonl.

  computers.py up --bin <svrngs> --w <dir>
  computers.py freeze|thaw <who> --w <dir>      SIGSTOP / SIGCONT its svrngs
  computers.py page <who> <lend> --w <dir>      a tab's GET of <lend> at its origin on <who>'s computer
  computers.py down --w <dir>
"""
import argparse, json, os, re, signal, subprocess, sys, time, urllib.request
from pathlib import Path

TOKEN = re.compile(r"svrngs://join/[A-Za-z0-9_-]+")

def env(w, who):
    e = {k: v for k, v in os.environ.items() if k not in ("SVRNGS_DIR", "RUST_LOG")}
    return {**e, "HOME": str(Path(w) / who), "USER": who, "SVRNGS_DIR": str(Path(w) / who / ".svrngs"),
            "RUST_LOG": "info,origin=debug,door=debug", "NO_COLOR": "1"}

def say(w, event, **kw):
    line = json.dumps({"t": round(time.time(), 3), "event": event, **kw})
    print(line, flush=True)
    with open(Path(w) / "events.jsonl", "a") as f:
        f.write(line + "\n")

def api(w, who):
    p = Path(w) / who / ".svrngs" / "api"
    return p.read_text().strip().rstrip("/") if p.exists() else None

def get(url, headers=None, timeout=10):
    req = urllib.request.Request(url, headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, dict(r.headers), r.read()
    except urllib.error.HTTPError as e:
        return e.code, dict(e.headers), e.read()

def run(binary, w, who, *args):
    (Path(w) / who).mkdir(parents=True, exist_ok=True)
    out, err = open(Path(w) / f"{who}.out", "w"), open(Path(w) / f"{who}.log", "w")
    return subprocess.Popen([binary, *args], env=env(w, who), stdin=subprocess.DEVNULL, stdout=out, stderr=err)

def share(binary, w, who, port, name, kind):
    r = subprocess.run([binary, "share", str(port), "--name", name, "--kind", kind], env=env(w, who),
                       input="\n", capture_output=True, text=True, timeout=60)
    say(w, "shared", who=who, lend=name, kind=kind, code=r.returncode, said=(r.stdout + r.stderr).strip()[-300:])

def holds(w, who):
    base = api(w, who)
    if not base:
        return None
    try:
        status, _, body = get(base + "/v1/groups", timeout=5)
        groups = json.loads(body).get("groups", []) if status == 200 else []
        return groups[0]["id"] if groups else None
    except Exception:
        return None

def up(a):
    w = Path(a.w); w.mkdir(parents=True, exist_ok=True)
    pids = {}
    pids["shelf"] = subprocess.Popen([sys.executable, __file__, "shelf", "--w", str(w)],
                                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).pid
    llama = Path.home() / "dev/llama.cpp/build/bin/llama-server"
    pids["llama-server"] = subprocess.Popen([str(llama), "-hf", "Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M", "--port", "18089", "--host", "127.0.0.1", "-ngl", "0"],
                                            stdout=open(w / "llama.log", "w"), stderr=subprocess.STDOUT).pid
    say(a.w, "new-start", who="ada")
    ada = run(a.bin, a.w, "ada", "new", "House", "--as", "ada"); pids["ada"] = ada.pid
    link = gid = None
    end = time.time() + 60
    while time.time() < end and not (link and gid):
        m = TOKEN.search((w / "ada.out").read_text())
        link = link or (m[0] if m else None)
        gid = gid or holds(a.w, "ada")
        time.sleep(0.2)
    say(a.w, "founded", who="ada", gid=gid, link=bool(link))
    for who in ("cy", "lee"):
        say(a.w, "join-start", who=who)
        pids[who] = run(a.bin, a.w, who, "join", link, "--as", who).pid
        end = time.time() + 60
        while time.time() < end and holds(a.w, who) != gid:
            time.sleep(0.2)
        say(a.w, "joined" if holds(a.w, who) == gid else "join-not-in", who=who)
    share(a.bin, a.w, "cy", 18088, "shelf", "shelf")
    end = time.time() + 120
    while time.time() < end:
        try:
            if get("http://127.0.0.1:18089/health", timeout=2)[0] == 200:
                break
        except Exception:
            pass
        time.sleep(1)
    say(a.w, "llama-server-health", ok=time.time() < end)
    share(a.bin, a.w, "lee", 18089, "llama", "llama-server")
    status, _, body = get(api(a.w, "ada") + f"/v1/lends?namespace={gid}", timeout=30)
    (w / "state.json").write_text(json.dumps({"token": link.rsplit("/", 1)[1], "gid": gid, "pids": pids,
                                               "ada_api": api(a.w, "ada"), "lends": json.loads(body) if status == 200 else str(body)}))
    say(a.w, "up", lends_status=status)

def shelf(a):
    """cy's shelf on 127.0.0.1:18088: its page at every path, and a request
    asking `Upgrade: websocket` switched and its bytes echoed, as a live
    app's sync socket."""
    import asyncio
    page = b"<!doctype html><title>cy's shelf</title><h1>cy's shelf</h1>"
    async def one(r, w):
        head = await r.readuntil(b"\r\n\r\n")
        if b"upgrade: websocket" in head.lower():
            w.write(b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n")
            await w.drain()
            while data := await r.read(65536):
                w.write(data); await w.drain()
        else:
            w.write(b"HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: %d\r\nconnection: close\r\n\r\n" % len(page) + page)
            await w.drain()
        w.close()
    async def main():
        server = await asyncio.start_server(one, "127.0.0.1", 18088)
        await server.serve_forever()
    asyncio.run(main())

def signal_to(a, sig):
    pid = json.loads((Path(a.w) / "state.json").read_text())["pids"][a.who]
    os.kill(pid, sig)
    say(a.w, "freeze" if sig == signal.SIGSTOP else "thaw", who=a.who, pid=pid)

def page(a):
    st = json.loads((Path(a.w) / "state.json").read_text())
    base = api(a.w, a.who)
    status, _, body = get(base + "/v1/lends", timeout=30)
    text = body.decode(errors="replace")
    m = re.search(r'https?://([a-z0-9]{62})\.localhost:\d+/', text)
    origins = re.findall(r'"(http://[a-z0-9]{62}\.localhost:\d+/)"', text)
    print(json.dumps({"lends_status": status, "origins": origins}), flush=True)

def down(a):
    st = json.loads((Path(a.w) / "state.json").read_text())
    for name, pid in st["pids"].items():
        for sig in (signal.SIGCONT, signal.SIGTERM):
            try:
                os.kill(pid, sig)
            except ProcessLookupError:
                pass
    say(a.w, "down")

ap = argparse.ArgumentParser()
ap.add_argument("cmd"); ap.add_argument("who", nargs="?"); ap.add_argument("lend", nargs="?")
ap.add_argument("--bin"); ap.add_argument("--w", required=True)
a = ap.parse_args()
{"up": up, "shelf": shelf, "freeze": lambda a: signal_to(a, signal.SIGSTOP), "thaw": lambda a: signal_to(a, signal.SIGCONT),
 "page": page, "down": down}[a.cmd](a)
