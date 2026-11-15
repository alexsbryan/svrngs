#!/usr/bin/env python3
"""Run the existing publisher durably and keep its actual exit and site path."""
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import subprocess
import sys


worktree, record = (Path(argument).resolve() for argument in sys.argv[1:])
log = record / "publish.log"
status = record / "publish.json"
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=worktree, text=True).strip()
result = {"source": head, "worktree": str(worktree), "started": datetime.now(timezone.utc).isoformat(), "state": "running"}


def save():
    partial = status.with_suffix(".tmp")
    partial.write_text(json.dumps(result, indent=2) + "\n")
    partial.replace(status)


save()
try:
    dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=worktree, text=True)
    if dirty:
        raise RuntimeError(f"release worktree is not clean: {dirty}")
    with log.open("w") as output:
        completed = subprocess.run(
            ["lockf", "-k", "/tmp/svrngs-gate.lock", "bash", "scripts/deploy-page.sh"],
            cwd=worktree, stdout=output, stderr=subprocess.STDOUT,
        )
    result["exit"] = completed.returncode
    result["state"] = "passed" if completed.returncode == 0 else "failed"
    sites = re.findall(r"^deploy-page: [0-9a-f]+ to .+ from (.+)$", log.read_text(), re.MULTILINE)
    result["site"] = sites[-1] if sites else None
except Exception as error:
    result.update(state="failed", error=str(error))
finally:
    result["finished"] = datetime.now(timezone.utc).isoformat()
    save()
sys.exit(0 if result["state"] == "passed" else 1)
