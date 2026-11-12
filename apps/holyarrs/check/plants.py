#!/usr/bin/env python3
"""Each broken scene must fail at its named bar, rather than during setup."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "target/holyarrs-plants"
OUT.mkdir(parents=True, exist_ok=True)
COMMAND = ["cargo", "test", "-p", "svrngs", "--test", "scenes",
           "holyarrs::holyarrs::chromium", "--", "--exact", "--nocapture", "--test-threads=1"]
PLANTS = {
    "no-approval": ("H3", "H3: HolyArrs imports the downloaded film not observed"),
    "no-seed": ("H3", "H3: HolyArrs imports the downloaded film not observed"),
    "no-readers": ("H4", "H4: Movie Night finds Ana’s newly imported copy not observed"),
    "credential": ("H7", "H7: Jellyfin admin reached Bo"),
    "early-stream": ("H6", "H6: stream must still be open before removal"),
}
results = []
for plant in sys.argv[1:] or PLANTS:
    bar, why = PLANTS[plant]
    env = dict(os.environ, SCENES_PLANT=f"holyarrs:{plant}")
    env.pop("HOLYARRS_RECORD", None)
    ran = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=600)
    log = ran.stdout + ran.stderr
    (OUT / f"{plant}.log").write_text(log)
    passed = ran.returncode != 0 and why in log
    result = {"plant": plant, "expected_failure": bar, "exit": ran.returncode,
              "verdict": "passed" if passed else "failed", "matched": why if passed else None}
    results.append(result)
    print(json.dumps(result), flush=True)
    (OUT / "results.json").write_text(json.dumps(results, indent=2))
    if not passed:
        print(log)
        raise SystemExit(1)
