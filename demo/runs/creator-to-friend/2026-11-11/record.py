"""Retain exact commands, source identity and verdicts; raw session logs stay private."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

here = Path(__file__).resolve().parent
root = here.parents[3]
name, *command = sys.argv[1:]
if not name.replace("-", "").isalnum() or not command:
    raise SystemExit("record.py <unique-label> <command...>")
out = here / f"{name}.json"
if out.exists():
    raise SystemExit(f"refusing to overwrite {out}")
private = root / "target/creator-checks"
private.mkdir(parents=True, exist_ok=True)
paths = subprocess.check_output(["git", "ls-files", "-co", "--exclude-standard", "-z"], cwd=root).decode().split("\0")
inputs = {}
for name_in in paths:
    path = root / name_in
    if path.is_file() and not name_in.startswith("demo/runs/") and path.suffix in {".rs", ".js", ".mjs", ".html", ".css", ".md", ".py", ".sh", ".toml", ".lock"}:
        inputs[name_in] = hashlib.sha256(path.read_bytes()).hexdigest()
began = time.monotonic()
with (private / f"{name}.log").open("wb") as log:
    run = subprocess.run(command, cwd=root, stdout=log, stderr=subprocess.STDOUT)
raw = (private / f"{name}.log").read_bytes()
record = {
    "command": command,
    "exit": run.returncode,
    "elapsed_seconds": round(time.monotonic() - began, 3),
    "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root).decode().strip(),
    "inputs_sha256": inputs,
    "output_sha256": hashlib.sha256(raw).hexdigest(),
    "private_log": str((private / f"{name}.log").relative_to(root)),
    "recorder_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
}
out.write_text(json.dumps(record, indent=2) + "\n")
print(f"exit {run.returncode} in {record['elapsed_seconds']} s; record {out.relative_to(root)}; log {record['private_log']}")
raise SystemExit(run.returncode)
