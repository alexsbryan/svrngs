#!/usr/bin/env python3
"""Proof 1: run a rootless detached pod from a user unit, then really reboot."""

import importlib.util
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import time
import uuid
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPIKE = Path("/tmp/keeper-demo")
os.environ.setdefault("BIN", str(HERE.parents[3] / "target/debug"))
spec = importlib.util.spec_from_file_location("vm", HERE.parent / "2026-11-22/run.py")
vm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(vm)
vm.OUT = vm.W / "run"
vm.PORT = 2224
vm.SSH[vm.SSH.index("-p") + 1] = str(vm.PORT)
vm.SCP[vm.SCP.index("-P") + 1] = str(vm.PORT)
NAME = "svrngs-keeper-test-recipes"
UNIT = NAME + ".service"
APP = NAME + "-web"
PORT = 18081
IMAGE = "docker.io/library/python@sha256:1148dbb014615300fa7e713b2f94f550c0d1c69780c0805f84f093ceee34dd0b"
MANAGER = "systemctl --user -M sam@.host"
SAM = "systemd-run -M sam@.host --user --wait --pipe -q"


def guest(command, timeout=20, *, check=True):
    result = subprocess.run(["ssh", *vm.SSH, "root@127.0.0.1", command],
                            capture_output=True, text=True, timeout=timeout)
    if check and result.returncode:
        vm.say("guest command failed", command=command, exit=result.returncode, said=result.stderr + result.stdout)
        result.check_returncode()
    return result.stdout.strip()


def alex(*args, check=True):
    return guest("systemd-run -M alex@.host --user --wait --pipe -q " + shlex.join(args), 20, check=check)


def write(path, text):
    guest(f"install -d {shlex.quote(str(Path(path).parent))} && printf %s {shlex.quote(text)} > {shlex.quote(path)}")


def install(split):
    stage = vm.HERE / "stage"
    guest("install -d /root/stage")
    subprocess.run(["scp", *vm.SCP, str(vm.BIN / "svrngs"), str(stage / "svrngs.service"),
                    str(stage / "svrngs-relay.service"), "root@127.0.0.1:/root/stage/"], check=True, timeout=30)
    guest("install -m755 /root/stage/svrngs /usr/local/bin/ && "
          "install -m644 /root/stage/svrngs.service /etc/systemd/user/ && "
          "install -m644 /root/stage/svrngs-relay.service /etc/systemd/system/ && "
          "systemctl enable --now svrngs-relay && loginctl enable-linger sam alex")
    for user in ("sam", "alex"):
        guest(f"systemctl --user -M {user}@.host daemon-reload && "
              f"systemctl --user -M {user}@.host enable --now svrngs.service")
    if not guest("command -v podman", check=False):
        vm.say("packages", said=guest("dnf -y install podman passt", 180))
    vm.say("versions", said=guest("uname -r && rpm -q podman systemd passt"))
    vm.say("pull", said=guest(f"{SAM} podman pull --quiet {IMAGE}", 100))
    # The detached pod and mount are App::start's shape. The orderly stop is
    # only for this disposable pod on reboot; it says nothing about Proof 2.
    path = f"/home/sam/.config/systemd/user/{UNIT}"
    volume = NAME + "-data"
    cgroups, delegate = ("--cgroups=split ", "Delegate=yes") if split else ("", "")
    write(path, f"""[Unit]
Description=Scratch volume-only recipes pod
[Service]
{delegate}
ExecStart=/usr/bin/bash -ec 'podman volume create --ignore {volume}; podman pod create --name {NAME} --publish 127.0.0.1:{PORT}:8000/tcp --add-host web:127.0.0.1; podman run --detach {cgroups}--pod {NAME} --name {APP} --mount type=volume,source={volume},destination=/data -- {IMAGE} python -m http.server --directory /data 8000; exec podman wait {APP}'
ExecStop=/usr/bin/podman pod stop --time 2 {NAME}
ExecStopPost=-/usr/bin/podman pod rm {NAME}
[Install]
WantedBy=default.target
""")
    guest(f"chown -R sam:sam /home/sam/.config && {MANAGER} daemon-reload")
    guest(f"{MANAGER} enable --now {UNIT}", check=False)


def sample(tag):
    # Query a byte-free dial by its exact peer port, not whichever socket ss
    # happens to list first. This reads the socket cgroup, not the PID's.
    script = f"""import socket, subprocess
with socket.create_connection(('127.0.0.1', {PORT}), timeout=2) as s:
    client = s.getsockname()[1]
    for args in (['-ltnHp', '--cgroup', 'sport', '=', ':{PORT}'],
                 ['-tnHp', '--cgroup', 'state', 'established', 'sport', '=', ':{PORT}', 'dport', '=', ':' + str(client)]):
        subprocess.run(['ss', *args], check=True)
"""
    cg = guest(f"{MANAGER} show -p ControlGroup --value {UNIT}")
    vm.say("unit", tag=tag, said=guest(f"{MANAGER} show -p ActiveState -p ControlGroup -p Delegate -p MainPID {UNIT}"))
    result = guest("python3 -c " + shlex.quote(script), check=False).splitlines()
    journal = guest(f"journalctl -b -q _UID=1000 _SYSTEMD_USER_UNIT={UNIT} --no-pager")
    (vm.OUT / f"{tag}-unit.log").write_text(journal + "\n")
    paths = [line.split(" cgroup:", 1)[1].split()[0] for line in result if " cgroup:" in line]

    def inside(unit):
        return bool(unit) and len(paths) == 2 and all(p == unit or p.startswith(unit + "/") for p in paths)

    if tag == "before" and len(paths) == 2:
        # Plant the wrong job by moving the expected subtree, not the socket.
        plant = cg + "/wrong-job"
        vm.say("planted subtree", expected=plant, sockets=result, accepted=inside(plant))
        assert not inside(plant)
    vm.say("sockets", tag=tag, sockets=result, in_unit=inside(cg))
    return inside(cg)


def wait_for(url, at=guest, bound=16):
    deadline = time.monotonic() + bound
    command = ("curl", "-sS", "-m", "2", "-w", "\n%{http_code}", url)
    while True:
        said = at(command)
        if said.endswith("\n200") or time.monotonic() >= deadline:
            return said
        time.sleep(0.5)


def direct(args):
    return guest(shlex.join(args), check=False)


def declaration():
    return json.loads(guest("cat /home/sam/.svrngs/lends/recipes.json"))


def scene(split):
    if not wait_for(f"http://127.0.0.1:{PORT}/", direct).endswith("\n200"):
        sample("start-failed")
        vm.verdict("pod starts", "failed", unit_log="start-failed-unit.log")
        return False
    initial = sample("before")
    marker = uuid.uuid4().hex
    guest(f"{SAM} podman exec {APP} sh -c 'echo {marker} > /data/marker'")
    code, said = vm.terminal("svrngs new house --as Sam --relay http://127.0.0.1:7900")
    link = re.search(r"svrngs://join/\S+", said)
    assert code == 0 and link
    alex("svrngs", "join", link[0], "--as", "Alex")
    code, _ = vm.terminal(f"RUST_LOG=door=debug,job=debug svrngs share {PORT} --name recipes --program recipes --group house")
    assert code == 0
    before = declaration()
    vm.say("before declaration", keep=before.get("keep"), approved=before.get("approved"))
    groups = json.loads(alex("svrngs", "ask", "/v1/groups"))["groups"]
    lends = json.loads(alex("svrngs", "ask", "/v1/lends?namespace=" + groups[0]["id"]))["lent"]
    row = next(lend for lend in lends if lend["name"] == "recipes")
    alex("svrngs", "ask", "/v1/play", json.dumps({"on": {"key": row["key"], "name": "recipes"}}))
    url = row["origin"].rstrip("/") + "/marker"
    expected = marker + "\n\n200"
    seen = wait_for(url, lambda args: alex(*args, check=False))
    vm.verdict("Alex reads the marker before reboot", "passed" if seen == expected else "failed", url=url, body=seen)
    assert seen == expected
    boot = guest("cat /proc/sys/kernel/random/boot_id")
    vm.root("systemctl reboot", 15)
    time.sleep(6)
    assert vm.ssh_up(40)
    assert guest("cat /proc/sys/kernel/random/boot_id") != boot
    sessions = guest("loginctl list-sessions --no-legend").splitlines()
    local = wait_for(f"http://127.0.0.1:{PORT}/marker", direct)
    fresh = sample("after")
    seen = wait_for(url, lambda args: alex(*args, check=False) if local == expected else "", bound=5)
    after = declaration()
    door = guest("journalctl -b -q _UID=1000 _SYSTEMD_USER_UNIT=svrngs.service --no-pager "
                 "-g 'unit renewed|what is on the lend.s port'")
    (vm.OUT / "after-door.log").write_text(door + "\n")
    renewed = ((before.get("keep") or {}).get("unit") == UNIT and before.get("keep") == after.get("keep")
               and after.get("approved") != before.get("approved") and "renewed=Ok(())" in door)
    ok = initial and fresh and local == expected and seen == expected and renewed and vm.no_login(sessions)
    vm.verdict("same marker at same address after reboot", "passed" if ok else "failed", split=split,
               url=url, boot_before=boot, boot_after=guest("cat /proc/sys/kernel/random/boot_id"),
               sessions=sessions, expected=marker, local=local, alex=seen,
               renewed=renewed, approved=after.get("approved"))
    return ok


def main():
    vm.W.mkdir()
    vm.OUT.mkdir()
    for name in ("key", "seed.iso"):
        (vm.W / name).symlink_to(SPIKE / name)
    disk = vm.W / "disk.qcow2"
    shutil.copy(SPIKE / "prepared.qcow2", disk)
    qemu = vm.qemu()
    try:
        assert vm.ssh_up(40), "the scratch VM did not answer SSH"
        split = "--split" in sys.argv[1:]
        install(split)
        return 0 if scene(split) else 1
    finally:
        try:
            vm.root("systemctl poweroff", 10)
            qemu.wait(15)
        except subprocess.TimeoutExpired:
            qemu.kill()
            qemu.wait(10)
        disk.unlink()


if __name__ == "__main__":
    sys.exit(main())
