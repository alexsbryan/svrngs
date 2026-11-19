#!/usr/bin/env python3
"""Build this release's site with the existing publisher, deploying nothing, and keep its actual exit and the site directory it printed.

Publishing is three steps (scripts/deploy-page.sh's header): this runs the
first, `deploy-page.sh --build`; the operator then signs the site it
printed (`scripts/sign-release.sh`), and only then does `deploy-page.sh
--site` deploy it."""
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
    # --build prints the site's directory alone on stdout, and the builds'
    # own output on stderr, which is the log.
    with log.open("w") as output:
        completed = subprocess.run(
            ["lockf", "-k", "/tmp/svrngs-gate.lock", "bash", "scripts/deploy-page.sh", "--build"],
            cwd=worktree, stdout=subprocess.PIPE, stderr=output, text=True,
        )
    result["exit"] = completed.returncode
    printed = completed.stdout.splitlines()
    result["printed"] = printed
    result["site"] = printed[-1] if printed else None
    said = log.read_text()
    # The lines deploy-page.sh names for what follows, each naming the same site.
    result["sign"] = next(iter(re.findall(r"^  (scripts/sign-release\.sh .+)$", said, re.MULTILINE)), None)
    result["deploy"] = next(iter(re.findall(r"^  (scripts/deploy-page\.sh --site .+)$", said, re.MULTILINE)), None)
    if completed.returncode != 0:
        raise RuntimeError(f"deploy-page.sh --build exited {completed.returncode}")
    if len(printed) != 1 or not Path(printed[0]).is_dir():
        raise RuntimeError(f"deploy-page.sh --build printed no one site directory: {printed!r}")
    site = printed[0]
    if result["sign"] is None or not result["sign"].endswith(f" {head} {site}"):
        raise RuntimeError(f"the sign-release.sh line it names is not this commit's and this site's: {result['sign']!r}")
    if result["deploy"] != f"scripts/deploy-page.sh --site {site}":
        raise RuntimeError(f"the --site line it names is not this site's: {result['deploy']!r}")
    result["state"] = "passed"
except Exception as error:
    result.update(state="failed", error=str(error))
finally:
    result["finished"] = datetime.now(timezone.utc).isoformat()
    save()
sys.exit(0 if result["state"] == "passed" else 1)
