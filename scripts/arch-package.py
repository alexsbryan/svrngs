#!/usr/bin/env python3
"""packaging/arch from a signed release: `scripts/arch-package.py <version>`.

The release's last step (DESIGN §7, "Publishing"). It downloads GitHub
release v<version>'s Linux tarballs, their .sig files and SHA256SUMS with
its .sig. It refuses unless the release key in PKGBUILD's validpgpkeys signed
each one, in a keyring holding only svrngs-release-key.asc, and each
tarball's sum is its SHA256SUMS line. It then writes PKGBUILD's pkgver,
pkgrel and every sum, and runs makepkg --verifysource on it in an x86_64 Arch
container: first on a copy with one tarball sum changed, which must be
refused, then on PKGBUILD. Its last line is its verdict, as JSON.
"""
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PKG = ROOT / "packaging/arch"
ARCHES = ("x86_64", "aarch64")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def array(text, name):
    return re.search(rf"^{name}=\((.*?)\)", text, re.S | re.M)


def written(name, sums):
    pad = "\n" + " " * (len(name) + 2)
    return f"{name}=(" + pad.join(f"'{s}'" for s in sums) + ")"


def verdict(state, **said):
    print(json.dumps({"state": state, **said}))
    return 0 if state == "passed" else 1


def main(version):
    text = (PKG / "PKGBUILD").read_text()
    repo = re.search(r'^url="https://github.com/(.+)"', text, re.M)[1]
    key = re.search(r"^validpgpkeys=\('([0-9A-F]{40})'\)", text, re.M)[1]
    names = {a: re.findall(r'\$_get/([^"]+)"', array(text, f"source_{a}")[1]) for a in ARCHES}
    with tempfile.TemporaryDirectory() as tmp:
        got, home = Path(tmp, "release"), Path(tmp, "gnupg")
        home.mkdir(mode=0o700)
        wanted = [n for a in ARCHES for n in names[a]] + ["SHA256SUMS", "SHA256SUMS.sig"]
        pull = ["gh", "release", "download", f"v{version}", "-R", repo, "-D", str(got)]
        subprocess.run(pull + [p for n in wanted for p in ("-p", n)], check=True, capture_output=True)
        gpg = ["gpg", "--homedir", str(home), "--batch"]
        subprocess.run(gpg + ["--import", str(ROOT / "svrngs-release-key.asc")], check=True, capture_output=True)
        for sig in sorted(got.glob("*.sig")):
            status = subprocess.run(gpg + ["--status-fd", "1", "--verify", str(sig), str(sig.with_suffix(""))], capture_output=True, text=True)
            if f"[GNUPG:] VALIDSIG {key} " not in status.stdout:
                return verdict("failed", why=f"{sig.name} is not signed by {key}")
        listed = {n: s for s, n in (line.split() for line in (got / "SHA256SUMS").read_text().splitlines())}
        for name in (n for a in ARCHES for n in names[a] if not n.endswith(".sig")):
            if listed.get(name) != sha(got / name):
                return verdict("failed", why=f"{name}'s sum is not its SHA256SUMS line")
        common = array(text, "source")[1].split()
        new = re.sub(r"^pkgver=.*$", f"pkgver={version}", text, flags=re.M)
        new = re.sub(r"^pkgrel=.*$", "pkgrel=1", new, flags=re.M)
        new = new.replace(array(new, "sha256sums")[0], written("sha256sums", [sha(PKG / n) for n in common]))
        for a in ARCHES:
            old = array(new, f"sha256sums_{a}")[0]
            new = new.replace(old, written(f"sha256sums_{a}", [sha(got / n) for n in names[a]]))
        (PKG / "PKGBUILD").write_text(new)
        if not shutil.which("podman"):
            return verdict("could-not-judge", why="no podman for the Arch container", pkgver=version)
        box = Path(tmp, "box")
        shutil.copytree(PKG, box / "pkg")
        planted = new.replace(sha(got / names["x86_64"][0]), "0" * 64)
        (box / "planted").write_text(planted)
        shutil.copy(ROOT / "svrngs-release-key.asc", box / "key.asc")
        steps = """set -e; useradd -m b; cp -R /w/pkg /home/b/pkg; cp -R /w/pkg /home/b/plant
cp /w/planted /home/b/plant/PKGBUILD; chown -R b /home/b
su b -c 'gpg -q --import /w/key.asc 2>/dev/null; cd ~/plant && ! makepkg --verifysource >/dev/null 2>&1'
echo planted refused; su b -c 'cd ~/pkg && makepkg --verifysource' 2>&1 | tail -n 12"""
        run = ["podman", "run", "--rm", "--platform", "linux/amd64", "-v", f"{box}:/w:Z", "docker.io/library/archlinux:latest", "bash", "-c", steps]
        done = subprocess.run(run, capture_output=True, text=True, timeout=1800)
        print(done.stdout + done.stderr, file=sys.stderr)
        state = "passed" if done.returncode == 0 and "planted refused" in done.stdout else "failed"
        return verdict(state, pkgver=version, signatures=len(list(got.glob("*.sig"))), makepkg=done.returncode)


if __name__ == "__main__":
    if len(sys.argv) != 2 or not re.fullmatch(r"\d+\.\d+\.\d+", sys.argv[1]):
        sys.exit(__doc__.splitlines()[0])
    sys.exit(main(sys.argv[1]))
