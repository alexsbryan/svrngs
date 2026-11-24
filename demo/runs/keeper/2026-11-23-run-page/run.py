"""Run only in the disposable gate, behind its cgroup-delegating entrypoint."""
import os
from pathlib import Path
import pty
import shutil
import subprocess as sp
import sys

sys.path.insert(0, "crates/svrngs-node/tests")
from keeper_gate import built_test, in_unit, require_gate, user_manager


def main():
    require_gate()
    env = dict(os.environ, CARGO_TARGET_DIR="/work/target/run-page-gate", TOKIO_WORKER_THREADS="4")
    sp.run(["cargo", "build", "--locked", "-p", "svrngs-keeper"], env=env, check=True)
    binary = built_test(env, "--test", "browser")
    keeper = Path(env["CARGO_TARGET_DIR"]) / "debug/svrngs-keeper"
    with user_manager() as (home, private):
        shutil.copy2(binary, home / "browser")
        shutil.copy2(keeper, home / "svrngs-keeper")
        plant = "Environment=SVRNGS_RUN_PAGE_PLANT=1\n" if "--plant" in sys.argv else ""
        scene = "explicit_run_then_share_in_webkit"
        if "--prefill" in sys.argv:
            scene = "run_prefill_waits_for_confirmation_in_webkit"
            for arm in ("submit", "edit"):
                if f"--plant-{arm}" in sys.argv:
                    plant = f"Environment=SVRNGS_PREFILL_PLANT={arm}\n"
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
            private["PATH"] = f'{home}:{private["PATH"]}'
            stdin = (f"StandardInput=tty\nTTYPath={os.ttyname(slave)}\n"
                     "Environment=SVRNGS_RUN_PAGE_TTY=1\n")
        try:
            outcome, log = in_unit(home, private, home / "browser", scene, plant + stdin)
            outcome.check_returncode()
            if tty:
                calls = (home / "linger.log").read_text().splitlines()
                assert calls and set(calls) == {"show-user"}, calls
                assert not (home / "lingers").exists()
                assert "Type y, then press Enter:" not in log
                print(f"Page Run and controls: unanswered PTY, loginctl {calls}; linger=no", flush=True)
        finally:
            if tty:
                os.close(slave)
                os.close(master)


if __name__ == "__main__":
    main()
