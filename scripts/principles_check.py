#!/usr/bin/env python3
"""svrngs is held to two documents, pinned verbatim in docs/principles/.

  principles_check.py          exit 0 when both copies match their pins, 1 when one
                               was edited here; a pin behind upstream is reported,
                               never failed (upstream's state is not svrngs' gate)
  principles_check.py --sync   copy upstream's current text and re-pin: a
                               deliberate commit, never a silent drift

docs/principles/SOURCE holds one line per document: path here, path upstream,
the upstream commit, and the sha256 of the pinned text. Upstream is
commonwealth-ai, found at $COMMONWEALTH_AI or ~/dev/commonwealth-ai.
"""
import hashlib
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PIN = ROOT / "docs" / "principles" / "SOURCE"
DOCS = {"docs/principles/ARCH_PRINCIPLES.md": "sovereign/ARCH_PRINCIPLES.md",
        "docs/principles/RUST_ARCH.md": "docs/RUST_ARCH.md"}
UPSTREAM = Path(os.environ.get("COMMONWEALTH_AI", Path.home() / "dev" / "commonwealth-ai"))


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def upstream_commit(path):
    r = subprocess.run(["git", "log", "-1", "--format=%h", "--", path], cwd=UPSTREAM,
                       capture_output=True, text=True)
    return r.stdout.strip() if r.returncode == 0 else ""


def sync():
    lines = []
    for here, there in DOCS.items():
        shutil.copyfile(UPSTREAM / there, ROOT / here)
        lines.append(f"{here} {there} {upstream_commit(there)} {sha(ROOT / here)}")
    PIN.write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


def check():
    pins = {l.split()[0]: l.split() for l in PIN.read_text().splitlines() if l.strip()}
    broken = 0
    for here, there in DOCS.items():
        _, _, commit, pinned = pins[here]
        if sha(ROOT / here) != pinned:
            print(f"BROKEN  {here} was edited here; it is pinned to commonwealth-ai {there} at {commit}")
            broken = 1
            continue
        print(f"pinned  {here} = commonwealth-ai {there} at {commit}")
        if (UPSTREAM / there).exists() and sha(UPSTREAM / there) != pinned:
            print(f"notice  upstream {there} has moved past {commit}; --sync to take it, in its own commit")
    return broken


if __name__ == "__main__":
    sys.exit(sync() if sys.argv[1:] == ["--sync"] else check())
