#!/usr/bin/env python3
"""The examples ladder, from release v1.11.1 and the guide alone (PLAN §1,
"Examples, a ladder", unit 4). PREDICTIONS.md was committed before this ran.

Ana and Bo are programs on this host, each with a HOME and an SVRNGS_DIR of
their own. The binary is the release's, checked against its SHA256SUMS. The
guide's lines and Spot's are read from the files a person reads, not
retyped here. One host, one run: not a rate.

    python3 drive.py          the whole run
    python3 drive.py stop     stop whatever is still running
"""
import hashlib
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
DATA = HERE / "data"
RESP, HOMES, LOGS, WORK, REL = (DATA / d for d in ("responses", "home", "logs", "work", "release"))
BIN = REL / "svrngs"
NODE = Path(shutil.which("node")).parent
PROVIDER, SPOT, KEY = 8311, 8090, "examples-run-q3Lx"
procs = {}


def save(name, body):
    RESP.mkdir(parents=True, exist_ok=True)
    text = body if isinstance(body, str) else json.dumps(body, indent=2, ensure_ascii=False)
    text = text.replace(KEY, "<the key>").replace(str(HERE), "<this run>")
    (RESP / name).write_text(re.sub(r"svrngs://join/\S+", "svrngs://join/<token>", text) + "\n")


def env(who):
    home = HOMES / who
    return {**os.environ, "HOME": str(home), "USER": who, "NO_COLOR": "1", "SVRNGS_DIR": str(home / ".svrngs"),
            "PATH": f"{REL}:{NODE}:/usr/bin:/bin", "RUST_LOG": "info,share=debug,door=debug"}


def start(name, argv, who=None, cwd=None):
    if who:
        (HOMES / who).mkdir(parents=True, exist_ok=True)
    out, err = open(LOGS / f"{name}.out", "w"), open(LOGS / f"{name}.err", "w")
    p = subprocess.Popen(argv, env=env(who) if who else os.environ, cwd=cwd, stdin=subprocess.DEVNULL,
                         stdout=out, stderr=err, start_new_session=True)
    (LOGS / f"{name}.pid").write_text(str(p.pid))
    procs[name] = p


def stop(name):
    pidfile = LOGS / f"{name}.pid"
    p = procs.pop(name, None)
    pid = p.pid if p else (int(pidfile.read_text()) if pidfile.exists() else None)
    if pid is not None:
        for sig in (signal.SIGINT, signal.SIGKILL):
            try:
                os.killpg(os.getpgid(pid), sig)
                if p:
                    p.wait(20)
                break
            except (ProcessLookupError, subprocess.TimeoutExpired):
                continue
    pidfile.unlink(missing_ok=True)


def wait(cond, bound, every=0.5):
    t0 = time.time()
    while time.time() - t0 < bound:
        try:
            if got := cond():
                return got
        except Exception:
            pass
        time.sleep(every)
    return None


def run(who, argv, **kw):
    return subprocess.run(argv, env=env(who), capture_output=True, text=True, timeout=kw.pop("timeout", 120), **kw)


def groups(who):
    r = run(who, ["curl", "-s", "-H", f"Authorization: Bearer {(HOMES / who / '.svrngs/token').read_text().strip()}",
                  (HOMES / who / ".svrngs/api").read_text().strip() + "/v1/groups"])
    return json.loads(r.stdout)["groups"]


def node(script, *args, who=None):
    r = subprocess.run(["node", str(HERE / script), *map(str, args)], capture_output=True, text=True, timeout=600,
                       env={**os.environ, "KEY": KEY})
    lines = r.stdout.strip().splitlines()
    return json.loads(lines[-1]) if lines else {"error": r.stderr[-400:]}


def block(text, after):
    """The first fenced block after `after` in `text`: a line as a person copies it."""
    return text[text.index(after):].split("```")[1].split("\n", 1)[1].strip()


def main():
    if sys.argv[1:] == ["stop"]:
        for pidfile in sorted(LOGS.glob("*.pid")):
            stop(pidfile.stem)
        return
    for d in (RESP, HOMES, LOGS, WORK):
        shutil.rmtree(d, ignore_errors=True)
        d.mkdir(parents=True)
    seen = {}
    try:
        sums = (REL / "SHA256SUMS").read_text()
        tar = hashlib.sha256((REL / "svrngs-linux-x86_64.tar.gz").read_bytes()).hexdigest()
        seen["release"] = {"tarball sha256 listed": tar in sums, "version": run("Ana", [str(BIN), "version"]).stdout.strip()}
        os.environ["KEY"] = KEY  # the stand-in answers only this key
        start("provider", ["node", str(HERE / "provider.mjs"), str(LOGS / "provider-seen.jsonl"), str(PROVIDER)])
        start("Ana", [str(BIN), "new", "Elm", "--as", "ana"], who="Ana")
        link = wait(lambda: re.search(r"svrngs://join/\S+", (LOGS / "Ana.out").read_text()), 120)
        if not link or not wait(lambda: groups("Ana"), 120):
            raise RuntimeError("Ana made no group")
        start("Bo", [str(BIN), "join", link.group(0), "--as", "bo"], who="Bo")
        if not wait(lambda: groups("Bo") and groups("Bo")[0]["id"] == groups("Ana")[0]["id"], 180):
            raise RuntimeError("Bo did not join Elm")
        # An open is spent once used and lasts a minute: a fresh one each time.
        def opened(who):
            r = run(who, [str(BIN), "open", "--print"])
            return re.search(r"http://\S+", r.stdout + r.stderr).group(0)

        # Rung 2, open one up.
        s = seen["1-2 sandbox"] = node("ana.mjs", "sandbox", opened("Ana"), RESP)
        folder = Path(re.search(r" at (/\S+)\s*$", s.get("folder", "") or "").group(1)) if s.get("folder") else None
        elm = Path(re.search(r" at (/\S+)\s*$", s.get("elmFolder", "") or "").group(1)) if s.get("elmFolder") else None
        money = folder / "apps/Money" if folder else None
        seen["2 index.js there"] = bool(money and (money / "index.js").exists())
        if seen["2 index.js there"]:
            js = money / "index.js"
            js.write_text(js.read_text().replace("'Add what you paid'", "'Add what you paid for'", 1))
            seen["3 opened again"] = node("ana.mjs", "money", opened("Ana"), RESP)
            c = run("Ana", ["svrngs", "check", str(money)])
            seen["4 check"] = {"exit": c.returncode, "said": (c.stdout + c.stderr).strip()[-600:]}
            if elm:
                shutil.copytree(money, elm / "apps/Money")
                seen["5 bo"] = node("bo.mjs", "money", opened("Bo"), RESP)

        # Rung 3, run one: the guide's lines as a person copies them.
        guide = (REPO / "docs/guide/examples.md").read_text()
        lines = block(guide, "2. Get the examples")
        got = run("Ana", ["bash", "-c", lines + "\npwd"], cwd=WORK, timeout=300)
        tree = Path(got.stdout.strip().splitlines()[-1]) if got.returncode == 0 and got.stdout.strip() else None
        readme = (tree / "apps/spot/README.md").read_text() if tree else ""
        seen["6 run one"] = {"node": run("Ana", ["node", "--version"]).stdout.strip(), "lines exit": got.returncode,
                             "at": tree.name if tree else got.stderr[-400:],
                             "spot there": bool(tree and (tree / "apps/spot/server.mjs").exists()),
                             "examples.md there": bool(tree and (tree / "docs/guide/examples.md").exists()),
                             "README has Run it": "## Run it" in readme}
        if tree:
            # Spot's own line from the README of this version, the provider the stand-in.
            line = next(l.strip() for l in readme.splitlines() if l.strip().startswith("node apps/spot/server.mjs"))
            line = line.split(" & ")[0].replace("https://api.deepinfra.com/v1/openai", f"http://127.0.0.1:{PROVIDER}/v1/openai")
            seen["7 spot line"] = line
            with socket.socket() as probe:
                if probe.connect_ex(("127.0.0.1", SPOT)) == 0:
                    raise RuntimeError(f"port {SPOT} is taken on this host")
            start("spot", ["bash", "-c", line], who="Ana", cwd=tree)
            if wait(lambda: "spot:" in (LOGS / "spot.out").read_text(), 30):
                seen["7 lend"] = node("ana.mjs", "lend", opened("Ana"), RESP, SPOT, KEY)
                seen["8 bo"] = node("bo.mjs", "chat", opened("Bo"), RESP)
                sent = [json.loads(l) for l in (LOGS / "provider-seen.jsonl").read_text().splitlines()] if (LOGS / "provider-seen.jsonl").exists() else []
                seen["8 provider saw the key"] = [s["headers"].get("authorization") == f"Bearer {KEY}" for s in sent]
            else:
                seen["7 spot"] = (LOGS / "spot.err").read_text()[-600:]
    except Exception as e:
        seen["stopped"] = str(e)
    finally:
        for name in ["spot", "Bo", "Ana", "provider"]:
            stop(name)
        save("seen.json", seen)
    print(json.dumps(seen, indent=2, ensure_ascii=False).replace(KEY, "<the key>"))


if __name__ == "__main__":
    main()
