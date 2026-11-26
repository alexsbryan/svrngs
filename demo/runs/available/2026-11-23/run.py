"""Use the release's local install site, not binaries placed by the VM harness."""

import importlib.util
import json
import os
from pathlib import Path
import re
import select
import shlex
import shutil
import subprocess
import sys
import time
import uuid

HERE = Path(__file__).resolve().parent
KEEPER = HERE.parents[1] / "keeper"
spec = importlib.util.spec_from_file_location("vm_run", KEEPER / "2026-11-23-run-containers/run.py")
proof = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proof)
vm, guest = proof.vm, proof.guest
vm.SSH[3] = vm.SCP[3] = str(2270)
vm.PORT = 2270
NAME = "test-available"
PORT = 8190
proof.UNIT = f"svrngs-keeper-{NAME}.service"
LOCAL = f"http://127.0.0.1:{PORT}"
FOLDER = "/home/sam/available"
SV = "/home/sam/.local/bin/svrngs"
STATE = "/home/sam/.svrngs"
SHARE = f"{SV} share {PORT} --name {NAME} --program {NAME} --group house"
RUN = f"{SV} run {NAME} --port {PORT} --folder {FOLDER} -- /usr/bin/python3 {FOLDER}/app.py {PORT}"
CASES = ("busy port", "failed save", "concurrent withdrawal", "old link and data", "stop sharing")


def say(case, ok, **facts):
    vm.verdict(case, "passed" if ok else "failed", **facts)
    assert ok, case


def scp(*args):
    subprocess.run(["scp", *vm.SCP, *args], check=True, timeout=60)


def install(site):
    web = vm.W / "site"
    shutil.copytree(site / "get", web / "get")
    shutil.copy(KEEPER.parents[2] / "web/install.sh", web)
    scp("-r", str(web), "root@127.0.0.1:/root/available-site")
    guest("systemd-run --unit=svrngs-keeper-test-available-site /usr/bin/python3 -m http.server "
          "8187 --bind 127.0.0.1 --directory /root/available-site")
    proof.write("/var/tmp/available-opener/xdg-open", "#!/bin/sh\nexit 0\n")
    guest("chmod 755 /var/tmp/available-opener/xdg-open && loginctl enable-linger sam alex")
    for user in ("sam", "alex"):
        cmd = ('set -e -o pipefail; test ! -e "$HOME/.local/bin/svrngs"; test ! -e "$HOME/.svrngs"; '
               'export SVRNGS_DIR="$HOME/.svrngs" PATH="/var/tmp/available-opener:$PATH"; '
               'curl -fsSL http://127.0.0.1:8187/install.sh | sh -s -- http://127.0.0.1:8187/; '
               'test -x "$HOME/.local/bin/svrngs-keeper"; "$HOME/.local/bin/svrngs" version')
        install_as = f"systemd-run -M {user}@.host --user --wait --pipe -q /usr/bin/bash -c "
        out = subprocess.run(["ssh", *vm.SSH, "root@127.0.0.1", install_as + shlex.quote(cmd)],
                             capture_output=True, text=True, timeout=60)
        vm.say("install", user=user, said=out.stdout + out.stderr)
        out.check_returncode()
    proof.write("/home/sam/.config/systemd/user/svrngs.service.d/proof.conf",
                "[Service]\nEnvironment=RUST_LOG=info,door=debug\n")
    guest(f"{proof.MANAGER} daemon-reload && {proof.MANAGER} restart svrngs.service")
    relay = "svrngs-keeper-test-available-relay.service"
    text = (vm.HERE / "stage/svrngs-relay.service").read_text().replace("/usr/local/bin/svrngs", SV)
    text = text.replace("/var/lib/svrngs-relay", "/var/lib/svrngs-keeper-test-available")
    proof.write("/etc/systemd/system/" + relay, text)
    guest(f"systemctl daemon-reload && systemctl enable --now {relay}")
    guest(f"install -d -o sam -g sam {FOLDER}; install -d -o sam -g sam {STATE}/lends")
    scp(str(HERE / "app.py"), f"sam@127.0.0.1:{FOLDER}/app.py")
    scp(str(HERE / "cut.py"), "root@127.0.0.1:/root/cut.py")


def alex(*args, check=True):
    return proof.alex(*args, check=check)


def pid():
    return guest(f"{proof.MANAGER} show -p MainPID --value {proof.UNIT}")


def revision():
    return int(guest(f"cat {STATE}/locks/{NAME}.rev"))


def absent():
    guest(f"test ! -e {STATE}/lends/{NAME}.json")


def review_then(withdraw):
    """Pause at the real tty prompt, not at a guessed delay."""
    with subprocess.Popen(["ssh", *vm.SSH, "-tt", "sam@127.0.0.1", SHARE], stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT) as review:
        try:
            seen = b""
            deadline = time.monotonic() + 15
            while b"Type y, then press Enter: " not in seen:
                assert select.select([review.stdout], [], [], max(0, deadline - time.monotonic()))[0], seen
                chunk = os.read(review.stdout.fileno(), 65536)
                assert chunk, seen
                seen += chunk
            if withdraw:
                guest(f"{proof.SAM} {SV} unshare {NAME}")
                absent()
            out, _ = review.communicate(b"y\n\n", timeout=15)
            return review.returncode, (seen + out).decode().replace("\r", "")
        finally:
            if review.poll() is None:
                review.terminate()
                review.wait(5)


def scene(site):
    try:
        install(site)
        said = guest(f"{proof.SAM} {SV} new house --as Sam --relay http://127.0.0.1:7900")
        link = re.search(r"svrngs://join/\S+", said)
        assert link, "group creation did not print a link"
        alex("/home/alex/.local/bin/svrngs", "join", link[0], "--as", "Alex")
        assert vm.terminal(RUN)[0] == 0
        marker = uuid.uuid4().hex
        guest(f"{proof.SAM} /usr/bin/bash -c " + shlex.quote(f"echo {marker} > {FOLDER}/marker"))
        before = pid()
        assert int(before) > 0
        assert guest(f"curl -fsS {LOCAL}/marker") == marker
        absent()

        busy = NAME + "-busy"
        code, said = vm.terminal(RUN.replace(f"run {NAME} ", f"run {busy} "))
        loaded = guest(f"{proof.MANAGER} show -p LoadState --value svrngs-keeper-{busy}.service")
        guest(f"test ! -e /home/sam/.local/share/svrngs-keeper/{busy} && test ! -e {STATE}/lends/{busy}.json")
        absent()
        say("busy port", code != 0 and f"Port {PORT} is in use" in said and loaded == "not-found"
            and pid() == before and guest(f"curl -fsS {LOCAL}/marker") == marker,
            exit=code, pid=before, load_state=loaded, marker=marker)

        partial = f"{STATE}/lends/.{NAME}.json.partial"
        guest(f"{proof.SAM} mkdir {partial}")
        code, said = vm.terminal(SHARE)
        absent()
        assert code != 0 and "Running here; not shared." in said and f".{NAME}.json.partial" in said
        assert pid() == before and guest(f"curl -fsS {LOCAL}/marker") == marker
        guest(f"{proof.SAM} rmdir {partial}")
        retry, _ = vm.terminal(SHARE)
        say("failed save", retry == 0 and pid() == before and proof.declaration(NAME)["keep"]["unit"] == proof.UNIT,
            exit=code, retry=retry, pid=before)

        code, said = review_then(False)
        vm.say("control: no withdrawal during review", exit=code, said=said)
        assert code == 0 and proof.declaration(NAME)["keep"]["unit"] == proof.UNIT
        rev = revision()
        code, said = review_then(True)
        vm.say("withdrew during review", exit=code, said=said)
        absent()
        after = revision()
        say("concurrent withdrawal", code != 0 and "Something changed since you reviewed this." in said and after == rev + 1,
            exit=code, before=rev, after=after)
        assert vm.terminal(SHARE)[0] == 0
        prior = proof.declaration(NAME)
        args = ("/home/alex/.local/bin/svrngs", "ask")
        group = json.loads(alex(*args, "/v1/groups"))["groups"][0]["id"]
        lends = json.loads(alex(*args, "/v1/lends?namespace=" + group))["lent"]
        row = next(lend for lend in lends if lend["name"] == NAME)
        alex(*args, "/v1/play", json.dumps({"on": {"key": row["key"], "name": NAME}}))
        origin = row["origin"].rstrip("/")
        url = origin + "/marker"
        at = lambda cmd: alex(*cmd, check=False)
        expected = marker + "\n\n200"
        first = proof.wait_for(url, at)
        assert first == expected, first
        actions = len(vm.typed)
        boot, sessions = proof.reboot()
        seen = proof.wait_for(url, at, bound=25)
        current = proof.declaration(NAME)
        journal = guest("journalctl -b -q _UID=1000 _SYSTEMD_USER_UNIT=svrngs.service --no-pager "
                        "-g 'a kept lend renewed'", check=False)
        vm.say("renewal journal", said=journal)
        renewed = (current.get("keep") == prior["keep"] and prior["keep"]["unit"] == proof.UNIT
                   and current.get("approved") != prior["approved"] and "renewed=Ok(())" in journal)
        wrong = "wrong-" + expected
        vm.say("planted wrong marker", expected=wrong, received=seen, accepted=seen == wrong)
        assert seen != wrong
        sessions_after = guest("loginctl list-sessions --no-legend").splitlines()
        say("old link and data", seen == expected and renewed and len(vm.typed) == actions and
            vm.no_login(sessions + sessions_after) and any(" sam " in s and "manager" in s for s in sessions_after),
            url=url, boot_before=boot, boot_after=guest("cat /proc/sys/kernel/random/boot_id"),
            before=first, after=seen, sessions=sessions_after, renewed=renewed, approved=current.get("approved"))

        cut = f"/usr/bin/python3 /root/cut.py {shlex.quote(origin + '/stream')} {marker}"
        planted = vm.root(cut + " --skip-withdrawal", 20)
        vm.say("planted omitted withdrawal", said=planted.strip())
        assert planted.strip() == "the member's answer still carries on after 2 s"
        stopped = guest(cut + " " + NAME)
        absent()
        denied = at(("curl", "-sS", "-o", "/dev/null", "-w", "%{http_code}", "-m", "5", url))
        local = guest(f"curl -fsS {LOCAL}/marker")
        say("stop sharing", "member stream ended" in stopped and local == marker and denied.isdecimal()
            and 400 <= int(denied) < 600, cut=stopped, local=local, fresh_request_status=denied)
        guest(f"{proof.SAM} /home/sam/.local/bin/svrngs-keeper remove {NAME}")
        return 0
    finally:
        for case in CASES:
            if case not in vm.cases:
                vm.verdict(case, "never-ran", why="run stopped before this case's verdict")


if __name__ == "__main__":
    sys.exit(proof.with_vm(lambda: scene(Path(sys.argv[1]).resolve())))
