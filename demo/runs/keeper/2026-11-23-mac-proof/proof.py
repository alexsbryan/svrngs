"""On beefymac: /usr/bin/python3 proof.py <scratch keeper> lineage|ownership|managed."""
from concurrent.futures import ThreadPoolExecutor
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import plistlib
import re
import socket
import subprocess
import sys
from threading import Event
import time
from urllib.request import urlopen


class Reply(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def log_message(self, *_):
        pass


def until(test):
    for _ in range(120):
        if value := test():
            return value
        time.sleep(.1)
    raise AssertionError("timed out waiting for the scratch agent")


def proof(keeper, case):
    scratch = Path(__file__).resolve().parent
    assert scratch == Path.home() / "svrngs-keeper-scratch"
    home, data = scratch / "home", scratch / "data"
    name = f"test-mac-proof-{case}"
    label = f"svrngs-keeper-{name}"
    domain = f"gui/{os.getuid()}"
    target = f"{domain}/{label}"
    live = home / "Library/LaunchAgents" / f"{label}.plist"
    parked = live.with_suffix(".stopped")
    card = data / "svrngs-keeper" / name / "compose.yaml"
    env = dict(os.environ, HOME=str(home), SVRNGS_DIR=str(scratch / "node"),
               XDG_DATA_HOME=str(data), RUST_LOG="debug")

    def call(*args, check=True):
        return subprocess.run(args, env=env, cwd=scratch, text=True,
                              capture_output=True, timeout=45, check=check)

    def ctl(*args, check=True):
        return call("/bin/launchctl", *args, check=check)

    def show():
        result = ctl("print", target, check=False)
        assert result.returncode in (0, 113), result.stderr
        return result

    def registered():
        fields = dict(re.findall(r"^\t(path|state|pid|runs) = (.+)$", show().stdout, re.M))
        return fields if fields.get("state") == "running" and "pid" in fields else None

    @contextmanager
    def fixture():
        assert show().returncode == 113
        assert not any(p.exists() or p.is_symlink() for p in (live, parked, card))
        try:
            yield
        finally:
            result = show()
            if result.returncode == 0:
                assert f"\tpath = {live}\n" in result.stdout, result.stdout
                ctl("bootout", target)
                until(lambda: show().returncode == 113)
            for path in (live, parked, card):
                path.unlink(missing_ok=True)
            print(f"cleanup {target}: print exit={show().returncode}; no test plist", flush=True)

    def run(port, *command):
        result = call(keeper, "run", name, "--json", "--port", str(port), "--", *command)
        for line in result.stderr.splitlines():
            if "installed agent and port observed" in line:
                print(re.sub(r"\x1b\[[0-9;]*m", "", line), flush=True)
        return json.loads(result.stdout)

    if case == "lineage":
        with fixture(), HTTPServer(("127.0.0.1", 0), Reply, bind_and_activate=False) as foreign:
            foreign.server_bind()  # Reserve a port, but do not listen before the busy checks.
            foreign.timeout = .2
            port = foreign.server_port
            finished = Event()

            def answer_after_launch():
                before = until(registered)
                foreign.server_activate()
                while not finished.is_set():
                    foreign.handle_request()
                return before

            with ThreadPoolExecutor(1) as pool:
                answering = pool.submit(answer_after_launch)
                try:
                    verdict = run(port, "/bin/sleep", "180")
                    with urlopen(f"http://127.0.0.1:{port}", timeout=2) as reply:
                        status = reply.status
                    holder = call("/usr/sbin/lsof", "-nP", "-a", f"-iTCP:{port}", "-sTCP:LISTEN", "-t")
                finally:
                    finished.set()
                before = answering.result()
            print(f"foreign HTTP {status}: listener={holder.stdout.strip()} agent={before['pid']} "
                  f"state={before['state']} verdict={json.dumps(verdict)}", flush=True)
            assert status == 200 and holder.stdout.strip() == str(os.getpid()) != before["pid"]
            assert before == registered() and before["path"] == str(live)
            assert verdict == {"kind": "unverified", "state": "running", "ready": True, "port": port}, verdict

    elif case == "ownership":
        failures = []
        for verb in ("stop", "start", "restart", "remove"):
            with fixture():
                live.parent.mkdir(parents=True, exist_ok=True)
                plist = plistlib.dumps(dict(Label=label, ProgramArguments=["/bin/sleep", "180"], RunAtLoad=True))
                live.write_bytes(plist)
                ctl("bootstrap", domain, live)
                before = until(registered)
                assert before["path"] == str(live) and not card.exists()
                result = call(keeper, verb, name, check=False)
                after = registered()
                lines = (result.stdout or result.stderr).splitlines()
                why = next((line for line in lines if line.startswith("svrngs-keeper:")), lines[-1])
                print(f"{verb}: exit={result.returncode} {why}\n  before={before}\n  after={after}", flush=True)
                if not (result.returncode == 2 and "saved card" in why and before == after
                        and live.is_file() and live.read_bytes() == plist and not parked.exists() and not card.exists()):
                    failures.append(verb)
        assert not failures, f"unmanaged same-path registration accepted: {failures}"

    elif case == "managed":
        with fixture():
            with socket.socket() as probe:
                probe.bind(("127.0.0.1", 0))
                port = probe.getsockname()[1]
            verdict = run(port, "/usr/bin/python3", __file__, "serve", str(port))
            print(f"managed HTTP: {json.dumps(verdict)}", flush=True)
            assert verdict == {"kind": "verified", "unit": label, "port": port}
            before = until(registered)
            assert card.is_file()
            moved = live.with_suffix(".proof")
            live.rename(moved)
            try:
                live.symlink_to(moved)
                result = call(keeper, "start", name, check=False)
                print(f"symlink start: exit={result.returncode}; agent PID={registered()['pid']}", flush=True)
                assert result.returncode == 2 and "not a regular file" in result.stderr and before == registered()
            finally:
                live.unlink(missing_ok=True)
                moved.rename(live)
            call(keeper, "stop", name)
            assert show().returncode == 113 and parked.is_file() and not live.exists()
            call(keeper, "start", name)
            resumed = until(registered)
            assert resumed["pid"] != before["pid"] and live.is_file() and not parked.exists()
            call(keeper, "remove", name)
            assert show().returncode == 113 and card.is_file() and not live.exists() and not parked.exists()
            print(f"managed stop/start/remove: {before['pid']} -> {resumed['pid']}; card retained", flush=True)
    else:
        raise ValueError(case)


if sys.argv[1] == "serve":
    with HTTPServer(("127.0.0.1", int(sys.argv[2])), Reply) as server:
        server.serve_forever()
else:
    proof(sys.argv[1], sys.argv[2])
