#!/usr/bin/env python3
"""models-scope's measured run (ralph/next/models/STATE.md, row models-scope).

One host, one run: PREDICTIONS.md was committed before the first install.
Each stage here starts a server on 127.0.0.1, lends it with this checkout's
`svrngs share` in a data dir of the run's own, and asks the same questions
through the lend's origin. Every raw answer is written under data/responses/;
nothing is rounded to a pass.

    python3 drive.py pieces     # llama-server single, router, llama-swap,
                                # whisper-server, Kokoro, MCP native and stdio
    python3 drive.py interface  # the interface against one stand-in address
    python3 drive.py bridge     # three screenshots, vision alone and coder
    python3 drive.py stop       # stop anything this run started

A server that cannot be started or pinned is written as could-not-judge with
the command and its output, never dropped.
"""
import http.client
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
DATA = HERE / "data"
RESP = DATA / "responses"
WEIGHTS = DATA / "weights"
LOGS = DATA / "logs"
BIN = REPO / "target/debug/svrngs"
LOG = "info,cli=debug,door=debug,surface=debug,origin=debug,lend=debug"

RESP.mkdir(parents=True, exist_ok=True)
LOGS.mkdir(parents=True, exist_ok=True)


def save(name, body):
    """Write one raw answer under data/responses/, as it came."""
    p = RESP / name
    p.parent.mkdir(parents=True, exist_ok=True)
    if not isinstance(body, (str, bytes)):
        body = json.dumps(body, indent=2, ensure_ascii=False,
                          default=lambda o: o.decode("utf-8", "replace")
                          if isinstance(o, bytes) else repr(o))
    if isinstance(body, bytes):
        p.write_bytes(body)
    else:
        p.write_text(body)
    return name


def now():
    return time.strftime("%Y-%m-%dT%H:%M:%S%z")


def call(port, method, path, body=None, headers=None, host=None, timeout=180):
    """One HTTP call to 127.0.0.1:<port>, with an optional Host header, as a
    program on this computer makes it (no Sec-Fetch-*, so the lend's own)."""
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
        data = r.read()
        return {"status": r.status, "headers": dict(r.getheaders()), "body": data}
    finally:
        c.close()


def said(res):
    try:
        return res["body"].decode()
    except Exception:
        return repr(res["body"])


def record(name, res):
    save(name, json.dumps({"status": res["status"],
                           "headers": res["headers"],
                           "body": said(res)}, indent=2, ensure_ascii=False))
    return res


def wait_for(port, path, bound=120, headers=None, host=None):
    t0 = time.time()
    last = None
    while time.time() - t0 < bound:
        try:
            r = call(port, "GET", path, headers=headers, host=host, timeout=5)
            if r["status"] < 500:
                return r
            last = r
        except Exception as e:
            last = str(e)
        time.sleep(0.5)
    raise RuntimeError(f"127.0.0.1:{port}{path} did not answer in {bound}s: {last}")


class Svrngs:
    """One svrngs node for the run, in a HOME and data dir of the run's own."""

    def __init__(self):
        self.home = DATA / "home"
        self.dir = DATA / "svrngs"
        self.home.mkdir(parents=True, exist_ok=True)
        self.dir.mkdir(parents=True, exist_ok=True)
        self.out = LOGS / "svrngs.out"
        self.err = LOGS / "svrngs.err"
        self.proc = None
        self.api = None
        self.group = None

    def env(self):
        return {**os.environ, "HOME": str(self.home), "SVRNGS_DIR": str(self.dir),
                "USER": "Bo", "NO_COLOR": "1", "RUST_LOG": LOG}

    def start(self, name="Pantry Scope"):
        self.proc = subprocess.Popen([str(BIN), "new", name, "--as", "Bo"],
                                     env=self.env(), stdin=subprocess.DEVNULL,
                                     stdout=open(self.out, "w"), stderr=open(self.err, "w"))
        t0 = time.time()
        while time.time() - t0 < 120:
            api = self.dir / "api"
            if api.exists() and api.read_text().strip():
                self.api = api.read_text().strip().rstrip("/")
                break
            if self.proc.poll() is not None:
                raise RuntimeError(f"svrngs new stopped: {self.out.read_text()[-800:]}")
            time.sleep(0.25)
        if not self.api:
            raise RuntimeError("svrngs new printed no api address")
        port = int(self.api.rsplit(":", 1)[1])
        t0 = time.time()
        g = None
        while time.time() - t0 < 60:
            g = call(port, "GET", "/v1/groups")
            groups = json.loads(said(g)).get("groups", [])
            if groups:
                self.group = groups[0]["id"]
                break
            time.sleep(0.25)
        if not self.group:
            raise RuntimeError(f"svrngs new made no group: {said(g)}")
        save("svrngs/groups.json", said(g))
        # Redact the join token from the saved stdout.
        text = self.out.read_text()
        text = re.sub(r"svrngs://join/\S+", "svrngs://join/<the token>", text)
        self.out.write_text(text)
        return self

    def port(self):
        return int(self.api.rsplit(":", 1)[1])

    def lends(self):
        r = call(self.port(), "GET", f"/v1/lends?namespace={self.group}", timeout=60)
        return json.loads(said(r))

    def share(self, port, kind, name, refuse=()):
        args = [str(BIN), "share", str(port), "--kind", kind, "--name", name]
        for p in refuse:
            args += ["--refuse", p]
        r = subprocess.run(args, env=self.env(), input="\n", capture_output=True,
                           text=True, timeout=120)
        out = r.stdout + r.stderr
        save(f"svrngs/share-{name}.txt",
             f"$ {' '.join(a.replace(str(self.home), '$HOME') for a in args[1:])}\n"
             f"exit {r.returncode}\n{re.sub(r'svrngs://join/\\S+', '<token>', out)}")
        if r.returncode != 0:
            raise RuntimeError(f"share {name} failed: {out}")
        return self.lend(name)

    def unshare(self, name):
        r = subprocess.run([str(BIN), "unshare", name], env=self.env(),
                           capture_output=True, text=True, timeout=60)
        save(f"svrngs/unshare-{name}.txt", f"exit {r.returncode}\n{r.stdout}{r.stderr}")
        return r.returncode == 0

    def lend(self, name):
        for _ in range(40):
            for l in self.lends().get("lent", []):
                if l.get("name") == name:
                    return l
            time.sleep(0.25)
        raise RuntimeError(f"{name} is not in /v1/lends: {self.lends()}")

    def stop(self):
        if self.proc and self.proc.poll() is None:
            self.proc.send_signal(signal.SIGINT)
            try:
                self.proc.wait(20)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()
        self.proc = None


def origin_of(lend):
    """(port, host) for the lend's origin on this computer."""
    o = lend["origin"]
    m = re.match(r"http://([^/]+)/?$", o)
    if not m:
        raise RuntimeError(f"not an origin: {o}")
    host = m.group(1)
    if ":" in host:
        name, port = host.rsplit(":", 1)
        return int(port), host
    return 80, host


def through(sv, lend, method, path, body=None, headers=None, timeout=180):
    port, host = origin_of(lend)
    return call(port, method, path, body=body, headers=headers, host=host, timeout=timeout)


# ── pieces ──────────────────────────────────────────────────────────────────

def versions():
    out = {"when": now(), "host": {}, "servers": {}}
    cmds = {
        "sw_vers": ["sw_vers"],
        "uname": ["uname", "-a"],
        "disk": ["df", "-h", "/"],
        "llama-server": ["llama-server", "--version"],
        "whisper-server": ["whisper-server", "--version"],
        "mcp-proxy": ["mcp-proxy", "--version"],
        "uv": ["uv", "--version"],
        "node": ["node", "--version"],
        "podman": ["podman", "--version"],
        "brew": ["brew", "--version"],
    }
    for k, c in cmds.items():
        try:
            r = subprocess.run(c, capture_output=True, text=True, timeout=30)
            out["host"][k] = (r.stdout + r.stderr).strip()[:4000]
        except Exception as e:
            out["host"][k] = f"could-not-judge: {e}"
    save("versions.json", out)
    return out


def measure_llama(sv, name, port, args, alias, routing_model, *, lend_name=None,
                  tools=False, transcription=False, admin=()):
    """Serve one llama-server, lend it, and ask every question the row names."""
    log = open(LOGS / f"{name}.err", "w")
    proc = subprocess.Popen(["llama-server", *args], stdout=subprocess.DEVNULL,
                            stderr=log, stdin=subprocess.DEVNULL)
    piece = {"piece": name, "serve": ["llama-server", *args], "port": port,
             "alias": alias, "when": now()}
    try:
        wait_for(port, "/health", bound=180)
        piece["models_direct"] = record(f"{name}/direct-models.json", call(port, "GET", "/v1/models"))
        piece["props_direct"] = record(f"{name}/direct-props.json", call(port, "GET", "/props"))
        piece["admin_direct"] = {}
        for path in admin:
            piece["admin_direct"][path] = record(
                f"{name}/direct-{path.strip('/').replace('/', '_')}.json",
                call(port, "GET", path, timeout=20))

        refuse = [p for p in admin]
        lend = sv.share(port, "llama-server", lend_name or alias, refuse=refuse)
        piece["lend"] = lend
        piece["models_origin"] = record(f"{name}/origin-models.json",
                                        through(sv, lend, "GET", "/v1/models"))
        piece["props_origin"] = record(f"{name}/origin-props.json",
                                       through(sv, lend, "GET", "/props"))
        piece["admin_origin"] = {}
        for path in admin:
            piece["admin_origin"][path] = record(
                f"{name}/origin-{path.strip('/').replace('/', '_')}.json",
                through(sv, lend, "GET", path, timeout=20))

        # A stock chat call, and the same with a model this lend did not answer.
        chat = {"model": routing_model,
                "messages": [{"role": "user", "content": "Reply with the single word: ok"}],
                "max_tokens": 8, "stream": False}
        piece["chat_origin"] = record(f"{name}/origin-chat.json",
                                      through(sv, lend, "POST", "/v1/chat/completions", chat))
        wrong = dict(chat, model="someone/not-this-lend")
        piece["chat_wrong_model_origin"] = record(
            f"{name}/origin-chat-wrong-model.json",
            through(sv, lend, "POST", "/v1/chat/completions", wrong))

        if tools:
            tooled = {
                "model": routing_model,
                "messages": [{"role": "user", "content": "What is the weather in Paris? Use the tool."}],
                "tools": [{"type": "function", "function": {
                    "name": "weather", "description": "Current weather for a city",
                    "parameters": {"type": "object", "properties": {
                        "city": {"type": "string"}}, "required": ["city"]}}}],
                "tool_choice": "auto", "max_tokens": 120, "stream": False}
            piece["tools_origin"] = record(f"{name}/origin-tools.json",
                                           through(sv, lend, "POST", "/v1/chat/completions", tooled))
        if transcription:
            piece["transcription_origin"] = record(
                f"{name}/origin-transcription.json",
                through(sv, lend, "POST", "/v1/audio/transcriptions",
                        body=b"", headers={"Content-Type": "multipart/form-data; boundary=x"}))
        part = sv.unshare(lend_name or alias)
        piece["unshared"] = part
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        proc.send_signal(signal.SIGINT)
        try:
            proc.wait(30)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        log.close()
    save(f"{name}/piece.json", piece)
    return piece


def stage_pieces():
    versions()
    sv = Svrngs().start()
    results = {}
    coder = WEIGHTS / "qwen2.5-coder-7b-instruct-q4_k_m-00001-of-00002.gguf"
    vision = WEIGHTS / "Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf"
    mmproj = WEIGHTS / "mmproj-Qwen2.5-VL-3B-Instruct-f16.gguf"

    # 1.1 the code model alone, with tools.
    results["coder"] = measure_llama(
        sv, "llama-coder", 8101,
        ["-m", str(coder), "--jinja", "--alias", "ana/coder", "--host", "127.0.0.1",
         "--port", "8101", "-c", "4096", "-ngl", "99"],
        "ana/coder", "ana/coder", lend_name="ana-coder", tools=True,
        admin=["/slots", "/v1/stream", "/v1/streams", "/v1/chat/completions/control",
               "/lora-adapters", "/metrics", "/tools", "/cors-proxy", "/models"])

    # 1.2 the vision model.
    results["vision"] = measure_llama(
        sv, "llama-vision", 8102,
        ["-m", str(vision), "--mmproj", str(mmproj), "--jinja", "--alias", "ana/vision",
         "--host", "127.0.0.1", "--port", "8102", "-c", "4096", "-ngl", "99"],
        "ana/vision", "ana/vision", lend_name="ana-vision",
        admin=["/slots", "/v1/stream", "/v1/streams", "/v1/chat/completions/control",
               "/lora-adapters", "/metrics", "/tools", "/cors-proxy", "/models"])

    save("pieces.json", results)
    sv.stop()
    print(json.dumps({k: (v.get("could_not_judge") or v.get("models_origin", {}).get("status"))
                      for k, v in results.items()}, indent=2))


if __name__ == "__main__":
    stage = sys.argv[1] if len(sys.argv) > 1 else "pieces"
    {"pieces": stage_pieces}.get(stage, lambda: print(f"no stage {stage}"))()
