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
        (DATA / "svrngs" / "pid").write_text(str(self.proc.pid))
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

    @classmethod
    def attach(cls):
        """The node a previous stage started, if it still answers."""
        self = cls()
        api = self.dir / "api"
        if api.exists():
            url = api.read_text().strip().rstrip("/")
            try:
                port = int(url.rsplit(":", 1)[1])
                groups = json.loads(said(call(port, "GET", "/v1/groups", timeout=5)))
                if groups.get("groups"):
                    self.api, self.group = url, groups["groups"][0]["id"]
                    pid = self.dir / "pid"
                    if pid.exists():
                        self.proc = type("P", (), {"pid": int(pid.read_text()),
                                                   "poll": lambda s: None})()
                    return self
            except Exception:
                pass
        return None

    def stop(self):
        if not self.proc:
            return
        if hasattr(self.proc, "send_signal"):
            if self.proc.poll() is None:
                self.proc.send_signal(signal.SIGINT)
                try:
                    self.proc.wait(20)
                except subprocess.TimeoutExpired:
                    self.proc.kill()
                    self.proc.wait()
        else:
            try:
                os.kill(self.proc.pid, signal.SIGINT)
            except ProcessLookupError:
                pass
            for _ in range(60):
                try:
                    os.kill(self.proc.pid, 0)
                except ProcessLookupError:
                    break
                time.sleep(0.25)
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


def ensure_node():
    sv = Svrngs.attach()
    if sv is None:
        sv = Svrngs().start()
    return sv


def stage_pieces():
    versions()
    sv = ensure_node()
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



def shutdown(proc, log=None, bound=30):
    if proc is None:
        return
    proc.send_signal(signal.SIGINT)
    try:
        proc.wait(bound)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
    if log:
        log.close()


def ids_of(res):
    try:
        return [m["id"] for m in json.loads(said(res)).get("data", [])]
    except Exception:
        return []


def chat_body(model, text="Reply with the single word: ok", **kw):
    b = {"model": model,
         "messages": [{"role": "user", "content": text}],
         "max_tokens": 24, "stream": False}
    b.update(kw)
    return b


def stage_router():
    sv = ensure_node()
    mdir = WEIGHTS / "router"
    argv = ["llama-server", "--models-dir", str(mdir), "--models-max", "2",
            "--host", "127.0.0.1", "--port", "8104", "--jinja", "-ngl", "99"]
    log = open(LOGS / "llama-router.err", "w")
    proc = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=log,
                            stdin=subprocess.DEVNULL)
    piece = {"piece": "llama-router", "serve": argv, "port": 8104, "when": now()}
    try:
        wait_for(8104, "/health", bound=120)
        piece["models_direct"] = record("router/direct-models.json",
                                        call(8104, "GET", "/v1/models"))
        piece["models_path_direct"] = record("router/direct-models-path.json",
                                             call(8104, "GET", "/models"))
        ids = ids_of(piece["models_direct"])
        piece["ids"] = ids
        piece["props_direct"] = record("router/direct-props.json",
                                       call(8104, "GET", "/props"))
        if ids:
            piece["load_direct"] = record("router/direct-load.json",
                                          call(8104, "POST", "/models", {"model": ids[0]}))
            piece["chat_direct"] = record("router/direct-chat.json",
                                          call(8104, "POST", "/v1/chat/completions",
                                               chat_body(ids[0])))
        piece["load_unknown_direct"] = record(
            "router/direct-load-unknown.json",
            call(8104, "POST", "/models", {"model": "not/a-model"}))
        piece["chat_unknown_direct"] = record(
            "router/direct-chat-unknown.json",
            call(8104, "POST", "/v1/chat/completions", chat_body("not/a-model")))
        piece["admin_direct"] = {}
        for method, path, body in [
                ("GET", "/slots", None), ("POST", "/slots/0", {"action": "save"}),
                ("GET", "/metrics", None), ("GET", "/props", None)]:
            piece["admin_direct"][f"{method} {path}"] = record(
                f"router/direct-{method}-{path.strip('/').replace('/', '_')}.json",
                call(8104, method, path, body, timeout=30))
        lend = sv.share(8104, "llama-server", "ana-router",
                        refuse=["/models", "/slots", "/metrics"])
        piece["lend"] = lend
        piece["models_origin"] = record("router/origin-models.json",
                                        through(sv, lend, "GET", "/v1/models"))
        piece["models_path_origin"] = record("router/origin-models-path.json",
                                             through(sv, lend, "GET", "/models"))
        piece["load_origin"] = record("router/origin-load.json",
                                      through(sv, lend, "POST", "/models",
                                              {"model": ids[0] if ids else "x"}))
        if ids:
            piece["chat_origin"] = record("router/origin-chat.json",
                                          through(sv, lend, "POST",
                                                  "/v1/chat/completions",
                                                  chat_body(ids[0])))
        piece["chat_unknown_origin"] = record(
            "router/origin-chat-unknown.json",
            through(sv, lend, "POST", "/v1/chat/completions",
                    chat_body("not/a-model")))
        piece["slots_origin"] = record("router/origin-slots.json",
                                       through(sv, lend, "GET", "/slots"))
        piece["unshared"] = sv.unshare("ana-router")
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        shutdown(proc, log)
    save("router/piece.json", piece)
    print(json.dumps({k: v for k, v in piece.items() if k in
                      ("ids", "could_not_judge", "unshared")}, indent=2))


SWAP_YAML = """\
models:
  "tiny-1.5b":
    cmd: |
      llama-server --port ${PORT} -m @CODER@ --jinja --host 127.0.0.1 -c 2048 -ngl 99
  "tiny-0.5b":
    cmd: |
      llama-server --port ${PORT} -m @SMALL@ --jinja --host 127.0.0.1 -c 2048 -ngl 99
"""


def stage_swap():
    sv = ensure_node()
    cfg = DATA / "llama-swap.yaml"
    cfg.write_text(SWAP_YAML.replace(
        "@CODER@", str(WEIGHTS / "router/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf")
    ).replace(
        "@SMALL@", str(WEIGHTS / "router/qwen2.5-coder-0.5b-instruct-q4_k_m.gguf")))
    argv = [str(DATA / "bin/llama-swap"), "--config", str(cfg),
            "--listen", "127.0.0.1:8105"]
    log = open(LOGS / "llama-swap.err", "w")
    proc = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=log,
                            stdin=subprocess.DEVNULL)
    piece = {"piece": "llama-swap", "serve": argv, "config": cfg.read_text(),
             "port": 8105, "when": now()}
    try:
        wait_for(8105, "/v1/models", bound=60)
        piece["models_direct"] = record("swap/direct-models.json",
                                        call(8105, "GET", "/v1/models"))
        ids = ids_of(piece["models_direct"])
        piece["ids"] = ids
        if ids:
            piece["chat_direct"] = record("swap/direct-chat.json",
                                          call(8105, "POST", "/v1/chat/completions",
                                               chat_body(ids[0]), timeout=300))
        piece["chat_unknown_direct"] = record(
            "swap/direct-chat-unknown.json",
            call(8105, "POST", "/v1/chat/completions", chat_body("not/a-model")))
        piece["admin_direct"] = {}
        for path in ["/ui", "/logs", "/running", "/unload", "/metrics", "/health"]:
            piece["admin_direct"][path] = record(
                f"swap/direct-{path.strip('/')}.json",
                call(8105, "GET", path, timeout=30))
        lend = sv.share(8105, "llama-swap", "sam-swap",
                        refuse=["/ui", "/logs", "/running", "/unload", "/metrics"])
        piece["lend"] = lend
        piece["models_origin"] = record("swap/origin-models.json",
                                        through(sv, lend, "GET", "/v1/models"))
        if ids:
            piece["chat_origin"] = record("swap/origin-chat.json",
                                          through(sv, lend, "POST",
                                                  "/v1/chat/completions",
                                                  chat_body(ids[0]), timeout=300))
        piece["admin_origin"] = {}
        for path in ["/ui", "/logs", "/running", "/unload", "/metrics"]:
            piece["admin_origin"][path] = record(
                f"swap/origin-{path.strip('/')}.json",
                through(sv, lend, "GET", path, timeout=30))
        piece["unshared"] = sv.unshare("sam-swap")
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        shutdown(proc, log)
    save("swap/piece.json", piece)
    print(json.dumps({k: v for k, v in piece.items() if k in
                      ("ids", "could_not_judge", "unshared")}, indent=2))


def wav():
    out = DATA / "audio/say.wav"
    if not out.exists():
        subprocess.run(["say", "-o", str(DATA / "audio/say.aiff"),
                        "The quick brown fox jumps over the lazy dog."], check=True)
        subprocess.run(["ffmpeg", "-y", "-i", str(DATA / "audio/say.aiff"),
                        "-ar", "16000", "-ac", "1", str(out)],
                       capture_output=True, check=True)
    return out


def multipart(fields, files):
    boundary = "----scope-boundary"
    body = b""
    for k, v in fields.items():
        body += f"--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n".encode()
    for k, (fn, data) in files.items():
        body += (f"--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"; "
                 f"filename=\"{fn}\"\r\nContent-Type: audio/wav\r\n\r\n").encode()
        body += data + b"\r\n"
    body += f"--{boundary}--\r\n".encode()
    return body, {"Content-Type": f"multipart/form-data; boundary={boundary}"}


def stage_whisper():
    sv = ensure_node()
    model = WEIGHTS / "ggml-base.en.bin"
    argv = ["whisper-server", "-m", str(model), "--host", "127.0.0.1",
            "--port", "8106", "-l", "en"]
    log = open(LOGS / "whisper.err", "w")
    proc = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=log,
                            stdin=subprocess.DEVNULL)
    piece = {"piece": "whisper", "serve": argv, "port": 8106, "when": now()}
    try:
        wait_for(8106, "/", bound=120)
        piece["models_direct"] = record("whisper/direct-models.json",
                                        call(8106, "GET", "/v1/models"))
        piece["props_direct"] = record("whisper/direct-props.json",
                                       call(8106, "GET", "/props"))
        audio = wav()
        body, headers = multipart({"model": "whisper-1", "language": "en"},
                                  {"file": ("say.wav", audio.read_bytes())})
        piece["transcription_direct"] = record(
            "whisper/direct-transcription.json",
            call(8106, "POST", "/v1/audio/transcriptions", body, headers, timeout=300))
        body2, headers2 = multipart({"model": "smaller-model-that-does-not-exist"},
                                    {"file": ("say.wav", audio.read_bytes())})
        piece["transcription_other_model_direct"] = record(
            "whisper/direct-transcription-other-model.json",
            call(8106, "POST", "/v1/audio/transcriptions", body2, headers2, timeout=300))
        piece["inference_direct"] = record(
            "whisper/direct-inference.json",
            call(8106, "POST", "/inference", body, headers, timeout=300))
        piece["admin_direct"] = {}
        for path in ["/load", "/unload", "/metrics"]:
            piece["admin_direct"][path] = record(
                f"whisper/direct-{path.strip('/')}.json",
                call(8106, "POST" if path in ("/load", "/unload") else "GET",
                     path, {}, timeout=30))
        lend = sv.share(8106, "whisper-server", "sam-stt",
                        refuse=["/load", "/unload", "/metrics"])
        piece["lend"] = lend
        piece["models_origin"] = record("whisper/origin-models.json",
                                        through(sv, lend, "GET", "/v1/models"))
        piece["transcription_origin"] = record(
            "whisper/origin-transcription.json",
            through(sv, lend, "POST", "/v1/audio/transcriptions", body, headers,
                    timeout=300))
        piece["admin_origin"] = {}
        for path in ["/load", "/unload", "/metrics"]:
            piece["admin_origin"][path] = record(
                f"whisper/origin-{path.strip('/')}.json",
                through(sv, lend, "GET", path, timeout=30))
        piece["unshared"] = sv.unshare("sam-stt")
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        shutdown(proc, log)
    save("whisper/piece.json", piece)
    print(json.dumps({k: v for k, v in piece.items() if k in
                      ("could_not_judge", "unshared")}, indent=2))


def stage_kokoro():
    sv = ensure_node()
    piece = {"piece": "kokoro", "when": now(),
             "image": "ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0",
             "port": 8107}
    name = "pantry-scope-kokoro"
    try:
        pull = subprocess.run(["podman", "pull", piece["image"]],
                              capture_output=True, text=True, timeout=900)
        save("kokoro/pull.txt", f"exit {pull.returncode}\n{pull.stdout}{pull.stderr}")
        if pull.returncode != 0:
            raise RuntimeError("podman pull failed")
        inspect = subprocess.run(
            ["podman", "image", "inspect", "--format",
             "{{.Digest}} {{.RepoDigests}}", piece["image"]],
            capture_output=True, text=True, timeout=60)
        piece["digest"] = inspect.stdout.strip()
        subprocess.run(["podman", "rm", "-f", name], capture_output=True)
        run = subprocess.run(
            ["podman", "run", "-d", "--name", name,
             "-p", "127.0.0.1:8107:8880", piece["image"]],
            capture_output=True, text=True, timeout=120)
        save("kokoro/run.txt", f"exit {run.returncode}\n{run.stdout}{run.stderr}")
        if run.returncode != 0:
            raise RuntimeError("podman run failed")
        wait_for(8107, "/v1/models", bound=300)
        piece["models_direct"] = record("kokoro/direct-models.json",
                                        call(8107, "GET", "/v1/models"))
        speech = {"model": "kokoro", "input": "The pantry is open.",
                  "voice": "af_heart", "response_format": "mp3"}
        res = call(8107, "POST", "/v1/audio/speech", speech, timeout=300)
        (RESP / "kokoro/direct-speech.mp3").write_bytes(res["body"])
        piece["speech_direct"] = {"status": res["status"],
                                  "headers": res["headers"],
                                  "bytes": len(res["body"]),
                                  "saved": "kokoro/direct-speech.mp3"}
        piece["speech_wrong_model_direct"] = record(
            "kokoro/direct-speech-wrong-model.json",
            call(8107, "POST", "/v1/audio/speech",
                 dict(speech, model="not/a-model"), timeout=120))
        piece["admin_direct"] = {}
        for path in ["/v1/audio/voices", "/docs", "/openapi.json"]:
            piece["admin_direct"][path] = record(
                f"kokoro/direct-{path.strip('/').replace('/', '_')}.json",
                call(8107, "GET", path, timeout=30))
        lend = sv.share(8107, "kokoro", "sam-voice",
                        refuse=["/docs", "/openapi.json"])
        piece["lend"] = lend
        piece["models_origin"] = record("kokoro/origin-models.json",
                                        through(sv, lend, "GET", "/v1/models"))
        res = through(sv, lend, "POST", "/v1/audio/speech", speech, timeout=300)
        (RESP / "kokoro/origin-speech.mp3").write_bytes(res["body"])
        piece["speech_origin"] = {"status": res["status"],
                                  "headers": res["headers"],
                                  "bytes": len(res["body"]),
                                  "saved": "kokoro/origin-speech.mp3"}
        piece["speech_wrong_model_origin"] = record(
            "kokoro/origin-speech-wrong-model.json",
            through(sv, lend, "POST", "/v1/audio/speech",
                    dict(speech, model="not/a-model"), timeout=120))
        piece["unshared"] = sv.unshare("sam-voice")
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        subprocess.run(["podman", "rm", "-f", name], capture_output=True)
    save("kokoro/piece.json", piece)
    print(json.dumps({k: v for k, v in piece.items() if k in
                      ("could_not_judge", "digest", "unshared")}, indent=2))


MCP_SERVER = """\
import json, os, sys
from mcp.server.fastmcp import FastMCP, Context
from mcp.types import ToolAnnotations

mcp = FastMCP("scope-echo")


@mcp.tool(annotations=ToolAnnotations(readOnlyHint=True))
def whoami(ctx: Context) -> str:
    req = getattr(ctx.request_context, "request", None)
    headers = dict(req.headers) if req is not None else {}
    env = {k: v for k, v in os.environ.items()
           if k.startswith("HTTP_") or k.startswith("X_SVRNG") or k.startswith("X-")}
    return json.dumps({"headers": headers, "env": env}, sort_keys=True)


@mcp.tool(annotations=ToolAnnotations(readOnlyHint=True))
def add(a: int, b: int) -> int:
    return a + b


@mcp.tool(annotations=ToolAnnotations(destructiveHint=True))
def erase(name: str) -> str:
    # a stand-in destructive tool; it writes nothing here
    return f"erased {name}"



if __name__ == "__main__":
    if "--stdio" in sys.argv:
        mcp.run(transport="stdio")
    else:
        mcp.settings.host = "127.0.0.1"
        mcp.settings.port = int(os.environ.get("MCP_PORT", "8108"))
        mcp.run(transport="streamable-http")
"""

MCP_CLIENT = """\
import asyncio, json, sys
from mcp import ClientSession
from mcp.client.streamable_http import streamablehttp_client


async def main(url, host):
    async with streamablehttp_client(url, headers={"Host": host}) as (r, w, _):
        async with ClientSession(r, w) as s:
            out = {}
            out["initialize"] = (await s.initialize()).model_dump(mode="json")
            out["tools_list"] = (await s.list_tools()).model_dump(mode="json")
            out["whoami"] = (await s.call_tool("whoami", {})).model_dump(mode="json")
            print(json.dumps(out, indent=2, default=str))


asyncio.run(main(sys.argv[1], sys.argv[2]))
"""


def stage_mcp():
    sv = ensure_node()
    src = DATA / "src"
    src.mkdir(parents=True, exist_ok=True)
    (src / "mcp_echo.py").write_text(MCP_SERVER)
    (src / "mcp_client.py").write_text(MCP_CLIENT)
    py = DATA / "venv/mcp/bin/python"
    piece = {"piece": "mcp", "when": now(), "python": str(py)}
    httpproc = stdproc = proxyproc = None
    httplog = open(LOGS / "mcp-http.err", "w")
    proxylog = open(LOGS / "mcp-proxy.err", "w")
    stdlog = open(LOGS / "mcp-stdio.err", "w")
    try:
        env = {**os.environ, "MCP_PORT": "8108"}
        httpproc = subprocess.Popen([str(py), str(src / "mcp_echo.py")],
                                    env=env, stdout=subprocess.DEVNULL,
                                    stderr=httplog, stdin=subprocess.DEVNULL)
        wait_for(8108, "/mcp", bound=120)
        piece["native_serve"] = [str(py), str(src / "mcp_echo.py")]
        lend = sv.share(8108, "mcp", "sam-tools")
        piece["native_lend"] = lend
        port, host = origin_of(lend)
        url = f"http://127.0.0.1:{port}/mcp"
        r = subprocess.run([str(py), str(src / "mcp_client.py"), url, host],
                           capture_output=True, text=True, timeout=300)
        save("mcp/native-through-door.txt",
             f"$ mcp_client http://<lend-host>:{port}/mcp\n"
             f"exit {r.returncode}\n{r.stdout}{r.stderr}")
        piece["native_through_door"] = {"exit": r.returncode}
        piece["native_unshared"] = sv.unshare("sam-tools")
        shutdown(httpproc, httplog)
        httpproc = None

        proxyproc = subprocess.Popen(
            ["mcp-proxy", "--transport", "streamablehttp", "--port", "8109",
             "--host", "127.0.0.1", "--", str(py), str(src / "mcp_echo.py"), "--stdio"],
            stdout=subprocess.DEVNULL, stderr=proxylog, stdin=subprocess.DEVNULL)
        piece["proxy_serve"] = ["mcp-proxy", "--transport", "streamablehttp",
                                "--port", "8109", "--", str(py),
                                str(src / "mcp_echo.py"), "--stdio"]
        wait_for(8109, "/mcp", bound=120)
        lend = sv.share(8109, "mcp", "sam-stdio-tools")
        piece["proxy_lend"] = lend
        port, host = origin_of(lend)
        url = f"http://127.0.0.1:{port}/mcp"
        r = subprocess.run([str(py), str(src / "mcp_client.py"), url, host],
                           capture_output=True, text=True, timeout=300)
        save("mcp/stdio-through-door.txt",
             f"$ mcp_client http://<lend-host>:{port}/mcp\n"
             f"exit {r.returncode}\n{r.stdout}{r.stderr}")
        piece["stdio_through_door"] = {"exit": r.returncode}
        piece["proxy_unshared"] = sv.unshare("sam-stdio-tools")
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        shutdown(proxyproc, proxylog)
        shutdown(stdproc, stdlog)
        if httpproc is not None:
            shutdown(httpproc, httplog)
    save("mcp/piece.json", piece)
    print(json.dumps({k: v for k, v in piece.items() if k in
                      ("could_not_judge", "native_through_door",
                       "stdio_through_door")}, indent=2))



STANDIN = """\
import base64, http.server, json, os, sys, urllib.parse

LOG = os.environ["STANDIN_LOG"]
COUNT = {"n": 0}


def note(method, path, headers, body):
    rec = {"method": method, "path": path, "headers": dict(headers),
           "body": body.decode("utf-8", "replace")[:4000]}
    with open(LOG, "a") as f:
        f.write(json.dumps(rec) + "\\n")


class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def _body(self):
        n = int(self.headers.get("Content-Length") or 0)
        return self.rfile.read(n) if n else b""

    def _json(self, code, obj):
        data = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        note("GET", self.path, self.headers, b"")
        if self.path.rstrip("/") in ("/v1/models", "/models"):
            self._json(200, {"object": "list", "data": [
                {"id": "ana/coder", "object": "model", "owned_by": "standin"},
                {"id": "sam/stt", "object": "model", "owned_by": "standin"},
                {"id": "sam/voice", "object": "model", "owned_by": "standin"}]})
        elif self.path == "/health":
            self._json(200, {"ok": True})
        else:
            self._json(404, {"error": "not found"})

    def do_POST(self):
        body = self._body()
        note("POST", self.path, self.headers, body)
        if self.path.rstrip("/") == "/v1/chat/completions":
            try:
                req = json.loads(body or b"{}")
            except Exception:
                req = {}
            text = "stand-in answered for " + str(req.get("model"))
            self._json(200, {"id": "chatcmpl-standin", "object": "chat.completion",
                             "created": 0, "model": req.get("model"),
                             "choices": [{"index": 0, "finish_reason": "stop",
                                          "message": {"role": "assistant",
                                                      "content": text}}]})
        elif self.path.rstrip("/") == "/v1/audio/transcriptions":
            self._json(200, {"text": "stand-in heard the quick brown fox"})
        elif self.path.rstrip("/") == "/v1/audio/speech":
            data = b"ID3" + b"stand-in-speech" * 100
            self.send_response(200)
            self.send_header("Content-Type", "audio/mpeg")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
        else:
            self._json(404, {"error": "not found"})


http.server.ThreadingHTTPServer(("0.0.0.0", 8111), H).serve_forever()
"""


def stage_interface():
    src = DATA / "src"
    src.mkdir(parents=True, exist_ok=True)
    standin = src / "standin.py"
    standin.write_text(STANDIN)
    (RESP / "standin").mkdir(parents=True, exist_ok=True)
    log = RESP / "standin/requests.jsonl"
    log.write_text("")
    piece = {"piece": "interface", "when": now(),
             "candidates": [
                 {"name": "Open WebUI", "licence": "BSD-3 + 2025 branding clause",
                  "pinned": "ghcr.io/open-webui/open-webui:v0.12.3", "measured": True},
                 {"name": "LibreChat", "licence": "MIT",
                  "pinned": "not measured: Mongo + Meilisearch stack",
                  "measured": False, "why": "could-not-judge inside this run's budget"},
                 {"name": "AnythingLLM", "licence": "MIT",
                  "pinned": "not measured", "measured": False,
                  "why": "judged from its docs: no one-address STT/TTS + MCP"},
                 {"name": "Jan", "licence": "Apache-2.0",
                  "pinned": "not measured", "measured": False,
                  "why": "a desktop app, not an address"}],
             "image": "ghcr.io/open-webui/open-webui:v0.12.3",
             "standin": "0.0.0.0:8111, reached from a container at host.containers.internal, OpenAI only, every request logged"}
    name = "pantry-scope-webui"
    proc = None
    try:
        pull = subprocess.run(["podman", "pull", piece["image"]],
                              capture_output=True, text=True, timeout=600)
        save("interface/pull.txt", f"exit {pull.returncode}\n{pull.stdout}{pull.stderr}")
        if pull.returncode != 0:
            raise RuntimeError("podman pull failed")
        digest = subprocess.run(
            ["podman", "image", "inspect", "--format", "{{.Digest}}", piece["image"]],
            capture_output=True, text=True, timeout=60)
        piece["digest"] = digest.stdout.strip()
        proc = subprocess.Popen([sys.executable, str(standin)],
                                env={**os.environ, "STANDIN_LOG": str(log)},
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        wait_for(8111, "/health", bound=30)
        probe = ("import urllib.request;print(urllib.request.urlopen("
                 "'http://host.containers.internal:8111/health', timeout=5).read().decode())")
        reach = subprocess.run(
            ["podman", "run", "--rm", "--entrypoint", "python",
             "ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0", "-c", probe],
            capture_output=True, text=True, timeout=180)
        save("interface/standin-reach.txt",
             f"exit {reach.returncode}\n{reach.stdout}{reach.stderr}")
        if reach.returncode != 0:
            raise RuntimeError("a container cannot reach the stand-in")
        piece["standin_reachable_at"] = "host.containers.internal:8111"
        base = "http://host.containers.internal:8111/v1"
        subprocess.run(["podman", "rm", "-f", name], capture_output=True)
        run = subprocess.run(
            ["podman", "run", "-d", "--name", name, "-p", "127.0.0.1:8110:8080",
             "-e", "OPENAI_API_BASE_URL=" + base,
             "-e", "OPENAI_API_KEY=sk-scope", "-e", "ENABLE_OPENAI_API=true",
             "-e", "AUDIO_STT_ENGINE=openai",
             "-e", "AUDIO_STT_OPENAI_API_BASE_URL=" + base,
             "-e", "AUDIO_STT_OPENAI_API_KEY=sk-scope",
             "-e", "AUDIO_TTS_ENGINE=openai",
             "-e", "AUDIO_TTS_OPENAI_API_BASE_URL=" + base,
             "-e", "AUDIO_TTS_OPENAI_API_KEY=sk-scope",
             "-e", "AUDIO_TTS_VOICE=af_heart",
             piece["image"]],
            capture_output=True, text=True, timeout=120)
        save("interface/run.txt", f"exit {run.returncode}\n{run.stdout}{run.stderr}")
        if run.returncode != 0:
            raise RuntimeError("podman run failed")
        piece["settings"] = {
            "chat": "OPENAI_API_BASE_URL=" + base,
            "speech to text": "AUDIO_STT_OPENAI_API_BASE_URL=" + base,
            "speech": "AUDIO_TTS_OPENAI_API_BASE_URL=" + base,
            "note": "three separate settings, all pointed at one address"}
        # wait for the interface itself
        t0 = time.time()
        while time.time() - t0 < 300:
            try:
                r = call(8110, "GET", "/health", timeout=5)
                if r["status"] < 500:
                    break
            except Exception:
                pass
            time.sleep(1)
        piece["health"] = record("interface/health.json", call(8110, "GET", "/health"))
        # first user, then the rest of the API with its token
        signup = call(8110, "POST", "/api/v1/auths/signup",
                      {"name": "Bo", "email": "bo@example.com",
                       "password": "scope-run-password"})
        record("interface/signup.json", signup)
        token = ""
        try:
            token = json.loads(said(signup)).get("token", "")
        except Exception:
            pass
        piece["signed_up"] = bool(token)
        auth = {"Authorization": "Bearer " + token} if token else {}
        piece["models"] = record("interface/models.json",
                                 call(8110, "GET", "/api/models", headers=auth))
        piece["chat"] = record("interface/chat.json",
                               call(8110, "POST", "/api/chat/completions",
                                    {"model": "ana/coder",
                                     "messages": [{"role": "user",
                                                   "content": "Say one word."}]},
                                    headers=auth, timeout=120))
        wavdata = (DATA / "audio/say.wav").read_bytes()
        body, headers = multipart({"model": "sam/stt"}, {"file": ("say.wav", wavdata)})
        piece["transcription"] = record(
            "interface/transcription.json",
            call(8110, "POST", "/api/v1/audio/transcriptions", body,
                 {**headers, **auth}, timeout=120))
        piece["speech"] = record(
            "interface/speech.json",
            call(8110, "POST", "/api/v1/audio/speech",
                 {"model": "sam/voice", "input": "hello", "voice": "af_heart"},
                 headers=auth, timeout=120))
        # what the interface itself can take for a tool, read from the pinned image
        grep = subprocess.run(
            ["podman", "run", "--rm", "--entrypoint", "sh", piece["image"], "-c",
             "grep -ril 'streamable' /app/backend/open_webui | head -20; "
             "grep -ril 'mcp' /app/backend/open_webui | head -20"],
            capture_output=True, text=True, timeout=180)
        save("interface/mcp-grep.txt", f"exit {grep.returncode}\n{grep.stdout}{grep.stderr}")
        piece["mcp_grep"] = grep.stdout.splitlines()[:40]
        piece["requests_to_standin"] = [
            json.loads(l) for l in log.read_text().splitlines() if l.strip()]
        piece["could_not_judge"] = None
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        subprocess.run(["podman", "rm", "-f", name], capture_output=True)
        shutdown(proc)
    save("interface/piece.json", piece)
    print(json.dumps({"could_not_judge": piece.get("could_not_judge"),
                      "digest": piece.get("digest"),
                      "signed_up": piece.get("signed_up"),
                      "mcp_grep": piece.get("mcp_grep"),
                      "requests": len(piece.get("requests_to_standin", []))}, indent=2))


def stage_stop():
    sv = Svrngs.attach()
    if sv is None:
        print("no node answers")
        return
    lends = sv.lends()
    save("stop/lends-before.json", lends)
    for l in lends.get("lent", []):
        sv.unshare(l["name"])
    sv.stop()
    (DATA / "svrngs" / "api").unlink(missing_ok=True)
    print("stopped")


UI_PAGE = """<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>The house</title>
<style>
body{font:16px system-ui;margin:0;background:#faf9f7;color:#222}
header{background:#24495e;color:#fff;padding:18px 24px;display:flex;gap:18px;align-items:center}
header a{color:#cfe3ee;text-decoration:none;margin-right:14px}
main{padding:24px;max-width:1100px}
.card{background:#fff;border:1px solid #ddd;border-radius:10px;padding:18px;margin:18px 0;position:relative}
.badge{display:inline-block;background:#ffe9c7;color:#7a4b00;border-radius:6px;padding:3px 8px;
       font-size:13px;white-space:nowrap;width:86px;overflow:hidden}
.actions{display:flex;gap:10px;align-items:center;margin-top:12px;position:relative}
.primary{background:#24495e;color:#fff;border:0;border-radius:8px;padding:10px 16px;font-size:15px;position:relative;left:26px}
.ghost{background:#fff;border:1px solid #24495e;color:#24495e;border-radius:8px;padding:10px 16px}
.tablewrap{width:620px;overflow:hidden;border:1px solid #ddd;border-radius:8px}
table{border-collapse:collapse;width:900px}
th,td{text-align:left;padding:10px 14px;border-bottom:1px solid #eee;white-space:nowrap}
th{background:#f1f5f7}
.note{color:#666;font-size:14px}
</style></head><body>
<header><strong>The house</strong><nav><a href="#">Rings</a><a href="#">Pantry</a><a href="#">People</a></nav></header>
<main>
  <div class="card">
    <h2>Garden room</h2>
    <span class="badge">Subscription required</span>
    <p class="note">Booked by Ana for Saturday. The room before it is listed is in the pantry.</p>
    <div class="actions"><button class="primary">Open the room</button><button class="ghost">Invite someone</button></div>
  </div>
  <div class="card">
    <h2>What the group keeps</h2>
    <div class="tablewrap"><table>
      <tr><th>Thing</th><th>Kept by</th><th>Updated</th><th>Status</th></tr>
      <tr><td>Garden room</td><td>Ana</td><td>Thursday</td><td>Ready</td></tr>
      <tr><td>Album</td><td>Sam</td><td>Wednesday</td><td>Waiting</td></tr>
      <tr><td>Money owed</td><td>Bo</td><td>Monday</td><td>Paid</td></tr>
    </table></div>
  </div>
</main></body></html>
"""
SHOT = """\
const { chromium } = require(process.env.PW);
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: Number(process.argv[3]), height: 720 } });
  await p.goto(process.argv[2]);
  await p.screenshot({ path: process.argv[4], fullPage: true });
  await b.close();
})();
"""
VISION_PROMPT = (
    "You are describing a screenshot of a web page to a programmer who cannot see it. "
    "Describe the layout, every visible string, the element hierarchy, and any visual "
    "defects: clipping, overlap, cut-off content, or errors. Name the exact element and "
    "what is wrong with it. Be concrete.")
CODER_PROMPT = (
    "A vision model described a screenshot of a web page. Write the smallest HTML/CSS "
    "change that fixes the defect it names. Reply with the CSS rule or HTML change only, "
    "and name the element it applies to. If the description names no defect, say so.")


def stage_bridge():
    src = DATA / "src"
    ui = DATA / "ui"
    src.mkdir(parents=True, exist_ok=True)
    ui.mkdir(parents=True, exist_ok=True)
    (ui / "index.html").write_text(UI_PAGE)
    (src / "shot.js").write_text(SHOT)
    coder = WEIGHTS / "qwen2.5-coder-7b-instruct-q4_k_m-00001-of-00002.gguf"
    vision = WEIGHTS / "Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf"
    mmproj = WEIGHTS / "mmproj-Qwen2.5-VL-3B-Instruct-f16.gguf"
    piece = {"piece": "bridge", "when": now(), "prompt_vision": VISION_PROMPT,
             "prompt_coder": CODER_PROMPT,
             "pages": [{"name": "clip", "width": 480},
                       {"name": "overlap", "width": 760},
                       {"name": "table", "width": 1024}]}
    vlog = open(LOGS / "bridge-vision.err", "w")
    clog = open(LOGS / "bridge-coder.err", "w")
    vproc = cproc = srv = None
    try:
        vproc = subprocess.Popen(
            ["llama-server", "-m", str(vision), "--mmproj", str(mmproj), "--jinja",
             "--alias", "ana/vision", "--host", "127.0.0.1", "--port", "8102",
             "-c", "4096", "-ngl", "99"],
            stdout=subprocess.DEVNULL, stderr=vlog, stdin=subprocess.DEVNULL)
        cproc = subprocess.Popen(
            ["llama-server", "-m", str(coder), "--jinja", "--alias", "ana/coder",
             "--host", "127.0.0.1", "--port", "8101", "-c", "4096", "-ngl", "99"],
            stdout=subprocess.DEVNULL, stderr=clog, stdin=subprocess.DEVNULL)
        wait_for(8101, "/health", bound=180)
        wait_for(8102, "/health", bound=180)
        srv = subprocess.Popen([sys.executable, "-m", "http.server", "8112",
                                "--bind", "127.0.0.1"], cwd=str(ui),
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        wait_for(8112, "/index.html", bound=30)
        piece["serve"] = {"vision": ["llama-server", str(vision), "--mmproj", str(mmproj)],
                          "coder": ["llama-server", str(coder)],
                          "ui": "python3 -m http.server 8112 (cwd data/ui)"}
        results = []
        for page in piece["pages"]:
            name, width = page["name"], page["width"]
            # every page is the same home page; the width is what shows one defect.
            url = f"http://127.0.0.1:8112/index.html?{name}"
            shot = ui / f"{name}-{width}.png"
            subprocess.run(["node", str(src / "shot.js"), url, str(width), str(shot)],
                           check=True, timeout=120,
                           env={**os.environ, "PW": str(REPO / "web/node_modules/playwright")})
            b64 = __import__("base64").b64encode(shot.read_bytes()).decode()
            vbody = {"model": "ana/vision", "max_tokens": 500, "temperature": 0,
                     "messages": [{"role": "user", "content": [
                         {"type": "text", "text": VISION_PROMPT},
                         {"type": "image_url", "image_url": {"url": "data:image/png;base64," + b64}}]}]}
            vr = call(8102, "POST", "/v1/chat/completions", vbody, timeout=300)
            vtext = ""
            try:
                vtext = json.loads(said(vr))["choices"][0]["message"]["content"]
            except Exception:
                vtext = said(vr)
            record(f"bridge/{name}-vision.json", vr)
            cbody = {"model": "ana/coder", "max_tokens": 400, "temperature": 0,
                     "messages": [{"role": "user",
                                   "content": CODER_PROMPT + "\n\nDescription:\n" + vtext}]}
            cr = call(8101, "POST", "/v1/chat/completions", cbody, timeout=300)
            ctext = ""
            try:
                ctext = json.loads(said(cr))["choices"][0]["message"]["content"]
            except Exception:
                ctext = said(cr)
            record(f"bridge/{name}-coder.json", cr)
            results.append({"page": page, "screenshot": str(shot.relative_to(DATA)),
                            "vision_status": vr["status"], "vision": vtext,
                            "coder_status": cr["status"], "coder": ctext})
        piece["results"] = results
        piece["rubric"] = (
            "bug found: the vision answer names the planted element and its symptom; "
            "fix runs: the coder answer gives a concrete CSS/HTML change for that element, "
            "and applying it to a copy of the page removes the symptom in a fresh screenshot.")
    except Exception as e:
        piece["could_not_judge"] = f"{type(e).__name__}: {e}"
    finally:
        shutdown(srv)
        shutdown(cproc, clog)
        shutdown(vproc, vlog)
    save("bridge/piece.json", piece)
    print(json.dumps({"results": [{ "page": r["page"], "vision": r["vision"][:200],
                                    "coder": r["coder"][:200]} for r in piece.get("results", [])],
                      "could_not_judge": piece.get("could_not_judge")}, indent=2))


if __name__ == "__main__":
    stage = sys.argv[1] if len(sys.argv) > 1 else "pieces"
    {"pieces": stage_pieces, "router": stage_router, "swap": stage_swap,
     "whisper": stage_whisper, "kokoro": stage_kokoro, "mcp": stage_mcp,
     "bridge": stage_bridge, "interface": stage_interface,
     "stop": stage_stop}.get(stage, lambda: print(f"no stage {stage}"))()
