#!/usr/bin/env python3
"""models-pantry (0): the curated pair, measured on one host.

Row `models-pantry` (`ralph/next/models/STATE.md`), part (0). Each curated
file is checked against its pinned sha256, served by `llama-server` on
127.0.0.1, lent with this checkout's `svrngs share`, and asked once through
the lend. The responses are kept raw under `data/responses/`; a server that
cannot start or an ask that does not finish is written as could-not-judge,
never dropped.

    python3 drive.py
"""
import base64
import hashlib
import http.client
import json
import os
import re
import signal
import struct
import subprocess
import sys
import time
import urllib.request
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
DATA = HERE / "data"
RESP = DATA / "responses"
CURATED = REPO / "target/models/curated"
LOGS = DATA / "logs"
BIN = REPO / "target/debug/svrngs"
LOG = "info,cli=debug,door=debug,surface=debug,origin=debug,lend=debug"

for d in (RESP, LOGS):
    d.mkdir(parents=True, exist_ok=True)

# The operator's shortlist, pinned (CHARTER.md, "What the operator decided at
# models-scope"). The revision is the HuggingFace commit whose tree carries
# this sha256; the public file name is the one in that tree.
MODELS = [
    {
        "key": "qwen36-35b",
        "role": "code",
        "alias": "ana/coder",
        "repo": "unsloth/Qwen3.6-35B-A3B-MTP-GGUF",
        "revision": "5bc3e238d916f48a861bac2f8a1990a0e9b7e98d",
        "file": "Qwen3.6-35B-A3B-UD-IQ4_NL.gguf",
        "local": "Qwen3.6-35B-A3B-UD-MTP-IQ4_NL.gguf",
        "sha256": "d0ad78047bd0a583bfcb484e9defc63112c5f1f856e310dcf88dc48f2ebeae5b",
        "port": 8201,
    },
    {
        "key": "qwen3-8b",
        "role": "code",
        "alias": "ana/coder-small",
        "repo": "unsloth/Qwen3-8B-GGUF",
        "revision": "a6adef130ffb23ddaf1a62fec9dced968c9bc482",
        "file": "Qwen3-8B-Q4_K_M.gguf",
        "local": "Qwen3-8B-Q4_K_M.gguf",
        "sha256": "120307ba529eb2439d6c430d94104dabd578497bc7bfe7e322b5d9933b449bd4",
        "port": 8202,
    },
    {
        "key": "gemma4-26b",
        "role": "vision",
        "alias": "ana/vision",
        "repo": "unsloth/gemma-4-26B-A4B-it-GGUF",
        "revision": "c462057f7ed65ccdb7f7e0778fae67894d425d92",
        "file": "gemma-4-26B-A4B-it-UD-Q6_K_XL.gguf",
        "local": "gemma-4-26B-A4B-it-UD-Q6_K_XL.gguf",
        "sha256": "5cfb7ab424c01388538005f26573f3bd374d3140cc021a1c44249e69928882a4",
        "mmproj_file": "mmproj-F16.gguf",
        "mmproj_local": "mmproj-gemma-4-26B-A4B-it-F16.gguf",
        "mmproj_sha256": "418a6d8723067cd712235facbbc5cba6c8fbbd413fc1292d2aace5a027d5a42f",
        "port": 8203,
    },
    {
        "key": "gemma4-e4b",
        "role": "vision",
        "alias": "ana/vision-small",
        "repo": "unsloth/gemma-4-E4B-it-GGUF",
        "revision": "ce152932ac27bc40bc9c727386760424d50bb456",
        "file": "gemma-4-E4B-it-Q5_K_M.gguf",
        "local": "gemma-4-E4B-it-Q5_K_M.gguf",
        "sha256": "8022e25d4417ec60d3b70918341e29444f253fe5b0322a0ce2e7cf9479ebe510",
        "mmproj_file": "mmproj-F16.gguf",
        "mmproj_local": "mmproj-gemma-4-E4B-it-F16.gguf",
        "mmproj_sha256": "4b3bf6e16b19d7f85fe4dd4ab2f8c8037ec14e656f8017fcfb42b30c3db26f67",
        "port": 8204,
    },
]


def save(name, body):
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


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def now():
    return time.strftime("%Y-%m-%dT%H:%M:%S%z")


def call(port, method, path, body=None, headers=None, host=None, timeout=600):
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
        return {"status": r.status, "headers": dict(r.getheaders()), "body": r.read()}
    finally:
        c.close()


def said(res):
    try:
        return res["body"].decode()
    except Exception:
        return repr(res["body"])


def record(name, res):
    save(name, json.dumps({"status": res["status"], "headers": res["headers"],
                           "body": said(res)}, indent=2, ensure_ascii=False))
    return res


def wait_for(port, path, bound=600):
    t0 = time.time()
    last = None
    while time.time() - t0 < bound:
        try:
            r = call(port, "GET", path, timeout=5)
            if r["status"] < 500:
                return r
            last = r
        except Exception as e:
            last = str(e)
        time.sleep(1.0)
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

    def start(self):
        for stale in ("api", "pid"):
            try:
                (self.dir / stale).unlink()
            except FileNotFoundError:
                pass
        self.proc = subprocess.Popen([str(BIN), "new", "Pantry Models", "--as", "Bo"],
                                     env=self.env(), stdin=subprocess.DEVNULL,
                                     stdout=open(self.out, "w"), stderr=open(self.err, "w"))
        (self.dir / "pid").write_text(str(self.proc.pid))
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
        last = None
        while time.time() - t0 < 60:
            try:
                g = call(port, "GET", "/v1/groups")
                groups = json.loads(said(g)).get("groups", [])
                if groups:
                    self.group = groups[0]["id"]
                    break
            except Exception as e:
                last = e
            time.sleep(0.25)
        if not self.group:
            raise RuntimeError(f"svrngs new made no group: {said(g) if g else last}")
        save("svrngs/groups.json", said(g))
        text = self.out.read_text()
        text = re.sub(r"svrngs://join/\S+", "svrngs://join/<the token>", text)
        self.out.write_text(text)
        return self

    def port(self):
        return int(self.api.rsplit(":", 1)[1])

    def lends(self):
        r = call(self.port(), "GET", f"/v1/lends?namespace={self.group}", timeout=60)
        return json.loads(said(r))

    def share(self, port, kind, name):
        args = [str(BIN), "share", str(port), "--kind", kind, "--name", name]
        r = subprocess.run(args, env=self.env(), input="\n", capture_output=True,
                           text=True, timeout=120)
        out = r.stdout + r.stderr
        save(f"svrngs/share-{name}.txt",
             f"$ {' '.join(args[1:])}\nexit {r.returncode}\n"
             f"{re.sub(r'svrngs://join/\\S+', '<token>', out)}")
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
        if not self.proc:
            return
        if self.proc.poll() is None:
            self.proc.send_signal(signal.SIGINT)
            try:
                self.proc.wait(20)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()
        self.proc = None


def origin_of(lend):
    m = re.match(r"http://([^/]+)/?$", lend["origin"])
    if not m:
        raise RuntimeError(f"not an origin: {lend['origin']}")
    host = m.group(1)
    if ":" in host:
        name, port = host.rsplit(":", 1)
        return int(port), host
    return 80, host


def through(lend, method, path, body=None, headers=None, timeout=600):
    port, host = origin_of(lend)
    return call(port, method, path, body=body, headers=headers, host=host, timeout=timeout)


def png_square(rgb, size=64):
    """A PNG of one colour, made here so the run needs no image library."""
    raw = b"".join(b"\x00" + bytes(rgb) * size for _ in range(size))

    def chunk(kind, data):
        return (struct.pack(">I", len(data)) + kind + data
                + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF))

    return (b"\x89PNG\r\n\x1a\n"
            + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw))
            + chunk(b"IEND", b""))


def chat_body(model, text="Reply with the single word: ok", **kw):
    b = {"model": model,
         "messages": [{"role": "user", "content": text}],
         "max_tokens": 512, "stream": False}
    b.update(kw)
    return b


def tool_body(model):
    return {
        "model": model,
        "messages": [{"role": "user",
                      "content": "What is the weather in Paris? You must call the weather tool."}],
        "tools": [{"type": "function", "function": {
            "name": "weather", "description": "Current weather for a city",
            "parameters": {"type": "object", "properties": {
                "city": {"type": "string"}}, "required": ["city"]}}}],
        "tool_choice": "auto", "max_tokens": 256, "stream": False,
    }


def vision_body(model):
    url = "data:image/png;base64," + base64.b64encode(png_square((220, 30, 30))).decode()
    return {
        "model": model,
        "messages": [{"role": "user", "content": [
            {"type": "text", "text": "What colour is this square? Answer with one word."},
            {"type": "image_url", "image_url": {"url": url}}]}],
        "max_tokens": 512, "stream": False,
    }


def tool_calls(res):
    try:
        body = json.loads(said(res))
        msg = body["choices"][0]["message"]
        return msg.get("tool_calls") or []
    except Exception:
        return []


def ensure_node():
    return Svrngs().start()


def measure(sv, m):
    path = CURATED / m["local"]
    piece = {"key": m["key"], "role": m["role"], "when": now(),
             "repo": m["repo"], "revision": m["revision"],
             "file": m["file"], "local": m["local"], "expected_sha256": m["sha256"],
             "port": m["port"], "alias": m["alias"]}
    actual = sha256(path)
    piece["sha256"] = actual
    piece["sha256_ok"] = actual == m["sha256"]
    if not piece["sha256_ok"]:
        piece["could_not_judge"] = "the local file is not the pinned one"
        return piece
    argv = ["llama-server", "-m", str(path), "--jinja", "--alias", m["alias"],
            "--host", "127.0.0.1", "--port", str(m["port"]),
            "-c", "8192", "-ngl", "99", "--metrics"]
    if m["role"] == "vision":
        mm = CURATED / m["mmproj_local"]
        actual_mm = sha256(mm)
        piece["mmproj_file"] = m["mmproj_file"]
        piece["mmproj_sha256"] = actual_mm
        piece["mmproj_sha256_ok"] = actual_mm == m["mmproj_sha256"]
        if not piece["mmproj_sha256_ok"]:
            piece["could_not_judge"] = "the projector is not the pinned one"
            return piece
        argv += ["--mmproj", str(mm)]
    piece["serve"] = argv
    log = open(LOGS / f"{m['key']}.err", "w")
    proc = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=log,
                            stdin=subprocess.DEVNULL)
    try:
        t0 = time.time()
        wait_for(m["port"], "/health")
        piece["load_seconds"] = round(time.time() - t0, 1)
        piece["models_direct"] = record(f"{m['key']}/direct-models.json",
                                        call(m["port"], "GET", "/v1/models"))
        piece["props_direct"] = record(f"{m['key']}/direct-props.json",
                                       call(m["port"], "GET", "/props"))
        piece["chat_direct"] = record(f"{m['key']}/direct-chat.json",
                                      call(m["port"], "POST", "/v1/chat/completions",
                                           chat_body(m["alias"])))
        if m["role"] == "code":
            piece["tools_direct"] = record(f"{m['key']}/direct-tools.json",
                                           call(m["port"], "POST", "/v1/chat/completions",
                                                tool_body(m["alias"])))
            piece["tool_calls"] = len(tool_calls(piece["tools_direct"]))

        lend = sv.share(m["port"], "llama-server", m["key"])
        piece["lend"] = lend
        piece["models_origin"] = record(f"{m['key']}/origin-models.json",
                                        through(lend, "GET", "/v1/models"))
        piece["chat_origin"] = record(f"{m['key']}/origin-chat.json",
                                      through(lend, "POST", "/v1/chat/completions",
                                              chat_body(m["alias"])))
        if m["role"] == "code":
            piece["tools_origin"] = record(f"{m['key']}/origin-tools.json",
                                           through(lend, "POST", "/v1/chat/completions",
                                                   tool_body(m["alias"])))
            piece["tool_calls_origin"] = len(tool_calls(piece["tools_origin"]))
        if m["role"] == "vision":
            piece["vision_origin"] = record(f"{m['key']}/origin-vision.json",
                                            through(lend, "POST", "/v1/chat/completions",
                                                    vision_body(m["alias"])))
        piece["unshared"] = sv.unshare(m["key"])
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
    return piece


def versions():
    out = {"when": now(), "host": {}, "models": []}
    for k, c in {
        "sw_vers": ["sw_vers"], "uname": ["uname", "-a"],
        "llama-server": ["llama-server", "--version"], "node": ["node", "--version"],
        "memory": ["sysctl", "-n", "hw.memsize"],
    }.items():
        try:
            r = subprocess.run(c, capture_output=True, text=True, timeout=30)
            out["host"][k] = (r.stdout + r.stderr).strip()[:4000]
        except Exception as e:
            out["host"][k] = f"could-not-judge: {e}"
    save("versions.json", out)
    return out


def main():
    wanted = set(sys.argv[1:])
    versions()
    sv = ensure_node()
    results = []
    try:
        for m in MODELS:
            if wanted and m["key"] not in wanted:
                continue
            print(f"serving {m['key']} …", flush=True)
            piece = measure(sv, m)
            save(f"{m['key']}/piece.json", piece)
            results.append(piece)
            print(json.dumps({k: piece.get(k) for k in
                              ("key", "sha256_ok", "mmproj_sha256_ok", "load_seconds",
                               "tool_calls", "tool_calls_origin", "could_not_judge")},
                             indent=2), flush=True)
    finally:
        sv.stop()
    old = []
    if (RESP / "pieces.json").exists():
        try:
            old = json.loads((RESP / "pieces.json").read_text())
        except Exception:
            old = []
    merged = {p["key"]: p for p in old}
    merged.update({p["key"]: p for p in results})
    save("pieces.json", list(merged.values()))


if __name__ == "__main__":
    main()
