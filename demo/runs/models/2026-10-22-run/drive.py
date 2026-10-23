#!/usr/bin/env python3
"""The story on one host (row models-run (a)): Ana, Sam, Bo, Cy and Dee are
programs here, each with a HOME and an SVRNGS_DIR of its own, in one group.
Ana lends the curated coder and vision model; Sam lends whisper, Kokoro and
an MCP tool; Cy lends a model through llama-swap; Dee runs an adapter written
in this run; Bo runs `pantry serve` and the vision bridge.

PREDICTIONS.md was committed before the first server started. Every answer is
written under data/responses/; a step that cannot run is written as
could-not-judge, never dropped. One host, one run: not a rate.

    python3 drive.py            the whole story
    python3 drive.py stop       stop whatever is still running
"""
import hashlib
import http.client
import json
import os
import re
import shutil
import signal
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
CMDS = DATA / "commands"
HOMES = DATA / "home"
LOGS = DATA / "logs"
UI = DATA / "ui"
BIN = REPO / "target/debug/svrngs"
NODE = shutil.which("node") or "node"
CURATED = REPO / "target/models/curated"
WHISPER = REPO / "target/models/run-whisper/ggml-base.en.bin"
VENV = REPO / "target/models/run-venv/bin/python"
SWAP = REPO / "demo/runs/models/2026-10-22-scope/data/bin/llama-swap"
KOKORO = "ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0"
# The name each group knows a person by, as the story writes it.
PEOPLE = {"Ana": "ana", "Sam": "sam", "Bo": "bo", "Cy": "cy", "Dee": "dee"}
LOG = "info,cli=debug,door=debug,surface=debug,origin=debug,lend=debug"

MODELS = {
    "coder": (CURATED / "Qwen3.6-35B-A3B-UD-MTP-IQ4_NL.gguf",
              "d0ad78047bd0a583bfcb484e9defc63112c5f1f856e310dcf88dc48f2ebeae5b"),
    "vision": (CURATED / "gemma-4-26B-A4B-it-UD-Q6_K_XL.gguf",
               "5cfb7ab424c01388538005f26573f3bd374d3140cc021a1c44249e69928882a4"),
    "mmproj": (CURATED / "mmproj-gemma-4-26B-A4B-it-F16.gguf",
               "418a6d8723067cd712235facbbc5cba6c8fbbd413fc1292d2aace5a027d5a42f"),
    "small": (CURATED / "Qwen3-8B-Q4_K_M.gguf",
              "120307ba529eb2439d6c430d94104dabd578497bc7bfe7e322b5d9933b449bd4"),
}

for d in (RESP, CMDS, HOMES, LOGS, UI):
    d.mkdir(parents=True, exist_ok=True)

procs = {}


def save(name, body):
    p = RESP / name
    p.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(body, bytes):
        p.write_bytes(body)
    elif isinstance(body, str):
        p.write_text(body)
    else:
        p.write_text(json.dumps(body, indent=2, ensure_ascii=False, default=str))
    return str(name)


def cmd(name, argv, code=0, out=""):
    text = f"$ {' '.join(argv)}\nexit {code}\n{out}"
    (CMDS / f"{name}.txt").write_text(re.sub(r"svrngs://join/\S+", "svrngs://join/<token>", text))


def now():
    return time.strftime("%Y-%m-%dT%H:%M:%S%z")


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def call(port, method, path, body=None, headers=None, host=None, timeout=300):
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


def wait_for(port, path, bound=180, host=None):
    t0, last = time.time(), None
    while time.time() - t0 < bound:
        try:
            r = call(port, "GET", path, host=host, timeout=5)
            if r["status"] < 500:
                return r
            last = r
        except Exception as e:
            last = e
        time.sleep(0.5)
    raise RuntimeError(f"127.0.0.1:{port}{path} did not answer in {bound}s: {last}")


def wait(cond, bound, every=0.5):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got:
            return got
        time.sleep(every)
    return None


def env(who):
    return {**os.environ, "HOME": str(HOMES / who), "USER": who,
            "SVRNGS_DIR": str(HOMES / who / ".svrngs"), "NO_COLOR": "1",
            "RUST_LOG": LOG}


def start(name, argv, who=None, stdin=subprocess.DEVNULL):
    (HOMES / who).mkdir(parents=True, exist_ok=True) if who else None
    out = open(LOGS / f"{name}.out", "w") if who else subprocess.DEVNULL
    err = open(LOGS / f"{name}.err", "w")
    p = subprocess.Popen(argv, env=env(who) if who else os.environ, stdin=stdin,
                         stdout=out, stderr=err, start_new_session=True)
    (LOGS / f"{name}.pid").write_text(str(p.pid))
    procs[name] = (p, err)
    return p


def stop(name, sig=signal.SIGINT):
    got = procs.pop(name, None)
    pidfile = LOGS / f"{name}.pid"
    pid = got[0].pid if got else (int(pidfile.read_text()) if pidfile.exists() else None)
    if pid is not None:
        try:
            os.killpg(os.getpgid(pid), sig)
        except ProcessLookupError:
            pass
        if got:
            try:
                got[0].wait(20)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(os.getpgid(pid), signal.SIGKILL)
                except ProcessLookupError:
                    pass
                got[0].wait()
    if got and got[1]:
        got[1].close()
    pidfile.unlink(missing_ok=True)


def api(who):
    p = HOMES / who / ".svrngs/api"
    if not p.exists():
        return None
    return p.read_text().strip().rstrip("/")


def group(who):
    a = api(who)
    if not a:
        return None
    try:
        return json.loads(said(call(int(a.rsplit(":", 1)[1]), "GET", "/v1/groups")))["groups"][0]["id"]
    except Exception:
        return None


def lends(who):
    a, g = api(who), group(who)
    if not a or not g:
        return {}
    try:
        body = json.loads(said(call(int(a.rsplit(":", 1)[1]), "GET", f"/v1/lends?namespace={g}", timeout=90)))
        return {l["name"]: l for l in body.get("lent", [])}
    except Exception:
        return {}


def start_person(who, *args):
    start(who, [str(BIN), *args], who=who)
    got = wait(lambda: api(who), 120, 0.5)
    if not got:
        raise RuntimeError(f"{who}: no api file")
    wait(lambda: group(who), 120)
    return got


def join(who, link):
    start_person(who, "join", link, "--as", PEOPLE[who])


def share(who, port, kind, name, refuse=(), limit=None):
    argv = [str(BIN), "share", str(port), "--kind", kind, "--name", name]
    if limit:
        argv += ["--limit", str(limit)]
    for p in refuse:
        argv += ["--refuse", p]
    t0 = time.time()
    r = subprocess.run(argv, env=env(who), input="\n", capture_output=True, text=True, timeout=180)
    dt = round((time.time() - t0) * 1000)
    cmd(f"share-{name}", [x.replace(str(HOMES / who), "$HOME") for x in argv[1:]], r.returncode, r.stdout + r.stderr)
    if r.returncode != 0:
        raise RuntimeError(f"share {name} failed: {r.stdout}{r.stderr}")
    got = wait(lambda: (lends(who).get(name)), 60)
    if not got:
        raise RuntimeError(f"{name} is not lent")
    return got, dt


def unshare(who, name):
    r = subprocess.run([str(BIN), "unshare", name], env=env(who), capture_output=True, text=True, timeout=60)
    cmd(f"unshare-{name}", ["unshare", name], r.returncode, r.stdout + r.stderr)


def origin_of(lend):
    m = re.match(r"http://([^/]+)/?$", lend["origin"])
    host = m.group(1)
    port = int(host.rsplit(":", 1)[1]) if ":" in host else 80
    return port, host


def through(lend, method, path, body=None, headers=None, timeout=300):
    port, host = origin_of(lend)
    return call(port, method, path, body=body, headers=headers, host=host, timeout=timeout)


def story(step, *args, timeout=900):
    argv = [str(VENV), str(HERE / "story.py"), step, *args]
    r = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
    try:
        return json.loads(r.stdout)
    except Exception:
        return {"couldNotJudge": (r.stdout + r.stderr)[-3000:], "exit": r.returncode}


def version():
    out = {"when": now(), "host": {}, "pins": {}}
    for key, c in {"sw_vers": ["sw_vers"], "uname": ["uname", "-a"], "node": ["node", "--version"],
                   "llama-server": ["llama-server", "--version"], "whisper-server": ["whisper-server", "--version"],
                   "podman": ["podman", "--version"]}.items():
        try:
            r = subprocess.run(c, capture_output=True, text=True, timeout=30)
            out["host"][key] = (r.stdout + r.stderr).strip()[:2000]
        except Exception as e:
            out["host"][key] = f"could-not-judge: {e}"
    for key, (path, pinned) in MODELS.items():
        out["pins"][key] = {"path": str(path), "pinned": pinned,
                            "got": sha(path) if path.exists() else None,
                            "ok": path.exists() and sha(path) == pinned}
    for key, path in {"whisper": WHISPER, "llama-swap": SWAP} .items():
        out["pins"][key] = {"path": str(path), "got": sha(path) if path.exists() else None}
    out["pins"]["kokoro"] = {"image": KOKORO}
    out["pantry"] = {f: sha(REPO / "apps/pantry" / f) for f in ["pantry.mjs", "mcp.mjs"]}
    save("versions.json", out)
    return out


def prepare():
    # The whisper model the scope pinned; copied into the run's own target
    # path so its server keeps its data there.
    src = REPO / "demo/runs/models/2026-10-22-scope/data/weights/ggml-base.en.bin"
    WHISPER.parent.mkdir(parents=True, exist_ok=True)
    if not WHISPER.exists() and src.exists():
        shutil.copyfile(src, WHISPER)
    if not WHISPER.exists():
        raise RuntimeError("the pinned whisper ggml-base.en.bin is not here and no download is made")
    # The run's screenshot fixtures, copied so the run holds its own bytes.
    if not (UI / "clip.png").exists():
        shutil.copyfile(REPO / "demo/runs/models/2026-10-22-scope/data/ui/clip-480.png",
                        UI / "clip.png")
        subprocess.run(["sips", "-Z", "480", str(UI / "clip.png")], capture_output=True)
    if not VENV.exists():
        raise RuntimeError(f"no stock-client venv at {VENV}")
    for key, (path, pinned) in MODELS.items():
        if not path.exists() or sha(path) != pinned:
            raise RuntimeError(f"{key} is not the pinned file: {path}")


TIMES = {}


def servers():
    coder = MODELS["coder"][0]
    vision = MODELS["vision"][0]
    mmproj = MODELS["mmproj"][0]
    small = MODELS["small"][0]

    t0 = time.time()
    start("llama-coder", ["llama-server", "-m", str(coder), "--jinja", "--alias", "coder",
                          "--host", "127.0.0.1", "--port", "8201", "-c", "8192", "-ngl", "99"])
    start("llama-vision", ["llama-server", "-m", str(vision), "--mmproj", str(mmproj), "--jinja",
                           "--alias", "vision", "--host", "127.0.0.1", "--port", "8202",
                           "-c", "8192", "-ngl", "90", "-ub", "512"])
    start("whisper-server", ["whisper-server", "-m", str(WHISPER), "--host", "127.0.0.1",
                             "--port", "8206", "-l", "en"])
    start("whisper-wrapper", [NODE, str(REPO / "apps/pantry/examples/whisper/server.mjs"),
                              "--upstream", "http://127.0.0.1:8206", "--port", "8210", "--model", "whisper-1"])
    os.environ["MCP_PORT"] = "8208"
    start("sam-tool", [str(VENV), str(HERE / "tool.py")])

    cfg = DATA / "llama-swap.yaml"
    cfg.write_text(f"""models:
  "cy-model":
    cmd: |
      llama-server --port ${{PORT}} -m {small} --jinja --alias cy-model --host 127.0.0.1 -c 8192 -ngl 99
""")
    start("llama-swap", [str(SWAP), "--config", str(cfg), "--listen", "127.0.0.1:8209"])

    subprocess.run(["podman", "rm", "-f", "pantry-run-kokoro"], capture_output=True)
    r = subprocess.run(["podman", "run", "-d", "--name", "pantry-run-kokoro",
                        "-p", "127.0.0.1:8207:8880", KOKORO], capture_output=True, text=True)
    cmd("kokoro-run", ["podman", "run", "-d", "--name", "pantry-run-kokoro", "-p",
                       "127.0.0.1:8207:8880", KOKORO], r.returncode, r.stdout + r.stderr)
    if r.returncode != 0:
        raise RuntimeError(f"kokoro did not start: {r.stdout}{r.stderr}")

    wait_for(8201, "/health", bound=600)
    TIMES["serve_ms"] = round((time.time() - t0) * 1000)
    wait_for(8202, "/health", bound=600)
    wait_for(8206, "/", bound=300)
    wait_for(8210, "/v1/models", bound=120)
    wait_for(8208, "/mcp", bound=300)
    wait_for(8209, "/v1/models", bound=120)
    wait_for(8207, "/v1/models", bound=600)


def story_run():
    bo = "Bo"
    p = api(bo)
    port = 8280

    # C1: everything lent, by protocol, with whose computer and its address.
    r = subprocess.run([NODE, str(REPO / "apps/pantry/pantry.mjs"), "list"],
                       env=env(bo), capture_output=True, text=True, timeout=180)
    cmd("pantry-list", ["node", "apps/pantry/pantry.mjs", "list"], r.returncode, r.stdout + r.stderr)
    save("story/list.txt", r.stdout + r.stderr)
    if r.returncode != 0:
        raise RuntimeError("pantry list failed")

    start("pantry", [NODE, str(REPO / "apps/pantry/pantry.mjs"), "serve", "--port", str(port)], who=bo)
    wait_for(port, "/v1/models", bound=120)
    save("story/address.txt", f"http://127.0.0.1:{port}/v1\n")
    base = f"http://127.0.0.1:{port}"

    # C2: one address, OpenAI SDK, streamed chat, transcription, speech.
    save("story/c2-chat.json", story("chat", "--pantry", base, "--model", "ana/coder", timeout=900))
    save("story/c2-transcribe.json", story("transcribe", "--pantry", base, "--model", "sam/whisper-1",
                                           "--audio", str(REPO / "demo/runs/models/2026-10-22-scope/data/audio/say.wav")))
    save("story/c2-speech.json", story("speak", "--pantry", base, "--model", "sam/kokoro",
                                       "--save", str(RESP / "story/c2-speech.mp3")))

    # The address stays: stop and start without --port; the same client answers.
    stop("pantry")
    start("pantry", [NODE, str(REPO / "apps/pantry/pantry.mjs"), "serve"], who=bo)
    wait_for(port, "/v1/models", bound=120)
    same = wait_for(port, "/v1/models", bound=10)
    save("story/c2-after-restart.json", story("once", "--pantry", base, "--model", "ana/coder",
                                              "--prompt", "Reply with the single word: again", "--max-tokens", "8"))

    # C3: Cy's model through a server no pantry test has seen.
    save("story/c3-cy.json", story("once", "--pantry", base, "--model", "cy/cy-model",
                                   "--prompt", "Reply with the single word: cy", "--max-tokens", "8"))

    # C3: Dee's adapter, written in the run, in front of Bo's pantry.
    start("dee", [NODE, str(HERE / "dee.mjs"), base, "8282"])
    wait_for(8282, "/v1/models", bound=60)
    save("story/c3-dee.json", story("dee", "--dee", "http://127.0.0.1:8282"))

    # C7 first, while Sam is awake: the door's kept-back route and its limit.
    ana = lends(bo).get("ana-coder")
    slots = through(ana, "GET", "/slots")
    record("story/c7-slots.json", slots)
    limit = story("limit-chat", "--pantry", base, "--model", "ana/coder", timeout=900)
    save("story/c7-limit.json", limit)
    # Cy's 8B is not needed again; unloading it leaves the two large models
    # room for the bridge.
    record("story/cy-unload.json", call(8209, "GET", "/unload"))

    # C4: Bo's bridge, Ana's vision describes, Ana's coder answers; both hops named.
    start("bridge", [NODE, str(REPO / "apps/pantry/examples/bridge/server.mjs"),
                     "--upstream", base, "--vision", "ana/vision", "--port", "8281"], who=bo)
    wait_for(8281, "/", bound=60)
    save("story/c4-bridge.json", story("bridge", "--bridge", "http://127.0.0.1:8281",
                                       "--model", "ana/coder", "--shot", str(UI / "clip.png"), timeout=900))
    ana_log = (LOGS / "Ana.err").read_text(errors="replace") if (LOGS / "Ana.err").exists() else ""
    hits = [l for l in ana_log.splitlines() if "door" in l and ("Bo" in l or "bo" in l)]
    save("story/c4-door.txt", "\n".join(hits[-40:]) or "no door line naming Bo in Ana's log")

    # C5: Sam's tool listed with effects and called as Bo; Bo's name arrives.
    text = (RESP / "story/list.txt").read_text() if (RESP / "story/list.txt").exists() else ""
    line = next((l for l in text.splitlines() if "tool" in l and "whoami" in l), "")
    m = re.search(r"http://\S+", line)
    if not m:
        save("story/c5-tool.json", {"couldNotJudge": f"no tool address in list: {line!r}"})
    else:
        save("story/c5-tool.json", story("tool", "--origin", m.group(0), timeout=300))

    # C6 last: Sam asleep, the refusal names Sam and why, nothing substitutes.
    stop("Sam")
    time.sleep(5)
    save("story/c6-asleep.json", story("asleep", "--pantry", base, "--model", "sam/whisper-1", timeout=120))

    save("story/c6-lends.txt", json.dumps(lends(bo), indent=2)[:4000])
    save("story/ps.txt", subprocess.run(["ps", "-o", "pid,command", "-ax"], capture_output=True,
                                        text=True).stdout)


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "stop":
        for pidfile in sorted(LOGS.glob("*.pid")):
            stop(pidfile.stem)
        subprocess.run(["podman", "rm", "-f", "pantry-run-kokoro"], capture_output=True)
        return
    # A full run is fresh: each person's HOME and svrngs dir start empty, so
    # a run after a failed one joins the same group again rather than half of
    # two.
    shutil.rmtree(HOMES, ignore_errors=True)
    shutil.rmtree(RESP, ignore_errors=True)
    RESP.mkdir(parents=True, exist_ok=True)
    for pidfile in LOGS.glob("*.pid"):
        pidfile.unlink()
    version()
    prepare()
    try:
        ana = start_person("Ana", "new", "Pantry Run", "--as", PEOPLE["Ana"])
        link = wait(lambda: re.search(r"svrngs://join/(\S+)",
                                      (LOGS / "Ana.out").read_text(errors="replace")), 120)
        if not link:
            raise RuntimeError("Ana printed no join link")
        for who in ["Sam", "Bo", "Cy", "Dee"]:
            join(who, link.group(0))
        for who in ["Ana", "Sam", "Bo", "Cy", "Dee"]:
            if not group(who):
                raise RuntimeError(f"{who} is in no group")
        save("setup/group.txt", group("Ana"))
        servers()
        lends("Ana")
        # Ana: the code model and the vision model.
        share("Ana", 8201, "llama-server", "ana-coder", limit=1,
              refuse=["/slots", "/v1/stream", "/v1/streams", "/v1/chat/completions/control",
                      "/lora-adapters", "/metrics", "/tools", "/cors-proxy", "/models"])
        share("Ana", 8202, "llama-server", "ana-vision",
              refuse=["/slots", "/v1/stream", "/v1/streams", "/v1/chat/completions/control",
                      "/lora-adapters", "/metrics", "/tools", "/cors-proxy", "/models"])
        # Sam: whisper through the wrapper, Kokoro at the limit of one, and a tool.
        share("Sam", 8210, "whisper-server", "sam-stt")
        share("Sam", 8207, "kokoro", "sam-voice", refuse=["/docs", "/openapi.json"], limit=1)
        share("Sam", 8208, "mcp", "sam-tools")
        # Cy: llama-swap, a server none of the pantry's tests use.
        _, timed = share("Cy", 8209, "llama-swap", "cy-model",
                         refuse=["/ui", "/logs", "/running", "/unload", "/metrics"])
        save("story/provider-two-lines.json", {"serve_ms": TIMES.get("serve_ms"), "share_ms": timed,
             "total_ms": (TIMES.get("serve_ms") or 0) + timed,
             "download": "none: every weight was already under target/models/curated/ and its sha256 was checked"})
        story_run()
        save("story/pantry-hashes.json", {f: sha(REPO / "apps/pantry" / f) for f in ["pantry.mjs", "mcp.mjs"]})
    except Exception as e:
        save("story/could-not-judge.txt", f"{type(e).__name__}: {e}")
        raise
    finally:
        for name in list(procs):
            stop(name)
        subprocess.run(["podman", "rm", "-f", "pantry-run-kokoro"], capture_output=True)
        save("story/ps-after.txt", subprocess.run(["ps", "-o", "pid,command", "-ax"],
                                                  capture_output=True, text=True).stdout)


if __name__ == "__main__":
    main()
