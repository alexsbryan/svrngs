"""Run on beefymac from ~/svrngs-keeper-scratch using its scratch build."""
import json
import os
from pathlib import Path
import re
import shlex
import signal
import socket
import subprocess
import time
from urllib.request import urlopen

scratch = Path.home() / "svrngs-keeper-scratch"
home = scratch / "home"
data = scratch / "data"
model = scratch / "qwen2.5-0.5b.gguf"
name = "test-avail-run-mac"
label = f"svrngs-keeper-{name}"
target = f"gui/{os.getuid()}/{label}"
with socket.socket() as probe:
    probe.bind(("127.0.0.1", 0))
    port = probe.getsockname()[1]
svr, keeper = (scratch / "repo/target/debug" / bin for bin in ("svrngs", "svrngs-keeper"))
active = home / "Library/LaunchAgents" / f"{label}.plist"
parked = active.with_suffix(".stopped")
card = data / "svrngs-keeper" / name
env = dict(os.environ, HOME=str(home), SVRNGS_DIR=str(scratch / "node"), XDG_DATA_HOME=str(data))


def call(*args, check=True):
    result = subprocess.run(args, cwd=scratch, env=env, text=True, capture_output=True, timeout=120)
    if check:
        assert result.returncode == 0, result.stderr or result.stdout
    return result


def show():
    return call("/bin/launchctl", "print", target, check=False)


def pid():
    match = re.search(r"^\tpid = (\d+)$", show().stdout, re.M)
    return int(match[1]) if match else None


def health():
    try:
        with urlopen(f"http://127.0.0.1:{port}/health", timeout=1) as reply:
            return reply.status == 200
    except OSError:
        return False


def until(test, message):
    for _ in range(120):
        if test():
            return
        time.sleep(.25)
    raise AssertionError(message)


def run():
    return call(svr, "run", name, "--port", str(port), "--",
                "llama-server", "-m", model.name,
                "--host", "127.0.0.1", "--port", str(port),
                "--device", "MTL0", "--gpu-layers", "99", "--verbose",
                "-c", "512", "--parallel", "1", check=False)


assert model.is_file() and show().returncode == 113
assert not active.exists() and not parked.exists() and not card.exists()
home.mkdir(exist_ok=True)
started = False
try:
    for family, address in ((socket.AF_INET, "0.0.0.0"), (socket.AF_INET, "127.0.0.1"),
                            (socket.AF_INET6, "::1")):
        with socket.socket(family) as listener:
            listener.bind((address, port))
            listener.listen()
            result = run()
            print(f"busy {address}: exit={result.returncode} {result.stderr.strip() or result.stdout.strip()}", flush=True)
            assert result.returncode and f"Port {port} is in use by" in result.stderr
            assert not active.exists() and not card.exists() and show().returncode == 113
            held = call("/usr/sbin/lsof", "-a", f"-p{os.getpid()}", f"-iTCP:{port}", "-sTCP:LISTEN", "-t")
            assert held.stdout.strip() == str(os.getpid())

    started = True
    result = run()
    print(f"run: exit={result.returncode} {result.stdout.strip() or result.stderr.strip()}", flush=True)
    assert result.returncode == 0 and "running here" in result.stdout
    assert health() and active.is_file()
    first = pid()
    assert first and call("/bin/ps", "-p", str(first), "-o", "comm=").stdout.strip().endswith("llama-server")
    prefill = json.loads(call(keeper, "facts", str(port)).stdout)
    assert prefill["kind"] == "prefill" and prefill["folder"] == str(scratch), prefill
    assert shlex.split(prefill["command"]) == [
        "llama-server", "--model", str(model), "--host", "127.0.0.1", "--port", str(port),
    ], prefill
    print(f"prefill: {model.name} resolved from relative path")
    listing = json.loads(call(keeper, "list", "--json").stdout)
    assert listing[0]["status"]["starts"] == "at_login" and listing[0]["status"]["running"] == {"since": None}
    log = (card / "program.log").read_text()
    offload = re.search(r"offloaded (\d+)/\d+ layers to GPU", log)
    assert "ggml_metal" in log and offload and int(offload[1]) > 0
    print(f"Metal: {offload[0]}, HTTP /health 200, pid {first}")

    then = time.monotonic()
    os.kill(first, signal.SIGKILL)
    until(lambda: pid() not in (None, first) and health(), "launchd did not recover the model")
    print(f"SIGKILL: {first} -> {pid()}, /health 200 after {time.monotonic() - then:.1f}s")

    print(call(keeper, "stop", name).stdout.strip())
    call(keeper, "restart", name)
    time.sleep(12)
    assert not health() and show().returncode == 113 and not active.exists() and parked.is_file()
    listing = json.loads(call(keeper, "list", "--json").stdout)
    assert listing[0]["status"]["running"] == "stopped"
    print("stop: still stopped after 12s and restart request; no login plist or port")

    call(keeper, "start", name)
    until(lambda: pid() and health(), "start did not restore HTTP")
    resumed = pid()
    call(keeper, "restart", name)
    until(lambda: pid() not in (None, resumed) and health(), "restart did not restore HTTP")
    print(f"start then restart: {resumed} -> {pid()}, HTTP 200")
    print(call(keeper, "remove", name).stdout.strip())
    until(lambda: show().returncode == 113 and not health(), "unrun did not stop the service")
    assert not active.exists() and not parked.exists() and (card / "compose.yaml").is_file() and model.is_file()
    print("unrun: agent and plists removed; card and model retained")
finally:
    if started:
        if show().returncode == 0:
            assert str(active) in show().stdout, "refusing cleanup of a different registration"
            call("/bin/launchctl", "bootout", target)
            until(lambda: show().returncode == 113, "cleanup could not unload test agent")
        active.unlink(missing_ok=True)
        parked.unlink(missing_ok=True)
