#!/usr/bin/env python3
"""Foreground build/gate/probe recorder. No deployment or persistent listener."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone

run = Path(__file__).resolve().parent
root = run.parents[3]
source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
site = root / "target/onboarding-scene/site"
data = run / "data"
if "--probes-only" not in sys.argv:
    data = data / source[:12]
data.mkdir(parents=True, exist_ok=True)
record = {"source": source, "started": datetime.now(timezone.utc).isoformat(), "steps": []}


def step(name, command, expected=0, plant=None):
    print(f"{name}: running (expected exit {expected})", flush=True)
    with (data / f"{name}.txt").open("w") as log:
        log.write(f"source: {source}\ncommand: {json.dumps(command)}\n")
        log.flush()
        result = subprocess.run(command, cwd=root, stdout=log, stderr=subprocess.STDOUT)
        log.write(f"\nexit: {result.returncode}\n")
    record["steps"].append({"name": name, "command": command, "exit": result.returncode, "expected": expected})
    (data / "prepare.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"{name}: exit {result.returncode}; log {data / (name + '.txt')}", flush=True)
    if result.returncode != expected:
        sys.exit(1)
    if plant and plant not in (data / f"{name}.txt").read_text():
        sys.exit(f"{name}: failed before reaching the named plant")


if "--probes-only" not in sys.argv:
    step("build-web", ["lockf", "-k", "/tmp/svrngs-gate.lock", "scripts/build-web.sh", "--out", str(site)])
    step("build-downloads", ["lockf", "-k", "/tmp/svrngs-gate.lock", "scripts/build-release.sh", "--out", str(site)])
    artifacts = {}
    for path in sorted(site.rglob("*")):
        if path.is_file():
            artifacts[str(path.relative_to(site))] = hashlib.sha256(path.read_bytes()).hexdigest()
    (data / "artifacts.json").write_text(json.dumps({"source": source, "sha256": artifacts}, indent=2) + "\n")
probe = ["node", str(run / "verify.mjs"), str(site)]
step("checksum-red", probe + ["--plant-checksum"], expected=1, plant="download checksum: svrngs-darwin-aarch64.tar.gz")
step("staging-green", probe)
if "--probes-only" not in sys.argv:
    step("check", ["lockf", "-k", "/tmp/svrngs-gate.lock", "scripts/check.sh"])
print("Requested preparation steps passed; people scene never-ran. Servers stopped. Operator judgment owed.", flush=True)
