#!/usr/bin/env python3
"""Run Actual from its card; Alex signs into the same server after Sam reboots."""

import hashlib
import json
import sys
import time
import uuid

import run as proof

NAME = "test-actual"
CARD = "/home/sam/actual.compose.yaml"
proof.PORT = 5006
LOCAL = f"http://127.0.0.1:{proof.PORT}"
proof.UNIT = f"svrngs-keeper-{NAME}.service"


def request(at, url, verb, marker):
    args = ("curl", "-sS", "-m", "5", "-w", "\n%{http_code}", "-H", "Content-Type: application/json",
            "-d", json.dumps({"loginMethod": "password", "password": marker}), f"{url}/account/{verb}")
    body, _, status = at(args).rpartition("\n")
    try:
        answer = json.loads(body)
        return status, answer.get("status"), bool((answer.get("data") or {}).get("token"))
    except (ValueError, AttributeError):
        return status, body[:120], False


def scene(plant):
    proof.stage_svrngs("svrngs-keeper")
    text = (proof.HERE / "actual.compose.yaml").read_text()
    invalid = text.replace('"data:/data"', '"/tmp:/data"')
    proof.write(CARD, invalid)
    run = f"svrngs run {NAME} --port {proof.PORT} --card {CARD}"
    code, said = proof.vm.terminal(run)
    assert code != 0 and "bind mount" in said, said
    assert proof.guest(f"{proof.MANAGER} show -p LoadState --value {proof.UNIT}") == "not-found"
    proof.vm.verdict("host bind refused before install", "passed", exit=code)
    proof.write(CARD, text)
    image = next(line.split()[1] for line in text.splitlines() if line.strip().startswith("image:"))
    proof.vm.say("pull", said=proof.guest(f"{proof.SAM} podman pull --quiet {image}", 180))
    code, said = proof.vm.terminal(run)
    assert code == 0 and f"{NAME} is running here. To share it: `svrngs share {proof.PORT} " in said, said
    proof.guest("test ! -e /home/sam/.svrngs/lends/actual.json")
    assert proof.sample("before")
    marker = uuid.uuid4().hex
    first = request(proof.direct, LOCAL, "bootstrap", marker)
    assert first[0:2] == ("200", "ok"), first
    origin, before = proof.share("actual")
    alex = lambda args: proof.alex(*args, check=False)
    login = request(alex, origin, "login", "wrong-" + marker if plant else marker)
    proof.vm.verdict("Alex signs in with the marker before reboot", "passed" if login == ("200", "ok", True) else "failed",
                     url=origin, status=login, plant=plant, marker_hash=hashlib.sha256(marker.encode()).hexdigest())
    assert login == ("200", "ok", True)
    boot, sessions = proof.reboot()
    assert proof.wait_for(LOCAL + "/", proof.direct).endswith("\n200")
    assert proof.sample("after")
    login = request(alex, origin, "login", marker)
    renewed, after = proof.renewal("actual", before)
    ok = login == ("200", "ok", True) and renewed and proof.vm.no_login(sessions)
    proof.vm.verdict("same Actual link and marker after reboot", "passed" if ok else "failed", url=origin,
                     boot_before=boot, boot_after=proof.guest("cat /proc/sys/kernel/random/boot_id"),
                     status=login, sessions=sessions, renewed=renewed, approved=after.get("approved"))
    assert ok
    def pod_id():
        return proof.guest(f"{proof.SAM} podman pod inspect --format '{{{{.Id}}}}' {NAME}", check=False)

    old = pod_id()
    assert old
    proof.guest(f"{proof.MANAGER} kill --kill-whom=main --signal=KILL {proof.UNIT}")
    for _ in range(40):
        new = pod_id()
        if new and new != old:
            break
        time.sleep(0.25)
    restarts = proof.guest(f"{proof.MANAGER} show -p NRestarts --value {proof.UNIT}")
    assert new and new != old and restarts == "1", (old, new, restarts)
    assert proof.wait_for(LOCAL + "/", proof.direct).endswith("\n200")
    login = request(alex, origin, "login", marker)
    proof.vm.verdict("installed foreground death recovers its pod", "passed" if login == ("200", "ok", True) else "failed",
                     old=old, new=new, restarts=restarts, status=login, url=origin)
    assert login == ("200", "ok", True)
    assert proof.vm.terminal(f"svrngs-keeper stop {NAME}")[0] == 0
    assert proof.guest(f"{proof.SAM} podman pod exists {NAME}; printf %s $?") == "1"
    assert proof.guest(f"{proof.SAM} podman volume exists {NAME}-data; printf %s $?") == "0"
    assert proof.vm.terminal(f"svrngs-keeper remove {NAME}")[0] == 0
    proof.vm.verdict("stop removes pod and keeps volume; remove takes out unit", "passed")
    return 0


if __name__ == "__main__":
    sys.exit(proof.with_vm(lambda: scene("--plant" in sys.argv[1:])))
