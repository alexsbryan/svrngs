#!/usr/bin/env python3
"""Replay the spike VM with separate Run and Share. Set W to a new scratch directory."""
import importlib.util
import json
import os
import re
import shutil
import subprocess
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPIKE = Path("/tmp/keeper-demo")
spec = importlib.util.spec_from_file_location("keeper_demo", HERE.parent / "2026-11-22" / "run.py")
demo = importlib.util.module_from_spec(spec)
os.environ.setdefault("BIN", str(HERE.parents[3] / "target/debug"))
spec.loader.exec_module(demo)
W = demo.W
demo.OUT = W / "run"
root, alex, terminal = demo.root, demo.alex, demo.terminal
unit = "svrngs-keeper-coder.service"
manager = "systemctl --user -M sam@.host"
state = "/home/sam/.svrngs"
start = "llama-server --model /home/sam/models/qwen2.5-0.5b.gguf --host 127.0.0.1 --port 8101 -c 4096"
run = "svrngs run coder --port 8101 -- /usr/bin/" + start
share = "svrngs share 8101 --program llama-server --name coder --group house"


def prove(case, passed, **facts):
    demo.verdict(case, "passed" if passed else "failed", **facts)
    assert passed, case


def origin():
    groups = json.loads(alex("/usr/local/bin/svrngs ask /v1/groups"))["groups"]
    lends = json.loads(alex("/usr/local/bin/svrngs ask /v1/lends?namespace=" + groups[0]["id"]))["lent"]
    return next(row["origin"] for row in lends if row["name"] == "coder")


def pid():
    return root(f"{manager} show --value -p MainPID {unit}").strip()


def complete(lines):
    return demo.answered(lines) and any(l["what"] == "chat" and l.get("status") == 200 for l in lines)


def scene():
    code, said = terminal("systemctl --user enable --now svrngs.service; sleep 2; svrngs new house --as Sam --relay http://127.0.0.1:7900")
    link = re.search(r"svrngs://join/\S+", said)
    assert code == 0 and link, "Sam did not make a group"
    demo.say("joined", said=alex(f"/usr/local/bin/svrngs join {link[0]} --as Alex", 100).replace(link[0], "<the link>"))
    with (demo.OUT / "terminal-llama.log").open("w") as log:
        llama = subprocess.Popen(["ssh", *demo.SSH, "-tt", "sam@127.0.0.1", "exec " + start], stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT)
        demo.typed.append(start)
        try:
            for _ in range(60):
                if '"status":"ok"' in root("curl -fsS -m 2 http://127.0.0.1:8101/health"):
                    break
                time.sleep(1)
            else:
                raise RuntimeError("the terminal copy never answered")
            assert terminal(share)[0] == 0
            alex("/usr/bin/python3 /home/alex/ask.py --play --out /home/alex/asks/play.jsonl --bound 25", 45)
            prove("Alex opens the terminal copy", complete(demo.ask("terminal.jsonl", 40)))
            saved_origin = origin()
            code, said = terminal(run)
            prove("a busy port leaves the terminal copy and its share alone", code != 0 and
                  "Port 8101 is in use by llama-server, the copy you started: stop it, then press Run." in said and
                  llama.poll() is None and pid() == "0" and origin() == saved_origin, address=saved_origin)
            llama.stdin.write(b"\x03")
            llama.stdin.flush()
            llama.wait(30)
        finally:
            if llama.poll() is None:
                llama.terminate()
                llama.wait(10)

    assert terminal("svrngs unshare coder")[0] == 0
    code, said = terminal(run)
    before = pid()
    prove("Run starts the service without writing a share", code == 0 and "Share it separately" in said and
          before.isdecimal() and before != "0" and root(f"test ! -e {state}/lends/coder.json && loginctl show-user sam -p Linger --value").strip() == "yes", pid=before)
    assert root(f"sudo -u sam mkdir {state}/lends/.coder.json.partial && echo planted").strip() == "planted"
    code, said = terminal(share)
    prove("failed save leaves the app running and unshared", code != 0 and
          "Running here; not shared." in said and ".coder.json.partial" in said and pid() == before and
          root(f"test ! -e {state}/lends/coder.json && echo absent").strip() == "absent", pid=pid())
    assert root(f"rmdir {state}/lends/.coder.json.partial && echo cleared").strip() == "cleared"
    code, said = terminal(share)
    declaration = json.loads(root(f"cat {state}/lends/coder.json"))
    prove("retry shares the same running service", code == 0 and pid() == before and
          declaration["keep"]["unit"] == unit and origin() == saved_origin and
          complete(demo.ask("managed.jsonl", 45)), pid=before, address=saved_origin)
    typed = len(demo.typed)

    boot_ids = {root("cat /proc/sys/kernel/random/boot_id").strip()}
    for n in (1, 2, 3):
        lines, side = demo.reboot(n, bound=100)
        first = lines[0] if lines else {}
        boot_id = first.get("boot_id")
        prove(f"no-login boot {n}", bool(lines and complete(lines) and boot_id and boot_id not in boot_ids and
              demo.no_login(first["sessions"]) and len(demo.typed) == typed and origin() == saved_origin),
              boot=boot_id, sessions=first.get("sessions"), answer=demo.answered(lines or []), unit=side and side["unit"])
        boot_ids.add(boot_id)

    killed = pid()
    assert root(f"{manager} kill --signal=KILL --kill-whom=main {unit} && echo sent").strip() == "sent"
    lines = demo.ask("kill.jsonl", 60)
    side = demo.collect("kill")
    prove("crash restarts and reopens the old address", complete(lines) and "NRestarts=1" in side["unit"] and
          pid() != killed and origin() == saved_origin and len(demo.typed) == typed, before=killed, after=pid(), unit=side["unit"])


def main():
    W.mkdir()
    demo.OUT.mkdir()
    for name in ("key", "seed.iso"):
        (W / name).symlink_to(SPIKE / name)
    shutil.copy(SPIKE / "prepared.qcow2", W / "disk.qcow2")
    q = demo.qemu()
    try:
        assert demo.ssh_up(90), "the scratch VM did not answer ssh"
        root("install -d /root/stage")
        files = [demo.BIN / name for name in ("svrngs", "svrngs-keeper")]
        files += [p for p in (demo.HERE / "stage").iterdir() if p.name != "user-data.in"]
        subprocess.run(["scp", *demo.SCP, *map(str, files), "root@127.0.0.1:/root/stage/"], check=True)
        subprocess.run(["scp", *demo.SCP, str(SPIKE / "qwen2.5-0.5b.gguf"), "root@127.0.0.1:/root/stage/model.gguf"], check=True)
        demo.say("setup", said=root("bash /root/stage/setup.sh 2>&1"))
        scene()
    finally:
        try:
            root("systemctl poweroff", 10)
            q.wait(20)
        except subprocess.TimeoutExpired:
            q.terminate()
            q.wait(10)
        (W / "disk.qcow2").unlink()
        (demo.OUT / "summary.json").write_text(json.dumps({"cases": demo.cases, "typed_as_sam": demo.typed}, indent=2) + "\n")
        demo.say("end", verdicts={c: v["verdict"] for c, v in demo.cases.items()})


if __name__ == "__main__":
    main()
