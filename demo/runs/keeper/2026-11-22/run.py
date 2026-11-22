#!/usr/bin/env python3
"""keeper-demo's run (README.md here): Sam's scene in a Fedora 45 VM. Sam and
Alex are two accounts, each with a node, meeting at a local `svrngs relay`.
llama-server in Sam's terminal, `svrngs share`, `svrngs-keeper keep 8101
--lend coder`; then three no-login reboots, a kill -9, `svrngs unshare` and a
reboot, and `svrngs-keeper stop` and a reboot. Alex's request is timed each
time, from Alex's account (stage/ask.py).

    BASE=<Fedora-Cloud-Base-Generic-45-1.4.x86_64.qcow2> MODEL=<a gguf> \
    BIN=<dir with svrngs and svrngs-keeper> W=<scratch> python3 run.py

Data in run/ beside this file, the group's link never in it. scripts/check.sh
runs none of this."""
import json, os, re, select, shutil, subprocess, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent; OUT = HERE / "run"
W, BIN = Path(os.environ["W"]), Path(os.environ["BIN"])
PORT = 2223
SSH = ["-i", str(W / "key"), "-p", str(PORT), "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null",
       "-o", "LogLevel=ERROR", "-o", "ConnectTimeout=5"]
SCP = ["-P" if a == "-p" else a for a in SSH]
hidden, cases, typed = [], {}, []


def say(event, **kw):
    line = json.dumps({"t": round(time.time(), 3), "event": event, **kw})
    for h in hidden: line = line.replace(h, "<the link>")
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def verdict(case, v, **why):
    assert v in ("passed", "failed", "could-not-judge", "never-ran")
    cases[case] = {"verdict": v, **why}; say("case", case=case, verdict=v, **why)


def root(cmd, timeout=600):
    r = subprocess.run(["ssh", *SSH, "root@127.0.0.1", cmd], capture_output=True, text=True, timeout=timeout)
    return r.stdout + r.stderr


def alex(cmd, timeout=600): return root(f"systemd-run -M alex@.host --user --wait --pipe -q {cmd}", timeout)


def terminal(cmd, bound=300):
    """`cmd` typed in Sam's terminal (ssh -tt, a login): each "Type y" answered
    y, and share's password question Enter alone, for none."""
    typed.append(cmd)
    p = subprocess.Popen(["ssh", *SSH, "-tt", "sam@127.0.0.1", cmd], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.STDOUT)
    said, answered, t0 = b"", 0, time.time()
    while time.time() - t0 < bound:
        if select.select([p.stdout], [], [], 0.5)[0]:
            chunk = os.read(p.stdout.fileno(), 65536)
            if not chunk: break
            said += chunk
        asks = said.count(b"Type y") + said.count(b"(Enter alone for none)")
        if asks > answered:
            last = max(said.rfind(b"Type y"), said.rfind(b"(Enter alone for none)"))
            reply = b"y\n" if said[last:].startswith(b"Type y") else b"\n"
            answered += 1; time.sleep(0.3); p.stdin.write(reply); p.stdin.flush()
    if p.poll() is None: p.kill()
    code = p.wait(30)
    text = said.decode(errors="replace").replace("\r", "")
    say("terminal", cmd=cmd, exit=code, answered=answered, said=text)
    return code, text


def asked(name):
    return [json.loads(l) for l in root(f"cat /home/alex/asks/{name}").splitlines() if l.startswith("{")]


def ask(name, bound):
    alex(f"/usr/bin/python3 /home/alex/ask.py --out /home/alex/asks/{name} --bound {bound}", bound + 60)
    got = asked(name); say("asked", name=name, lines=got); return got


def answered(lines): return next((l for l in lines if l["what"] == "answered"), None)


def ssh_up(bound=180):
    t0 = time.time()
    while time.time() - t0 < bound:
        if "up" in root("echo up", 10): return True
        time.sleep(2)
    return False


def qemu():
    return subprocess.Popen(["qemu-system-x86_64", "-enable-kvm", "-cpu", "host", "-smp", "4", "-m", "4096", "-display", "none",
                             "-drive", f"file={W / 'disk.qcow2'},if=virtio", "-cdrom", str(W / "seed.iso"),
                             "-netdev", f"user,id=n0,hostfwd=tcp:127.0.0.1:{PORT}-:22", "-device", "virtio-net-pci,netdev=n0",
                             "-serial", f"file:{W / 'serial.log'}"], stdin=subprocess.DEVNULL)


def vm():
    """The VM, its cloud-init done. The first time, the disk just after
    cloud-init is kept as prepared.qcow2 (its packages fetched, nothing of
    the run's yet), and a run again starts from a copy of it."""
    (W / "disk.qcow2").unlink(missing_ok=True)
    if (W / "prepared.qcow2").exists():
        shutil.copy(W / "prepared.qcow2", W / "disk.qcow2"); say("disk", from_="prepared.qcow2")
        q = qemu()
        if not ssh_up(300): sys.exit("the VM never answered ssh")
        return q
    for f in ("seed.iso", "key", "key.pub"): (W / f).unlink(missing_ok=True)
    subprocess.run(["qemu-img", "create", "-q", "-f", "qcow2", "-F", "qcow2", "-b", os.environ["BASE"], str(W / "disk.qcow2"), "20G"], check=True)
    subprocess.run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", "keeper-demo-vm", "-f", str(W / "key")], check=True)
    seed = W / "seed"; seed.mkdir(exist_ok=True)
    (seed / "user-data").write_text((HERE / "stage/user-data.in").read_text().replace("@KEY@", (W / "key.pub").read_text().strip()))
    shutil.copy(HERE / "stage/meta-data", seed / "meta-data")
    subprocess.run(["genisoimage", "-quiet", "-output", str(W / "seed.iso"), "-volid", "cidata", "-joliet", "-rock",
                    str(seed / "user-data"), str(seed / "meta-data")], check=True)
    q = qemu()
    if not ssh_up(300): sys.exit("the VM never answered ssh")
    said = root("cloud-init status --wait --long", 1200).strip().splitlines()
    say("cloud-init", said=[l for l in said if not l.startswith(".")])
    root("systemctl poweroff", 10); q.wait(120)
    shutil.copy(W / "disk.qcow2", W / "prepared.qcow2"); say("disk", kept="prepared.qcow2")
    q = qemu()
    if not ssh_up(300): sys.exit("the VM never answered ssh")
    return q


def collect(tag):
    """What sam's side shows now: its sessions, the kept unit, the door's lines this boot."""
    got = {"sessions": root("loginctl list-sessions --no-legend").strip().splitlines(),
           "unit": root("systemctl --user -M sam@.host show svrngs-keeper-coder -p ActiveState -p SubState -p NRestarts "
                        "-p ActiveEnterTimestampMonotonic -p UnitFileState").split(),
           "straight": root("curl -s -o /dev/null -w '%{http_code}' -m 5 http://127.0.0.1:8101/health").strip(),
           "boot": root("systemd-analyze | head -1").strip()}
    door = root("journalctl -b -q -o short-monotonic _UID=1000 _SYSTEMD_USER_UNIT=svrngs.service | grep -E 'door: a kept|door: held|renew|lend: ' | tail -40")
    (OUT / f"{tag}-door.log").write_text(door)
    say("collect", tag=tag, **got); return got


def no_login(sessions): return not any(" sam " in f" {s} " and "manager" not in s for s in sessions)


def reboot(n, bound=420):
    root("systemctl reboot", 20); time.sleep(10)
    if not ssh_up(): return None, None
    t0 = time.time()
    while time.time() - t0 < bound:
        lines = asked(f"boot-{n}.jsonl")
        if lines and lines[-1]["what"] == "end": break
        time.sleep(3)
    say("asked", name=f"boot-{n}.jsonl", lines=lines)
    return lines, collect(f"boot-{n}")


def main():
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(); W.mkdir(parents=True, exist_ok=True)
    head = subprocess.run(["git", "-C", str(HERE), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("start", head=head, base=os.environ["BASE"], model=os.environ["MODEL"])
    q = vm()
    try:
        root("mkdir -p /root/stage")
        files = [BIN / "svrngs", BIN / "svrngs-keeper", *[p for p in (HERE / "stage").iterdir() if p.name != "user-data.in"]]
        subprocess.run(["scp", *SCP, *map(str, files), "root@127.0.0.1:/root/stage/"], check=True)
        subprocess.run(["scp", *SCP, os.environ["MODEL"], "root@127.0.0.1:/root/stage/model.gguf"], check=True)
        say("setup", said=root("bash /root/stage/setup.sh 2>&1").strip().splitlines(),
            os=root("cat /etc/fedora-release; uname -r; systemctl --version | head -1").strip().splitlines())

        # Sam's terminal: svrngs at login, the group, llama-server, the lend.
        code, said = terminal("systemctl --user enable --now svrngs.service; sleep 3; svrngs new house --as Sam --relay http://127.0.0.1:7900")
        link = re.search(r"svrngs://join/\S+", said)
        if not link: sys.exit("no link")
        hidden.append(link[0])
        say("join", said=alex(f"/usr/local/bin/svrngs join {link[0]} --as Alex", 120).strip()[-400:].replace(link[0], "<the link>"))
        llama = subprocess.Popen(["ssh", *SSH, "-tt", "sam@127.0.0.1", "llama-server -m ~/models/qwen2.5-0.5b.gguf --port 8101 -c 4096"],
                                 stdin=subprocess.PIPE, stdout=open(OUT / "terminal-llama.log", "w"), stderr=subprocess.STDOUT)
        typed.append("llama-server -m ~/models/qwen2.5-0.5b.gguf --port 8101 -c 4096")
        for _ in range(120):
            if root("curl -s -m 2 http://127.0.0.1:8101/health").strip().startswith('{"status":"ok"'): break
            time.sleep(1)
        code, said = terminal("svrngs share 8101 --program llama-server --name coder --group house")
        say("alex's play", said=alex("/usr/bin/python3 /home/alex/ask.py --play --out /home/alex/asks/play.jsonl --bound 60", 120),
            lines=asked("play.jsonl"))
        got = answered(ask("terminal.jsonl", 90))
        verdict("lent from the terminal", "passed" if got else "failed", answered=got)

        # The promotion: one command, Sam's yes to each question.
        code, said = terminal("svrngs-keeper keep 8101 --lend coder")
        unit = collect("promoted")
        ok = code == 0 and "ActiveState=active" in unit["unit"] and "each time it starts again" in said
        verdict("promoted", "passed" if ok else "failed", exit=code, unit=unit["unit"])
        try: llama.wait(30); say("terminal llama-server ended", exit=llama.returncode)
        except subprocess.TimeoutExpired: say("terminal llama-server still running"); llama.kill()
        got = answered(ask("promoted.jsonl", 120))
        verdict("lent after the promotion, nothing approved again", "passed" if got else "failed", answered=got)
        promoted_typed = len(typed)
        time.sleep(5)
        say("sessions before the boots", sessions=root("loginctl list-sessions --no-legend").strip().splitlines())

        # Three reboots with nobody logged in.
        for n in (1, 2, 3):
            lines, side = reboot(n)
            if lines is None: verdict(f"boot {n}", "never-ran", why="the VM did not come back"); continue
            got, start = answered(lines), lines[0]
            sam_in = not no_login(start.get("sessions", []))
            v = "passed" if got and not sam_in and len(typed) == promoted_typed else "failed"
            verdict(f"boot {n}: Alex's link opens, Sam typed nothing", v, answered_at_uptime=got and got["uptime"],
                    request_ms=got and got["ms"], chat=next((l for l in lines if l["what"] == "chat"), None),
                    sessions_at_boot=start.get("sessions"), boot=side["boot"], unit=side["unit"])

        # kill -9 of the kept llama-server.
        k = root("P=$(systemctl --user -M sam@.host show -p MainPID --value svrngs-keeper-coder); cut -d' ' -f1 /proc/uptime; kill -9 $P; echo $P").split()
        lines = ask("kill.jsonl", 120); got = answered(lines); side = collect("kill")
        verdict("kill -9: restarted and renewed", "passed" if got and "NRestarts=1" in side["unit"] else "failed",
                killed_at_uptime=float(k[0]), pid=k[-1], answered_after_s=got and round(got["uptime"] - float(k[0]), 2),
                request_ms=got and got["ms"], unit=side["unit"])

        # Stop lending: the lend stays stopped; coder keeps running for Sam.
        code, said = terminal("svrngs unshare coder")
        lines = ask("unshared.jsonl", 30); side = collect("unshared")
        verdict("unshare: Alex is answered nothing, coder runs for Sam", "passed" if not answered(lines) and side["straight"] == "200" else "failed",
                last=lines[-2] if len(lines) > 1 else None, straight=side["straight"])
        lines, side = reboot(4, bound=300)
        if lines is None: verdict("boot 4 after unshare", "never-ran", why="the VM did not come back")
        else: verdict("boot 4 after unshare: still not lent, still running", "passed" if not answered(lines) and side["straight"] == "200" else "failed",
                      last=lines[-2] if len(lines) > 1 else None, straight=side["straight"], unit=side["unit"])

        # A stop stays stopped, at boot too.
        code, said = terminal("svrngs-keeper stop coder")
        root("systemctl reboot", 20); time.sleep(10)
        if not ssh_up(): verdict("stop stays stopped at boot", "never-ran", why="the VM did not come back")
        else:
            time.sleep(20); side = collect("boot-5")
            verdict("stop stays stopped at boot", "passed" if "ActiveState=inactive" in side["unit"] and side["straight"] == "000" else "failed",
                    unit=side["unit"], straight=side["straight"])
    finally:
        (OUT / "summary.json").write_text(json.dumps({"head": head, "cases": cases, "typed_as_sam": typed}, indent=1))
        say("end", verdicts={c: v["verdict"] for c, v in cases.items()})
        root("systemctl poweroff", 10)
        try: q.wait(60)
        except subprocess.TimeoutExpired: q.kill()


if __name__ == "__main__":
    main()
