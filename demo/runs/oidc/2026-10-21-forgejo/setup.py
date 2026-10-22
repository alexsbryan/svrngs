#!/usr/bin/env python3
"""What the run was made of, beside its data: pinned versions, the trust
floor and the lean check.

    setup.py <data dir> <unchanged|changed> <version line> <binary sha256> <bottle sha256>
"""
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
floor = sys.argv[2]
setup = {
    "forgejo": {
        "version": sys.argv[3] if len(sys.argv) > 3 else "17.0.0",
        "install": "Homebrew bottle forgejo--17.0.0.arm64_golden_gate.bottle.tar.gz",
        "bottle_sha256": sys.argv[5] if len(sys.argv) > 5 else None,
        "binary_sha256": sys.argv[4] if len(sys.argv) > 4 else None,
        "root_url": "http://localhost:18097/",
    },
    "container": {
        "used": False,
        "why": "the row's Podman premise on a Mac; data/podman-probe.txt",
    },
    "trust_floor": floor,
    "lean": (out / "lean.txt").read_text() if (out / "lean.txt").exists() else None,
    "podman_probe": (out / "podman-probe.txt").read_text() if (out / "podman-probe.txt").exists() else None,
}
(out / "setup.json").write_text(json.dumps(setup, indent=2) + "\n")
