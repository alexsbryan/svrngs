#!/usr/bin/env python3
"""Install the release site in a fresh overlay of the prepared Fedora guest."""

import importlib.util
import os
import shutil
import subprocess
import sys
import tarfile
from hashlib import sha256
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
SITE = Path(sys.argv[1]).resolve()
SPIKE = Path("/tmp/keeper-demo")
W = Path(os.environ["W"])
ARCHIVE = "svrngs-linux-x86_64.tar.gz"
NAME = "test-ship"
UNIT = "svrngs-keeper-" + NAME + ".service"
os.environ.setdefault("BIN", str(REPO / "target/debug"))
spec = importlib.util.spec_from_file_location("demo", HERE.parent / "2026-11-22/run.py")
demo = importlib.util.module_from_spec(spec)
spec.loader.exec_module(demo)
demo.PORT = 2269
demo.SSH[3] = demo.SCP[3] = str(demo.PORT)


def guest(command, user="root", want=0):
    print(f"{user}$ {command}", flush=True)
    out = subprocess.run(
        ["ssh", *demo.SSH, f"{user}@127.0.0.1", command],
        capture_output=True, text=True, timeout=100,
    )
    # The fixed opener keeps only the fact of the request, not its one-use code.
    print(out.stdout + out.stderr, end="", flush=True)
    assert out.returncode == want, (out.returncode, want)
    return out.stdout + out.stderr


def arch(archive, members, want):
    # The pinned public release predates the keeper. Exercise the packaging
    # step for its successor without pretending the scratch archive is signed.
    staging = W / f"arch-{want}"
    shutil.copytree(REPO / "packaging/arch", staging)
    subprocess.run(["tar", "-xzf", archive, "-C", staging], check=True)
    pkgdir = staging / "pkg"
    result = subprocess.run(
        ["bash", "-c", 'source ./PKGBUILD; pkgver=1.13.4; pkgdir=$1; package_svrngs-bin',
         "package_svrngs-bin", pkgdir], cwd=staging, check=False,
    )
    assert result.returncode == want
    if want == 0:
        for member in members:
            path = pkgdir / "usr/bin" / member
            assert os.access(path, os.X_OK) and path.read_bytes() == (staging / member).read_bytes()
    print(f"passed: Arch package step exited {want} with archive members {members}", flush=True)


def scene():
    web = W / "site"
    shutil.copytree(SITE / "get", web / "get")
    shutil.copy(REPO / "web/install.sh", web)
    red = web / "plant/get"
    red.mkdir(parents=True)
    shutil.copy(web / "install.sh", red.parent)
    with tarfile.open(web / "get" / ARCHIVE) as original, tarfile.open(red / ARCHIVE, "w:gz") as plant:
        svrngs = original.getmember("svrngs")
        plant.addfile(svrngs, original.extractfile(svrngs))
    (red / "SHA256SUMS").write_text(f"{sha256((red / ARCHIVE).read_bytes()).hexdigest()}  {ARCHIVE}\n")
    arch(red / ARCHIVE, ("svrngs",), 1)
    arch(web / "get" / ARCHIVE, ("svrngs", "svrngs-keeper"), 0)

    subprocess.run(["scp", *demo.SCP, "-r", str(web), "root@127.0.0.1:/root/svrngs-ship"], check=True)
    guest("systemd-run --unit=svrngs-keeper-test-ship-site /usr/bin/python3 -m http.server 8187 "
          "--bind 127.0.0.1 --directory /root/svrngs-ship")
    guest("set -e; command -v svrngs >/dev/null && exit 2; test ! -e ~/.local/bin/svrngs; "
          "test ! -e ~/.local/bin/svrngs-keeper; test ! -e ~/.svrngs; "
          "mkdir -p ~/ship-fixture; printf 'ship witness\n' > ~/ship-fixture/index.html; "
          "printf '#!/bin/sh\nprintf open > \"$HOME/ship-fixture/opened\"\n' > ~/ship-fixture/xdg-open; "
          "chmod 755 ~/ship-fixture/xdg-open; . /etc/os-release; printf '%s\n' \"$PRETTY_NAME\"", "sam")

    env = 'export SVRNGS_DIR="$HOME/.svrngs" PATH="$HOME/ship-fixture:$HOME/.local/bin:$PATH"; '
    page = "http://127.0.0.1:8187/"
    line = lambda page, arg: f"curl -fsSL {page}install.sh | sh -s -- '{arg}'"
    out = guest(env + "set -o pipefail; " + line(page + "plant/", page + "plant/"), "sam", want=1)
    assert "must hold each of these programs: svrngs svrngs-keeper" in out
    guest("test ! -e ~/.local/bin/svrngs && test ! -e ~/.local/bin/svrngs-keeper && test ! -e ~/.svrngs", "sam")
    print("passed: a correctly checksummed archive with no keeper installs nothing", flush=True)

    guest(env + "set -o pipefail; " + line(page, page), "sam")
    guest(env + "set -e; test -x ~/.local/bin/svrngs-keeper; test ! -e ~/.local/bin/give; "
          "test -f ~/ship-fixture/opened; systemctl --user is-active svrngs; svrngs version", "sam")
    out = guest(env + f'svrngs run {NAME} --port 8188 --folder "$HOME/ship-fixture" '
                "-- /usr/bin/python3 -m http.server 8188 --bind 127.0.0.1", "sam")
    assert "is running here. To share it:" in out
    guest(f'test "$(curl -fsS http://127.0.0.1:8188/)" = "ship witness" && '
          f'test ! -e ~/.svrngs/lends/{NAME}.json && systemctl --user is-active {UNIT}', "sam")
    print("passed: the stock line installed both siblings, not Give; Run answers and made no share", flush=True)

    out = guest(env + "set -o pipefail; " + line(page, "--uninstall"), "sam")
    assert "Apps you told svrngs to run are not stopped or removed." in out
    assert "managed containers on Linux need svrngs-keeper to start and stop." in out
    guest(f"set -e; test ! -e ~/.local/bin/svrngs; test ! -e ~/.local/bin/svrngs-keeper; "
          f'test "$(systemctl --user is-active svrngs)" != active; systemctl --user restart {UNIT}; '
          'test "$(curl --retry 3 --retry-connrefused -fsS http://127.0.0.1:8188/)" = "ship witness"; '
          f"systemctl --user disable --now {UNIT}", "sam")
    print("passed: uninstall removed both binaries, and the native job still restarts", flush=True)


def main():
    W.mkdir()
    for name in ("key", "seed.iso"):
        (W / name).symlink_to(SPIKE / name)
    subprocess.run(["qemu-img", "create", "-q", "-f", "qcow2", "-F", "qcow2", "-b",
                    str(SPIKE / "prepared.qcow2"), str(W / "disk.qcow2")], check=True)
    q = demo.qemu()
    try:
        assert demo.ssh_up(60), "could-not-judge: the guest did not answer SSH"
        scene()
    finally:
        try:
            demo.root("systemctl poweroff", 10)
            q.wait(20)
        except subprocess.TimeoutExpired:
            q.terminate()
            q.wait(10)
        (W / "disk.qcow2").unlink()


if __name__ == "__main__":
    main()
