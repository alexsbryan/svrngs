#!/usr/bin/env python3
"""Proof 2: test the restart recipe with real Podman and systemd before shipping it."""

import shlex
import sys
import time
import uuid

import run as proof

NAME, UNIT, APP, PORT = proof.NAME, proof.UNIT, proof.APP, proof.PORT
IMAGE, MANAGER, SAM = proof.IMAGE, proof.MANAGER, proof.SAM
VOLUME = NAME + "-data"
LABEL = "org.svrngs.keeper.owner"
TOKEN = uuid.uuid4().hex


def pod(*args):
    return proof.guest(f"{SAM} podman " + shlex.join(args))


def show(prop):
    return proof.guest(f"{MANAGER} show -p {prop} --value {UNIT}")


def inspect():
    out = proof.guest(f"{SAM} podman pod inspect --format '{{{{.Id}}}} {{{{.State}}}}' {NAME}", check=False)
    return out.split() if out else (None, None)


def until(read, accepts, seconds=25):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        value = read()
        if accepts(value):
            return value
        time.sleep(0.2)
    raise AssertionError(f"timed out: last read {value!r}")


def app_args():
    return ["run", "--detach", "--pod", NAME, "--name", APP,
            "--mount", f"type=volume,source={VOLUME},destination=/data",
            "--health-cmd", "test -f /data/ready", "--health-interval", "2s",
            "--", IMAGE, "python", "-m", "http.server", "--directory", "/data", "8000"]


def install(plant, control_group):
    assert proof.guest("command -v podman"), "use the Podman-prepared spike disk"
    first = f"podman pod create --name {NAME} --publish 127.0.0.1:{PORT}:8000/tcp --add-host web:127.0.0.1"
    guard = "true" if plant else f'test "$owner" = {TOKEN}'
    proof.write("/usr/local/bin/svrngs-keeper-test-recover", f"""#!/bin/bash
set -euo pipefail
if podman pod exists {NAME}; then
    read -r id owner <<< "$(podman pod inspect --format '{{{{.Id}}}} {{{{index .Labels "{LABEL}"}}}}' {NAME})"
    if ! {guard}; then echo "refused foreign pod $id ($owner)" >&2; exit 2; fi
    echo "cleaning pod $id ($owner)"
    podman pod stop --time 1 "$id"
    podman pod rm "$id"
else
    test "$?" -eq 1
fi
podman volume create --ignore {VOLUME}
{first} --label {LABEL}={TOKEN}
id=$(podman {shlex.join(app_args())})
echo "detached container $id"
podman wait --condition=exited "$id"
exit 1
""")
    proof.guest("chmod 755 /usr/local/bin/svrngs-keeper-test-recover")
    proof.write(f"/home/sam/.config/systemd/user/{UNIT}", f"""[Unit]
StartLimitIntervalSec=60
StartLimitBurst=4
[Service]
Type=exec
KillMode={'control-group' if control_group else 'process'}
Restart=on-failure
RestartSec=2
ExecStart=/usr/local/bin/svrngs-keeper-test-recover
""")
    proof.guest(f"loginctl enable-linger sam && chown -R sam:sam /home/sam/.config && {MANAGER} daemon-reload")
    proof.vm.say("versions", said=proof.guest("uname -r && rpm -q podman systemd passt"))
    proof.vm.say("pull", said=pod("pull", "--quiet", IMAGE))


def journal():
    return proof.guest(f"journalctl -b -q _UID=1000 _SYSTEMD_USER_UNIT={UNIT} --no-pager -o cat")


def manager(verb):
    return proof.guest(f"{MANAGER} {verb} {UNIT}")


def health(container):
    return pod("container", "inspect", "--format", "{{.State.Health.Status}}", container)


def kill_and_recover(ready, marker):
    old, state = inspect()
    before = pod("container", "inspect", "--format", "{{.Id}}", APP)
    observed = health(before)
    assert state == "Running" and (observed == "healthy") == ready
    was = show("NRestarts")
    proof.guest(f"{MANAGER} kill --kill-whom=main --signal=KILL {UNIT}")
    try:
        new, _ = until(inspect, lambda view: view[0] not in (old, None) and view[1] == "Running")
    except AssertionError:
        evidence = proof.guest(f"journalctl -b -q _UID=1000 -g '{UNIT}' --no-pager -o cat")
        proof.vm.verdict("foreground death recovers", "failed", old=old, pod=inspect(),
                         manager=show("ActiveState"), restarts=show("NRestarts"), journal=evidence)
        raise
    new_container = until(lambda: proof.guest(f"{SAM} podman ps --filter name=^{APP}$ --format '{{{{.ID}}}}'"), bool)
    body = proof.wait_for(f"http://127.0.0.1:{PORT}/marker", proof.direct)
    assert f"cleaning pod {old} ({TOKEN})" in journal() and new_container != before[:len(new_container)]
    evidence = proof.guest(f"journalctl -b -q _UID=1000 -g '{UNIT}.*status=9/KILL' --no-pager -o cat")
    assert "status=9/KILL" in evidence
    assert not show("ExecStop") and not show("ExecStopPost")
    assert show("NRestarts") == str(int(was) + 1) and body == marker + "\n\n200"
    proof.vm.verdict(f"kill {'after' if ready else 'before'} readiness", "passed", health=observed,
                     killed="main only; no ExecStop", restarts=show("NRestarts"), old=old, new=new,
                     old_container=before, new_container=new_container, body=body)


def scene(plant, control_group):
    install(plant, control_group)
    # Same name and runnable, with a different label. It has a running app
    # on a different port so this isolates the ownership guard from busy-port refusal.
    foreign = pod("pod", "create", "--name", NAME, "--label", f"{LABEL}=another-job")
    foreign_name = NAME + "-foreign"
    proof.guest(f"systemd-run -M sam@.host --user --unit={foreign_name} --quiet "
                f"podman run --pod {foreign} --name {foreign_name} -- {IMAGE} sleep 120")
    until(inspect, lambda view: view[1] == "Running")
    foreign_app = pod("container", "inspect", "--format", "{{.Id}}", foreign_name)
    manager("start")
    seen = until(journal, lambda said: "refused foreign pod" in said or "cleaning pod" in said)
    if plant:
        until(inspect, lambda view: view[0] != foreign or view[1] != "Running")
    after, state = inspect()
    intact = (after == foreign and state == "Running"
              and pod("container", "inspect", "--format", "{{.State.Status}}", foreign_app) == "running")
    proof.vm.verdict("same-name foreign pod is untouched", "passed" if intact else "failed",
                     plant=plant, expected=foreign, after=after, state=state, journal=seen.splitlines()[-4:])
    assert intact, "the same-name foreign pod was touched"
    assert "refused foreign pod" in seen
    manager("stop")
    pod("pod", "stop", "--time", "1", foreign)
    pod("pod", "rm", foreign)
    manager("start")
    until(inspect, lambda view: view[1] == "Running")
    marker = uuid.uuid4().hex
    pod("exec", APP, "sh", "-c", f"echo {marker} > /data/marker")
    kill_and_recover(False, marker)
    running = show("MainPID")
    inspected = pod("container", "inspect", "--format", "{{.Id}}", APP)
    probe = proof.guest(f"{SAM} /bin/sh -c 'podman healthcheck run {inspected}; echo unhealthy_exit=$?; "
                        "podman container inspect no-such-container >/dev/null 2>&1; echo unreadable_exit=$?'")
    assert probe.splitlines()[-2:] == ["unhealthy_exit=1", "unreadable_exit=125"]
    assert show("MainPID") == running
    proof.vm.verdict("nonready and unreadable are not exits", "passed", probe=probe, main_pid=running)
    pod("exec", APP, "touch", "/data/ready")
    until(lambda: health(APP), lambda status: status == "healthy")
    kill_and_recover(True, marker)
    pod("stop", "--time", "1", APP)
    until(lambda: show("NRestarts"), lambda count: count == "3")
    assert proof.wait_for(f"http://127.0.0.1:{PORT}/marker", proof.direct) == marker + "\n\n200"
    proof.vm.verdict("actual container exit restarts the unit", "passed", restarts=show("NRestarts"))
    (proof.vm.OUT / "recovery-unit.log").write_text(journal() + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(proof.with_vm(lambda: scene("--plant" in sys.argv[1:], "--control-group" in sys.argv[1:])))
