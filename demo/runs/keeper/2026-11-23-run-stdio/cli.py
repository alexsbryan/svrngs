"""Drive the packaged pair through SSH's PTY in a disposable Fedora VM."""
import importlib.util
import shutil
import subprocess as sp
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "keeper_demo", Path(__file__).resolve().parent.parent / "2026-11-22/run.py"
)
demo = importlib.util.module_from_spec(spec)
spec.loader.exec_module(demo)

SPIKE = Path("/tmp/keeper-demo")
BIN = "/tmp/svrngs-keeper-test-bin"
UNIT = "svrngs-keeper-test-stdio.service"


def main():
    demo.OUT = demo.W / "run"
    demo.OUT.mkdir(parents=True)
    for name in ("key", "seed.iso"):
        (demo.W / name).symlink_to(SPIKE / name)
    shutil.copyfile(SPIKE / "prepared.qcow2", demo.W / "disk.qcow2")
    try:
        with demo.qemu() as vm:
            try:
                assert demo.ssh_up(60), "the scratch VM did not answer SSH"
                assert demo.root(f"install -d {BIN} && loginctl disable-linger sam && echo ready").strip() == "ready"
                sp.run(["scp", *demo.SCP, demo.BIN / "svrngs", demo.BIN / "svrngs-keeper",
                        f"root@127.0.0.1:{BIN}/"], check=True)
                code, said = demo.terminal(
                    "test -t 0 && test -t 2 && "
                    'printf "PTY, linger=%s\\n" "$(loginctl show-user sam -p Linger --value)" && '
                    "env HOME=/home/sam SVRNGS_DIR=/home/sam/svrngs-keeper-test-state "
                    f"{BIN}/svrngs run test-stdio --port 18191 -- "
                    "/usr/bin/python3 -m http.server 18191 --bind 127.0.0.1",
                    bound=12,
                )
                linger = demo.root("loginctl show-user sam -p Linger --value").strip()
                demo.say("linger after Run", value=linger)
                assert "PTY, linger=no" in said, "the scratch account did not begin with linger off on a PTY"
                assert "Turn that on? Type y, then press Enter:" in said, "boot consent prompt was not visible"
                assert code == 0 and "test-stdio is running here." in said and linger == "yes"
            finally:
                try:
                    demo.root(f"systemctl --user -M sam@.host disable --now {UNIT}; "
                              f"rm -f /home/sam/.config/systemd/user/{UNIT}; systemctl poweroff", 8)
                    vm.wait(12)
                except sp.TimeoutExpired:
                    vm.terminate()
                    vm.wait(8)
    finally:
        (demo.W / "disk.qcow2").unlink()


if __name__ == "__main__":
    main()
