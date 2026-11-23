"""Run only in the disposable gate, behind its cgroup-delegating entrypoint."""
import json
import os
from pathlib import Path
import pty
import shutil
import subprocess as sp
import sys
import tempfile
import time


def systemctl(*args, check=True, **kwargs):
    return sp.run(["systemctl", "--user", *args], check=check, **kwargs)


def main():
    assert Path("/run/.containerenv").exists() and Path.cwd() == Path("/work")
    env = dict(os.environ, CARGO_TARGET_DIR="/work/target/run-page-gate", TOKIO_WORKER_THREADS="4")
    sp.run(["cargo", "build", "--locked", "-p", "svrngs-keeper"], env=env, check=True)
    built = sp.run(["cargo", "test", "--locked", "-p", "svrngs-node", "--test", "browser", "--no-run", "--message-format=json"],
                   env=env, text=True, stdout=sp.PIPE, check=True)
    binary = next(Path(item["executable"]) for line in built.stdout.splitlines()
                  if (item := json.loads(line)).get("executable") and item["target"]["name"] == "browser")
    keeper = Path(env["CARGO_TARGET_DIR"]) / "debug/svrngs-keeper"
    with tempfile.TemporaryDirectory(prefix="svrngs-keeper-test-") as scratch:
        home = Path(scratch)
        runtime = home / "runtime"
        runtime.mkdir(mode=0o700)
        units = home / ".config/systemd/user"
        units.mkdir(parents=True)
        shutil.copy2(binary, home / "browser")
        shutil.copy2(keeper, home / "svrngs-keeper")
        os.environ.update(HOME=scratch, XDG_RUNTIME_DIR=str(runtime), SVRNGS_DIR=str(home / "node"),
                          DBUS_SESSION_BUS_ADDRESS=f"unix:path={runtime}/bus")
        cg = Path(f"/sys/fs/cgroup/user@{os.getuid()}.service")
        cg.mkdir()
        unit = "svrngs-keeper-test-browser.service"
        log = home / "browser.log"
        plant = "Environment=SVRNGS_RUN_PAGE_PLANT=1\n" if "--plant" in sys.argv else ""
        stdin = ""
        tty = "--tty" in sys.argv
        if tty:
            master, slave = pty.openpty()
            stub = home / "loginctl"
            # This container has a private user manager but no logind. Model
            # the account's off state and record both reads and any enable.
            stub.write_text(
                f'#!/bin/sh\nprintf "%s\\n" "$1" >> {home}/linger.log\n'
                f'case "$1" in\n'
                f'  show-user) test ! -e {home}/lingers && echo Linger=no || echo Linger=yes ;;\n'
                f'  enable-linger) touch {home}/lingers ;;\n'
                '  *) exit 2 ;;\nesac\n'
            )
            stub.chmod(0o755)
            stdin = (f"StandardInput=tty\nTTYPath={os.ttyname(slave)}\n"
                     "Environment=SVRNGS_RUN_PAGE_TTY=1\n")
        (units / unit).write_text(
            f"[Service]\nType=oneshot\nWorkingDirectory=/work\n"
            f"Environment=HOME={home} XDG_RUNTIME_DIR={runtime} SVRNGS_DIR={home}/node\n"
            f"Environment=DBUS_SESSION_BUS_ADDRESS=unix:path={runtime}/bus\n"
            f"Environment=SVRNGS_RUN_PAGE_DEMO=scratch PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright\n"
            f"Environment=TOKIO_WORKER_THREADS=4 PATH={home}:/opt/node/bin:/usr/bin:/bin\n{plant}{stdin}"
            f"ExecStart={home}/browser --ignored --exact explicit_run_then_share_in_webkit --nocapture\n"
            f"StandardOutput=append:{log}\nStandardError=inherit\n"
            f"TimeoutStartSec=180\n[Install]\nWantedBy=default.target\n"
        )
        assert not systemctl("is-system-running", check=False, capture_output=True).returncode == 0
        with sp.Popen(["dbus-daemon", "--session", "--nofork", "--address=" + os.environ["DBUS_SESSION_BUS_ADDRESS"]]) as bus:
            try:
                with sp.Popen(["systemd", "--user"], preexec_fn=lambda: (cg / "cgroup.procs").write_text(str(os.getpid()))) as manager:
                    try:
                        for _ in range(100):
                            if systemctl("is-system-running", check=False, capture_output=True).stdout.strip() in (b"running", b"degraded"):
                                break
                            if manager.poll() is not None:
                                raise RuntimeError(f"private user manager exited {manager.returncode}")
                            time.sleep(.05)
                        systemctl("enable", unit)
                        outcome = systemctl("start", unit, check=False)
                        if log.exists():
                            print(log.read_text(), end="", flush=True)
                        outcome.check_returncode()
                        if tty:
                            calls = (home / "linger.log").read_text().splitlines()
                            assert calls and set(calls) == {"show-user"}, calls
                            assert not (home / "lingers").exists()
                            assert "Type y, then press Enter:" not in log.read_text()
                            print(f"Page Run and controls: unanswered PTY, loginctl {calls}; linger=no", flush=True)
                    finally:
                        systemctl("stop", "svrngs-keeper-test-page-coder.service", unit, check=False)
                        manager.terminate()
            finally:
                bus.terminate()
                if tty:
                    os.close(slave)
                    os.close(master)


if __name__ == "__main__":
    main()
