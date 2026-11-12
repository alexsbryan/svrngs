"""Copy the last real scene's sanitized observations, never create a verdict."""
import hashlib
import json
from pathlib import Path

here = Path(__file__).resolve().parent
root = here.parents[3]
original = root / "target/scenes/creator-chromium/report.json"
report = json.loads(original.read_text())
assert report["kind"] == "scripted-browser"
assert report["human_newcomer"] == "never-ran"
assert set(report["bars"]) == {"C1", "C2", "C3", "C4", "C5"}
assert all(b["verdict"] == "passed" for b in report["bars"].values())
out = here / "workflow.json"
assert not out.exists(), "refusing to replace recorded observations"
out.write_bytes(original.read_bytes())
print(f"Copied actual C1-C5 observations; SHA256 {hashlib.sha256(out.read_bytes()).hexdigest()}")
