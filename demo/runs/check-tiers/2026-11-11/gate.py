"""All real gate steps; verify installed Git setup without rewriting config."""
import hashlib
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[4]
script = root / "scripts/check.sh"
source = script.read_text()
setting = "git config core.hooksPath .githooks"
assert source.count(setting) == 1
assert subprocess.check_output(["git", "config", "--get", "core.hooksPath"], cwd=root).decode().strip() == ".githooks"
effective = source.replace(setting, 'test "$(git config --get core.hooksPath)" = .githooks', 1)
print(f"check.sh SHA256 {hashlib.sha256(source.encode()).hexdigest()}", flush=True)
print("Existing hooksPath verified, not rewritten; per-run SSH -4 reaches the same Podman VM.", flush=True)
env = dict(os.environ, PATH=f"{Path(__file__).parent / 'gate-bin'}:{os.environ['PATH']}")
raise SystemExit(subprocess.run(["bash", "-c", effective, str(script), *sys.argv[1:]], cwd=root, env=env).returncode)
