#!/usr/bin/env python3
"""Stop a kept Actual during systemd's restart delay in the disposable Fedora VM."""

import shlex
import subprocess
import uuid

import actual
from recover import until

proof = actual.proof
NAME, UNIT, LOCAL = actual.NAME, proof.UNIT, actual.LOCAL
MANAGER = proof.MANAGER


def pod(*args, timeout=20):
    return proof.guest(f"{proof.SAM} podman " + shlex.join(args), timeout)


def show(*props):
    return dict(line.split("=", 1) for line in
                proof.guest(f"{MANAGER} show -p {','.join(props)} {UNIT}").splitlines())


def port_read():
    return proof.guest(f"curl -sS -m 2 -o /dev/null -w '%{{http_code}} ' {LOCAL}/; printf %s $?")


def pod_exists(target):
    return proof.guest(f"{proof.SAM} podman pod exists {target}; printf %s $?")


def login_when_ready(marker):
    def read():
        answer = actual.request(proof.direct, LOCAL, "login", marker)
        if answer[0] == "000":
            proof.vm.say("Actual login did not answer", http=answer[0])
        return answer

    # The page answering does not establish that Actual's login route answers.
    login = until(read, lambda answer: answer[0] != "000", seconds=16)
    assert login == ("200", "ok", True), login
    return login


def scene():
    subprocess.run(["scp", *proof.vm.SCP, str(proof.vm.BIN / "svrngs-keeper"),
                    "root@127.0.0.1:/tmp/svrngs-keeper-test"], check=True, timeout=30)
    proof.guest("install -m755 /tmp/svrngs-keeper-test /usr/local/bin/svrngs-keeper && loginctl enable-linger sam")
    text = (proof.HERE / "actual.compose.yaml").read_text()
    proof.write(actual.CARD, text)
    image = next(line.split()[1] for line in text.splitlines() if line.strip().startswith("image:"))
    proof.vm.say("versions", said=proof.guest("rpm -q podman systemd"))
    pod("pull", "--quiet", image, timeout=45)
    unit_path = f"/home/sam/.config/systemd/user/{UNIT}"
    proof.write(unit_path + ".d/window.conf", "[Service]\nRestartSec=30s\nEnvironment=RUST_LOG=debug\n")
    proof.guest("chown -R sam:sam /home/sam/.config")
    assert proof.vm.terminal(f"svrngs-keeper run {NAME} --port {proof.PORT} --card {actual.CARD}")[0] == 0
    marker = uuid.uuid4().hex
    assert actual.request(proof.direct, LOCAL, "bootstrap", marker)[:2] == ("200", "ok")
    old = pod("pod", "inspect", "--format", "{{.Id}}", NAME)
    assert len(old) == 64 and port_read() == "200 0"

    proof.guest(f"{MANAGER} kill --kill-whom=main --signal=KILL {UNIT}")
    gap = until(lambda: show("SubState", "MainPID", "NRestarts"),
                lambda state: state["SubState"] == "auto-restart")
    assert gap == {"SubState": "auto-restart", "MainPID": "0", "NRestarts": "0"}, gap
    proof.vm.say("wrapper killed; waiting to restart", **gap, port=port_read())
    assert proof.vm.terminal(f"svrngs-keeper stop {NAME}")[0] == 0
    state = show("ActiveState", "UnitFileState", "NRestarts")
    port, pod_rc = port_read(), pod_exists(old)
    stopped = state == {"ActiveState": "inactive", "UnitFileState": "disabled", "NRestarts": "0"}
    ok = stopped and port == "000 7" and pod_rc == "1" and pod_exists(NAME) == "1"
    journal = proof.guest(f"journalctl -b -q _UID=1000 _SYSTEMD_USER_UNIT={UNIT} --no-pager -o cat")
    (proof.vm.OUT / "stop-gap-unit.log").write_text(journal + "\n")
    proof.vm.verdict("Stop during restart delay removes the pod", "passed" if ok else "failed",
                     **state, port=port, pod_exists_exit=pod_rc)
    assert ok

    invocation = proof.guest("systemctl show user@1000.service -p InvocationID --value")
    proof.guest("systemctl restart user@1000.service")
    new_invocation = proof.guest("systemctl show user@1000.service -p InvocationID --value")
    assert invocation and new_invocation and new_invocation != invocation
    proof.guest(f"{MANAGER} is-active default.target")
    state = show("ActiveState", "UnitFileState", "NRestarts")
    assert state == {"ActiveState": "inactive", "UnitFileState": "disabled", "NRestarts": "0"}, state
    assert port_read() == "000 7" and pod_exists(NAME) == "1"
    pod("volume", "exists", NAME + "-data")
    proof.vm.verdict("stays stopped after restarting Sam's user manager", "passed", **state,
                     port="000 7", volume="present")

    # Restore the normal restart delay and check that an unopposed crash
    # still starts a fresh pod with the same data.
    proof.write(unit_path + ".d/window.conf", "[Service]\nEnvironment=RUST_LOG=debug\n")
    proof.guest(f"{MANAGER} daemon-reload")
    assert proof.vm.terminal(f"svrngs-keeper start {NAME}")[0] == 0
    login = login_when_ready(marker)
    proof.vm.verdict("explicit Start reads the same marker", "passed", status=login)
    before = pod("pod", "inspect", "--format", "{{.Id}}", NAME)
    proof.guest(f"{MANAGER} kill --kill-whom=main --signal=KILL {UNIT}")
    until(lambda: show("SubState", "NRestarts"), lambda state: state == {"SubState": "running", "NRestarts": "1"})
    assert pod_exists(before) == "1"
    login = login_when_ready(marker)
    proof.vm.verdict("an unopposed crash still recovers", "passed", restarts="1", status=login)
    assert proof.vm.terminal(f"svrngs-keeper stop {NAME}")[0] == 0
    assert pod_exists(NAME) == "1"

    # Give the PID-less stop hook a running same-name pod it doesn't own.
    foreign = pod("pod", "create", "--name", NAME, "--label", "org.svrngs.keeper.owner=foreign")
    other = "svrngs-keeper-test-actual-foreign"
    proof.guest(f"systemd-run -M sam@.host --user --unit={other} --quiet podman run " +
                shlex.join(["--pod", foreign, "--name", other, "--entrypoint", "/bin/sleep", "--", image, "60"]))
    def foreign_running():
        return proof.guest(f"{proof.SAM} podman container inspect --format '{{{{.State.Status}}}}' {other}", check=False)

    until(foreign_running, lambda state: state == "running")
    container = pod("container", "inspect", "--format", "{{.Id}}", other)
    hook = next(line.removeprefix("ExecStopPost=") for line in proof.guest(f"cat {unit_path}").splitlines()
                if line.startswith("ExecStopPost="))
    code, said = proof.vm.terminal("RUST_LOG=debug " + hook)
    after = pod("pod", "inspect", "--format", "{{.Id}}", NAME)
    running = pod("container", "inspect", "--format", "{{.State.Status}}", container)
    intact = code != 0 and "pod label did not match" in said and after == foreign and running == "running"
    proof.vm.verdict("PID-less stop refuses a same-name foreign pod", "passed" if intact else "failed", exit=code,
                     same_id=after == foreign, container=running)
    assert intact
    pod("pod", "stop", "--time", "1", foreign)
    pod("pod", "rm", foreign)
    assert proof.vm.terminal(f"svrngs-keeper remove {NAME}")[0] == 0
    return 0


if __name__ == "__main__":
    raise SystemExit(proof.with_vm(scene))
